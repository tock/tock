use kernel::common::volatile_cell::VolatileCell;

#[repr(C, packed)]
struct BpmRegisters {
    interrupt_enable: VolatileCell<u32>,
    interrupt_disable: VolatileCell<u32>,
    interrupt_mask: VolatileCell<u32>,
    interrupt_status: VolatileCell<u32>,
    interrupt_clear: VolatileCell<u32>,
    status: VolatileCell<u32>,
    unlock: VolatileCell<u32>,
    control: VolatileCell<u32>,
    _reserved0: [u32; 2],
    backup_wake_cause: VolatileCell<u32>,
    backup_wake_enable: VolatileCell<u32>,
    backup_pin_mux: VolatileCell<u32>,
    io_retention: VolatileCell<u32>,
}

const BPM_BASE: usize = 0x400F0000;
const BPM_UNLOCK_KEY: u32 = 0xAA000000;

static mut BPM: *mut BpmRegisters = BPM_BASE as *mut BpmRegisters;

pub enum CK32Source {
    OSC32K = 0,
    RC32K = 1,
}

#[inline(never)]
pub unsafe fn set_ck32source(source: CK32Source) {
    let control = (*BPM).control.get();
    unlock_register(0x1c); // Control
    (*BPM).control.set(control | (source as u32) << 16);
}

unsafe fn unlock_register(register_offset: u32) {
    (*BPM).unlock.set(BPM_UNLOCK_KEY | register_offset);
}
