// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Secure bootloader for nRF52840DK
//! 
//! This bootloader verifies the Tock kernel signature and version before jumping to it.

#![no_std]
#![no_main]

mod config;
mod io;
mod panic;

use config::Nrf52840Config;
use io::Nrf52840IO;
use secure_boot_common::{verify_and_boot, BootloaderIO};
use secure_boot_common::error::BootError;

// const LED1_PIN: u32 = 13; // P0.13
// const LED2_PIN: u32 = 14; // P0.14
// const LED3_PIN: u32 = 15; // P0.15
// const LED4_PIN: u32 = 16; // P0.16

// Include the startup assembly code
core::arch::global_asm!(include_str!("startup.s"));

/// Main entry point called by startup code
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize I/O
    let io = Nrf52840IO::new();
    // io.debug("\r\n\r\n");
    // io.debug("Secure Boot for Tock");

    // let mut buf = [0u8; 32];

    // let reset_reason = read_reset_reason();
    // clear_reset_reason(reset_reason);

    // // no. of blinks is cause for reset (1=RESETPIN, 2=DOG, 3=SREQ, 4=LOCKUP)
    // let code = if reset_reason & (1<<3) != 0 { 4 }    // LOCKUP
    //     else if reset_reason & (1<<2) != 0 { 3 }      // SYSRESETREQ (soft reset)
    //     else if reset_reason & (1<<1) != 0 { 2 }      // watchdog
    //     else if reset_reason & (1<<0) != 0 { 1 }      // reset pin
    //     else { 5 };                                   // unknown/none
    // io.debug_blink(LED4_PIN, code);

    // io.debug("reset reason: ");
    // io.format(code, &mut buf);
    
    // Debug: 1 blink = bootloader started
    // io.debug_blink(LED1_PIN, 1);
    
    // Signature verification
    match verify_and_boot::<Nrf52840Config, _>(&io) {
        Ok((kernel_entry, kernel_end)) => {

            // verify stack pointer and reset vector are valid
            let vector_table = kernel_entry as *const u32;
            let sp = unsafe { core::ptr::read_volatile(vector_table) } as usize;
            let reset_vector = unsafe { core::ptr::read_volatile(vector_table.add(1)) } as usize;

            let sp_in_sram = (0x2000_0000..0x2004_0000).contains(&sp);
            let rv_in_flash = (kernel_entry..kernel_end).contains(&(reset_vector & !1)) && (reset_vector & 1) == 1;

            // let validity = if !sp_in_sram { 1 } else if !rv_in_flash { 2 } else { 3 };

            // io.debug("sp and rv validity:");
            // io.format(validity, &mut buf);

            if !(sp_in_sram && rv_in_flash){
                io.signal_failure();
                loop {} 
            } else {
                // Kernel Handoff
                // io.debug("handing off to kernel");

                unsafe { jump_to_kernel(kernel_entry, &io); } 
            }
        }
        Err(error) => {
            // Debug: Blink LED2 to show specific error code
            let _blink_count = match error {
                BootError::SentinelNotFound => 3,
                BootError::InvalidTLV => 4,
                BootError::InvalidSignature => 5,
                BootError::VerificationFailed => 6,
                BootError::UnsupportedAlgorithm => 7,
                BootError::InvalidKernelRegion => 8,
                BootError::SignatureMissing => 9,
                BootError::VersionTooOld => 10,
                BootError::HashError => 11,
                BootError::NoValidKernel => 12,
            };
            
            // io.debug_blink(LED2_PIN, blink_count);
            
            // Verification failed
            io.signal_failure();
            unreachable!();
        }
    }
}

/// Jump to the kernel entry point
/// 
/// This function performs the handoff from bootloader to kernel:
/// 1. Loads the kernel's initial stack pointer
/// 2. Jumps to the kernel's reset handler

