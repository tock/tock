#![feature(core)]
#![allow(unused_variables,dead_code)]

extern crate core;

use core::prelude::*;

mod timer;
mod alarm;

pub static mut TIMER: Option<&'static mut timer::Timer> = None;
pub static mut ALARM: Option<&'static mut alarm::Alarm> = None;
pub static mut TESTALARM:TestAlarm = TestAlarm{val: 0, request: None};
pub static mut TIMEROPT: Option<timer::TimerMux> = None;
pub static mut TIMERCBA: TestRequest = TestRequest {id: 10};
pub static mut TIMERCBB: TestRequest = TestRequest {id: 20};
pub static mut TREQUESTA: Option<timer::TimerRequest> = None; 
pub static mut TREQUESTB: Option<timer::TimerRequest> = None; 
pub struct TestAlarm {
    pub val: u32,
    pub request: Option<&'static mut alarm::Request>
}

impl TestAlarm {
  fn new() -> TestAlarm {
    TestAlarm {
     val: 0,
     request: None
    }
  }

  fn pop_fire(&'static mut self) -> bool {
    if self.request.is_none() {
        print!("ERROR: TestAlarm: Tried to fire when there is no outstanding alarm.\n");
        false
    } else {
       print!("TestAlarm: firing request at {}\n", self.val);
       let ropt: Option<&'static mut alarm::Request> = self.request.take();
       let request: &'static mut alarm::Request = ropt.unwrap();
       request.fired();
       true
    }

  }
}

impl alarm::Alarm for TestAlarm {
  fn now (&self) -> u32 {
    self.val
  }

  fn set_alarm(&'static mut self, when: u32, request: &'static mut alarm::Request) {
    print!("Setting alarm to {}\n", when);
    self.val = when;
    self.request = Some(request);
  }

  fn disable_alarm(&'static mut self) {

  }

  fn get_alarm(&'static mut self) -> u32 {
    self.val
  }
}

pub struct TestRequest {
  id: u32
}

impl timer::TimerCB for TestRequest {
  fn fired(&'static mut self,
           request: &'static mut timer::TimerRequest,
           now: u32) {
    print!("Timer {} fired at {}.\n", self.id, now);
  }
}

pub fn main() {
    unsafe {
        ALARM = Some(&mut TESTALARM);
        let alarm:&'static mut TestAlarm = &mut TESTALARM;
        TIMEROPT = Some(timer::TimerMux::new(alarm));
        TREQUESTA = Some(timer::TimerRequest::new(&mut TIMERCBA));
        TREQUESTB = Some(timer::TimerRequest::new(&mut TIMERCBB));
        let timer:&'static mut timer::Timer = TIMEROPT.as_mut().unwrap();
        let requesta:&'static mut timer::TimerRequest = TREQUESTA.as_mut().unwrap();
        let requestb:&'static mut timer::TimerRequest = TREQUESTB.as_mut().unwrap();
        timer.repeat(1001, requesta);
        timer.repeat(2001, requestb);
        while alarm.pop_fire() {}
    }

}
