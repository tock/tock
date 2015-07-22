pub trait Request {
    fn fired(&'static mut self);
}

pub trait Alarm {
    fn now(&self) -> u32;
    fn set_alarm(&'static mut self, u32, &'static mut Request);
    fn disable_alarm(&'static mut self);
}


