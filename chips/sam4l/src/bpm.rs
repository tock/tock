//! Implementation of the BPM peripheral.

use kernel::common::VolatileCell;

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

/// Which power scaling mode the chip should use for internal voltages
///
/// See Tables 42-6 and 42-8 (page 1125) for information of energy usage
/// of different power scaling modes
pub enum PowerScaling {
    /// Mode 0: Default out of reset
    ///   - Maximum system clock frequency is 32MHz
    ///   - Normal flash speed
    PS0,

    /// Mode 1: Reduced voltage
    ///   - Maximum system clock frequency is 12MHz
    ///   - Normal flash speed
    ///   - These peripherals are not available in Mode 1:
    ///      - USB
    ///      - DFLL
    ///      - PLL
    ///      - Programming/Erasing Flash
    PS1,

    /// Mode 2:
    ///   - Maximum system clock frequency is 48MHz
    ///   - High speed flash
    PS2,
}

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

unsafe fn power_scaling_ok() -> bool {
    let psok_mask = 0x1;
    ((*BPM).status.get() & psok_mask) == 1
}

// This approach based on `bpm_power_scaling_cpu` from ASF
pub unsafe fn set_power_scaling(ps_value: PowerScaling) {
    // The datasheet says to spin on this before doing anything, ASF
    // doesn't as far as I can tell, but it seems like a good idea
    while !power_scaling_ok() {}

    // Read existing values
    let mut control = (*BPM).control.get();

    // Clear prior PS and set new PS
    control &= !0x3;
    control |= ps_value as u32;

    // WARN: Undocumented!
    //
    // According to the datasheet (sec 6.2, p57) changing power scaling
    // requires waiting for an interrupt (presumably because flash is
    // inaccessible during the transition). However, the ASF code sets
    // bit 3 ('PSCM' bit) of the PMCON register, which is *blank* (not a '-')
    // in the datasheet with supporting comments that this allows a change
    // 'without CPU halt'
    control |= 0x8; // PSCM: without CPU halt

    // Request power scaling change
    control |= 0x4; // PSCREQ

    // Unlock PMCON register
    unlock_register(0x1c); // Control

    // Actually change power scaling
    (*BPM).control.set(control);
}
