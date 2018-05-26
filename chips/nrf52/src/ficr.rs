//! Factory Information Configuration Registers (FICR)
//!
//! Factory information configuration registers (FICR) are pre-programmed in
//! factory and cannot be erased by the user. These registers contain
//! chip-specific information and configuration.
//!
//! - Author: Pat Pannuto <ppannuto@berkeley.edu>
//! - Date: November 27, 2017

use core::fmt;
use kernel::common::regs::ReadOnly;

const FICR_BASE_ADDRESS: usize = 0x10000000;

/// Struct of the FICR registers
///
/// Section 13.1 of <http://infocenter.nordicsemi.com/pdf/nRF52832_PS_v1.0.pdf>
#[repr(C)]
struct FicrRegisters {
    /// Reserved
    _reserved0: [u32; 4],
    /// Code memory page size
    /// - Address: 0x010 - 0x014
    codepagesize: ReadOnly<u32, CodePageSize::Register>,
    /// Code memory size
    /// - Address: 0x014 - 0x018
    codesize: ReadOnly<u32, CodeSize::Register>,
    /// Reserved
    _reserved1: [u32; 18],
    /// Device identifier
    /// - Address: 0x060 - 0x064
    deviceid0: ReadOnly<u32, DeviceId0::Register>,
    /// Device identifier
    /// - Address: 0x064 - 0x068
    deviceid1: ReadOnly<u32, DeviceId1::Register>,
    /// Reserved
    _reserved2: [u32; 6],
    /// Encryption Root
    /// - Address: 0x080 - 0x090
    er: [ReadOnly<u32, EncryptionRoot::Register>; 4],
    /// Identity Root
    /// - Address: 0x090 - 0x0A0
    ir: [ReadOnly<u32, IdentityRoot::Register>; 4],
    /// Device address type
    /// - Address: 0x0A0 - 0x0A4
    deviceaddrtype: ReadOnly<u32, DeviceAddressType::Register>,
    /// Device address
    /// - Address: 0x0A4 - 0x0A8
    deviceaddr0: ReadOnly<u32, DeviceAddress0::Register>,
    /// Device address
    /// - Address: 0x0A8 - 0x0AC
    deviceaddr1: ReadOnly<u32, DeviceAddress1::Register>,
    /// Reserved
    _reserved3: [u32; 21],
    /// Part code
    /// - Address: 0x100 - 0x104
    info_part: ReadOnly<u32, InfoPart::Register>,
    /// Part Variant, Hardware version and Production configuration
    /// - Address: 0x104 - 0x108
    info_variant: ReadOnly<u32, InfoVariant::Register>,
    /// Package option
    /// - Address: 0x108 - 0x10C
    info_package: ReadOnly<u32, InfoPackage::Register>,
    /// RAM variant
    /// - Address: 0x10C - 0x110
    info_ram: ReadOnly<u32, InfoRam::Register>,
    /// Flash variant
    /// - Address: 0x110 - 0x114
    info_flash: ReadOnly<u32, InfoFlash::Register>,
}

register_bitfields! [u32,
    /// Code memory page size
    CodePageSize [
        /// Code memory page size
        CODEPAGESIZE OFFSET(0) NUMBITS(32)
    ],
    /// Code memory size
    CodeSize [
        /// Code memory size in number of pages
        CODESIZE OFFSET(0) NUMBITS(32)
    ],
    /// Device Identifier
    DeviceId0 [
        /// 32 LSB of 64 bit unique device identifier
        DEVICEID OFFSET(0) NUMBITS(32)
    ],
    /// Device Identifier
    DeviceId1 [
        /// 32 MSB of 64 bit unique device identifier
        DEVICEID OFFSET(0) NUMBITS(32)
    ],
    /// Encryption Root
    EncryptionRoot [
        /// Encryption Root, word n
        ER OFFSET(0) NUMBITS(32)
    ],
    /// Identity Root
    IdentityRoot [
        /// Identity Root, word n
        IR OFFSET(0) NUMBITS(32)
    ],
    /// Device address type
    DeviceAddressType [
        /// Device address type
        DEVICEADDRESSTYPE OFFSET(0) NUMBITS(1) [
            /// Public
            PUBLIC = 0,
            /// Random
            RANDOM = 1
        ]
    ],
    /// Device address 1
    DeviceAddress0 [
        /// 32 LSB of 48 bit device address
        DEVICEADDRESS OFFSET(0) NUMBITS(32)
    ],
    /// Device address 2
    DeviceAddress1 [
        /// 16 MSB of 48 bit device address
        DEVICEADDRESS OFFSET(0) NUMBITS(16)
    ],
    /// Part code
    InfoPart [
        PART OFFSET(0) NUMBITS(32) [
            /// nRF52838
            N52832 = 0x52832,
            /// Unspecified
            #[allow(overflowing_literals)]
            Unspecified = 0xffffffff
        ]
    ],
    /// Part Variant, Hardware version and Production configuration
    InfoVariant [
        /// Part Variant, Hardware version and Production configuration, encoded as ASCII
        // Note, some of these are not present in datasheet but is in nrf52.svd
        VARIANT OFFSET(0) NUMBITS(32) [
            /// AAAA
            AAAA = 0x41414141,
            /// AAAB
            AAAB = 0x41414142,
            /// AABA
            AABA = 0x41414241,
            /// AABB
            AABB = 0x41414242,
            /// AAB0
            AAB0 = 0x41414230,
            /// AAE0
            AAE0 = 0x41414530,
            /// Unspecified
            #[allow(overflowing_literals)]
            Unspecified = 0xffffffff
        ]
    ],
    /// Package option
    // Note, some of these are not present in datasheet but is in nrf52.svd
    InfoPackage [
        PACKAGE OFFSET(0) NUMBITS(32) [
            /// QFxx - 48-pin QFN
            QF = 0x2000,
            /// CHxx - 7x8 WLCSP 56 balls
            CH = 0x2001,
            /// CIxx - 7x8 WLCSP 56 balls<
            CI = 0x2002,
            /// CKxx - 7x8 WLCSP 56 balls with backside coating for light protection
            CK = 0x2005,
            /// Unspecified
            #[allow(overflowing_literals)]
            Unspecified = 0xffffffff
        ]
    ],
    /// RAM variant
    InfoRam [
        RAM OFFSET(0) NUMBITS(32) [
            /// 16 kByte RAM
            K16 = 0x10,
            /// 32 kByte RAM
            K32 = 0x20,
            /// 64 kByte RAM
            K64 = 0x40,
            #[allow(overflowing_literals)]
            Unspecified = 0xffffffff

        ]
    ],
    /// Flash
    InfoFlash [
        FLASH OFFSET(0) NUMBITS(32) [
            /// 128 kByte FLASH
            K128 = 0x80,
            /// 256 kByte FLASH
            K256 = 0x100,
            /// 512 kByte FLASH
            K512 = 0x200,
            /// Unspecified
            #[allow(overflowing_literals)]
            Unspecified = 0xffffffff
        ]
    ]
];