unsafe fn jump_to_kernel<IO: BootloaderIO>(kernel_entry: usize, io: &IO) -> ! {

    // let mut buf = [0u8; 32];
    // Set NVMC back to read
    const NVMC_CONFIG: *mut u32 = 0x4001_E504 as *mut u32;
    const NVMC_READY:  *const u32 = 0x4001_E400 as *const u32;
    core::ptr::write_volatile(NVMC_CONFIG, 0);
    while core::ptr::read_volatile(NVMC_READY) == 0 {}

    // Read vector table
    let vt = kernel_entry as *const u32;
    let msp = core::ptr::read_volatile(vt);
    let reset = core::ptr::read_volatile(vt.add(1));

    // io.debug("VT[0] MSP=");   io.format(msp as usize, &mut buf);
    // io.debug("VT[1] Reset="); io.format(reset as usize, &mut buf);

    if (reset & 1) == 0 {
        // io.debug("reset vector not Thumb");
        io.signal_failure();
        loop {}
    }

    // Mask interrupts
    core::arch::asm!("cpsid i", options(nomem, nostack, preserves_flags));

    // Stop systick and clear/disable all NVIC
    const SYST_CSR: *mut u32 = 0xE000_E010 as *mut u32;
    core::ptr::write_volatile(SYST_CSR, 0);

    const NVIC_ICER0: *mut u32 = 0xE000_E180 as *mut u32;
    const NVIC_ICER1: *mut u32 = 0xE000_E184 as *mut u32;
    const NVIC_ICPR0: *mut u32 = 0xE000_E280 as *mut u32;
    const NVIC_ICPR1: *mut u32 = 0xE000_E284 as *mut u32;
    core::ptr::write_volatile(NVIC_ICER0, 0xFFFF_FFFF);
    core::ptr::write_volatile(NVIC_ICER1, 0xFFFF_FFFF);
    core::ptr::write_volatile(NVIC_ICPR0, 0xFFFF_FFFF);
    core::ptr::write_volatile(NVIC_ICPR1, 0xFFFF_FFFF);

    // Disable MPU
    const MPU_CTRL: *mut u32 = 0xE000_ED94 as *mut u32;
    core::ptr::write_volatile(MPU_CTRL, 0);

    // let res = set_vtor_and_verify(kernel_entry as u32);
    if (kernel_entry as u32 & 0x7F) != 0 {
        // Bad vtor entry
        io.signal_failure();
        loop {} 
    }
    const SCB_VTOR: *mut u32 = 0xE000_ED08 as *mut u32;
    core::ptr::write_volatile(SCB_VTOR, kernel_entry as u32);
    core::arch::asm!("dsb; isb", options(nomem, nostack, preserves_flags));

    // Clear pending faults like SysTick/PendSV
    const SCB_ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
    core::ptr::write_volatile(SCB_ICSR, (1 << 25) | (1 << 27)); // CLRPEND SysTick, PendSV
    core::arch::asm!("dsb; isb", options(nomem, nostack, preserves_flags));

    // Unmask exceptions so SVC/PendSV work for the kernel
    // PRIMASK=0, BASEPRI=0, FAULTMASK=0
    // Without this, kernel keeps hardfaulting because one of these get escalated
    core::arch::asm!("msr basepri, {0}",  in(reg) 0u32, options(nomem, nostack, preserves_flags));
    core::arch::asm!("msr faultmask, {0}",in(reg) 0u32, options(nomem, nostack, preserves_flags));
    core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
    
    // Privileged thread mode
    core::arch::asm!("msr control, {0}", in(reg) 0u32, options(nostack, preserves_flags));
    core::arch::asm!("isb", options(nomem, nostack, preserves_flags));

    // Load MSP from vector[0]
    core::arch::asm!("msr msp, {0}", in(reg) msp, options(nostack, preserves_flags));
    core::arch::asm!("dsb; isb", options(nomem, nostack, preserves_flags));

    // Branch to reset handler
    core::arch::asm!(
        "bx {0}",
        in(reg) (reset | 1),
        options(noreturn)
    );
}


// Exception handlers - all weak so they can be overridden
#[no_mangle]
#[link_section = ".text.Default_Handler"]
pub extern "C" fn Default_Handler() -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn NMI_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn HardFault_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn MemManage_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn BusFault_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn UsageFault_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn SVC_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn DebugMon_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn PendSV_Handler() -> ! {
    Default_Handler()
}

#[no_mangle]
pub extern "C" fn SysTick_Handler() -> ! {
    Default_Handler()
}