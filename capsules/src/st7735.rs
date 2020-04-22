use crate::driver;
use core::cell::Cell;
use kernel::hil::time::{self, Alarm, Frequency};

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::St7735 as usize;

// The states the program can be in.
#[derive(Copy, Clone, PartialEq)]
enum LCDStatus {
    Idle,
}

pub struct ST7735<'a, A: Alarm<'a>> {
    alarm: &'a A,
    lcd_status: Cell<LCDStatus>,
}

impl<'a, A: Alarm<'a>> ST7735<'a, A> {
    pub fn new(alarm: &'a A) -> ST7735<'a, A> {
        ST7735 {
            alarm: alarm,
            lcd_status: Cell::new(LCDStatus::Idle),
        }
    }

    /// set_delay sets an alarm and saved the next state after that.
    ///
    /// As argument, there are:
    ///  - the duration of the alarm:
    ///      - 10 means 100 ms
    ///      - 100 means 10 ms
    ///      - 500 means 2 ms
    ///  - the status of the program after the alarm fires
    ///
    /// Example:
    ///  self.set_delay(10, LCDStatus::Idle);
    fn set_delay(&self, timer: u32, next_status: LCDStatus) {
        self.lcd_status.set(next_status);
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency() / timer),
        )
    }
}
