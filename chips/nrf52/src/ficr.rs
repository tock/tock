//! Factory Information Configuration Registers (FICR)
//!
//! Factory information configuration registers (FICR) are pre-programmed in
//! factory and cannot be erased by the user. These registers contain
//! chip-specific information and configuration.
//!
//! - Author: Pat Pannuto <ppannuto@berkeley.edu>
//! - Date: November 27, 2017

use kernel::common::VolatileCell;

/// Struct of the FICR registers
///
/// Section 13.1 of http://infocenter.nordicsemi.com/pdf/nRF52832_PS_v1.0.pdf
#[repr(C, packed)]
struct FicrRegisters {
    _reserved0: [VolatileCell<u32>; 4], // (0x10 - 0x00) / 4 = 4
    codepagesize: VolatileCell<u32>,
    codesize: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 18], // (0x60 - 0x18) / 4 = 18
    deviceid0: VolatileCell<u32>,
    deviceid1: VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 6], // (0x80 - 0x68) / 4 = 6
    er0: VolatileCell<u32>,
    er1: VolatileCell<u32>,
    er2: VolatileCell<u32>,
    er3: VolatileCell<u32>,
    ir0: VolatileCell<u32>,
    ir1: VolatileCell<u32>,
    ir2: VolatileCell<u32>,
    ir3: VolatileCell<u32>,
    deviceaddrtype: VolatileCell<u32>,
    deviceaddr0: VolatileCell<u32>,
    deviceaddr1: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 21], // (0x100 - 0xac) / 4 = 21
    info_part: VolatileCell<u32>,
    info_variant: VolatileCell<u32>,
    info_package: VolatileCell<u32>,
    info_ram: VolatileCell<u32>,
    info_flash: VolatileCell<u32>,
}
const FICR_BASE_ADDRESS: usize = 0x10000000;

/// Variant describes part variant, hardware version, and production configuration.
#[derive(PartialEq)]
#[repr(u32)]
pub enum Variant {
    AAAA = 0x41414141,
    AAAB = 0x41414142,
    AABA = 0x41414241,
    AABB = 0x41414242,
    AAB0 = 0x41414230,
    AAE0 = 0x41414530,
    Unspecified = 0xffffffff,
}

pub struct FICR {
    registers: *const FicrRegisters,
}

impl FICR {
    const fn new(base_addr: usize) -> FICR {
        FICR { registers: base_addr as *const FicrRegisters }
    }

    pub fn variant(&self) -> Variant {
        let regs = unsafe { &*self.registers };
        match regs.info_variant.get() {
            0x41414141 => Variant::AAAA,
            0x41414142 => Variant::AAAB,
            0x41414241 => Variant::AABA,
            0x41414242 => Variant::AABB,
            0x41414230 => Variant::AAB0,
            0x41414530 => Variant::AAE0,
            _ => Variant::Unspecified,
        }
    }
}

/// Static instance for the board. Only one (read-only) set of factory registers.
pub static mut FICR_INSTANCE: FICR = FICR::new(FICR_BASE_ADDRESS);
