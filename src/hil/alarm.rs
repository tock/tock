/**
 * alarm.rs - Trait for a hardware timer based on a counter. Assumes 32 bits.
 *
 * Author: Amit Levy <alevy@stanford.edu>
 * Date: 7/15/15
 */ 
pub trait Request {
    fn fired(&'static mut self);
}

pub trait Alarm {
    fn now(&self) -> u32;
    fn set_alarm(&'static mut self, when: u32, request: &'static mut Request);
    fn disable_alarm(&'static mut self);
    fn get_alarm(&'static mut self) -> u32;
}


