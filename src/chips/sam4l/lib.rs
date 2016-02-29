#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(asm,core_intrinsics,concat_idents,const_fn)]
#![no_std]

extern crate common;
extern crate hil;
extern crate process;

mod helpers;

pub mod chip;
pub mod ast;
pub mod dma;
pub mod i2c;
pub mod spi;
pub mod nvic;
pub mod pm;
pub mod gpio;
pub mod usart;
pub mod scif;
pub mod adc;

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

extern {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();

    // Defined in src/main/main.rs
    fn main();

    // Defined in src/arch/cortex-m4/ctx_switch.S
    fn SVC_Handler();

    static mut _szero : u32;
    static mut _ezero : u32;
    static mut _etext : u32;
    static mut _srelocate : u32;
    static mut _erelocate : u32;
}

#[link_section=".vectors"]
pub static ISR_VECTOR: [Option<unsafe extern fn()>; 96] = [
    // First 16 are defined in the Cortex M4 user guide section 2.3.4

    /* Stack top */     Option::Some(_estack),
    /* Reset */         Option::Some(reset_handler),
    /* NMI */           Option::Some(unhandled_interrupt),
    /* Hard Fault */    Option::Some(hard_fault_handler),
    /* MemManage */     Option::Some(unhandled_interrupt),
    /* BusFault */      Option::Some(unhandled_interrupt),
    /* UsageFault*/     Option::Some(unhandled_interrupt),
    None, None, None, None,
    /* SVC */           Option::Some(SVC_Handler),
    /* DebugMon */      Option::Some(unhandled_interrupt),
    None,
    /* PendSV */        Option::Some(unhandled_interrupt),
    /* SysTick */       Option::Some(unhandled_interrupt),

    // Perhipheral vectors are defined by Atmel in the SAM4L datasheet section
    // 4.7.
    /* HFLASHC */       Option::Some(unhandled_interrupt),
    /* PDCA0 */         Option::Some(dma::PDCA_0_Handler),
    /* PDCA1 */         Option::Some(dma::PDCA_1_Handler),
    /* PDCA2 */         Option::Some(dma::PDCA_2_Handler),
    /* PDCA3 */         Option::Some(dma::PDCA_3_Handler),
    /* PDCA4 */         Option::Some(dma::PDCA_4_Handler),
    /* PDCA5..PDCA15 */ None, None, None, None, None, None, None, None,
                        None, None, None,
    /* CRCCU */         Option::Some(unhandled_interrupt),
    /* USBC */          Option::Some(unhandled_interrupt),
    /* PEVC_TR */       Option::Some(unhandled_interrupt),
    /* PEVC_OV */       Option::Some(unhandled_interrupt),
    /* AESA */          Option::Some(unhandled_interrupt),
    /* PM */            Option::Some(unhandled_interrupt),
    /* SCIF */          Option::Some(unhandled_interrupt),
    /* FREQM */         Option::Some(unhandled_interrupt),
    /* GPIO0 */         Option::Some(gpio::GPIO_0_Handler),
    /* GPIO1 */         Option::Some(gpio::GPIO_1_Handler),
    /* GPIO2 */         Option::Some(gpio::GPIO_2_Handler),
    /* GPIO3 */         Option::Some(gpio::GPIO_3_Handler),
    /* GPIO4 */         Option::Some(gpio::GPIO_4_Handler),
    /* GPIO5 */         Option::Some(gpio::GPIO_5_Handler),
    /* GPIO6 */         Option::Some(gpio::GPIO_6_Handler),
    /* GPIO7 */         Option::Some(gpio::GPIO_7_Handler),
    /* GPIO8 */         Option::Some(gpio::GPIO_8_Handler),
    /* GPIO9 */         Option::Some(gpio::GPIO_9_Handler),
    /* GPIO10 */        Option::Some(gpio::GPIO_10_Handler),
    /* GPIO11 */        Option::Some(gpio::GPIO_11_Handler),
    /* BPM */           Option::Some(unhandled_interrupt),
    /* BSCIF */         Option::Some(unhandled_interrupt),
    /* AST_ALARM */     Option::Some(ast::AST_ALARM_Handler),
    /* AST_PER */       Option::Some(unhandled_interrupt),
    /* AST_OVF */       Option::Some(unhandled_interrupt),
    /* AST_READY */     Option::Some(unhandled_interrupt),
    /* AST_CLKREADY */  Option::Some(unhandled_interrupt),
    /* WDT */           Option::Some(unhandled_interrupt),
    /* EIC1 */          Option::Some(unhandled_interrupt),
    /* EIC2 */          Option::Some(unhandled_interrupt),
    /* EIC3 */          Option::Some(unhandled_interrupt),
    /* EIC4 */          Option::Some(unhandled_interrupt),
    /* EIC5 */          Option::Some(unhandled_interrupt),
    /* EIC6 */          Option::Some(unhandled_interrupt),
    /* EIC7 */          Option::Some(unhandled_interrupt),
    /* EIC8 */          Option::Some(unhandled_interrupt),
    /* IISC */          Option::Some(unhandled_interrupt),
    /* SPI */           Option::Some(unhandled_interrupt),
    /* TC00 */          Option::Some(unhandled_interrupt),
    /* TC01 */          Option::Some(unhandled_interrupt),
    /* TC02 */          Option::Some(unhandled_interrupt),
    /* TC10 */          Option::Some(unhandled_interrupt),
    /* TC11 */          Option::Some(unhandled_interrupt),
    /* TC12 */          Option::Some(unhandled_interrupt),
    /* TWIM0 */         Option::Some(unhandled_interrupt),
    /* TWIS0 */         Option::Some(unhandled_interrupt),
    /* TWIM1 */         Option::Some(unhandled_interrupt),
    /* TWIS1 */         Option::Some(unhandled_interrupt),
    /* USART0 */        Option::Some(unhandled_interrupt),
    /* USART1 */        Option::Some(unhandled_interrupt),
    /* USART2 */        Option::Some(usart::USART2_Handler),
    /* USART3 */        Option::Some(usart::USART3_Handler),
    /* ADCIFE */        Option::Some(adc::ADCIFE_Handler),
    /* DACC */          Option::Some(unhandled_interrupt),
    /* ACIFC */         Option::Some(unhandled_interrupt),
    /* ABDACB */        Option::Some(unhandled_interrupt),
    /* TRNG */          Option::Some(unhandled_interrupt),
    /* PARC */          Option::Some(unhandled_interrupt),
    /* CATB */          Option::Some(unhandled_interrupt),
    None,
    /* TWIM2 */         Option::Some(i2c::twim2_interrupt),
    /* TWIM3 */         Option::Some(unhandled_interrupt),
    /* LCDCA */         Option::Some(unhandled_interrupt),
];

