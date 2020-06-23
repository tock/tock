//! Test that a Timer implementation is working by trying a few edge
//! cases on the interval, including intervals of 1 and 0. Depends
//! on a working UART and debug! macro. Tries repeating as well as
//! one-shot Timers.
//!
//! Author: Philip Levis <plevis@google.com>
//! Last Modified: 6/22/2020
use core::cell::Cell;
use kernel::debug;
use kernel::hil::time::{Timer, TimerClient, Ticks};

pub struct TestRandomTimer<'a, T: 'a> {
    timer: &'a T,
    interval: Cell<u32>,
    counter: Cell<u32>,
    iv: Cell<u32>,
    _id: char,
}

impl<'a, T: Timer<'a>> TestRandomTimer<'a, T> {
    pub fn new(timer: &'a T, value: usize, ch: char) -> TestRandomTimer<'a, T> {
        TestRandomTimer {
            timer: timer,
            interval: Cell::new(0),
            counter: Cell::new(0),
            iv: Cell::new(value as u32),
            _id: ch,
        }
    }

    pub fn run(&self) {
        debug!("Starting random timer test Test{}.", self._id);
        self.set_next_timer();
    }

    fn set_next_timer(&self) {
        let iv = self.iv.get();
        self.iv.set(iv + 1);

        let counter = self.counter.get();
        if counter == 0 {
            let mut us: u32 = ((iv * 745939) % 115843) as u32;
            if us % 11 == 0 {
                // Try delays of zero in 1 of 11 cases
                us = 0;
            }
            let new_interval = T::ticks_from_us(us);
            self.interval.set(new_interval.into_u32());
            if us % 7 == 0 {
                let new_counter = 2 + self.interval.get() * 23 % 13;
                self.counter.set(new_counter);
                //debug!("Timer{} repeating with interval {}", self._id, self.interval.get());
                self.timer.repeating(new_interval);
            } else {
                //debug!("Timer{} oneshot with interval {}", self._id, self.interval.get());
                self.timer.oneshot(new_interval);
            }
        } else {
            self.counter.set(counter - 1);
        }
    }
}

impl<'a, T: Timer<'a>> TimerClient for TestRandomTimer<'a, T> {
    fn timer(&self) {
        debug!("Timer{} fired with interval {}, count {},  fired at {}.", self._id, self.interval.get(), self.counter.get(), self.timer.now().into_u32());
        self.set_next_timer();
    }
}
