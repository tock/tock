use core::intrinsics::{volatile_load,volatile_store};

struct DWT {
    control:    u32,
    cycle_count: u32
}

const BASE_ADDR : *mut DWT = 0xE0001000 as *mut DWT;

pub fn start_counter() {
    unsafe {
        let dwt : &mut DWT = &mut *BASE_ADDR;
        volatile_store(0xE000EDFC as *mut u32,
                       volatile_load(0xE000EDFC as *const u32) | 1 << 24);
        volatile_store(&mut dwt.control, volatile_load(&dwt.control) | 1);
    }
}

pub fn stop_counter() {
    unsafe {
        let dwt : &mut DWT = &mut *BASE_ADDR;
        volatile_store(&mut dwt.control, volatile_load(&dwt.control) ^ 1);
    }
}

pub fn reset_counter() {
    unsafe {
        let dwt : &mut DWT = &mut *BASE_ADDR;
        volatile_store(&mut dwt.cycle_count, 0);
    }
}

pub fn cycle_count() -> u32 {
    unsafe {
        let dwt : &mut DWT = &mut *BASE_ADDR;
        volatile_load(&dwt.cycle_count)
    }
}