unsafe extern "C" fn reset_handler() {

    // Relocate data segment.
    // Assumes data starts right after text segment as specified by the linker
    // file.
    let mut pdest  = &mut _srelocate as *mut u32;
    let pend  = &mut _erelocate as *mut u32;
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

    main();
}

unsafe extern "C" fn hard_fault_handler() {
    use core::intrinsics::offset;

    let faulting_stack: *mut u32;

    asm!(
        "tst    lr, #4                      \n\
         ite    eq                          \n\
         mrseq  r0, msp                     \n\
         mrsne  r0, psp                     "
        : "={r0}"(faulting_stack)
        :
        : "r0"
        :
        );

    let stacked_r0  :u32 = *offset(faulting_stack, 0);
    let stacked_r1  :u32 = *offset(faulting_stack, 1);
    let stacked_r2  :u32 = *offset(faulting_stack, 2);
    let stacked_r3  :u32 = *offset(faulting_stack, 3);
    let stacked_r12 :u32 = *offset(faulting_stack, 4);
    let stacked_lr  :u32 = *offset(faulting_stack, 5);
    let stacked_pc  :u32 = *offset(faulting_stack, 6);
    let stacked_prs :u32 = *offset(faulting_stack, 7);

    panic!("HardFault.\n\
           \tr0  0x{:x}\n\
           \tr1  0x{:x}\n\
           \tr2  0x{:x}\n\
           \tr3  0x{:x}\n\
           \tr12 0x{:x}\n\
           \tlr  0x{:x}\n\
           \tpc  0x{:x}\n\
           \tprs 0x{:x}\n\
           ", stacked_r0, stacked_r1, stacked_r2, stacked_r3,
           stacked_r12, stacked_lr, stacked_pc, stacked_prs);
}

