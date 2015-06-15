/* Simplest attempt at an ADC interface.
 *
 *   Author: Philip Levis <pal@cs.stanford.edu>
 *   Date: 6/10/15
*/

pub trait Callback {
  fn read_done(&mut self, val: u16);
}

pub struct Request {
  pub channel: u8,
  pub callback: &'static mut Callback
}

pub trait AdcInternal {
    fn initialize(&mut self) -> bool;
    fn sample(&mut self, &'static mut Request) -> bool;
    fn handle_interrupt(&mut self);
}
