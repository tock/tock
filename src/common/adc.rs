// hil::adc is an ADC mux/demux; it allows multiple outstanding requests

use hil::adc;
use core::prelude::*;

pub struct AdcMux {
  request: Option<&'static mut adc::Request>,
  last: Option<&'static mut adc::Request>,
  internal: Option<&'static mut adc::AdcImpl>,
}

impl AdcMux {
  fn start_request(&'static mut self) {
     self.internal.as_mut().unwrap().sample(self); 
  }
}

impl adc::ImplRequest for AdcMux {
   fn read_done(&'static mut self, val: u16) {
      if self.request.is_some() {
          let mut current: Option<&'static mut adc::Request> = self.request.take();
          let mut req    = current.as_mut().unwrap();
          let mut next   = req.next.take();
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

impl adc::Adc for AdcMux {
  fn sample(&mut self,
            chan: u8,
            callback: &'static FnMut(u16),
            request: &'static mut adc::Request) {
    request.chan = chan;
    request.callback = callback;
    match self.request {
      Some(ref mut r) => {
        self.last.as_mut().unwrap().next = Some(request);
        self.last = Some(request);
      }
      None => {
        self.request = Some(request);
        self.last = Some(request);
//        self.start_request();
      }
    }
  }
}

