// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::cell::{Cell, RefCell};
use kernel::debug;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Depth of the scan-code ring buffer
const BUFFER_SIZE: usize = 32;

/// Timeout limit for spin loops
const TIMEOUT_LIMIT: usize = 1_000_000;

/// Controller Configuration Byte bits (check OSDev)
const CFG_IRQ1: u8 = 1 << 0; // keyboard IRQ enable
const CFG_IRQ12: u8 = 1 << 1; // mouse IRQ enable
const CFG_DISABLE_KBD: u8 = 1 << 4; // 1=disable keyboard clock
const CFG_DISABLE_AUX: u8 = 1 << 5; // 1=disable mouse clock
const CFG_TRANSLATION: u8 = 1 << 6; // 1=translate Set-2->Set-1

// Status-register bits returned by inb(0x64)
register_bitfields![u8,
    pub STATUS [
        OUTPUT_FULL OFFSET(0) NUMBITS(1), // OB has data
        INPUT_FULL  OFFSET(1) NUMBITS(1), // IB is full (busy)
        // (bit2 SYSFLAG and bit3 CMD/DATA not used here)
        AUX_OBF     OFFSET(5) NUMBITS(1), // 1 = from mouse/port2
        TIMEOUT_ERR OFFSET(6) NUMBITS(1),
        PARITY_ERR  OFFSET(7) NUMBITS(1),
    ]
];

/// Error types that  the controller can return
/// so we don't bring the whole kernel down
/// if something breaks in a peripheral
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ps2Error {
    TimeoutIB,
    TimeoutOB,
    SelfTestFailed,
    Port1TestFailed,
    AckError,
    ControllerTimeout,
    UnexpectedResponse(u8),
}

pub type Ps2Result<T> = core::result::Result<T, Ps2Error>;

/// Note: There is no hardware interrupt when the input buffer empties, so we must poll bit 1.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
///
/// Block until the controller’s input buffer is empty (ready for a command).

#[inline(always)]
fn wait_ib_empty() -> Ps2Result<()> {
    let mut status = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut loops = 0;
    while {
        status.set(read_status());
        status.is_set(STATUS::INPUT_FULL)
    } {
        loops += 1;
        if loops >= TIMEOUT_LIMIT {
            return Err(Ps2Error::TimeoutIB);
        }
    }
    Ok(())
}

/// Data-ready events trigger IRQ1, handled asynchronously in `handle_interrupt()`.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
///
/// Block until there is data ready to read in the output buffer.
#[inline(always)]
fn wait_ob_full() -> Ps2Result<()> {
    let mut status = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut loops = 0;
    while {
        status.set(read_status());
        !status.is_set(STATUS::OUTPUT_FULL)
    } {
        loops += 1;
        if loops >= TIMEOUT_LIMIT {
            return Err(Ps2Error::TimeoutOB);
        }
    }
    Ok(())
}

/// Read one byte from the data port (0x60).
#[inline(always)]
fn read_data() -> Ps2Result<u8> {
    wait_ob_full()?;
    Ok(unsafe { io::inb(PS2_DATA_PORT) })
}

/// Send a command byte to the controller (port 0x64).
#[inline(always)]
fn write_command(c: u8) -> Ps2Result<()> {
    wait_ib_empty()?;
    unsafe { io::outb(PS2_STATUS_PORT, c) }
    Ok(())
}

/// Write a data byte to the data port (0x60).
#[inline(always)]
fn write_data(d: u8) -> Ps2Result<()> {
    wait_ib_empty()?;
    unsafe { io::outb(PS2_DATA_PORT, d) }
    Ok(())
}

#[inline(always)]
fn read_status() -> u8 {
    unsafe { io::inb(PS2_STATUS_PORT) }
}

fn read_config() -> Ps2Result<u8> {
    write_command(0x20)?; // Read Controller Configuration Byte
    read_data()
}

fn write_config(cfg: u8) -> Ps2Result<()> {
    write_command(0x60)?; // Write Controller Configuration Byte
    write_data(cfg)
}

fn update_config<F: FnOnce(u8) -> u8>(f: F) -> Ps2Result<u8> {
    let cur = read_config()?;
    let new = f(cur);
    write_config(new)?;
    Ok(new)
}

/// Config helpers so we don't break the whole address in the init
/// we can do it sequentially

fn cfg_set_translation(enabled: bool) -> Ps2Result<u8> {
    update_config(|mut c| {
        if enabled {
            c |= CFG_TRANSLATION
        } else {
            c &= !CFG_TRANSLATION
        }
        c
    })
}

