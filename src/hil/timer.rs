/* hil::timer -- Traits and structures for software timers.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/16/15
 */
#![allow(dead_code)]

use core::prelude::*;
use alarm;

pub trait TimerCB {
  fn fired(&'static mut self, &'static mut TimerRequest, now: u32);
}

pub trait Timer {
  fn now(&'static mut self) -> u32;
  fn cancel(&'static mut self, &'static mut TimerRequest);
  fn oneshot(&'static mut self, interval: u32, &'static mut TimerRequest);
  fn repeat(&'static mut self, interval: u32, &'static mut TimerRequest);

}

pub struct TimerRequest {
  pub next: Option<&'static mut TimerRequest>,
  pub is_active: bool,
  pub is_repeat: bool,
  pub when: u32,
  pub interval: u32,
  pub callback: Option<&'static mut TimerCB>
}

impl TimerRequest {
  pub fn new(cb: &'static mut TimerCB) -> TimerRequest {
    TimerRequest {
      next:      None,
      is_active: false,
      is_repeat: false,
      when:      0,
      interval:  0,
      callback:  Some(cb)
    }
  }
}

pub struct TimerMux {
  request: Option<&'static mut TimerRequest>,
  internal: Option<&'static mut alarm::Alarm>
}

impl TimerMux {
  pub fn new(internal: &'static mut alarm::Alarm) -> TimerMux {
    TimerMux {
      request: None,
      internal: Some(internal)
    }
  }

  fn start_request(&'static mut self) {
    if self.request.is_none() {return;}

    let aopt: Option<&'static mut alarm::Alarm> = self.internal.take();
    let alarm: &'static mut alarm::Alarm = aopt.unwrap();
    let ropt = self.request.take();
    let request: &'static mut TimerRequest = ropt.unwrap();
    let when = request.when;

    //let mut me = self as &'static mut alarm::Request;
    alarm.set_alarm(when, self);// as &mut alarm::Request);

    self.internal = Some(alarm);
    self.request = Some(request);

  }

  fn add(&'static mut self, request: &'static mut TimerRequest) -> bool {
    let changed = self.insert(request);
    if changed {
      self.start_request();
    }
    changed
  }

  fn remove(&'static mut self, request: &'static mut TimerRequest) -> bool {
    let changed = self.delete(request);
    if changed {
      self.start_request();
    }
    changed
  }

  // Returns whether hardware clock has to be recalculated (inserted
  // timer is now first timer)
  fn insert(&'static mut self, request: &'static mut TimerRequest) -> bool {
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
  fn delete(&'static mut self, request: &'static mut TimerRequest) -> bool {
    if self.request.is_none() {return false;}

    let mut done = false;
    let mut copt = &mut self.request;
    let mut first = true;
    while !done {
      let mycopt = copt.take();
      let mut curr = mycopt.unwrap();
      let cptr: *const TimerRequest = curr;
      let rptr: *const TimerRequest = request;
      if cptr == rptr {
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

impl alarm::Request for TimerMux {
  fn fired(&'static mut self) {
    if self.request.is_none() {return;}
    let ropt = self.request.take();
    let request: &'static mut TimerRequest = ropt.unwrap();
    self.request = request.next.take();

    // Note this implementation is inefficient: if the repeat timer
    // would be at the head of the queue again, we recalculate the
    // timer, then re-insert so recalculate a second time.
    // A better implementation would check this and conditionally
    // remove/insert. -pal 7/22/15
    let cbopt = request.callback.take();
    let cb: &'static mut TimerCB = cbopt.unwrap();
    request.callback = Some(cb);

    if request.is_repeat {
      request.when = request.when + request.interval;
      let until = request.when - self.now();
      self.add(request); 
    } else {
      request.is_active = false;
    }

    cb.fired(request, self.now());
  }
}

impl Timer for TimerMux {
  fn now(&'static mut self) -> u32 {
     let mut alarm = self.internal.as_mut().unwrap();
     let val = alarm.now();
     self.internal = Some(*alarm);
     val
  }

  fn cancel(&'static mut self, request: &'static mut TimerRequest) {
    if !request.is_active {return;}

    request.is_active = false;
    self.remove(request);
  }

  fn oneshot(&'static mut self, interval: u32, request: &'static mut TimerRequest) {
    request.interval = interval;
    request.is_active = true;
    request.when = self.now() + interval;
    request.is_repeat = false;
    self.add(request);
  }

  fn repeat(&'static mut self, interval: u32, request: &'static mut TimerRequest) {
    request.interval = interval;
    request.is_active = true;
    request.when = self.now() + interval;
    request.is_repeat = true;
    self.add(request);
  }
}
