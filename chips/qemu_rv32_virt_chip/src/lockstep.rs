// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! QEMU-specific lockstep plumbing.
//!
//! Provides [`QemuTransport`], the [`lockstep::Transport`] implementation for
//! the QEMU RISC-V `virt` board (BiChannel + CLINT MSIP + `read_mtime_low`).
//! All arch-agnostic lockstep logic lives in the `lockstep` library crate and
//! is re-exported here for convenience.

use kernel::platform::{UpcallAction, UpcallVerifier};
use kernel::upcall::UpcallId;

// ---------------------------------------------------------------------------
// Re-exports from the shared lockstep crate
// ---------------------------------------------------------------------------

pub use lockstep::{
    lockstep_barrier, store_pending_syscall, BulkTag, DriverUpcallRules, LockstepDriver,
    LockstepUart, SyncEntry, SyscallDesc, Transport, UartHooks, UpcallMode, UpcallRule,
};

// ---------------------------------------------------------------------------
// Timeout constants (kept as pub consts so main.rs can reference them by name)
// ---------------------------------------------------------------------------

pub const SYNC_TIMEOUT_MTIME_TICKS: u32 = 1_000_000_000;
pub const DRAIN_TIMEOUT_MTIME_TICKS: u32 = 1_000_000_000;

// ---------------------------------------------------------------------------
// QemuTransport — Transport impl for QEMU BiChannel + CLINT MSIP
// ---------------------------------------------------------------------------

/// Zero-sized transport token for the QEMU `virt` dual-hart setup.
///
/// `try_push` / `try_pop` dispatch to the correct `BiChannel` direction
/// (`a_*` for hart 0, `b_*` for hart 1) based on `core_id()` which reads
/// `mhartid` on each call. `kick_peer` writes CLINT MSIP1 on hart 0 only;
/// hart 1 never needs to kick hart 0 because hart 0 spin-polls.
pub struct QemuTransport;

pub static QEMU_TRANSPORT: QemuTransport = QemuTransport;

impl lockstep::Transport for QemuTransport {
    fn core_id(&self) -> u8 {
        crate::chip::current_hart() as u8
    }

    fn now_ticks(&self) -> u32 {
        crate::chip::read_mtime_low()
    }

    fn try_push(&self, e: lockstep::SyncEntry) -> bool {
        if self.core_id() == 0 {
            crate::chip::LOCKSTEP_CHAN.a_send(e)
        } else {
            crate::chip::LOCKSTEP_CHAN.b_send(e)
        }
    }

    fn try_pop(&self) -> Option<lockstep::SyncEntry> {
        if self.core_id() == 0 {
            crate::chip::LOCKSTEP_CHAN.a_recv()
        } else {
            crate::chip::LOCKSTEP_CHAN.b_recv()
        }
    }

    fn kick_peer(&self) {
        if self.core_id() == 0 {
            unsafe { core::ptr::write_volatile(crate::chip::CLINT_MSIP1, 1) };
        }
        // Hart 1 does not kick hart 0 — hart 0 spin-polls the channel.
    }

    fn bulk_write(&self, tag: lockstep::BulkTag, bytes: &[u8]) {
        match tag {
            lockstep::BulkTag::UartRx => {
                let copy_len = bytes.len().min(crate::chip::UART_RX_REPLAY_MAX);
                unsafe {
                    (&mut *crate::chip::UART_RX_REPLAY_BUF.0.get())[..copy_len]
                        .copy_from_slice(&bytes[..copy_len]);
                }
            }
        }
    }

    fn bulk_read(&self, tag: lockstep::BulkTag, out: &mut [u8]) -> usize {
        match tag {
            lockstep::BulkTag::UartRx => {
                let len = out.len().min(crate::chip::UART_RX_REPLAY_MAX);
                unsafe {
                    out[..len].copy_from_slice(
                        &(&*crate::chip::UART_RX_REPLAY_BUF.0.get())[..len],
                    );
                }
                len
            }
        }
    }

    fn on_spin(&self) {
        crate::chip::clear_irq_active();
    }

