/* Simplest attempt at an ADC interface.
 *
 *   Author: Philip Levis <pal@cs.stanford.edu>
 *   Date: 6/10/15
*/

use core::prelude::*;

pub struct Request {
  pub callback: &'static FnMut(u16),
  pub next: Option<&'static mut Request>,
  pub chan: u8
}

pub trait Adc {
  fn sample(&mut self, chan: u8, &'static FnMut(u16), &'static mut Request);
}

pub trait ImplRequest {
  fn read_done(&mut self, val: u16);
  fn channel(&self) -> u8;
}

pub trait AdcImpl {
    fn initialize(&mut self) -> bool;
    fn sample(&mut self, &'static mut ImplRequest) -> bool;
}
