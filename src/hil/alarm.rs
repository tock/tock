pub trait Request {
    fn fired(&mut self);
}

pub trait Alarm {
    fn now(&self) -> u32;
    fn set_alarm(&mut self, u32, &'static mut Request);
    fn disable_alarm(&mut self);
}


