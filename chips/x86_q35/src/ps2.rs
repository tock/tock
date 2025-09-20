// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! i8042 PS/2 controller for x86_q35
//!
//! Init policy (chip owns bring-up -> `init_early_with()`):
//!  - Disable ports, flush OB, self-test
//!  - Config: translation OFF (Set-2), IRQ bits off during tests
//!  - Port1 test + enable; device: F5, FF -> AA, F0 02 (Set-2), enable IRQ1, F4
//!
//! ISR/BH split:
//!  - ISR reads OB, drops on parity/timeout, queues bytes, schedules deferred call
//!  - Deferred call drains ring; for now it logs; and attempts to send data to a present client

use core::cell::Cell;
use kernel::debug;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Depth of the scan-code ring buffer
const BUFFER_SIZE: usize = 32;

/// This is not time-based. We repeatedly read the controller status (0x64)
/// and check the relevant bit; if we’ve spun this many iterations without the
/// condition becoming true, we return a Timeout error.

/// Big spin budget used during controller/device bring-up
const SPINS_INIT: usize = 32768;

/// Small budget for non-init use
const SPINS_RUNTIME: usize = 256;

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

register_bitfields![u8,
pub CONFIG [
        IRQ1 OFFSET(0) NUMBITS(1),
        IRQ12 OFFSET(1) NUMBITS(1),
        SYSFLAG OFFSET(2) NUMBITS(1),
        RESERVED3 OFFSET(3) NUMBITS(1),
        DISABLE_KBD OFFSET(4) NUMBITS(1),
        DISABLE_AUX OFFSET(5) NUMBITS(1),
        TRANSLATION OFFSET(6) NUMBITS(1),
        RESERVED7 OFFSET(7) NUMBITS(1),
    ]
];

/// Raw byte sink for PS/2 controller clients (keyboard, mouse).
pub trait Ps2Client {
    /// Called in deferred context for each byte pulled from the ring.
    fn receive_scancode(&self, byte: u8);
}

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

/// Since we really want to minimise the risk of the controller
/// bringing the whole kernel down if a device (or even the controller itself)
/// breaks, we will add this simple display and its struct,

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

pub type Ps2Result<T> = core::result::Result<T, Ps2Error>;

/// Note: There is no hardware interrupt when the input buffer empties, so we must poll bit 1.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
///
/// Block until the controller’s input buffer is empty (ready for a command).

