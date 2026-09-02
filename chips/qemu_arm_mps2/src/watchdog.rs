// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! ARM CMSDK APB Watchdog (SP805-style), as found on the MPS2 AN385/AN386
//! FPGA images.
//!
//! The interrupt line is wired to NMI, not a normal NVIC line
//! (`hw/arm/mps2.c`), so it is not dispatched through
//! [`kernel::platform::chip::InterruptService`] the way the other
//! peripherals in this crate are — this driver only ever pokes registers,
//! it never installs an NMI handler of its own.
//!
//! The first countdown-to-zero with `INTEN` set raises the (non-maskable)
//! interrupt; if nobody kicks the watchdog (`WDOGINTCLR`) before the
//! *second* countdown-to-zero with `RESEN` also set, QEMU performs an actual
//! system reset (`watchdog_perform_action()`), independent of whatever the
//! guest's NMI handler does.
//!
//! This board's vector table maps NMI to `unhandled_interrupt` (a panic), so
//! in practice that panic — not the hardware reset — is what happens on a
//! first missed kick; the reset only fires if something prevents that panic
//! from halting execution first.
//!
//! Normally, the panic handler uses semihosting to terminate the machine, but
//! if the terminate is ignored/continued, then the panic handler loops forever
//! without kicking the watchdog, and the second countdown-to-zero (and the
//! reset it triggers) will follow shortly after.

use kernel::platform::watchdog::WatchDog;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

use crate::SYSCLK_FRQ;

pub const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x4000_8000 as *const WatchdogRegisters) };

/// Unlock value documented in the CMSDK Watchdog TRM; writing anything else
/// to `WDOGLOCK` re-locks the device.
const WDOG_UNLOCK_VALUE: u32 = 0x1ACC_E551;

/// How long before the watchdog fires in ticks (2 seconds wall time).
///
/// `WDOGLOAD` holds a tick count. `SYSCLK_FRQ` is ticks/sec.
const WDOG_RELOAD_TICKS: u32 = SYSCLK_FRQ * 2;

register_structs! {
    pub WatchdogRegisters {
        (0x000 => wdogload: ReadWrite<u32>),
        (0x004 => wdogvalue: ReadOnly<u32>),
        (0x008 => wdogcontrol: ReadWrite<u32, WDOGCONTROL::Register>),
        (0x00c => wdogintclr: ReadWrite<u32>),
        (0x010 => wdogris: ReadOnly<u32>),
        (0x014 => wdogmis: ReadOnly<u32>),
        (0x018 => _reserved0),
        (0xc00 => wdoglock: ReadWrite<u32>),
        (0xc04 => @END),
    }
}

register_bitfields![u32,
    WDOGCONTROL [
        IntEn OFFSET(0) NUMBITS(1) [],
        ResEn OFFSET(1) NUMBITS(1) [],
    ],
];

pub struct Watchdog {
    registers: StaticRef<WatchdogRegisters>,
}

impl Watchdog {
    pub const fn new(registers: StaticRef<WatchdogRegisters>) -> Self {
        Watchdog { registers }
    }

    fn unlock(&self) {
        self.registers.wdoglock.set(WDOG_UNLOCK_VALUE);
    }
}

impl WatchDog for Watchdog {
    fn setup(&self) {
        self.unlock();
        // Writing WDOGLOAD also reloads the live counter.
        self.registers.wdogload.set(WDOG_RELOAD_TICKS);
        self.registers
            .wdogcontrol
            .write(WDOGCONTROL::IntEn::SET + WDOGCONTROL::ResEn::SET);
    }

    fn tickle(&self) {
        // Any value clears the pending interrupt and reloads from WDOGLOAD.
        self.registers.wdogintclr.set(1);
        // The HIL requires tickle() to resume a suspended watchdog. Clearing
        // the interrupt does not restart a stopped counter, so set IntEn here.
        self.registers.wdogcontrol.modify(WDOGCONTROL::IntEn::SET);
    }

    fn suspend(&self) {
        self.registers.wdogcontrol.modify(WDOGCONTROL::IntEn::CLEAR);
    }

    fn resume(&self) {
        // Setting IntEn high after being disabled reloads from WDOGLOAD,
        // so this resumes with a full margin rather than wherever the
        // counter happened to be left -- an inherent property of this
        // hardware (a real SP805 characteristic, not a QEMU quirk).
        //
        // As a result, every idle sleep/wake cycle incidentally re-arms the
        // watchdog with a fresh margin, same as tickle() does. Harmless for
        // catching real hangs, but this also has to be modified to manually
        // provoke an expiry when testing.
        self.registers.wdogcontrol.modify(WDOGCONTROL::IntEn::SET);
    }
}
