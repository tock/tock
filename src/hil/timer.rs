/* hil::timer -- Traits and structures for software timers.
 *
 * Author: Philip Levis <pal@cs.stanford.edu>
 * Date: 7/16/15
 */

use core::prelude::*;

pub trait Request {
  fn read_done(&'static mut self, now: u32);
}

pub trait Timer {
  fn now(&self) -> u32; 
  fn cancel(&mut self, &'static mut Request);
  fn oneshot(&mut self, interval: u32, &'static mut Request);
  fn periodic(&mut self, interval: u32, &'static mut Request);
}

pub struct RequestInternal {
  pub next: Option<&'static mut RequestInternal>,
  pub is_active: bool,
  pub when: u32,
  pub interval: u32,
  pub callback: Option<&'static mut Request>
}
