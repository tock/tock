#![no_std]
// #![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

pub mod delay;
pub mod examples;
pub mod executor;

extern crate alloc;

use critical_section::RawRestoreState;
use embedded_alloc::LlffHeap as Heap;

struct MyCriticalSection;
critical_section::set_impl!(MyCriticalSection);

// Tock is single threaded, so locking is not required
unsafe impl critical_section::Impl for MyCriticalSection {
    unsafe fn acquire() -> RawRestoreState {}

    unsafe fn release(_token: RawRestoreState) {}
}

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub fn init() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
}
