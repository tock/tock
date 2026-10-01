// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Fixture exercising each category of unsafe operation this tool
//! recognizes.

static mut COUNTER: u32 = 0;

unsafe fn danger(x: u32) -> u32 {
    x + 1
}

union Pun {
    as_u32: u32,
    as_f32: f32,
}

pub fn deref_example(ptr: *const u32) -> u32 {
    unsafe { *ptr }
}

pub fn call_example() -> u32 {
    unsafe { danger(COUNTER) }
}

pub fn transmute_example(x: u32) -> f32 {
    unsafe { core::mem::transmute(x) }
}

pub fn union_example(p: Pun) -> u32 {
    unsafe { p.as_u32 }
}

pub fn nested_example(ptr: *const u32) -> u32 {
    unsafe {
        let a = *ptr;
        let b = unsafe { danger(a) };
        a + b
    }
}

pub fn boring_example() -> u32 {
    unsafe { 1 + 1 }
}
