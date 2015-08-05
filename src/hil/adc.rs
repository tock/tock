/* Simplest attempt at an ADC interface.
 *
 *   Author: Philip Levis <pal@cs.stanford.edu>
 *   Date: 6/10/15
*/

pub trait Request {
  fn sample_done(&'static mut self, val: u16, request: &'static mut Request);
}

pub trait AdcInternal {
    fn initialize(&'static mut self) -> bool;
    fn sample(&'static mut self, channel: u8, callback: &'static mut Request) -> bool;
}
