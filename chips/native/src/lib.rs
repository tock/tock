#![crate_name = "tock_native_chip"]
#![crate_type = "rlib"]
#![feature(attr_literals, const_cell_new)]
#![feature(const_atomic_usize_new, const_ptr_null_mut, integer_atomics)]
#![feature(asm, core_intrinsics, concat_idents, const_fn)]
#![no_std]

extern crate tock_native_arch;
#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init, register_bitfields, register_bitmasks)]
extern crate kernel;

pub mod chip;

use tock_native_arch::{generic_isr, svc_handler, systick_handler};

unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!("unhandled_interrupt");
    //panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

extern "C" {
    /*
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
    */

    // Defined by platform
    fn reset_handler();

    /*
    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
    */
}

//#[link_section = ".vectors"]
// no_mangle Ensures that the symbol is kept until the final binary
#[no_mangle]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    reset_handler, // FIXME
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

//#[link_section = ".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 111] = [generic_isr; 111];

pub unsafe fn init() {

    unimplemented!("Manual relocation needed on native?");
    /*
    // Relocate data segment.
    // Assumes data starts right after text segment as specified by the linker
    // file.
    let mut pdest = &mut _srelocate as *mut u32;
    let pend = &mut _erelocate as *mut u32;
    let mut psrc = &_etext as *const u32;

    if psrc != pdest {
        while (pdest as *const u32) < pend {
            *pdest = *psrc;
            pdest = pdest.offset(1);
            psrc = psrc.offset(1);
        }
    }

    // Clear the zero segment (BSS)
    let pzero = &_ezero as *const u32;
    pdest = &mut _szero as *mut u32;

    while (pdest as *const u32) < pzero {
        *pdest = 0;
        pdest = pdest.offset(1);
    }
    */

    unimplemented!("Call native interrupt init (probably signal setup)");
}

unsafe extern "C" fn hard_fault_handler() {
    unimplemented!("Hard fault hander");
}

