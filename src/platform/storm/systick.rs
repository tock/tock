use core::intrinsics;

struct SysTick {
    control:     u32,
    reload:      u32,
    value:       u32,
    calibration: u32
}

const BASE_ADDR : *mut SysTick = 0xE000E010 as *mut SysTick;

/// Sets the timer as close as possible to the given interval in microseconds.
/// The clock is 24-bits wide and specific timing is dependent on the driving
/// clock. Increments of 10ms are most accurate and, in practice 466ms is the
/// approximate maximum.
pub unsafe fn set_timer(us: u32) {
    let systick : &mut SysTick = &mut *BASE_ADDR;

    let tenms = intrinsics::volatile_load(&systick.calibration) & 0xffffff;
    let reload = tenms * us / 10000;

    intrinsics::volatile_store(&mut systick.value, 0);
    intrinsics::volatile_store(&mut systick.reload, reload);
}

/// Returns the time left in approximate microseconds
pub unsafe fn value() -> u32 {
    let systick : &SysTick = &*BASE_ADDR;

    let tenms = intrinsics::volatile_load(&systick.calibration) & 0xffffff;
    let value = intrinsics::volatile_load(&systick.value) & 0xffffff;

    value * 10000 / tenms
}

pub unsafe fn overflowed() -> bool {
    let systick : &SysTick = &*BASE_ADDR;
    intrinsics::volatile_load(&systick.control) & 1 << 16 != 0
}

pub unsafe fn reset() {
    let systick : &mut SysTick = &mut *BASE_ADDR;

    intrinsics::volatile_store(&mut systick.control, 0);
    intrinsics::volatile_store(&mut systick.reload, 0);
    intrinsics::volatile_store(&mut systick.value, 0);
    intrinsics::volatile_store(&mut OVERFLOW_FIRED, 0);
}

#[inline(never)]
pub unsafe fn enable(with_interrupt: bool) {
    let systick : &mut SysTick = &mut *BASE_ADDR;

    if with_interrupt {
        intrinsics::volatile_store(&mut systick.control, 0b111);
    } else {
        intrinsics::volatile_store(&mut systick.control, 0b101);
    }
}

#[no_mangle]
pub static mut OVERFLOW_FIRED : usize = 0;

pub unsafe fn overflow_fired() -> bool {
    intrinsics::volatile_load(&OVERFLOW_FIRED) == 1
}

