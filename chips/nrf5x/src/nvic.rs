use core::mem;
use kernel::common::VolatileCell;
use peripheral_interrupts::NvicIdx;

const NVIC_BASE: usize = 0xE000E100;
#[repr(C, packed)]
struct NVIC {
    pub iser: [VolatileCell<u32>; 7],
    _reserved1: [u32; 25],
    pub icer: [VolatileCell<u32>; 7],
    _reserved2: [u32; 25],
    pub ispr: [VolatileCell<u32>; 7],
    _reserved3: [u32; 25],
    pub icpr: [VolatileCell<u32>; 7],
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
