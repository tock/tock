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


