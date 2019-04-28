//
//  These are generic event handling routines for any cortex-m architecture
//
use crate::support::atomic;
use core::ptr;
use enum_primitive::cast::{FromPrimitive, ToPrimitive};

pub static mut FLAGS: usize = 0;

pub fn has_event() -> bool {
    let event_flags;
    unsafe { event_flags = ptr::read_volatile(&FLAGS) }
    event_flags != 0
}

pub fn next_pending<T: FromPrimitive>() -> Option<T> {
    let mut event_flags;
    unsafe { event_flags = ptr::read_volatile(&FLAGS) }

    let mut count = 0;
    // stay in loop until we found the flag
    while event_flags != 0 {
        // if flag is found, return the count
        if (event_flags & 0b1) != 0 {
            return Some(T::from_u8(count).expect("Unmapped EVENT_PRIORITY"));
        }
        // otherwise increment
        count += 1;
        event_flags >>= 1;
    }
    None
}

pub fn set_event_flag<T: ToPrimitive>(priority: T) {
    unsafe {
        let bm = 0b1
            << priority
                .to_usize()
                .expect("Could not cast priority enum as usize");
        atomic(|| {
            let new_value = ptr::read_volatile(&FLAGS) | bm;
            FLAGS = new_value;
        })
    };
}

#[naked]
pub unsafe fn set_event_flag_from_isr(priority: usize) {
    // Set PRIMASK
    asm!("cpsid i" :::: "volatile");

    asm!("
        // Set event flag
        orr $0, $2
        isb
        "
        : "={r0}"(FLAGS)
        : "{r0}"(FLAGS), "{r1}"(0b1<<(priority))
        : : "volatile" "volatile"
    );

    // Unset PRIMASK
    asm!("cpsie i" :::: "volatile");
}

pub fn clear_event_flag<T: ToPrimitive>(priority: T) {
    unsafe {
        let bm = !(0b1
            << priority
                .to_usize()
                .expect("Could not cast priority enum as usize"));
        atomic(|| {
            let new_value = ptr::read_volatile(&FLAGS) & bm;
            FLAGS = new_value;
        })
    };
}
