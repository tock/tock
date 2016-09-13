use kernel;
use kernel::common::volatile_cell::VolatileCell;

pub struct SysTick {
    control: VolatileCell<u32>,
    reload: VolatileCell<u32>,
    value: VolatileCell<u32>,
    calibration: VolatileCell<u32>,
}

#[no_mangle]
pub static mut OVERFLOW_FIRED: VolatileCell<usize> = VolatileCell::new(0);

const BASE_ADDR: *const SysTick = 0xE000E010 as *const SysTick;

impl SysTick {
    pub unsafe fn new() -> &'static SysTick {
        &*BASE_ADDR
    }
}

impl kernel::SysTick for SysTick {
    fn set_timer(&self, us: u32) {
        let tenms = self.calibration.get() & 0xffffff;
        let reload = tenms * us / 10000;

        self.value.set(0);
        self.reload.set(reload);
    }

    fn value(&self) -> u32 {
        let tenms = self.calibration.get() & 0xffffff;
        let value = self.value.get() & 0xffffff;

        value * 10000 / tenms
    }

    fn overflowed(&self) -> bool {
        self.control.get() & 1 << 16 != 0
    }

    fn reset(&self) {
        self.control.set(0);
        self.reload.set(0);
        self.value.set(0);
        unsafe {
            OVERFLOW_FIRED.set(0);
        }
    }

    fn enable(&self, with_interrupt: bool) {
        if with_interrupt {
            self.control.set(0b111);
        } else {
            self.control.set(0b101);
        }
    }

    fn overflow_fired() -> bool {
        unsafe { OVERFLOW_FIRED.get() == 1 }
    }
}
