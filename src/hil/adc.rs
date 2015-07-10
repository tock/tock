/* Simplest attempt at an ADC interface.
 *
 *   Author: Philip Levis <pal@cs.stanford.edu>
 *   Date: 6/10/15
*/

use core::prelude::*;

pub trait Request {
  fn read_done(&'static mut self, val: u16, &'static mut Request);
}

pub trait Adc {
  fn sample(&'static mut self,
            chan: u8,
            request: &'static mut Request,
            internal: &'static mut RequestInternal);
}

pub struct RequestInternal {
    pub next: Option<&'static mut RequestInternal>,
    pub chan: u8,
    pub callback: Option<&'static mut Request>
}

// Needed because a RequestInternal has a reference to a Request
// Since we know RequestInternals are only accessed by one thread
// we can tell Rust it's thread safe.
unsafe impl Sync for RequestInternal { }

pub trait ImplRequest {
    fn read_done(&'static mut self, val: u16);
    fn channel(&self) -> u8;
}

pub trait AdcImpl {
    fn initialize(&mut self) -> bool;
    fn sample(&mut self, &'static mut ImplRequest) -> bool;
}

pub struct AdcMux {
  request: Option<&'static mut RequestInternal>,
  last: Option<&'static mut RequestInternal>,
  internal: Option<&'static mut AdcImpl>,
}

impl AdcMux {
  pub fn new(internal: &'static mut AdcImpl) -> AdcMux {
    AdcMux {
      request: None,
      last: None,
      internal: Some(internal)
    }
  }

  fn start_request(&'static mut self) {
     let mut opt: Option<&'static mut AdcImpl> = self.internal.take();
     let mut adc = opt.as_mut().unwrap();
     adc.sample(self as &'static mut ImplRequest);
  }
}

impl ImplRequest for AdcMux {
   fn read_done(&'static mut self, val: u16) {
      if self.request.is_some() {
          let mut current: Option<&'static mut RequestInternal> = self.request.take();
          let mut req    = current.as_mut().unwrap();
          let next   = req.next.take();
          self.request = next;
      }

      if self.request.is_some() {
          self.start_request();
      }
   }
   fn channel(&self) -> u8 {
      match self.request {
         Some(ref r) => {r.chan}
         None => {0}
      }
   }
}

impl Adc for AdcMux {
  fn sample(&'static mut self,
            chan: u8,
            request: &'static mut Request,
            internal: &'static mut RequestInternal) {

    internal.chan = chan;
    internal.callback = Some(request);
    match self.request {
      Some(ref mut r) => {
        self.last.as_mut().unwrap().next = Some(internal);
        self.last = Some(internal);
      }
      None => {
        self.request = Some(internal);
        self.last = Some(internal);
        self.start_request();
      }
    }
  }
}

