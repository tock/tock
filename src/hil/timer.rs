/* hil::timer -- Traits and structures for software timers.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/16/15
 */
#[allow(dead_code)]

use core::prelude::*;
use alarm;

pub trait Request {
  fn fired(&'static mut self, now: u32);
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
  internal: Option<&'static mut alarm::Alarm>
}

#[allow(dead_code,unused_variables)]
impl TimerMux {
  pub fn new(internal: &'static mut alarm::Alarm) -> TimerMux {
    TimerMux {
      request: None,
      internal: Some(internal)
    }
  }
 
  fn start_request(&mut self) {
    if self.request.is_none() {return;}

    let aopt: Option<&'static mut alarm::Alarm> = self.internal.take();
    let alarm: &'static mut alarm::Alarm = aopt.unwrap();
    let ropt = self.request.take();
    let request: &'static mut RequestInternal = ropt.unwrap();
    let when = request.when;

    //let mut me = self as &'static mut alarm::Request;
    //alarm.set_alarm(when, me);// as &mut alarm::Request);

    self.internal = Some(alarm);
    self.request = Some(request);

  }

  // Returns whether hardware clock has to be recalculated (inserted
  // timer is now first timer)
  fn insert(&mut self, request: &'static mut RequestInternal) -> bool {
    if request.next.is_some() { // Already on a list, this is an error!
      false
    }
    else if self.request.is_none() {
      self.request = Some(request);
      true
    } else {
      let mut first = true; 
      let mut done = false;
      let mut copt = &mut self.request;
      while !done {
        let mycopt = copt.take();
        let mut curr = mycopt.unwrap();
        // 'request' is earlier than current element, insert here by making
        // the current Option point to 'request' and have 'request''s next
        // Option point to the element the current Option held.
        if request.when < curr.when {
           request.next = Some(curr);
           *copt = Some(request);
           done = true;
        } else {
           // We need to insert later. We therefore are not inserting in
           // the first element. There are two cases:
           //   1. last element and we need to insert at the end, or
           //   2. we need to traverse the next hop.
           first = false;
           if curr.next.is_none() {
             // Reached end of list, insert here
             curr.next = Some(request);
             *copt = Some(curr);
             done = true;
           } else {
             let mut nopt = &mut curr.next;
             let mynopt = nopt.take();
             *copt = Some(curr);
             copt = nopt;
           }

        }
      }
      first
    }
  }

  // Returns whether hardware clock has to be recalculated (removed first
  // timer)
  fn remove(&mut self, request: &'static mut RequestInternal) -> bool {
    if self.request.is_none() {return false;}
    
    let mut done = false;
    let mut copt = &mut self.request;
    let mut first = true;
    while !done {
      let mycopt = copt.take();
      let mut curr = mycopt.unwrap();
      if false {
        *copt = curr.next.take();
        done = true;
      } else if curr.next.is_none() {
        *copt = Some(curr);
        first = false;
        done = true;
      } else {
        let mut nopt = &mut curr.next;
        *copt = Some(curr);
        copt = nopt;
        first = false;
      }
    }
    first
  }
}

#[allow(dead_code,unused_variables)]
impl alarm::Request for TimerMux {
  fn fired(&mut self) {
    if self.request.is_none() {return;}

    let ropt = self.request.take();
    let request: &'static mut RequestInternal = ropt.unwrap();

    if request.is_repeat {
    //  let t = self as &mut Timer;
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
