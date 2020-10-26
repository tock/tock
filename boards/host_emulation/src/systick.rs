use kernel;
use std::cell::Cell;
use std::time::{SystemTime, SystemTimeError};

pub struct SysTick {
    start_time: Cell<SystemTime>,
    set_duration_us: Cell<u32>,
    enabled: Cell<bool>,
    interrupt_enabled: Cell<bool>,
}

impl SysTick {
    pub fn new() -> SysTick {
        SysTick {
            start_time: Cell::new(SystemTime::now()),
            set_duration_us: Cell::new(0),
            enabled: Cell::new(true),
            interrupt_enabled: Cell::new(false),
        }
    }

    fn elapsed_us(&self) -> Result<u128, SystemTimeError> {
        let now = SystemTime::now();
        let elapsed_us = match now.duration_since(self.start_time.get()) {
            Ok(time) => time,
            Err(e) => return Err(e),
        };
        Ok(elapsed_us.as_micros())
    }

    pub fn get_systick_left(&self) -> Option<u128> {
        if !self.enabled.get() {
            return None;
        }
        let elapsed_us = match self.elapsed_us() {
            Ok(time) => time,
            Err(_) => 0,
        };
        let left = self.set_duration_us.get() as u128 - elapsed_us;
        return Some(left);
    }
}

impl kernel::SysTick for SysTick {
    fn set_timer(&self, us: u32) {
        self.start_time.set(SystemTime::now());
        self.set_duration_us.set(us);
    }

    fn greater_than(&self, us: u32) -> bool {
        if !self.enabled.get() {
            return false;
        }
        let elapsed_us = match self.elapsed_us() {
            Ok(time) => time,
            Err(_) => return false,
        };
        let remaining_us = if self.set_duration_us.get() as u128 > elapsed_us {
            self.set_duration_us.get() as u128 - elapsed_us
        } else {
            0
        };
        return remaining_us >= us as u128;
    }

    fn overflowed(&self) -> bool {
        if !self.enabled.get() {
            return true;
        }

        let elapsed_us = match self.elapsed_us() {
            Ok(time) => time,
            Err(_) => return true,
        };
        return elapsed_us > self.set_duration_us.get() as u128;
    }

    fn reset(&self) {
        self.enabled.set(false);
        self.set_timer(0);
        self.interrupt_enabled.set(false);
    }

    fn enable(&self, with_interrupt: bool) {
        self.enabled.set(true);
        if with_interrupt {
            self.interrupt_enabled.set(true);
        }
    }
}