#[inline(always)]
fn wait_ib_empty_with_timeout(limit: usize) -> Ps2Result<()> {
    let mut spins = 0usize;
    while read_status().is_set(STATUS::INPUT_FULL) {
        spins += 1;
        if spins >= limit {
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
fn wait_ob_full_with_timeout(limit: usize) -> Ps2Result<()> {
    let mut spins = 0usize;
    while !read_status().is_set(STATUS::OUTPUT_FULL) {
        spins += 1;
        if spins >= limit {
            return Err(Ps2Error::TimeoutOB);
        }
    }
    Ok(())
}

/// Read one byte from the data port (0x60).
#[inline(always)]
fn read_data_with(limit: usize) -> Ps2Result<u8> {
    wait_ob_full_with_timeout(limit)?;
    Ok(unsafe { io::inb(PS2_DATA_PORT) })
}

/// Send a command byte to the controller (port 0x64).
#[inline(always)]
fn write_command_with(c: u8, limit: usize) -> Ps2Result<()> {
    wait_ib_empty_with_timeout(limit)?;
    unsafe { io::outb(PS2_STATUS_PORT, c) }
    Ok(())
}

/// Write a data byte to the data port (0x60).
#[inline(always)]
fn write_data_with(d: u8, limit: usize) -> Ps2Result<()> {
    wait_ib_empty_with_timeout(limit)?;
    unsafe { io::outb(PS2_DATA_PORT, d) }
    Ok(())
}
#[inline(always)]
fn read_status() -> LocalRegisterCopy<u8, STATUS::Register> {
    LocalRegisterCopy::new(unsafe { io::inb(PS2_STATUS_PORT) })
}

#[inline(always)]
fn read_config_reg_with(limit: usize) -> Ps2Result<LocalRegisterCopy<u8, CONFIG::Register>> {
    write_command_with(0x20, limit)?; // Read Controller Configuration Byte
    Ok(LocalRegisterCopy::new(read_data_with(limit)?))
}

#[inline(always)]
fn write_config_reg_with(
    cfg: LocalRegisterCopy<u8, CONFIG::Register>,
    limit: usize,
) -> Ps2Result<()> {
    write_command_with(0x60, limit)?; // Write Controller Configuration Byte
    write_data_with(cfg.get(), limit)
}

fn update_config_with<F>(limit: usize, f: F) -> Ps2Result<u8>
where
    F: FnOnce(LocalRegisterCopy<u8, CONFIG::Register>) -> LocalRegisterCopy<u8, CONFIG::Register>,
{
    let cur = read_config_reg_with(limit)?;
    let new = f(cur);
    write_config_reg_with(new, limit)?;
    Ok(new.get())
}

fn flush_output_buffer() -> Ps2Result<()> {
    // Poll status; if OB has data, read once and loop.
    // Use the small runtime budget for the read.
    while read_status().is_set(STATUS::OUTPUT_FULL) {
        let _ = read_data_with(SPINS_RUNTIME)?;
    }
    Ok(())
}

/// PS/2 controller driver (the “8042” peripheral)
pub struct Ps2Controller {
    buffer: MapCell<[u8; BUFFER_SIZE]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    count: Cell<usize>, // new field to track number of valid entries

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

    // raw byte client (keyboard device)
    client: OptionalCell<&'static dyn Ps2Client>,
}

impl Ps2Controller {
    pub fn new() -> Self {
        Self {
            buffer: MapCell::new([0; BUFFER_SIZE]),
            head: Cell::new(0),
            tail: Cell::new(0),
            count: Cell::new(0),

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
    pub fn set_client(&self, client: &'static dyn Ps2Client) {
        self.client.set(client);
    }

    #[inline(always)]
    fn drain_and_deliver(&self) {
        while let Some(b) = self.pop_scan_code() {
            self.client.map(|c| c.receive_scancode(b));
        }
    }

    /// Count controller wait timeouts
    #[inline(always)]
    fn tally_timeout<T>(&self, r: Ps2Result<T>) -> Ps2Result<T> {
        if matches!(r, Err(Ps2Error::TimeoutIB) | Err(Ps2Error::TimeoutOB)) {
            self.timeouts.set(self.timeouts.get().wrapping_add(1));
        }
        r
    }

    /// Keyboard device commands (port 0x60)
    /// Thin wrappers over `send_with_ack`. Kept as methods so they can use the
    /// controller’s counters/state.

    /// Send a device command and wait for ACK (0xFA).
    /// Retries on RESEND (0xFE) up to `tries - 1` times and increments `self.resends`.
    /// Returns:
    /// `Ok(())` on ACK
    /// `Err(Ps2Error::AckError)` if RESEND persists after all retries
    /// `Err(Ps2Error::UnexpectedResponse(_))` for any other response byte
    fn send_with_ack_limit(&self, byte: u8, tries: u8, limit: usize) -> Ps2Result<()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            write_data_with(byte, limit)?;
            match read_data_with(limit)? {
                0xFA => return Ok(()),
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
    pub fn init_early_with_spins(&self, spins: usize) -> Ps2Result<()> {
        // disable ports; flush OB
        self.tally_timeout(write_command_with(0xAD, spins))?;
        self.tally_timeout(write_command_with(0xA7, spins))?;
        self.tally_timeout(flush_output_buffer())?;

        // controller self-test
        self.tally_timeout({
            write_command_with(0xAA, spins)?;
            match read_data_with(spins)? {
                0x55 => Ok(()),
                other => Err(Ps2Error::UnexpectedResponse(other)),
            }
        })?;

        // config policy (do not generate IRQs during tests)
        self.tally_timeout(update_config_with(spins, |mut c| {
            c.modify(CONFIG::IRQ1::CLEAR);
            c
        }))?;
        self.tally_timeout(update_config_with(spins, |mut c| {
            c.modify(CONFIG::IRQ12::CLEAR);
            c
        }))?;
        self.tally_timeout(update_config_with(spins, |mut c| {
            c.modify(CONFIG::TRANSLATION::CLEAR);
            c
        }))?;
        self.tally_timeout(update_config_with(spins, |mut c| {
            c.modify(CONFIG::DISABLE_KBD::CLEAR);
            c
        }))?;
        self.tally_timeout(update_config_with(spins, |mut c| {
            c.modify(CONFIG::DISABLE_AUX::SET);
            c
        }))?;

        // port1 test then enable keyboard clock at the controller command level
        self.tally_timeout({
            write_command_with(0xAB, spins)?;
            match read_data_with(spins)? {
                0x00 => Ok(()),
                other => Err(Ps2Error::UnexpectedResponse(other)),
            }
        })?;
        self.tally_timeout(write_command_with(0xAE, spins))?;

        // device sequence (keyboard)
        self.tally_timeout(self.send_with_ack_limit(0xF5, 3, spins))?; // F5 disable scan
        self.tally_timeout(self.send_with_ack_limit(0xFF, 3, spins))?; // FF reset
        self.tally_timeout({
            match read_data_with(spins)? {
                0xAA => Ok(()), // BAT passed
                other => Err(Ps2Error::UnexpectedResponse(other)),
            }
        })?;
        self.tally_timeout(self.send_with_ack_limit(0xF0, 3, spins))?; // select set
        self.tally_timeout(self.send_with_ack_limit(0x02, 3, spins))?; // set-2
        self.tally_timeout(update_config_with(spins, |mut c| {
            c.modify(CONFIG::IRQ1::SET);
            c
        }))?;
        self.tally_timeout(self.send_with_ack_limit(0xF4, 3, spins))?; // enable scan

        Ok(())
    }

    /// Back-compat wrapper: long spin budget during bring-up.
    pub fn init_early(&self) -> Ps2Result<()> {
        self.init_early_with_spins(SPINS_INIT)
    }
    pub fn handle_interrupt(&self) {
        let mut scheduled = false;

        loop {
            let status = read_status();
            if !status.is_set(STATUS::OUTPUT_FULL) {
                break;
            }

            let b = unsafe { io::inb(PS2_DATA_PORT) };

            if status.is_set(STATUS::PARITY_ERR) {
                self.parity_drops
                    .set(self.parity_drops.get().wrapping_add(1));
                continue;
            }
            if status.is_set(STATUS::TIMEOUT_ERR) {
                self.timeout_drops
                    .set(self.timeout_drops.get().wrapping_add(1));
                continue;
            }

            self.push_code(b);
            scheduled = true;
        }

        if scheduled {
            self.deferred_call.set();
        }
    }

    /// Producer-side enqueue: push one scan-code byte into the ring buffer.
    /// Safe without masking on x86_q35: both `handle_interrupt()` and
    /// deferred calls run on the kernel main loop (no preemption).
    #[inline(always)]
    fn push_code(&self, b: u8) {
        let h = self.head.get();
        self.buffer.map(|buf| {
            buf[h] = b;
        });
        self.head.set((h + 1) % BUFFER_SIZE);
        if self.count.get() < BUFFER_SIZE {
            self.count.set(self.count.get() + 1);
        } else {
            self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
            self.overruns.set(self.overruns.get().wrapping_add(1));
        }
        self.bytes_rx.set(self.bytes_rx.get().wrapping_add(1));
    }

    /// Consumer-side dequeue. Also safe without masking for the same reason.
    pub fn pop_scan_code(&self) -> Option<u8> {
        if self.count.get() == 0 {
            None
        } else {
            let t = self.tail.get();
            // without `const` buffer type, the
            // old declaration MapCell::new([0; BUFFER_SIZE]) was fine for initialization,
            // but it broke when we tried to write through the .map closure
            //
            // since MapCell::map returns Option<R> to prevent nested borrows
            // our closure returns an `u8`, so the whole call is Option<u8>,
            // wrap it (or use a temp Cell) so we hand back a `u8`
            let b = self.buffer.map(|buf| buf[t]).unwrap();
            self.tail.set((t + 1) % BUFFER_SIZE);
            self.count.set(self.count.get() - 1);
            Some(b)
        }
    }
}
impl DeferredCallClient for Ps2Controller {
    fn handle_deferred_call(&self) {
        // drain ring and deliver raw bytes to the client
        self.drain_and_deliver();

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
