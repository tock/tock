// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! RP2350-specific lockstep plumbing.
//!
//! Provides [`Rp2350Transport`], the [`lockstep::Transport`] implementation
//! for the RP2350's dual Cortex-M33 cores. All arch-agnostic lockstep logic
//! lives in the `lockstep` library crate and is re-exported here.
//!
//! # Wire design
//!
//! The RP2350's SIO inter-core FIFO is only 32 bits wide -- too narrow for
//! variants like `SyscallDesc`/`UpcallDesc` (4-5 fields each). Rather than
//! hand-roll a second shared-memory ring buffer, [`LOCKSTEP_CHAN`] reuses the
//! same [`BiChannel`] primitive the `qemu_rv32_virt` port already uses (note
//! that port's `LOCKSTEP_CHAN` is deliberately depth-4 to match *this* real
//! hardware FIFO's depth, per its own doc comment) -- it's already a
//! reviewed, correctly-fenced SPSC ring safe to share between two physically
//! concurrent cores. The real SIO FIFO is layered on top purely as the
//! doorbell/notification the plan calls for: a push writes any word to wake
//! the peer, which drains it and then reads the actual entry out of the
//! channel. Bulk byte payloads (UART RX) go through a separate raw shared
//! buffer, following the exact same pattern as the QEMU port's
//! `UART_RX_REPLAY_BUF`.

use core::cell::UnsafeCell;

use cortexm33::support::dmb;
use kernel::collections::spsc_channel::BiChannel;

use crate::chip::Processor;
use crate::gpio::SIO;

pub use lockstep::{
    lockstep_barrier, store_pending_syscall, BulkTag, DriverUpcallRules, LockstepDriver,
    LockstepUart, SyncEntry, SyscallDesc, Transport, UartHooks, UpcallMode, UpcallRule,
};

// ---------------------------------------------------------------------------
// Shared inter-core state
// ---------------------------------------------------------------------------

/// Inter-core lockstep channel.
///
/// Declared as a plain `static`: nothing on this target duplicates `.data`/
/// `.bss` per core (see the Stage A2 comment in `boards/raspberry_pi_pico_2/
/// layout.ld`), so both cores' shared `.text` computes the same absolute
/// address for this symbol -- only one instance exists, visible to both.
///
/// Depth of 4 matches the real SIO inter-core FIFO (32-bit wide, 4 entries
/// deep).
pub static LOCKSTEP_CHAN: BiChannel<4, SyncEntry> = BiChannel::new();

/// Maximum number of bytes that can be forwarded in one UART RX replay.
pub const UART_RX_REPLAY_MAX: usize = 256;

/// Bytes received by core 0's UART, to be replayed on core 1. Mirrors real
/// SIO FIFO usage: the tiny hardware FIFO only ever carries a short
/// notification ("data's ready"), with the bulk payload passed via ordinary
/// shared memory. `LOCKSTEP_CHAN`'s `UartRxReady { len }` message is that
/// notification; this buffer is the payload it points at.
pub struct UartRxReplayBuf(pub UnsafeCell<[u8; UART_RX_REPLAY_MAX]>);

// SAFETY: only core 0 writes the buffer (in receive()), and only before
// pushing UartRxReady onto LOCKSTEP_CHAN. Core 1 reads it only after popping
// that message. The channel's own push/pop ordering (Release before
// advancing the tail index, Acquire on read) provides the happens-before
// relationship that makes this raw shared-memory access sound.
unsafe impl Sync for UartRxReplayBuf {}

pub static UART_RX_REPLAY_BUF: UartRxReplayBuf =
    UartRxReplayBuf(UnsafeCell::new([0u8; UART_RX_REPLAY_MAX]));

// ---------------------------------------------------------------------------
// Rp2350Transport — Transport impl for SIO FIFO + BiChannel
// ---------------------------------------------------------------------------

pub const SYNC_TIMEOUT_TICKS: u32 = 5_000_000;
pub const DRAIN_TIMEOUT_TICKS: u32 = 5_000_000;

/// Zero-sized transport token for the RP2350 dual-core setup.
///
/// `try_push` / `try_pop` dispatch to the correct `LOCKSTEP_CHAN` direction
/// (`a_*` for core 0, `b_*` for core 1) based on `core_id()`, which reads
/// `SIO::get_processor()` fresh on each call (cheap: a single MMIO read, no
/// stored state needed since `SIO::new()` is just a fixed base-address
/// wrapper).
pub struct Rp2350Transport;

pub static RP2350_TRANSPORT: Rp2350Transport = Rp2350Transport;

impl Transport for Rp2350Transport {
    fn core_id(&self) -> u8 {
        match SIO::new().get_processor() {
            Processor::Processor0 => 0,
            Processor::Processor1 => 1,
        }
    }

    fn now_ticks(&self) -> u32 {
        crate::timer::now_ticks()
    }

    fn try_push(&self, e: SyncEntry) -> bool {
        let pushed = if self.core_id() == 0 {
            LOCKSTEP_CHAN.a_send(e)
        } else {
            LOCKSTEP_CHAN.b_send(e)
        };
        if pushed {
            // Doorbell: the FIFO word's value carries no meaning of its own
            // -- the payload already landed in LOCKSTEP_CHAN above, ordered
            // by its internal Release fence. This write just raises
            // SIO_IRQ_FIFO on the peer. `dmb` ensures the channel write is
            // visible before the peer observes the FIFO word.
            unsafe { dmb() };
            let _ = SIO::new().fifo_try_push(0);
        }
        pushed
    }

    fn try_pop(&self) -> Option<SyncEntry> {
        // Drain any doorbell word(s); LOCKSTEP_CHAN carries the real payload
        // regardless of how many doorbells coalesced into this poll.
        while SIO::new().fifo_try_pop().is_some() {}

        if self.core_id() == 0 {
            LOCKSTEP_CHAN.a_recv()
        } else {
            LOCKSTEP_CHAN.b_recv()
        }
    }

