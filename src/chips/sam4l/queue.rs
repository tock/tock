use hil::{queue};
use nvic;
use core::intrinsics::volatile_load;

pub const IQ_SIZE: usize = 100;

#[allow(dead_code)]
pub struct InterruptQueue {
  ring: [nvic::NvicIdx; IQ_SIZE],
  head: usize,
  tail: usize
}

#[allow(dead_code)]
impl InterruptQueue {
  pub fn new() -> InterruptQueue {
    InterruptQueue {
      head: 0,
      tail: 0,
      ring: [nvic::NvicIdx::INVALID; IQ_SIZE]
    }
  }
}

#[allow(dead_code)]
impl queue::Queue<nvic::NvicIdx> for InterruptQueue {
  fn has_elements(&self) -> bool {
    unsafe {
      let head = volatile_load(&self.head);
      let tail = volatile_load(&self.tail);
      head != tail
    }
  }

  fn is_full(&self) -> bool {
    unsafe {
      volatile_load(&self.head) == ((volatile_load(&self.tail) + 1) % IQ_SIZE)
    }
  }

  fn enqueue(&mut self, val: nvic::NvicIdx) -> bool {
    unsafe {
      let head = volatile_load(&self.head);
      if ((self.tail + 1) % IQ_SIZE) == head {
        // Incrementing tail will overwrite head
        return false;
      } else {
        self.ring[self.tail] = val;
        self.tail = (self.tail + 1) % IQ_SIZE;
        return true;
      }
    }
  }

  fn dequeue(&mut self) -> nvic::NvicIdx {
    let val: nvic::NvicIdx = self.ring[self.head];
    if self.has_elements() {
      self.head = (self.head + 1) % IQ_SIZE;
    }
    val
  }
}
