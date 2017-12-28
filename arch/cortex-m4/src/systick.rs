//! ARM SysTick peripheral.

use core::cmp;
use kernel;
use kernel::common::VolatileCell;

struct Registers {
    control: VolatileCell<u32>,
    reload: VolatileCell<u32>,
    value: VolatileCell<u32>,
    calibration: VolatileCell<u32>,
}

pub struct SysTick {
    regs: &'static Registers,
    hertz: u32,
}

#[no_mangle]
pub static mut OVERFLOW_FIRED: VolatileCell<usize> = VolatileCell::new(0);

const BASE_ADDR: *const Registers = 0xE000E010 as *const Registers;

impl SysTick {
    pub unsafe fn new() -> SysTick {
        SysTick {
            regs: &*BASE_ADDR,
            hertz: 0,
        }
    }

    pub unsafe fn new_with_calibration(clock_speed: u32) -> SysTick {
        let mut res = SysTick::new();
        res.hertz = clock_speed;
        res
    }

    fn hertz(&self) -> u32 {
        let tenms = self.regs.calibration.get() & 0xffffff;
        if tenms == 0 {
            self.hertz
        } else {
            tenms * 100
        }
    }
}

impl kernel::SysTick for SysTick {
    fn set_timer(&self, us: u32) {
        let us = cmp::min(us, 1_000_000);
        let hertz = self.hertz();
        let reload = hertz / (1_000_000 / us);

        self.regs.value.set(0);
        self.regs.reload.set(reload);
    }

    fn value(&self) -> u32 {
        let hertz = self.hertz();
        let value = self.regs.value.get() & 0xffffff;

        value / (hertz / 1_000_000)
    }

    fn overflowed(&self) -> bool {
        self.regs.control.get() & 1 << 16 != 0
    }

    fn reset(&self) {
        self.regs.control.set(0);
        self.regs.reload.set(0);
        self.regs.value.set(0);
        unsafe {
            OVERFLOW_FIRED.set(0);
        }
    }

    fn enable(&self, with_interrupt: bool) {
        if with_interrupt {
            self.regs.control.set(0b111);
        } else {
            self.regs.control.set(0b101);
        }
    }

    fn overflow_fired() -> bool {
        unsafe { OVERFLOW_FIRED.get() == 1 }
    }
}
