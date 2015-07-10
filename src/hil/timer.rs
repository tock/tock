use core::prelude::*;

pub struct Request {
    is_periodic: bool,
    active: bool,
    interval: u32,
    when: u32,
    next: Option<&'static mut Request> 
}

pub trait Timer {
    fn now(&self) -> u32; 
    fn cancel(&mut self, &'static mut Request);
    fn oneshot(&mut self, u32, &'static mut Request);
    fn periodic(&mut self, u32, &'static mut Request);
}


