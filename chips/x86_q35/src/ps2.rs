// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! i8042 PS/2 controller for x86_q35
//!
//! Init policy (chip owns bring-up -> `init_early()`):
//!  - Disable ports, flush OB, self-test
//!  - Config: translation OFF (Set-2), IRQ bits off during tests
//!  - Port1 test + enable; device: F5, FF -> AA, F0 02 (Set-2), enable IRQ1, F4
//!
//! ISR/BH split:
//!  - ISR reads OB, drops on parity/timeout, queues bytes, schedules deferred call
//!  - Deferred call drains ring; for now it logs; and attempts to send data to a present client

use core::cell::{Cell, RefCell};
use kernel::debug;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
use x86::registers::io;
use x86::support;

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

/// Lightweight health snapshot for observability.
#[derive(Debug, Clone, Copy)]
pub struct Ps2Health {
    pub bytes_rx: u32,
    pub overruns: u32,
    pub parity_err: u32,
    pub timeout_err: u32,
    pub timeouts: u32, // controller wait timeouts (IB/OB)
    pub resends: u32,  // times device asked us to resend (0xFE)
}

/// Since I really want to minimise the risk of the controller
/// bringing the whole kernel down if a device (or even the controller itself)
/// breaks, I will add this simple display + its struct,
///
/// Log it anywhere with
/// debug!("ps2 health: {}", ps2.health_snapshot());
/// check chip.rs for example
impl core::fmt::Display for Ps2Health {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "rx={} overruns={} parity_err={} timeout_err={} ctrl_timeouts={} resends={}",
            self.bytes_rx,
            self.overruns,
            self.parity_err,
            self.timeout_err,
            self.timeouts,
            self.resends
        )
    }
}

/// Client hook (future: keyboard capsule)
pub trait Ps2Client {
    fn receive_scancode(&self, code: u8);
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

/// PS/2 controller driver (the “8042” peripheral)
pub struct Ps2Controller {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    count: Cell<usize>, // new field to track number of valid entries
    // track prefix bytes for logging => press/release only inputs
    break_next: Cell<bool>, // saw 0xF0; next data byte is a BREAK
    ext_next: Cell<bool>,   // saw 0xE0; next data byte is extended

    // "health" counters
    parity_drops: Cell<u32>,
    timeout_drops: Cell<u32>,
    overruns: Cell<u32>,
    bytes_rx: Cell<u32>,
    resends: Cell<u32>,
    timeouts: Cell<u32>,
    // deferred call handle (bottom-half scheduling)
    deferred_call: DeferredCall,

    // actually count health logs in board
    prev_log_bytes: Cell<u32>,

    // optional client for keyboard capsule later on
    client: OptionalCell<&'static dyn Ps2Client>,
}

impl Ps2Controller {
    pub fn new() -> Self {
        Self {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head: Cell::new(0),
            tail: Cell::new(0),
            count: Cell::new(0),
            break_next: Cell::new(false),
            ext_next: Cell::new(false),

            parity_drops: Cell::new(0),
            timeout_drops: Cell::new(0),
            overruns: Cell::new(0),
            bytes_rx: Cell::new(0),
            resends: Cell::new(0),
            timeouts: Cell::new(0),

            deferred_call: DeferredCall::new(),
            prev_log_bytes: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }

    /// telemetry debugging for future devices
    pub fn health_snapshot(&self) -> Ps2Health {
        Ps2Health {
            bytes_rx: self.bytes_rx.get(),
            overruns: self.overruns.get(),
            parity_err: self.parity_drops.get(),
            timeout_err: self.timeout_drops.get(),
            timeouts: self.timeouts.get(),
            resends: self.resends.get(),
        }
    }

    /// Install the "keyboard" client, we'll wire calls to this later
    /// for now it's just the hook
    pub fn set_client(&self, client: &'static dyn Ps2Client) {
        self.client.set(client);
    }

    /// Count controller wait timeouts
    #[inline(always)]
    fn tally_timeout<T>(&self, r: Ps2Result<T>) -> Ps2Result<T> {
        if matches!(r, Err(Ps2Error::TimeoutIB) | Err(Ps2Error::TimeoutOB)) {
            self.timeouts.set(self.timeouts.get().wrapping_add(1));
        }
        r
    }

    /// I moved this helpers inside the impl of the Controller
    /// Instead of resending an exact N amount of times
    /// I changed it so the send_with_ack actually counts the ~good~ bytes it received

    fn kbd_disable_scan(&self) -> Ps2Result<()> {
        self.send_with_ack(0xF5, 3)
    }

    fn kbd_reset_and_wait_bat(&self) -> Ps2Result<()> {
        self.send_with_ack(0xFF, 3)?; // reset
        match read_data()? {
            0xAA => Ok(()), // BAT passed
            other => Err(Ps2Error::UnexpectedResponse(other)),
        }
    }

    fn kbd_set_scancode_set2(&self) -> Ps2Result<()> {
        self.send_with_ack(0xF0, 3)?;
        self.send_with_ack(0x02, 3)
    }

    fn kbd_enable_scan(&self) -> Ps2Result<()> {
        self.send_with_ack(0xF4, 3)
    }

    /// Send a byte to the device and wait for ACK (0xFA).
    /// Count 0xFE (RESEND) attempts in `resends`.
    fn send_with_ack(&self, byte: u8, tries: u8) -> Ps2Result<()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            write_data(byte)?;
            let resp = read_data()?;
            match resp {
                0xFA => return Ok(()), // ACK
                0xFE if attempts < tries => {
                    self.resends.set(self.resends.get().wrapping_add(1));
                    continue;
                }
                0xFE => {
                    self.resends.set(self.resends.get().wrapping_add(1));
                    return Err(Ps2Error::AckError);
                }
                other => return Err(Ps2Error::UnexpectedResponse(other)),
            }
        }
    }