    const SYNC_TIMEOUT_TICKS: u32 = SYNC_TIMEOUT_MTIME_TICKS;
    const DRAIN_TIMEOUT_TICKS: u32 = DRAIN_TIMEOUT_MTIME_TICKS;
}

// ---------------------------------------------------------------------------
// Layer-1 event dispatch
// ---------------------------------------------------------------------------

/// Replay a single Layer-1/Layer-2 channel event on hart 1.
///
/// Called from hart 1's Phase-1 drain loop for every entry that is not a
/// `Sync` or `SyscallDesc`. Hart 0 owns the real peripherals and never
/// dispatches these.
pub fn dispatch_layer1_event(entry: SyncEntry) {
    match entry {
        SyncEntry::UartRxReady { len } => crate::uart::replay_rx_done_for_hart1(len),
        SyncEntry::UartTxDone => crate::uart::replay_tx_done_for_hart1(),
        SyncEntry::SyscallDesc(desc) => store_pending_syscall(desc),
        SyncEntry::Sync { .. } | SyncEntry::UpcallDesc { .. } => {
            unreachable!("lockstep_barrier descriptors must not be dispatched as Layer-1 events")
        }
    }
}

// ---------------------------------------------------------------------------
// QemuUpcallVerifier — Layer-3 upcall verification (stub; TODO stage 3)
// ---------------------------------------------------------------------------

/// Lockstep upcall verifier for the QEMU dual-hart configuration.
pub struct QemuUpcallVerifier {
    hart_id: u32,
    registry: &'static [DriverUpcallRules],
}

impl QemuUpcallVerifier {
    pub fn new(registry: &'static [DriverUpcallRules]) -> Self {
        Self { hart_id: crate::chip::current_hart(), registry }
    }
}

impl UpcallVerifier for QemuUpcallVerifier {
    fn on_upcall(&self, id: UpcallId, r0: usize, r1: usize, r2: usize) -> UpcallAction {
        let rule = self
            .registry
            .iter()
            .find(|d| d.driver_num == id.driver_num)
            .and_then(|d| d.rules.iter().find(|u| u.subscribe_num == id.subscribe_num));

        let rule = match rule {
            Some(r) => r,
            None => return UpcallAction::Proceed,
        };

        // TODO(lockstep-stage3): cross-hart channel exchange via lockstep_barrier.
        let _ = (rule, r0, r1, r2, self.hart_id);
        UpcallAction::Proceed
    }
}

// ---------------------------------------------------------------------------
// QemuUartHooks — QEMU-specific UART lockstep hooks
// ---------------------------------------------------------------------------

/// QEMU-specific UART lockstep hooks.
///
/// On hart 0: `on_transmitted` pushes `UartTxDone` and kicks hart 1;
/// `on_received` copies RX bytes via `transport.bulk_write`, pushes
/// `UartRxReady`, and kicks hart 1. `on_transmit` is a no-op (TX buffer
/// comparison happens at the `LockstepDriver::command` syscall gate instead).
/// On hart 1: all hooks are no-ops — the replay mechanism drives callbacks.
pub struct QemuUartHooks {
    transport: &'static QemuTransport,
}

impl QemuUartHooks {
    pub fn new(transport: &'static QemuTransport) -> Self {
        Self { transport }
    }
}

impl UartHooks for QemuUartHooks {
    fn on_transmit(&self, _buf: &[u8]) {}

    fn on_transmitted(&self, _buf: &[u8]) {
        if self.transport.core_id() == 0 {
            while !self.transport.try_push(SyncEntry::UartTxDone) {
                core::hint::spin_loop();
            }
            self.transport.kick_peer();
        }
    }

    fn on_received(&self, buf: &[u8]) {
        if self.transport.core_id() == 0 {
            let copy_len = buf.len().min(crate::chip::UART_RX_REPLAY_MAX);
            self.transport.bulk_write(BulkTag::UartRx, &buf[..copy_len]);
            while !self
                .transport
                .try_push(SyncEntry::UartRxReady { len: copy_len as u8 })
            {
                core::hint::spin_loop();
            }
            self.transport.kick_peer();
        }
    }
}
