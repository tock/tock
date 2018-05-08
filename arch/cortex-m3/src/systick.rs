//! ARM Cortex-M3 SysTick peripheral.

use kernel;
use kernel::common::VolatileCell;

struct Registers {
    control: VolatileCell<u32>,
    reload: VolatileCell<u32>,
    value: VolatileCell<u32>,
    calibration: VolatileCell<u32>,
}

/// The ARM Cortex-M3 SysTick peripheral
///
/// Documented in the Cortex-M3 Devices Generic User Guide, Chapter 4.4
pub struct SysTick {
    regs: &'static Registers,
    hertz: u32,
}

const BASE_ADDR: *const Registers = 0xE000E010 as *const Registers;

impl SysTick {
    /// Initialize the `SysTick` with default values
    ///
    /// Use this constructor if the core implementation has a pre-calibration
    /// value in hardware.
    pub unsafe fn new() -> SysTick {
        SysTick {
            regs: &*BASE_ADDR,
            hertz: 0,
        }
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

    // Return the tic frequency in hertz. If the calibration value is set in
    // hardware, use `self.hertz`, which is set in the `new_with_calibration`
    // constructor.
    fn hertz(&self) -> u32 {
        let tenms = self.regs.calibration.get() & 0xffffff;
        if tenms == 0 {
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
            // if `us` is a power of 10 it can be simplified with the 1_000_000 divisor instead.
            let us = us as u64;
            let hertz = self.hertz() as u64;

            hertz * us / 1_000_000
        };

        self.regs.value.set(0);
        self.regs.reload.set(reload as u32);
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

        let value = self.regs.value.get() & 0xffffff;
        value > tics
    }

    fn overflowed(&self) -> bool {
        self.regs.control.get() & 1 << 16 != 0
    }

    fn reset(&self) {
        self.regs.control.set(0);
        self.regs.reload.set(0);
        self.regs.value.set(0);
    }

    fn enable(&self, with_interrupt: bool) {
        if with_interrupt {
            self.regs.control.set(0b111);
        } else {
            self.regs.control.set(0b101);
        }
    }
}
