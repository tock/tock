#![feature(asm,const_fn,linkage)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

extern "C" {
    pub fn init();
}

mod peripheral_registers;
mod peripheral_interrupts;
mod nvic;

pub mod aes;
pub mod chip;
pub mod gpio;
pub mod rtc;
pub mod timer;
pub mod clock;
pub mod uart;
pub mod pinmux;
pub use chip::NRF51;
pub mod temperature;
pub mod trng;

use core::ptr::write_volatile;

unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
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

    // Defined in src/arch/cortex-m4/ctx_switch.S
    fn SVC_Handler();
    fn systick_handler();

    fn generic_isr();

    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

#[link_section=".vectors"]
#[cfg_attr(rustfmt, rustfmt_skip)]
// no_mangle Ensures that the symbol is kept until the final binary
#[no_mangle]
pub static BASE_VECTORS: [unsafe extern fn(); 16] = [
    _estack, reset_handler,
    /* NMI */           unhandled_interrupt,
    /* Hard Fault */    unhandled_interrupt,
    /* MemManage */     unhandled_interrupt,
    /* BusFault */      unhandled_interrupt,
    /* UsageFault*/     unhandled_interrupt,
    unhandled_interrupt, unhandled_interrupt, unhandled_interrupt,
    unhandled_interrupt,
    /* SVC */           SVC_Handler,
    /* DebugMon */      unhandled_interrupt,
    unhandled_interrupt,
    /* PendSV */        unhandled_interrupt,
    /* SysTick */       systick_handler
];

#[link_section=".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 25] = [generic_isr; 25];

#[no_mangle]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub static INTERRUPT_TABLE: [Option<unsafe extern fn()>; 25] = [
    /* POWER_CLOCK_Handler */   Option::Some(unhandled_interrupt),
    /* RADIO_Handler */         Option::Some(unhandled_interrupt),
    /* UART0_Handler */         Option::Some(unhandled_interrupt),
    /* SPI0_TWI0_Handler */     Option::Some(unhandled_interrupt),
    /* SPI1_TWI1_Handler */     Option::Some(unhandled_interrupt),
    /* GPIOTE_Handler */        Option::Some(unhandled_interrupt),
    /* ADC_Handler */           Option::Some(unhandled_interrupt),
    /* TIMER0_Handler */        Option::Some(unhandled_interrupt),
    /* TIMER1_Handler */        Option::Some(unhandled_interrupt),
    /* TIMER2_Handler */        Option::Some(unhandled_interrupt),
    /* RTC0_Handler */          Option::Some(unhandled_interrupt),
    /* TEMP_Handler */          Option::Some(unhandled_interrupt),
    /* RNG_Handler */           Option::Some(unhandled_interrupt),
    /* ECB_Handler */           Option::Some(unhandled_interrupt),
    /* CCM_AAR_Handler */       Option::Some(unhandled_interrupt),
    /* WDT_Handler */           Option::Some(unhandled_interrupt),
    /* RTC1_Handler */          Option::Some(unhandled_interrupt),
    /* QDEC_Handler */          Option::Some(unhandled_interrupt),
    /* LPCOMP_Handler */        Option::Some(unhandled_interrupt),
    /* SWI0_Handler */          Option::Some(unhandled_interrupt),
    /* SWI1_Handler */          Option::Some(unhandled_interrupt),
    /* SWI2_Handler */          Option::Some(unhandled_interrupt),
    /* SWI3_Handler */          Option::Some(unhandled_interrupt),
    /* SWI4_Handler */          Option::Some(unhandled_interrupt),
    /* SWI5_Handler */          Option::Some(unhandled_interrupt),
];

pub unsafe fn init() {
    /* Apply early initialization workarounds for anomalies documented on
     * nRF51822-PAN v2.4. Note that they have been validated only for xxAA
     * variant. For other variants, please refer to the applicable
     * nRF51822-PAN. */

    /* Power on RAM blocks manually (PAN #16). Note that xxAA/xxAB variants
     * have only two RAM blocks. For xxAC, change to 0x0000000F. */
    //*((uint32_t volatile * ) 0x40000524) = 0x00000003;
    write_volatile(0x40000524, 0x00000003);

    /* Setup peripherals manually (PAN #26) */
    //*((uint32_t volatile * ) 0x40000504) = 0xC007FFDF;
    //*((uint32_t volatile * ) 0x40006C18) = 0x00008000;
    write_volatile(0x40000504, 0xC007FFDF);
    write_volatile(0x40006C18, 0x00008000);


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
}
