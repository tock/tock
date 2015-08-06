/* hil::timer -- Traits and structures for software timers.
 *
 * Support for virtualized software timers on top of a single hardware 
 * alarm (counter interrupt). Currently traits do not specify the time 
 * units of the timers; a future version will. 
 *
 * The four relevant types are:
 *   - Timer: the trait that provides virtualized timers
 *   - TimerMux: the struct that implements Timer on top of Alarm
 *   - TimerRequest: the structure a caller must pass in when requesting
 *                   a timer
 *   - TimerCB: the trait which has the callback for when the timer fires
 *
 * The TimerRequest structure contains a reference to a TimerCB. So
 * to invoke a timer, an application defines a structure that implements
 * TimerCB, creates a TimerRequest whose callback field is the TimerCB
 * (either through new() or initialization), then passes TimerRequest to
 * calls to Timer.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Author: Amit Levy <levya@cs.stanford.edu>
 * Date: 7/16/15
 */
#![allow(dead_code)]

use core::prelude::*;
use core::mem::transmute;
use alarm;

// If a timer is late (was supposed to fire in the past), LATE_DELAY
// specifies how far in the future to fire it. This number needs to be
// > 1 in case the counter ticks between reading it and setting the
// overflow value. Set it to 5 just to be safe.
const LATE_DELAY: u32 = 5;

pub trait TimerCB {
    fn fired(&mut self, &'static mut TimerRequest, now: u32);
}

pub trait Timer {
    fn now(&self) -> u32;
    fn cancel(&mut self, &'static mut TimerRequest);
    fn oneshot(&mut self, interval: u32, &'static mut TimerRequest);
    fn repeat(&mut self, interval: u32, &'static mut TimerRequest);
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

pub struct VirtualTimer {
    request: *mut TimerRequest,
    internal: &'static mut Timer
}

impl VirtualTimer {
    pub fn new(timer: &'static mut Timer, req: &'static mut TimerRequest)
        -> VirtualTimer {
        VirtualTimer { request: req, internal: timer }
    }

    pub fn repeat(&mut self, interval: u32) {
        unsafe {
            self.internal.repeat(interval, transmute(self.request))
        }
    }

    pub fn oneshot(&mut self, interval: u32) {
        unsafe {
            self.internal.oneshot(interval, transmute(self.request))
        }
    }

    pub fn cancel(&mut self) {
        unsafe {
            self.internal.cancel(transmute(self.request))
        }
    }

    pub fn now(&self) -> u32 {
        self.internal.now()
    }
}

pub struct TimerMux {
    request: Option<&'static mut TimerRequest>,
    internal: &'static mut alarm::Alarm
}

impl TimerMux {
    pub fn new(internal: &'static mut alarm::Alarm) -> TimerMux {
        TimerMux {
            request: None,
            internal: internal
        }
    }

    /*
     * There are two pairs of functions for manipulating the ordered
     * linked list of outstanding timers
     *
     *  insert/delete: operate on list, inserting or deleting an entry
     *  add/remove: operate on list, and reconfigure underlying hardware
     *              alarm if first element of list changes
     *
     * add/remove are wrappers around insert/delete.
     */

    /* Insert a TimerRequest into the linked list, reconfiguring the
     * hardware alarm if it is inserted as the first element. */
    fn add(&mut self, request: &'static mut TimerRequest) -> bool {
        let changed = self.insert(request);
        if changed {
            self.start_request();
        }
        changed
    }

    /* Remove a TimerRequest into the linked list, reconfiguring the
     * hardware alarm if it was the first element. */
    fn remove(&mut self, request: &'static mut TimerRequest) -> bool {
        let changed = self.delete(request);
        if changed {
            self.start_request();
        }
        changed
    }

    /* Insert a TimerRequest into the linked list. Returns whether hardware 
    * clock has to be recalculated (inserted timer is now first timer) */
    fn insert(&mut self, request: &'static mut TimerRequest) -> bool {
        if request.next.is_some() { // Already on a list, this is an error!
            false
        } else if self.request.is_none() {
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

    /* Delete a TimerRequest from the linked list. Returns whether hardware 
     * clock has to be recalculated (removed first timer) */
    fn delete(&mut self, request: &'static mut TimerRequest) -> bool {
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

    /* Schedule the hardware alarm based on the first TimerRequest.
     * Assumes that timers are at most 2^31 in the future. If the
     * duration until a timer is > 2^31, assumes this means the
     * timer has passed (is late) and so fires immediately (in LATE_DELAY
     * ticks).
     */
    fn start_request(&mut self) {
        match self.request {
            Some(ref mut request) => {
                let mut when = request.when;
                let curr = self.internal.now();
                let delay = request.when - curr;
                if delay > (0x80000000) {
                    when = curr + LATE_DELAY;
                    request.when = when;
                }
                self.internal.set_alarm(when);
            },
            None => {}
        }
    }

}

impl alarm::AlarmClient for TimerMux {

    // The hardware alarm fired. If its firing matches the expected
    // firing time, invoke the next software timer and schedule the
    // following one.
    fn fired(&mut self) {
        if self.request.is_none() {return;}
        let curr = self.now();
        let ropt = self.request.take();
        let request: &'static mut TimerRequest = ropt.unwrap();
        // The timer did not fire early
        if (request.when - curr) < 20 {
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
                self.add(request);
            } else {
                request.is_active = false;
            }
            self.start_request();
            cb.fired(request, curr);
        } else {
            // Timer fired early. Not sure why this happens,
            // But it does -pal 7/28/15
            self.request = Some(request);
            self.start_request();
        }
    }
}

impl Timer for TimerMux {
    fn now(&self) -> u32 {
        self.internal.now()
    }

    fn cancel(&mut self, request: &'static mut TimerRequest) {
        if !request.is_active {return;}

        request.is_active = false;
        self.remove(request);
    }

    fn oneshot(&mut self, interval: u32, request: &'static mut TimerRequest) {
        request.interval = interval;
        request.is_active = true;
        request.when = self.now() + interval;
        request.is_repeat = false;
        self.add(request);
    }

    fn repeat(&mut self, interval: u32, request: &'static mut TimerRequest) {
        request.interval = interval;
        request.is_active = true;
        request.when = self.now() + interval;
        request.is_repeat = true;
        self.add(request);
    }
}