fn cfg_set_port1_clock(enabled: bool) -> Ps2Result<u8> {
    // enabled => clear DISABLE_KBD bit
    update_config(|mut c| {
        if enabled {
            c &= !CFG_DISABLE_KBD
        } else {
            c |= CFG_DISABLE_KBD
        }
        c
    })
}

fn cfg_set_port2_clock(enabled: bool) -> Ps2Result<u8> {
    // enabled => clear DISABLE_AUX bit
    update_config(|mut c| {
        if enabled {
            c &= !CFG_DISABLE_AUX
        } else {
            c |= CFG_DISABLE_AUX
        }
        c
    })
}

fn cfg_set_irq1(enabled: bool) -> Ps2Result<u8> {
    update_config(|mut c| {
        if enabled {
            c |= CFG_IRQ1
        } else {
            c &= !CFG_IRQ1
        }
        c
    })
}

fn cfg_set_irq12(enabled: bool) -> Ps2Result<u8> {
    update_config(|mut c| {
        if enabled {
            c |= CFG_IRQ12
        } else {
            c &= !CFG_IRQ12
        }
        c
    })
}
fn disable_ports() -> Ps2Result<()> {
    write_command(0xAD)?; // disable keyboard (port 1)
    write_command(0xA7)?; // disable aux (port 2)
    Ok(())
}

fn flush_output_buffer() -> Ps2Result<()> {
    while read_status() & 0x01 != 0 {
        let _ = read_data()?; // non-blocking-ish: read_data waits only if OB set
    }
    Ok(())
}

fn controller_self_test() -> Ps2Result<()> {
    write_command(0xAA)?;
    match read_data()? {
        0x55 => Ok(()),
        other => Err(Ps2Error::UnexpectedResponse(other)),
    }
}

fn port1_interface_test() -> Ps2Result<()> {
    write_command(0xAB)?;
    match read_data()? {
        0x00 => Ok(()),
        other => Err(Ps2Error::UnexpectedResponse(other)),
    }
}

fn enable_port1_clock() -> Ps2Result<()> {
    write_command(0xAE)
}

fn kbd_disable_scan() -> Ps2Result<()> {
    send_with_ack(0xF5, 3)
}

fn kbd_reset_and_wait_bat() -> Ps2Result<()> {
    send_with_ack(0xFF, 3)?; // ACK for reset
    match read_data()? {
        // BAT result
        0xAA => Ok(()),
        other => Err(Ps2Error::UnexpectedResponse(other)),
    }
}

fn kbd_set_scancode_set2() -> Ps2Result<()> {
    send_with_ack(0xF0, 3)?;
    send_with_ack(0x02, 3)
}

fn kbd_enable_scan() -> Ps2Result<()> {
    send_with_ack(0xF4, 3)
}

/// Send a byte to the keyboard and wait for ACK (`0xFA`).
/// If the device replies RESEND (`0xFE`) we retry **once**.
///
/// Heads-up: this will be modified in the keyboard driver
/// to better handle command requests,
fn send_with_ack(byte: u8, tries: u8) -> Ps2Result<()> {
    let mut attempts = 0;
    loop {
        attempts += 1;
        write_data(byte)?;
        let resp = read_data()?;
        match resp {
            0xFA => return Ok(()),                // ACK
            0xFE if attempts < tries => continue, // RESEND -> retry
            0xFE => return Err(Ps2Error::AckError),
            other => return Err(Ps2Error::UnexpectedResponse(other)),
        }
    }
}

/// PS/2 controller driver (the “8042” peripheral)
pub struct Ps2Controller {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    count: Cell<usize>, // new field to track number of valid entries
    // track prefix bytes for logging => press/release only inputs
    break_next: Cell<bool>, // saw 0xF0; next data byte is a BREAK
    ext_next: Cell<bool>,   // saw 0xE0; next data byte is extended
}

impl Ps2Controller {
    pub const fn new() -> Self {
        Self {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head: Cell::new(0),
            tail: Cell::new(0),
            count: Cell::new(0),
            break_next: Cell::new(false),
            ext_next: Cell::new(false),
        }
    }

