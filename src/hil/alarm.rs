/**
 * alarm.rs - Trait for a hardware timer based on a counter. Assumes 32 bits.
 *
 * Author: Amit Levy <levya@cs.stanford.edu>
 * Date: 7/15/15
 */
pub trait AlarmClient {
    fn fired(&mut self);
}

pub trait Alarm {
    fn now(&self) -> u32;
    fn set_alarm(&mut self, when: u32);
    fn disable_alarm(&mut self);
    fn get_alarm(&mut self) -> u32;
}