    /// Pure controller + device bring-up.
    /// No logging, no PIC masking/unmasking, no CPU-IRQ enabling. (hopefully)
    /// Called by PcComponent::finalize() (chip layer).
    ///
    /// Whole goal of this change is to stop nagging the memory directly
    /// We created tiny wrappers and helpers, so we can configure the init
    /// much easier
    ///
    /// For testing and health of the controller, we'll wrap each call with
    /// tally_timeout helper
    pub fn init_early(&self) -> Ps2Result<()> {
        // disable ports; flush OB
        self.tally_timeout(disable_ports())?;
        self.tally_timeout(flush_output_buffer())?;

        // controller self-test
        self.tally_timeout(controller_self_test())?;

        // config policy (do not generate IRQs during tests)
        self.tally_timeout(cfg_set_irq1(false))?; // IRQ1 off
        self.tally_timeout(cfg_set_irq12(false))?; // IRQ12 off
        self.tally_timeout(cfg_set_translation(false))?; // translation OFF (we want Set2)
        self.tally_timeout(cfg_set_port1_clock(true))?; // keyboard clock enabled
        self.tally_timeout(cfg_set_port2_clock(false))?; // mouse clock disabled (for now)

        // port1 test then enable keyboard clock at the controller command level
        self.tally_timeout(port1_interface_test())?;
        self.tally_timeout(enable_port1_clock())?; // 0xAE

        // device sequence (keyboard)
        self.tally_timeout(self.kbd_disable_scan())?; // F5
        self.tally_timeout(self.kbd_reset_and_wait_bat())?; // FF -> BAT=AA
        self.tally_timeout(self.kbd_set_scancode_set2())?; // F0 02
        self.tally_timeout(cfg_set_irq1(true))?; // turn on controller-side IRQ1 (PIC policy lives in chip)
        self.tally_timeout(self.kbd_enable_scan())?; // F4

        Ok(())
    }

    /// Drain queued bytes and print clean MAKE/BREAK lines.
    /// Runs in the deferred bottom-half (not in IRQ context).
    #[inline(always)]
    fn decode_and_log_stream(&self) {
        while let Some(b) = self.pop_scan_code() {
            // hand the raw byte to a client if present
            self.client.map(|c| c.receive_scancode(b));

            // Track Set-2 prefixes locally; we keep state in the controller
            if b == 0xE0 {
                self.ext_next.set(true);
                continue;
            }
            if b == 0xF0 {
                self.break_next.set(true);
                continue;
            }

            let ext = self.ext_next.replace(false);
            let brk = self.break_next.replace(false);

            if brk {
                debug!("ps2: BREAK {}{:02X}", if ext { "E0 " } else { "" }, b);
            } else {
                debug!("ps2: MAKE  {}{:02X}", if ext { "E0 " } else { "" }, b);
            }
        }
    }
    pub fn handle_interrupt(&self) {
        let mut scheduled = false;

        loop {
            // Check if there is a byte waiting in the output buffer (OB)
            let status = read_status();
            if (status & 0x01) == 0 {
                // OUTPUT_FULL == 0 → done
                break;
            }

            // Reading from 0x60 consumes one byte from OB.
            let b = unsafe { io::inb(PS2_DATA_PORT) };

            // Drop corrupted bytes, bump counters. No logging here.
            if (status & (1 << 7)) != 0 {
                // PARITY_ERR
                self.parity_drops
                    .set(self.parity_drops.get().wrapping_add(1));
                continue;
            }
            if (status & (1 << 6)) != 0 {
                // TIMEOUT_ERR
                self.timeout_drops
                    .set(self.timeout_drops.get().wrapping_add(1));
                continue;
            }

            // Enqueue safely; overflow tracked inside.
            self.push_code_atomic(b);
            scheduled = true;
        }

        // Kick the bottom-half once if we queued anything.
        if scheduled {
            self.deferred_call.set();
        }
    }

    /// Pop the next scan-code, or None if buffer is empty.
    #[inline(always)]
    fn push_code_atomic(&self, b: u8) {
        // Mask IRQs briefly while we mutate head/tail/count.

        support::with_interrupts_disabled(|| {
            let h = self.head.get();
            self.buffer.borrow_mut()[h] = b;
            self.head.set((h + 1) % BUFFER_SIZE);

            if self.count.get() < BUFFER_SIZE {
                self.count.set(self.count.get() + 1);
            } else {
                // overwrite oldest and count overrun
                self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
                self.overruns.set(self.overruns.get().wrapping_add(1));
            }

            // count accepted bytes
            self.bytes_rx.set(self.bytes_rx.get().wrapping_add(1));
        })
    }
    /// Internal: push a scan-code into the ring buffer, dropping oldest if full.
    pub fn pop_scan_code(&self) -> Option<u8> {
        // Same tiny IRQ-masked critical section for the consumer.

        support::with_interrupts_disabled(|| {
            if self.count.get() == 0 {
                None
            } else {
                let t = self.tail.get();
                let b = self.buffer.borrow()[t];
                self.tail.set((t + 1) % BUFFER_SIZE);
                self.count.set(self.count.get() - 1);
                Some(b)
            }
        })
    }
}
impl DeferredCallClient for Ps2Controller {
    fn handle_deferred_call(&self) {
        // drain and print MAKE/BREAKs
        self.decode_and_log_stream();

        // log health only when bytes_rx advanced
        let cur = self.bytes_rx.get();
        // print every N bytes so we don't spam the console
        if cur.wrapping_sub(self.prev_log_bytes.get()) >= 16 {
            self.prev_log_bytes.set(cur);
            debug!("ps/2 health: {}", self.health_snapshot());
        }
    }
    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
