//! Peripheral implementations for the Apollo3 MCU.

#![crate_name = "apollo3"]
#![crate_type = "rlib"]
#![feature(llvm_asm, const_fn, naked_functions)]
#![no_std]
#![allow(unused_doc_comments)]

// Peripherals
pub mod chip;
pub mod clkgen;
pub mod gpio;
pub mod iom;
pub mod nvic;
pub mod pwrctrl;
pub mod stimer;
pub mod uart;

use cortexm4::{
    generic_isr, hard_fault_handler, scb, svc_handler, systick_handler, unhandled_interrupt,
};

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();

    // Defined by platform
    fn reset_handler();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MemManage
    unhandled_interrupt, // BusFault
    unhandled_interrupt, // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    svc_handler,         // SVC
    unhandled_interrupt, // DebugMon
    unhandled_interrupt,
    unhandled_interrupt, // PendSV
    systick_handler,     // SysTick
];

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 32] = [generic_isr; 32];

// The Patch table.
//
// The patch table should pad the vector table size to a total of 64 entries
// (16 core + 48 periph) such that code begins at offset 0x100.
#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static PATCH: [unsafe extern "C" fn(); 16] = [unhandled_interrupt; 16];

extern "C" {
    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe fn init() {
    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);

    // Enable the cache controller
    *(0x40018000i32 as *mut u32) = 1 | (1 << 10) | (1 << 20);

    // Explicitly tell the core where Tock's vector table is located. If Tock is the
    // only thing on the chip then this is effectively a no-op. If, however, there is
    // a bootloader present then we want to ensure that the vector table is set
    // correctly for Tock. The bootloader _may_ set this for us, but it may not
    // so that any errors early in the Tock boot process trap back to the bootloader.
    // To be safe we unconditionally set the vector table.
    scb::set_vector_table_offset(BASE_VECTORS.as_ptr() as *const ());

    // Disable the FPU (it might be enalbed by a prior stage)
    scb::disable_fpca();

    // This ensures the FPU is actually disabled
    llvm_asm!("svc 0xff" : : : "r0","r1","r2","r3","r12" : "volatile" );

    cortexm4::nvic::disable_all();
    cortexm4::nvic::clear_all_pending();
    cortexm4::nvic::enable_all();
}

// Mock implementation for tests
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe fn init() {
    // Prevent unused code warning.
    scb::disable_fpca();

    unimplemented!()
}
