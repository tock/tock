/* hil::timer -- Traits and structures for software timers.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/16/15
 */
#[allow(dead_code)]

use core::prelude::*;
use alarm;

pub trait Request {
  fn read_done(&'static mut self, now: u32);
}

pub trait Timer {
  fn now(&self) -> u32; 
  fn cancel(&mut self, &'static mut RequestInternal);
  fn oneshot(&mut self, interval: u32, &'static mut RequestInternal);
  fn repeat(&mut self, interval: u32, &'static mut RequestInternal);
  
}

#[allow(dead_code)]
pub struct RequestInternal {
  pub next: Option<&'static mut RequestInternal>,
  pub is_active: bool,
  pub is_repeat: bool,
  pub when: u32,
  pub interval: u32,
  pub callback: Option<&'static mut Request>
}

#[allow(dead_code)]
impl RequestInternal {
  pub fn new(request: &'static mut Request) -> RequestInternal {
    RequestInternal {
      next:      None,
      is_active: false,
      is_repeat: false,
      when:      0,
      interval:  0,
      callback:  Some(request)
    }
  }
}

#[allow(dead_code)]
pub struct TimerMux {
  request: Option<&'static mut RequestInternal>,
  last: Option<&'static mut RequestInternal>,
  internal: Option<&'static mut alarm::Alarm>
}

#[allow(dead_code,unused_variables)]
impl TimerMux {
  pub fn new(internal: &'static mut alarm::Alarm) -> TimerMux {
    TimerMux {
      request: None,
      last: None,
      internal: Some(internal)
    }
  }
 
  fn start_request(&'static mut self) {
    if self.request.is_none() {return;}

    let aopt: Option<&'static mut alarm::Alarm> = self.internal.take();
    let alarm: &'static mut alarm::Alarm = aopt.unwrap();
    let ropt = self.request.take();
    let request: &'static mut RequestInternal = ropt.unwrap();
    let when = request.when;

    alarm.set_alarm(when, self as &'static mut alarm::Request);
    self.internal = Some(alarm);
    self.request = Some(request);

  }


  fn remove(&mut self, request: &'static mut RequestInternal) {
    // No idea
  }

  fn insert(&mut self, request: &'static mut RequestInternal) {
    // No idea
  }
}

#[allow(dead_code,unused_variables)]
impl alarm::Request for TimerMux {
  fn fired(&mut self) {
    if self.request.is_none() {return;}

    let ropt = self.request.take();
    let request: &'static mut RequestInternal = ropt.unwrap();

    if request.is_repeat {
      let t = self as &mut Timer;
    }
  }

}

#[allow(dead_code)]
impl Timer for TimerMux {
  fn now(&self) -> u32 {
    0 as u32
  }
  fn cancel(&mut self, request: &'static mut RequestInternal) {
    if !request.is_active {return;}

    request.is_active = false;
    self.remove(request);
  }

  fn oneshot(&mut self, interval: u32, request: &'static mut RequestInternal) {
    request.interval = interval;
    request.is_active = true;
    request.when = self.now() + interval;
    request.is_repeat = false;
    self.insert(request);
  }

  fn repeat(&mut self, interval: u32, request: &'static mut RequestInternal) {
    request.interval = interval;
    request.is_active = true;
    request.when = self.now() + interval;
    request.is_repeat = true;
    self.insert(request);
  }
}
