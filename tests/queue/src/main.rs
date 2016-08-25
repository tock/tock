#![feature(core)]
#![feature(rand)]
extern crate core;
use core::intrinsics;

extern crate rand;
use rand::Rng;

mod queue;

pub fn volatile_load<T>(item: &T) -> T {
    unsafe { core::intrinsics::volatile_load(item) }
}

pub fn volatile_store<T>(item: &mut T, val: T) {
    unsafe { core::intrinsics::volatile_store(item, val) }
}

macro_rules! volatile {
    ($item:expr) => ({
        ::volatile_load(&$item)
    });

    ($item:ident = $value:expr) => ({
        ::volatile_store(&mut $item, $value)
    });

    ($item:ident |= $value:expr) => ({
        ::volatile_store(&mut $item, ::volatile_load(&$item) | $value)
    });

    ($item:ident &= $value:expr) => ({
        ::volatile_store(&mut $item, ::volatile_load(&$item) & $value)
    });
}
const SIZE: usize = 10;

fn main() {
    let mut iq: queue::InterruptQueue = queue::InterruptQueue::new();
    let mut q = &mut iq as &mut queue::Queue<usize>;
    let mut rng = rand::thread_rng();
    for x in 0..queue::IQ_SIZE * 100 {
        if rng.gen() {
            q.enqueue(x);
        } else {
            let e = q.is_empty();
            let y = q.dequeue();
            print!("{} ({}): {}\n", x, e, y);
        }
    }
}
