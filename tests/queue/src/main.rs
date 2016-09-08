#![feature(core)]
#![feature(rand)]
extern crate core;
use core::ptr;

extern crate rand;
use rand::Rng;

mod queue;

pub fn read_volatile<T>(item: &T) -> T {
    unsafe { core::ptr::read_volatile(item) }
}

pub fn write_volatile<T>(item: &mut T, val: T) {
    unsafe { core::ptr::write_volatile(item, val) }
}

macro_rules! volatile {
    ($item:expr) => ({
        ::read_volatile(&$item)
    });

    ($item:ident = $value:expr) => ({
        ::write_volatile(&mut $item, $value)
    });

    ($item:ident |= $value:expr) => ({
        ::write_volatile(&mut $item, ::read_volatile(&$item) | $value)
    });

    ($item:ident &= $value:expr) => ({
        ::write_volatile(&mut $item, ::read_volatile(&$item) & $value)
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
