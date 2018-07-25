#![crate_name = "tm4c129x"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, core_intrinsics, used)]
#![feature(tool_attributes)]
#![no_std]

extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init, register_bitfields, register_bitmasks)]
extern crate kernel;

pub mod chip;
pub mod gpio;
pub mod gpt;
pub mod nvic;
pub mod sysctl;
pub mod uart;

use cortexm4::{generic_isr, hard_fault_handler, svc_handler, systick_handler};

unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // TM4C129x IPSR[8:0] holds the currently active interrupt
    asm!(
        "mrs    r0, ipsr                    "
        : "={r0}"(interrupt_number)
        :
        : "r0"
        :
        );

    interrupt_number = interrupt_number & 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();

    // Defined by platform
    fn reset_handler();

    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

#[link_section = ".vectors"]
// used Ensures that the symbol is kept until the final binary
#[used]
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

#[link_section = ".vectors"]
#[used] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 111] = [generic_isr; 111];

pub unsafe fn init() {
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

    cortexm4::nvic::disable_all();
    cortexm4::nvic::clear_all_pending();
    cortexm4::nvic::enable_all();
}
