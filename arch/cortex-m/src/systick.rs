//! ARM Cortex-M SysTick peripheral.

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, FieldValue};
use kernel::common::StaticRef;

#[repr(C)]
struct SystickRegisters {
    syst_csr: ReadWrite<u32, ControlAndStatus::Register>,
    syst_rvr: ReadWrite<u32, ReloadValue::Register>,
    syst_cvr: ReadWrite<u32, CurrentValue::Register>,
    syst_calib: ReadOnly<u32, CalibrationValue::Register>,
}

register_bitfields![u32,
    ControlAndStatus [
        /// Returns 1 if timer counted to 0 since last time this was read.
        COUNTFLAG 16,

        /// Clock source is (0) External Clock or (1) Processor Clock.
        CLKSOURCE 2,

        /// Set to 1 to enable SysTick exception request.
        TICKINT 1,

        /// Enable the counter (1 == Enabled).
        ENABLE 0
    ],

    ReloadValue [
        /// Value loaded to `syst_csr` when counter is enabled and reaches 0.
        RELOAD          OFFSET(0)  NUMBITS(24)
    ],

    CurrentValue [
        /// Reads current value. Write of any value sets to 0.
        CURRENT         OFFSET(0)  NUMBITS(24)
    ],

    CalibrationValue [
        /// 0 if device provides reference clock to processor.
        NOREF           OFFSET(31) NUMBITS(1),

        /// 0 if TENMS value is exact, 1 if inexact or not given.
        SKEW            OFFSET(30) NUMBITS(1),

        /// Reload value for 10ms ticks, or 0 if no calibration.
        TENMS           OFFSET(0)  NUMBITS(24)
    ]
];

/// The ARM Cortex-M SysTick peripheral
///
/// Documented in the Cortex-MX Devices Generic User Guide, Chapter 4.4
pub struct SysTick {
    hertz: u32,
    external_clock: bool
}

const BASE_ADDR: *const SystickRegisters = 0xE000E010 as *const SystickRegisters;
const SYSTICK_BASE: StaticRef<SystickRegisters> =
    unsafe { StaticRef::new(BASE_ADDR as *const SystickRegisters) };

impl SysTick {
    /// Initialize the `SysTick` with default values
    ///
    /// Use this constructor if the core implementation has a pre-calibration
    /// value in hardware.
    pub unsafe fn new() -> SysTick {
        SysTick { hertz: 0, external_clock: false }
    }

    /// Initialize the `SysTick` with an explicit clock speed
    ///
    /// Use this constructor if the core implementation does not have a
    /// pre-calibration value.
    ///
    ///   * `clock_speed` - the frequency of SysTick tics in Hertz. For example,
    ///   if the SysTick is driven by the CPU clock, it is simply the CPU speed.
    pub unsafe fn new_with_calibration(clock_speed: u32) -> SysTick {
        let mut res = SysTick::new();
        res.hertz = clock_speed;
        res
    }

    pub unsafe fn new_with_calibration_and_external_clock(clock_speed: u32) -> SysTick {
        let mut res = SysTick::new();
        res.hertz = clock_speed;
        res.external_clock = true;
        res
    }

    // Return the tic frequency in hertz. If the calibration value is set in
    // hardware, use `self.hertz`, which is set in the `new_with_calibration`
    // constructor. However, if there is value configured by the user, choose
    //`self.hertz` instead.
    fn hertz(&self) -> u32 {
        let tenms = SYSTICK_BASE.syst_calib.read(CalibrationValue::TENMS);
        if tenms == 0 || self.hertz != 0 {
            self.hertz
        } else {
            // The `tenms` register is the reload value for 10ms, so
            // Hertz = number of tics in 1 second = tenms * 100
            tenms * 100
        }
    }
}

impl kernel::SysTick for SysTick {
    fn set_timer(&self, us: u32) {
        let reload = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us as u64;
            let hertz = self.hertz() as u64;

            hertz * us / 1_000_000
        };

        // n.b.: 4.4.5 'hints and tips' suggests setting reload before value
        SYSTICK_BASE
            .syst_rvr
            .write(ReloadValue::RELOAD.val(reload as u32));
        SYSTICK_BASE.syst_cvr.set(0);
    }

    fn greater_than(&self, us: u32) -> bool {
        let tics = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us as u64;
            let hertz = self.hertz() as u64;

            (hertz * us / 1_000_000) as u32
        };

        let value = SYSTICK_BASE.syst_cvr.read(CurrentValue::CURRENT);
        value > tics
    }

    fn overflowed(&self) -> bool {
        SYSTICK_BASE.syst_csr.is_set(ControlAndStatus::COUNTFLAG)
    }

    fn reset(&self) {
        SYSTICK_BASE.syst_csr.set(0);
        SYSTICK_BASE.syst_rvr.set(0);
        SYSTICK_BASE.syst_cvr.set(0);
    }

    fn enable(&self, with_interrupt: bool) {
        let clock_source: FieldValue<u32, self::ControlAndStatus::Register> =  if self.external_clock {
            ControlAndStatus::CLKSOURCE::CLEAR
        } else {
            ControlAndStatus::CLKSOURCE::SET
        };
        
        if with_interrupt {
            SYSTICK_BASE.syst_csr.write(
                ControlAndStatus::ENABLE::SET
                    + ControlAndStatus::TICKINT::SET
                    + clock_source,
            );
        } else {
            SYSTICK_BASE
                .syst_csr
                .write(ControlAndStatus::ENABLE::SET + clock_source);
        }
    }
}
