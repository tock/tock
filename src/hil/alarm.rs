pub trait Request {
    fn fired(&'static mut self);
}

pub trait Alarm {
    fn now(&self) -> u32;
    fn set_alarm(&'static mut self, when: u32, request: &'static mut Request);
    fn disable_alarm(&'static mut self);
    fn get_alarm(&'static mut self) -> u32;
}


