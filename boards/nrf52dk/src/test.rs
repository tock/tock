extern crate nrf52;

use kernel::common::VolatileCell;
use core::mem;


static mut uart_ptr: *mut u32 = nrf52::uart::UART_BASE as *mut u32;



pub fn test_rtc_regs() {
    let mut ptr: *mut u32 = nrf52::peripheral_registers::RTC1_BASE as *mut u32;
    let regs: &mut nrf52::peripheral_registers::RTC1 = unsafe { mem::transmute(ptr)};
    assert_eq!(0x40011000 as * const  VolatileCell<u32>, &regs.tasks_start as *const VolatileCell<u32>);
    assert_eq!(0x40011004 as * const  VolatileCell<u32>, &regs.tasks_stop as *const VolatileCell<u32>);
    assert_eq!(0x40011008 as * const  VolatileCell<u32>, &regs.tasks_clear as *const VolatileCell<u32>);
    assert_eq!(0x4001100c as * const  VolatileCell<u32>, &regs.tasks_trigovrflw as *const VolatileCell<u32>);
    assert_eq!(0x40011100 as * const  VolatileCell<u32>, &regs.events_tick as *const VolatileCell<u32>);
    assert_eq!(0x40011104 as * const  VolatileCell<u32>, &regs.events_ovrflw as *const VolatileCell<u32>);
    assert_eq!(0x40011140 as * const  VolatileCell<u32>, &regs.events_compare as *const VolatileCell<u32>);
    assert_eq!(0x40011304 as * const  VolatileCell<u32>, &regs.intenset as *const VolatileCell<u32>);
    assert_eq!(0x40011308 as * const  VolatileCell<u32>, &regs.intenclr as *const VolatileCell<u32>);
    assert_eq!(0x40011340 as * const  VolatileCell<u32>, &regs.evten as *const VolatileCell<u32>);
    assert_eq!(0x40011344 as * const  VolatileCell<u32>, &regs.evtenset as *const VolatileCell<u32>);
    assert_eq!(0x40011348 as * const  VolatileCell<u32>, &regs.evtenclr as *const VolatileCell<u32>);
    assert_eq!(0x40011504 as * const  VolatileCell<u32>, &regs.counter as *const VolatileCell<u32>);
    assert_eq!(0x40011508 as * const  VolatileCell<u32>, &regs.prescaler as *const VolatileCell<u32>);
    assert_eq!(0x40011540 as * const  VolatileCell<u32>, &regs.cc as *const VolatileCell<u32>);
}

/*
http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0439b/Cihfihfe.html

0xE000E004	ICTR	RO	-	Interrupt Controller Type Register, ICTR
0xE000E100 - 0xE000E11C	NVIC_ISER0 - NVIC_ISER7	RW	0x00000000	Interrupt Set-Enable Registers
0xE000E180 - 0E000xE19C	NVIC_ICER0 - NVIC_ICER7	RW	0x00000000	Interrupt Clear-Enable Registers
0xE000E200 - 0xE000E21C	NVIC_ISPR0 - NVIC_ISPR7	RW	0x00000000	Interrupt Set-Pending Registers
0xE000E280 - 0xE000E29C	NVIC_ICPR0 - NVIC_ICPR7	RW	0x00000000	Interrupt Clear-Pending Registers
0xE000E300 - 0xE000E31C	NVIC_IABR0 - NVIC_IABR7	RO	0x00000000	Interrupt Active Bit Register
0xE000E400 - 0xE000E41F	NVIC_IPR0 - NVIC_IPR59	RW	0x00000000	Interrupt Priority Register

*/

pub fn test_nvic_regs() {
    let mut ptr: *mut u32 = nrf52::nvic::NVIC_BASE as *mut u32;
    let regs: &mut nrf52::nvic::NVIC = unsafe { mem::transmute(ptr)};
    assert_eq!(0xE000E100 as * const  VolatileCell<u32>, &regs.iser as *const VolatileCell<u32>);
    assert_eq!(0xE000E180 as * const  VolatileCell<u32>, &regs.icer as *const VolatileCell<u32>);
    assert_eq!(0xE000E200 as * const  VolatileCell<u32>, &regs.ispr as *const VolatileCell<u32>);
    assert_eq!(0xE000E280 as * const  VolatileCell<u32>, &regs.icpr as *const VolatileCell<u32>);
}


#[inline(never)]
#[no_mangle]
pub fn test_uart_regs() {
    let regs: &mut nrf52::uart::Registers = unsafe { mem::transmute(uart_ptr)};
    assert_eq!(0x00 as *const VolatileCell<u32>, &regs.task_startrx as *const VolatileCell<u32>);
}

