/* hil::timer -- Traits and structures for software timers.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/16/15
 */

use core::prelude::*;
use alarm;

pub trait Request {
  fn read_done(&'static mut self, now: u32);
}

pub trait Timer {
  fn now(&self) -> u32; 
  fn cancel(&mut self, &'static mut Request);
  fn oneshot(&mut self, interval: u32, &'static mut Request);
  fn repeat(&mut self, interval: u32, &'static mut Request);
}

pub struct RequestInternal {
  pub next: Option<&'static mut RequestInternal>,
  pub is_active: bool,
  pub is_repeat: bool,
  pub when: u32,
  pub interval: u32,
  pub callback: Option<&'static mut Request>
}

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

/* common::timer -- Software timers (Timer trait), sitting on top of a
 * single physical hardware timer (Alarm trait).
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/16/15
 */

pub struct TimerMux {
  request: Option<&'static mut RequestInternal>,
  last: Option<&'static mut RequestInternal>,
  internal: Option<&'static mut alarm::Alarm>
}

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

    let mut aopt: Option<&'static mut alarm::Alarm> = self.internal.take();
    let alarm: &'static mut alarm::Alarm = aopt.unwrap();
    let mut ropt = self.request.take();
    let request: &'static mut RequestInternal = ropt.unwrap();
    let when = request.when;

    alarm.set_alarm(when, self as &'static mut alarm::Request);
    self.internal = Some(alarm);
    self.request = Some(request);

  }
}

impl alarm::Request for TimerMux {
  fn fired(&mut self) {
    if self.request.is_none() {return;}

    let mut ropt = self.request.take();
    let request: &'static mut RequestInternal = ropt.unwrap();

    if request.is_repeat {
      let t = self as &'static mut Timer;
    }
  }
}
