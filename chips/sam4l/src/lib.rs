//! Peripheral implementations for the SAM4L MCU.
//!
//! <http://www.atmel.com/microsite/sam4l/default.aspx>

#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(const_fn)]
#![no_std]

mod deferred_call_tasks;

pub mod acifc;
pub mod adc;
pub mod aes;
pub mod ast;
pub mod bpm;
pub mod bscif;
pub mod chip;
pub mod crccu;
pub mod dac;
pub mod dma;
pub mod eic;
pub mod flashcalw;
pub mod gloc;
pub mod gpio;
pub mod i2c;
pub mod nvic;
pub mod pm;
pub mod scif;
pub mod serial_num;
pub mod spi;
pub mod trng;
pub mod usart;
pub mod usbc;
pub mod wdt;

use cortexm4::{
    generic_isr, hard_fault_handler, svc_handler, systick_handler, unhandled_interrupt,
};

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
pub static IRQS: [unsafe extern "C" fn(); 80] = [generic_isr; 80];

pub unsafe fn init() {
    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);

    cortexm4::nvic::disable_all();
    cortexm4::nvic::clear_all_pending();
    cortexm4::nvic::enable_all();
}