    /// Pure controller + device bring-up.
    /// No logging, no PIC masking/unmasking, no CPU-IRQ enabling. (hopefully)
    /// Called by PcComponent::finalize() (chip layer).
    ///
    /// Whole goal of this change is to stop nagging the memory directly
    /// We created tiny wrappers and helpers, so we can configure the init
    /// much easier
    pub fn init_early(&self) -> Ps2Result<()> {
        // disable ports; flush OB
        disable_ports()?;
        flush_output_buffer()?;

        // controller self-test
        controller_self_test()?;

        // config policy (do not generate IRQs during tests)
        cfg_set_irq1(false)?; // IRQ1 off
        cfg_set_irq12(false)?; // IRQ12 off
        cfg_set_translation(false)?; // translation OFF (we want Set2)
        cfg_set_port1_clock(true)?; // keyboard clock enabled
        cfg_set_port2_clock(false)?; // mouse clock disabled (for now)

        // port1 test then enable keyboard clock at the controller command level
        port1_interface_test()?;
        enable_port1_clock()?; // 0xAE

        // device sequence (keyboard)
        kbd_disable_scan()?; // F5
        kbd_reset_and_wait_bat()?; // FF -> BAT=AA
        kbd_set_scancode_set2()?; // F0 02
        cfg_set_irq1(true)?; // turn on controller-side IRQ1 (PIC policy lives in chip)
        kbd_enable_scan()?; // F4

        Ok(())
    }

    /// Handle a keyboard interrupt: read a scan-code and buffer it.
    /// as of now it works, but it prints press/release as well
    /// which gets messy in the terminal
    /// for testing, i'll make it so we only log make/break events
    /* pub fn handle_interrupt(&self) {
        loop {
            if unsafe { io::inb(PS2_STATUS_PORT) } & 0x01 == 0 {
                break;
            }
            let byte = unsafe { io::inb(PS2_DATA_PORT) };
            self.push_code(byte);
            debug!("ps2 irq 0x{:02X}", byte);
        }
    }*/

    pub fn handle_interrupt(&self) {
        loop {
            // Snapshot the controller status. Bit0 (OUTPUT_FULL) says whether
            // there's a byte waiting in the output buffer (OB) at 0x60.
            let status = read_status();
            if (status & 0x01) == 0 {
                // OUTPUT_FULL == 0 => no more data
                break;
            }

            // Reading 0x60 pops one byte from OB and (for that byte) clears OUTPUT_FULL.
            let b = unsafe { io::inb(PS2_DATA_PORT) };

            // Check error flags associated with this byte:
            // bit7 PARITY_ERR
            // bit6 TIMEOUT_ERR
            // If either is set we drop the byte (we already consumed OB by reading).
            let parity = (status & (1 << 7)) != 0;
            let timeout = (status & (1 << 6)) != 0;
            if parity || timeout {
                debug!(
                    "ps2: dropped byte {:02X} ({}{})",
                    b,
                    if parity { "PARITY" } else { "" },
                    if timeout {
                        if parity {
                            "+"
                        } else {
                            ""
                        }
                    } else {
                        ""
                    }
                );
                continue; // don’t push/log corrupted data
            }

            // Always preserve the raw stream (including prefixes) for the future keyboard driver.
            self.push_code(b);

            // Set-2 uses prefixes:
            // - 0xE0 marks "extended" keys (next data byte is extended)
            // - 0xF0 marks a BREAK (key release) for the next data byte
            // We don't log prefixes themselves; we just remember them.
            if b == 0xE0 {
                self.ext_next.set(true);
                continue;
            }
            if b == 0xF0 {
                self.break_next.set(true);
                continue;
            }

            // At this point `b` is a real scancode byte (not a prefix).
            // Consume the remembered prefix flags to print exactly one MAKE/BREAK line.
            let ext = self.ext_next.get();
            let brk = self.break_next.get();
            if ext {
                self.ext_next.set(false);
            }
            if brk {
                self.break_next.set(false);
            }

            if brk {
                debug!("ps2: BREAK {}{:02X}", if ext { "E0 " } else { "" }, b);
            } else {
                debug!("ps2: MAKE  {}{:02X}", if ext { "E0 " } else { "" }, b);
            }
        }
    }

    /// Pop the next scan-code, or None if buffer is empty.
    pub fn pop_scan_code(&self) -> Option<u8> {
        if self.count.get() == 0 {
            return None;
        }
        let t = self.tail.get();
        let b = self.buffer.borrow()[t];
        self.tail.set((t + 1) % BUFFER_SIZE);
        self.count.set(self.count.get() - 1);
        Some(b)
    }

    /// Internal: push a scan-code into the ring buffer, dropping oldest if full.
    fn push_code(&self, b: u8) {
        let h = self.head.get();
        self.buffer.borrow_mut()[h] = b;
        let nh = (h + 1) % BUFFER_SIZE;
        self.head.set(nh);

        if self.count.get() < BUFFER_SIZE {
            self.count.set(self.count.get() + 1);
        } else {
            self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
        }
    }
}
