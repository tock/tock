/* Simplest attempt at an ADC interface.
 *
 *   Author: Philip Levis <pal@cs.stanford.edu>
 *   Date: 6/10/15
*/

pub trait Request {
  fn sample_done(&self, val: u16, request: &'static Request);
}

pub trait AdcInternal {
    fn initialize(&'static mut self) -> bool;
    fn sample(&self, channel: u8, callback: &'static Request) -> bool;
}