    fn kick_peer(&self) {
        // No-op: try_push's FIFO write already raised SIO_IRQ_FIFO on the
        // peer, and this transport's callers spin-poll rather than sleep.
    }

    fn bulk_write(&self, tag: BulkTag, bytes: &[u8]) {
        match tag {
            BulkTag::UartRx => {
                let copy_len = bytes.len().min(UART_RX_REPLAY_MAX);
                unsafe {
                    (&mut *UART_RX_REPLAY_BUF.0.get())[..copy_len]
                        .copy_from_slice(&bytes[..copy_len]);
                }
            }
        }
    }

    fn bulk_read(&self, tag: BulkTag, out: &mut [u8]) -> usize {
        match tag {
            BulkTag::UartRx => {
                let len = out.len().min(UART_RX_REPLAY_MAX);
                unsafe {
                    out[..len].copy_from_slice(&(&*UART_RX_REPLAY_BUF.0.get())[..len]);
                }
                len
            }
        }
    }

    const SYNC_TIMEOUT_TICKS: u32 = SYNC_TIMEOUT_TICKS;
    const DRAIN_TIMEOUT_TICKS: u32 = DRAIN_TIMEOUT_TICKS;
}

// ---------------------------------------------------------------------------
// Layer-1 event dispatch
// ---------------------------------------------------------------------------

/// Replay a single Layer-1 channel event on core 1.
///
/// Called from core 1's main loop for every `LOCKSTEP_CHAN` entry that isn't
/// a `Sync` (the only entries possible before Layer-2 syscall verification is
/// wired up in Step B2). Core 0 owns the real UART and never dispatches
/// these -- it only ever pushes them.
pub fn dispatch_layer1_event(entry: SyncEntry) {
    match entry {
        SyncEntry::UartRxReady { len } => crate::uart::replay_rx_done_for_core1(len),
        SyncEntry::UartTxDone => crate::uart::replay_tx_done_for_core1(),
        SyncEntry::SyscallDesc(desc) => store_pending_syscall(desc),
        SyncEntry::Sync { .. } | SyncEntry::UpcallDesc { .. } => {
            unreachable!("lockstep_barrier descriptors must not be dispatched as Layer-1 events")
        }
    }
}

// ---------------------------------------------------------------------------
// Rp2350UartHooks — RP2350-specific UART lockstep hooks
// ---------------------------------------------------------------------------

/// RP2350-specific UART lockstep hooks.
///
/// On core 0: `on_transmitted` pushes `UartTxDone`; `on_received` copies RX
/// bytes via `transport.bulk_write`, then pushes `UartRxReady`. Both pushes
/// implicitly kick core 1 (see `Rp2350Transport::try_push`). `on_transmit` is
/// a no-op (TX payload comparison happens at the `LockstepDriver::command`
/// syscall gate in Step B2, not here).
/// On core 1: all hooks are no-ops -- the replay mechanism
/// (`dispatch_layer1_event`) drives its callbacks instead.
pub struct Rp2350UartHooks {
    transport: &'static Rp2350Transport,
}

impl Rp2350UartHooks {
    pub const fn new(transport: &'static Rp2350Transport) -> Self {
        Self { transport }
    }
}

impl UartHooks for Rp2350UartHooks {
    fn on_transmit(&self, _buf: &[u8]) {}

    fn on_transmitted(&self, _buf: &[u8]) {
        if self.transport.core_id() == 0 {
            while !self.transport.try_push(SyncEntry::UartTxDone) {
                core::hint::spin_loop();
            }
        }
    }

    fn on_received(&self, buf: &[u8]) {
        if self.transport.core_id() == 0 {
            let copy_len = buf.len().min(UART_RX_REPLAY_MAX);
            self.transport.bulk_write(BulkTag::UartRx, &buf[..copy_len]);
            while !self
                .transport
                .try_push(SyncEntry::UartRxReady { len: copy_len as u8 })
            {
                core::hint::spin_loop();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rp2350UpcallVerifier — Layer-2 upcall verification (stub; TODO stage 3)
// ---------------------------------------------------------------------------

/// Lockstep upcall verifier for the RP2350 dual-core configuration.
///
/// Mirrors `QemuUpcallVerifier`: the registry/rule lookup is live, but the
/// actual cross-core exchange of masked upcall arguments (via
/// `lockstep_barrier` and `SyncEntry::UpcallDesc`) is not wired up yet --
/// same deferred scope as the QEMU port's own "stage3" TODO.
pub struct Rp2350UpcallVerifier {
    core_id: u8,
    registry: &'static [DriverUpcallRules],
}

impl Rp2350UpcallVerifier {
    pub fn new(registry: &'static [DriverUpcallRules]) -> Self {
        Self { core_id: RP2350_TRANSPORT.core_id(), registry }
    }
}

impl kernel::platform::UpcallVerifier for Rp2350UpcallVerifier {
    fn on_upcall(
        &self,
        id: kernel::upcall::UpcallId,
        r0: usize,
        r1: usize,
        r2: usize,
    ) -> kernel::platform::UpcallAction {
        let rule = self
            .registry
            .iter()
            .find(|d| d.driver_num == id.driver_num)
            .and_then(|d| d.rules.iter().find(|u| u.subscribe_num == id.subscribe_num));

        let rule = match rule {
            Some(r) => r,
            None => return kernel::platform::UpcallAction::Proceed,
        };

        // TODO(lockstep-stage3): cross-core channel exchange via lockstep_barrier.
        let _ = (rule, r0, r1, r2, self.core_id);
        kernel::platform::UpcallAction::Proceed
    }
}
