// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Panic handler for the bootloader

use core::panic::PanicInfo;

/// Panic handler - blinks LED2 rapidly to indicate a panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Configure LED2
    const GPIO_P0_BASE: usize = 0x5000_0000;
    const GPIO_OUTSET_OFFSET: usize = 0x508;
    const GPIO_OUTCLR_OFFSET: usize = 0x50C;
    const LED2_PIN: u32 = 14;
    
    unsafe {
        // Blink LED2 rapidly
        loop {
            core::ptr::write_volatile(
                (GPIO_P0_BASE + GPIO_OUTCLR_OFFSET) as *mut u32,
                1 << LED2_PIN
            );
            
            for _ in 0..500_000 {
                cortex_m::asm::nop();
            }
            
            core::ptr::write_volatile(
                (GPIO_P0_BASE + GPIO_OUTSET_OFFSET) as *mut u32,
                1 << LED2_PIN
            );
            
            for _ in 0..500_000 {
                cortex_m::asm::nop();
            }
        }
    }
}