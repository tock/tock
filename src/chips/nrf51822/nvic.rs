use helpers::*;
use core::intrinsics;

use core::mem;
use common::VolatileCell;
//use peripheral_interrupts::NvicIdx;

const NVIC_BASE: usize = 0xE000E100;
struct NVIC {
    pub iser: [VolatileCell<u32>; 7],
    _reserved1: [u32; 25],
    pub icer: [VolatileCell<u32>; 7],
    _reserved2: [u32; 25],
    pub ispr: [VolatileCell<u32>; 7],
    _reserved3: [u32; 25],
    pub icpr: [VolatileCell<u32>; 7],
}


#[allow(non_camel_case_types,dead_code)]
#[derive(Copy,Clone)]
pub enum NvicIdx {
	POWER_CLOCK = 0,
	RADIO = 1,
	UART0 = 2,
	SPI0_TWI0 = 3,
	SPI1_TWI1 = 4,
	GPIOTE = 6,
	ADC = 7,
	TIMER0 = 8,
	TIMER1 = 9,
	TIMER2 = 10,
	RTC0 = 11,
	TEMP = 12,
	RNG = 13,
	ECB = 14,
	CCM_AAR = 15,
	WDT = 16,
	RTC1 = 17,
	QDEC = 18,
	LPCOMP = 19,
	SWI0 = 20,
	SWI1 = 21,
	SWI2 = 22,
	SWI3 = 23,
	SWI4 = 24,
	SWI5 = 25,
}


fn nvic() -> &'static NVIC {
    unsafe { mem::transmute(NVIC_BASE as usize) }
}

pub fn enable(signal: NvicIdx) {
    let interrupt = signal as usize;
    nvic().iser[interrupt / 32].set(1 << (interrupt & 31));
}

pub fn disable(signal: NvicIdx) {
    let interrupt = signal as usize;
    nvic().icer[interrupt / 32].set(1 << (interrupt & 31));
}

pub fn clear_pending(signal: NvicIdx) {
    let interrupt = signal as usize;
    nvic().icpr[interrupt / 32].set(1 << (interrupt & 31));
}
