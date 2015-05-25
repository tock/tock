pub trait Timer {
    fn now(&self) -> u32;
    fn set_alarm(&mut self, u32);
    fn disable_alarm(&mut self);
}

pub trait TimerReceiver {
    fn alarm_fired(&mut self) {
    }
}

