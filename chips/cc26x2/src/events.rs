//
//  These are generic event handling routines which could be defined in cortexm
//

use core::ptr;
use cortexm::support::atomic;
use enum_primitive::cast::FromPrimitive;
use event_priority::{EVENT_PRIORITY, FLAGS};

pub fn has_event() -> bool {
    let event_flags;
    unsafe { event_flags = ptr::read_volatile(&FLAGS) }
    event_flags != 0
}

pub fn next_pending() -> Option<EVENT_PRIORITY> {
    let mut event_flags;
    unsafe { event_flags = ptr::read_volatile(&FLAGS) }

    let mut count = 0;
    // stay in loop until we found the flag
    while event_flags != 0 {
        // if flag is found, return the count
        if (event_flags & 0b1) != 0 {
            return Some(EVENT_PRIORITY::from_u8(count).expect("Unmapped EVENT_PRIORITY"));
        }
        // otherwise increment
        count += 1;
        event_flags >>= 1;
    }
    None
}

pub fn set_event_flag(priority: EVENT_PRIORITY) {
    unsafe {
        let bm = 0b1 << (priority as u8) as u32;
        atomic(|| {
            let new_value = ptr::read_volatile(&FLAGS) | bm;
            FLAGS = new_value;
        })
    };
}

#[naked]
pub unsafe fn set_event_flag_from_isr(priority: EVENT_PRIORITY) {
    // Set PRIMASK
    asm!("cpsid i" :::: "volatile");

    asm!("
        // Set event flag
        orr $0, $2
        isb
        "
        : "={r0}"(FLAGS)
        : "{r0}"(FLAGS), "{r1}"(0b1<<(priority as u8))
        : : "volatile" "volatile"
    );

    // Unset PRIMASK
    asm!("cpsie i" :::: "volatile");
}

pub fn clear_event_flag(priority: EVENT_PRIORITY) {
    unsafe {
        let bm = !0b1 << (priority as u8) as u32;
        atomic(|| {
            let new_value = ptr::read_volatile(&FLAGS) & bm;
            FLAGS = new_value;
        })
    };
}
