#![crate_name = "msp432"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, in_band_lifetimes)]
#![no_std]
#![allow(unused_doc_comments)]

use cortexm4::{generic_isr, hard_fault_handler, svc_handler, systick_handler};

pub mod sysctl;
pub mod wdt;

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
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

#[cfg(feature = "msp432p401r")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 64] = [
    generic_isr,         // Power Supply System (PSS) (0)
    generic_isr,         // Clock System (CS) (1)
    generic_isr,         // Power Control Manager (PCM) (2)
    generic_isr,         // Watchdog Timer A (WDT_A) (3)
    generic_isr,         // FPU_INT, Combined interrupt from flags in FPSCR (4)
    generic_isr,         // FLash Controller (FLCTL) (5)
    generic_isr,         // Comparator E0 (6)
    generic_isr,         // Comparator E1 (7)
    generic_isr,         // Timer A0 TA0CCTL0.CCIFG (8)
    generic_isr,         // Timer A0 TA0CCTLx.CCIFG (x = 1 to 4), TA0CTL.TAIFG (9)
    generic_isr,         // Timer A1 TA1CCTL0.CCIFG (10)
    generic_isr,         // Timer A1 TA1CCTLx.CCIFG (x = 1 to 4), TA1CTL.TAIFG (11)
    generic_isr,         // Timer A2 TA2CCTL0.CCIFG (12)
    generic_isr,         // Timer A2 TA2CCTLx.CCIFG (x = 1 to 4), TA2CTL.TAIFG (13)
    generic_isr,         // Timer A3 TA3CCTL0.CCIFG (13)
    generic_isr,         // Timer A3 TA3CCTLx.CCIFG (x = 1 to 4), TA3CTL.TAIFG (15)
    generic_isr,         // eUSCI A0 (16)
    generic_isr,         // eUSCI A1 (17)
    generic_isr,         // eUSCI A2 (18)
    generic_isr,         // eUSCI A3 (19)
    generic_isr,         // eUSCI B0 (20)
    generic_isr,         // eUSCI B1 (21)
    generic_isr,         // eUSCI B2 (22)
    generic_isr,         // eUSCI B3 (23)
    generic_isr,         // Precision ADC (24)
    generic_isr,         // Timer32 INT1 (25)
    generic_isr,         // Timer32 INT2 (26)
    generic_isr,         // Timer32 combined interrupt (27)
    generic_isr,         // AES256 (28)
    generic_isr,         // RTC_C (29)
    generic_isr,         // DMA error (30)
    generic_isr,         // DMA INT3 (31)
    generic_isr,         // DMA INT2 (32)
    generic_isr,         // DMA INT1 (33)
    generic_isr,         // DMA INT0 (34)
    generic_isr,         // IO Port 1 (35)
    generic_isr,         // IO Port 2 (36)
    generic_isr,         // IO Port 3 (37)
    generic_isr,         // IO Port 4 (38)
    generic_isr,         // IO Port 5 (39)
    generic_isr,         // IO Port 6 (40)
    unhandled_interrupt, // Reserved (41)
    unhandled_interrupt, // Reserved (42)
    unhandled_interrupt, // Reserved (43)
    unhandled_interrupt, // Reserved (44)
    unhandled_interrupt, // Reserved (45)
    unhandled_interrupt, // Reserved (46)
    unhandled_interrupt, // Reserved (47)
    unhandled_interrupt, // Reserved (48)
    unhandled_interrupt, // Reserved (49)
    unhandled_interrupt, // Reserved (50)
    unhandled_interrupt, // Reserved (51)
    unhandled_interrupt, // Reserved (52)
    unhandled_interrupt, // Reserved (53)
    unhandled_interrupt, // Reserved (54)
    unhandled_interrupt, // Reserved (55)
    unhandled_interrupt, // Reserved (56)
    unhandled_interrupt, // Reserved (57)
    unhandled_interrupt, // Reserved (58)
    unhandled_interrupt, // Reserved (59)
    unhandled_interrupt, // Reserved (60)
    unhandled_interrupt, // Reserved (61)
    unhandled_interrupt, // Reserved (62)
    unhandled_interrupt, // Reserved (63)
];

extern "C" {
    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

pub unsafe fn init() {
    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);

    cortexm4::nvic::disable_all();
    cortexm4::nvic::clear_all_pending();
    cortexm4::nvic::enable_all();
}
