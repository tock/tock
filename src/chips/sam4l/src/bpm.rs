use helpers::*;

#[repr(C, packed)]
struct BpmRegisters {
    interrupt_enable: u32,
    interrupt_disable: u32,
    interrupt_mask: u32,
    interrupt_status: u32,
    interrupt_clear: u32,
    status: u32,
    unlock: u32,
    control: u32,
    _reserved0: [u32; 2],
    backup_wake_cause: u32,
    backup_wake_enable: u32,
    backup_pin_mux: u32,
    io_retention: u32,
}

const BPM_BASE: isize = 0x400F0000;
const BPM_UNLOCK_KEY : u32 = 0xAA000000;

static mut bpm : *mut BpmRegisters = BPM_BASE as *mut BpmRegisters;

pub enum CK32Source {
    OSC32K = 0,
    RC32K = 1
}

#[inline(never)]
pub unsafe fn set_ck32source(source: CK32Source) {
    let control = volatile_load(&(*bpm).control);
    unlock_register(&(*bpm).control);
    volatile_store(&mut (*bpm).control, control | (source as u32) << 16);
}

unsafe fn unlock_register(reg: *const u32) {
    let addr = reg as u32 - bpm as u32;
    volatile_store(&mut (*bpm).unlock, BPM_UNLOCK_KEY | addr);
}