/// Variant describes part variant, hardware version, and production configuration.
#[derive(PartialEq, Debug)]
#[repr(u32)]
enum Variant {
    AAAA = 0x41414141,
    AAAB = 0x41414142,
    AABA = 0x41414241,
    AABB = 0x41414242,
    AAB0 = 0x41414230,
    AAE0 = 0x41414530,
    Unspecified = 0xffffffff,
}

#[derive(PartialEq, Debug)]
#[repr(u32)]
enum Part {
    N52832 = 0x52832,
    Unspecified = 0xffffffff,
}

#[derive(PartialEq, Debug)]
#[repr(u32)]
enum Package {
    QF = 0x2000,
    CH = 0x2001,
    CI = 0x2002,
    CK = 0x2005,
    Unspecified = 0xffffffff,
}

#[derive(PartialEq, Debug)]
#[repr(u32)]
enum Ram {
    K16 = 0x10,
    K32 = 0x20,
    K64 = 0x40,
    Unspecified = 0xffffffff,
}

#[derive(Debug)]
#[repr(u32)]
enum Flash {
    K128 = 0x80,
    K256 = 0x100,
    K512 = 0x200,
    Unspecified = 0xffffffff,
}

pub struct Ficr {
    registers: *const FicrRegisters,
}

impl Ficr {
    const fn new(base_addr: usize) -> Ficr {
        Ficr {
            registers: base_addr as *const FicrRegisters,
        }
    }

    fn part(&self) -> Part {
        let regs = unsafe { &*self.registers };
        match regs.info_part.get() {
            0x52832 => Part::N52832,
            _ => Part::Unspecified,
        }
    }

    fn variant(&self) -> Variant {
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

    fn package(&self) -> Package {
        let regs = unsafe { &*self.registers };
        match regs.info_package.get() {
            0x2000 => Package::QF,
            0x2001 => Package::CH,
            0x2002 => Package::CI,
            0x2005 => Package::CK,
            _ => Package::Unspecified,
        }
    }

    fn ram(&self) -> Ram {
        let regs = unsafe { &*self.registers };
        match regs.info_ram.get() {
            0x10 => Ram::K16,
            0x20 => Ram::K32,
            0x40 => Ram::K64,
            _ => Ram::Unspecified,
        }
    }

    fn flash(&self) -> Flash {
        let regs = unsafe { &*self.registers };
        match regs.info_flash.get() {
            0x80 => Flash::K128,
            0x100 => Flash::K256,
            0x200 => Flash::K512,
            _ => Flash::Unspecified,
        }
    }
}

impl fmt::Display for Ficr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NRF52-DK HW INFO: Variant: {:?}, Part: {:?}, Package: {:?}, Ram: {:?}, Flash: {:?}",
            self.variant(),
            self.part(),
            self.package(),
            self.ram(),
            self.flash()
        )
    }
}

/// Static instance for the board. Only one (read-only) set of factory registers.
pub static mut FICR_INSTANCE: Ficr = Ficr::new(FICR_BASE_ADDRESS);
