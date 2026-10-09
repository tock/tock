// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! A `SchedulerTimer` for QEMU's `mps2-an385`/`mps2-an386` machine types that
//! works around a QEMU SysTick emulation bug.
//!
//! This wraps [`cortexm::systick::SysTick`] -- the generic, hardware-accurate
//! SysTick driver shared by every Cortex-M board -- and forwards every call
//! to it unchanged, *except* for [`start()`](SchedulerTimer::start) and
//! [`reset()`](SchedulerTimer::reset), which this type implements itself.
//! The generic driver is left exactly as it models real Cortex-M silicon;
//! this is a QEMU-only shadow sitting in front of it.
//!
//! # The bug
//!
//! `start()`/`reset()` on real hardware re-arm a fresh timeslice by writing
//! `SYST_RVR` (the new reload value) and then writing `SYST_CVR` (which the
//! architecture defines as "clears to 0, regardless of the value written")
//! before enabling -- the standard CMSIS `SysTick_Config()` idiom. Real
//! silicon guarantees this silently reloads from `SYST_RVR` and continues
//! counting on the very next clock, with no spurious interrupt.
//!
//! QEMU's SysTick device (`hw/timer/armv7m_systick.c`, at least as of QEMU
//! 11.0.1) is implemented on top of its generic `ptimer` primitives with the
//! policy flags `PTIMER_POLICY_TRIGGER_ONLY_ON_DECREMENT` and
//! `PTIMER_POLICY_NO_IMMEDIATE_RELOAD`. Reload-from-limit is only triggered
//! by the counter *decrementing* to 0 from a positive value -- but writing
//! `SYST_CVR` sets the count to 0 directly, with nothing to decrement from.
//! So the counter is left permanently parked at 0: every subsequent read
//! reports the timeslice as already expired, and nothing ever un-wedges it.
//! Tock's round-robin scheduler fully resets and restarts the scheduler
//! timer on every single scheduling decision, so this fires continuously
//! once a process is runnable -- in practice, close to 100% of the CPU's
//! time goes to re-arming and immediately "expiring" the timer rather than
//! running any process.
//!
//! # The workaround
//!
//! Never write `SYST_CVR` after the very first activation (confirmed
//! empirically to behave correctly: QEMU's ptimer has no stale state yet).
//! [`reset()`](SchedulerTimer::reset) becomes a no-op, and
//! [`start()`](SchedulerTimer::start) only ever updates `SYST_RVR` (the
//! limit that takes effect on the *next* natural wrap) and ensures the
//! counter is enabled -- it never disables or re-enables it, so the counter
//! only ever reaches 0 by genuinely decrementing, the one path QEMU's
//! `ptimer` policy handles correctly.
//!
//! The trade-off: immediately after a call to `start()`, the in-progress
//! countdown (inherited from whatever it naturally was when the previous
//! process's timeslice ended) runs to completion using leftover time rather
//! than the newly requested duration; the new duration only takes full
//! effect from the following wrap onward. For a round-robin fairness
//! mechanism (not a hard real-time guarantee), that's a minor, self-
//! correcting inaccuracy -- far preferable to the alternative.

use core::cell::Cell;
use core::num::NonZeroU32;

use cortexm::systick::SysTick;
use kernel::platform::scheduler_timer::SchedulerTimer;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{ReadWrite, register_bitfields};

#[repr(C)]
struct SystickRegisters {
    syst_csr: ReadWrite<u32, ControlAndStatus::Register>,
    syst_rvr: ReadWrite<u32, ReloadValue::Register>,
}

register_bitfields![u32,
    ControlAndStatus [
        /// Clock source is (0) External Clock or (1) Processor Clock.
        CLKSOURCE 2,
        /// Set to 1 to enable SysTick exception request.
        TICKINT 1,
        /// Enable the counter (1 == Enabled).
        ENABLE 0
    ],
    ReloadValue [
        /// Value loaded to `syst_cvr` when counter is enabled and reaches 0.
        RELOAD OFFSET(0) NUMBITS(24)
    ]
];

const SYSTICK_BASE: StaticRef<SystickRegisters> =
    unsafe { StaticRef::new(0xE000E010 as *const SystickRegisters) };

/// Wraps a [`SysTick`] to work around a QEMU SysTick emulation bug -- see
/// the module documentation.
pub struct QemuMps2SysTick {
    inner: SysTick,
    /// `start()`'s own copy of the configured frequency, in Hertz, used to
    /// convert the requested timeslice into a `SYST_RVR` reload value.
    /// Independent of (but always set to the same value as) whatever `inner`
    /// was calibrated with, since `inner`'s own copy is private -- `inner`
    /// still needs its real calibration for the `get_remaining_us()`,
    /// `arm()`, and `disarm()` calls this forwards to it unchanged.
    hertz: Cell<u32>,
}

impl QemuMps2SysTick {
    /// `inner` should already be calibrated (e.g. via
    /// [`SysTick::new_with_calibration`]) with the same `clock_speed` passed
    /// here.
    pub fn new(inner: SysTick, clock_speed: u32) -> Self {
        Self {
            inner,
            hertz: Cell::new(clock_speed),
        }
    }
}

impl SchedulerTimer for QemuMps2SysTick {
    fn start(&self, us: NonZeroU32) {
        // Same conversion as the wrapped driver's own `start()`; see its
        // comment for why this goes through u64.
        let reload = {
            let us = us.get() as u64;
            let hertz = self.hertz.get() as u64;
            hertz * us / 1_000_000
        };

        // Update the limit for the *next* natural wrap -- never touch
        // `SYST_CVR`, which is exactly what triggers the QEMU bug (see the
        // module documentation). `n.b.: 4.4.5 'hints and tips' suggests
        // setting reload before value`, as the wrapped driver's own
        // comment notes, but there is no "value" (`SYST_CVR`) write here to
        // order against.
        SYSTICK_BASE
            .syst_rvr
            .write(ReloadValue::RELOAD.val(reload as u32));

        // Ensure the counter is enabled and generating interrupts. On every
        // call after the first, this is a no-op rewrite of bits already set
        // -- QEMU's device (like real hardware) only acts on an actual
        // transition of ENABLE, so leaving it continuously enabled here is
        // what keeps the counter free-running instead of being stopped and
        // restarted from a written (rather than decremented) 0.
        SYSTICK_BASE.syst_csr.write(
            ControlAndStatus::TICKINT::SET
                + ControlAndStatus::ENABLE::SET
                + ControlAndStatus::CLKSOURCE::SET,
        );
    }

    fn reset(&self) {
        // Deliberately not a passthrough to `inner.reset()`: that stops the
        // counter and zeroes `SYST_CVR`/`SYST_RVR`, which is the write that
        // wedges QEMU's ptimer (see the module documentation). Leaving the
        // counter running is harmless -- `do_process` always calls
        // `start()` again immediately after `reset()` when a real
        // timeslice is wanted at all (see `kernel/src/kernel.rs`), so there
        // is nothing useful to reset to in the meantime.
    }

    fn arm(&self) {
        self.inner.arm()
    }

    fn disarm(&self) {
        self.inner.disarm()
    }

    fn get_remaining_us(&self) -> Option<NonZeroU32> {
        self.inner.get_remaining_us()
    }
}
