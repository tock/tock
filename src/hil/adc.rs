/* Simplest attempt at an ADC interface.
 *
 *   Author: Philip Levis <pal@cs.stanford.edu>
 *   Date: 6/10/15
*/

pub trait Request {
  fn read_done(&mut self, val: u16);
  fn channel(&mut self) -> u8;
}

pub trait AdcInternal {
    fn initialize(&mut self) -> bool;
    fn sample(&mut self, &'static mut Request) -> bool;
//    fn handle_interrupt(&mut self);
}
