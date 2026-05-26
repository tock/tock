// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};

register_structs! {
    pub AesRegisters {
        // Control register
        (0x0000 => pub cr: ReadWrite<u32, Control::Register>),

        // Status register
        (0x0004 => pub sr: ReadOnly<u32, Status::Register>),

        // Data input register
        (0x0008 => pub dinr: WriteOnly<u32, Data::Register>),

        // Data output register
        (0x000C => pub doutr: ReadOnly<u32, Data::Register>),

        // Key registers 0-3
        (0x0010 => pub keyr: [WriteOnly<u32, Data::Register>; 4]),

        // Initialization vector registers 0-3
        (0x0020 => pub ivr: [ReadWrite<u32, Data::Register>; 4]),

        // Key registers 4-7
        (0x0030 => pub keyr2: [WriteOnly<u32, Data::Register>; 4]),

        // Suspend registers (context saving)
        (0x0040 => pub suspendr: [ReadWrite<u32, Data::Register>; 8]),

        // 0x0300 - 0x0060 = 0x02A0 bytes (672 bytes / 4 = 168 u32s)
        (0x0060 => _reserved: [u32; 168]),

        // Interrupt enable register
        (0x0300 => pub intenr: ReadWrite<u32, Interrupt::Register>),

        // Interrupt status register
        (0x0304 => pub intstr: ReadOnly<u32, Interrupt::Register>),

        // Interrupt clear register
        (0x0308 => pub intclr: WriteOnly<u32, Interrupt::Register>),

        (0x030C => @END),
    }
}

register_bitfields![u32,
    /// AES Control Register (AES_CR)
    pub Control [
        /// Software Reset Writing 1 resets the peripheral logic.
        IPRST    OFFSET(31) NUMBITS(1) [],

        /// Key Mode (Normal, Wrapped, Shared)
        KMOD     OFFSET(24) NUMBITS(2) [
            Normal = 0,
            Wrapped = 1,
            Shared = 2
        ],

        /// Number of Padding Bytes for GCM/CCM
        NPBLB    OFFSET(20) NUMBITS(4) [],

        /// Key Size
        KEYSIZE  OFFSET(18) NUMBITS(1) [
            AES128 = 0,
            AES256 = 1
        ],

        /// Chaining Mode Extension (MSB for CHMOD)
        CHMOD_2  OFFSET(16) NUMBITS(1) [],

        /// GCM/CCM State Selection
        GCMPH    OFFSET(13) NUMBITS(2) [
            Init = 0,
            Header = 1,
            Payload = 2,
            Final = 3
        ],

        /// DMA Output Enable
        DMAOUTEN OFFSET(12) NUMBITS(1) [],

        /// DMA Input Enable
        DMAINEN  OFFSET(11) NUMBITS(1) [],

        /// AES Chaining Mode
        CHMOD    OFFSET(5)  NUMBITS(2) [
            ECB = 0,
            CBC = 1,
            CTR = 2,
            GCM_CCM = 3
        ],

        /// AES Operating Mode
        MODE     OFFSET(3)  NUMBITS(2) [
            Encrypt = 0,
            KeyDerivation = 1,
            Decrypt = 2,
            KeyDerivationThenDecrypt = 3
        ],

        /// Data Type (Endianness / Swapping)
        DATATYPE OFFSET(1)  NUMBITS(2) [
            None = 0,       // 32-bit (No swapping)
            HalfWord = 1,   // 16-bit (Half-word swapping)
            Byte = 2,       // 8-bit (Byte swapping)
            Bit = 3         // 1-bit (Bit swapping)
        ],

        /// AES Peripheral Enable
        EN       OFFSET(0)  NUMBITS(1) []
    ],

    /// AES Status Register (AES_SR)
    pub Status [
        /// Key Valid Flag
        KEYVALID OFFSET(7) NUMBITS(1) [],
        /// Busy Flag
        BUSY     OFFSET(3) NUMBITS(1) [],
        /// Write Error Flag
        WRERR    OFFSET(2) NUMBITS(1) [],
        /// Read Error Flag
        RDERR    OFFSET(1) NUMBITS(1) [],
        /// Computation Complete Flag
        CCF      OFFSET(0) NUMBITS(1) []
    ],

    /// AES Interrupt Register
    pub Interrupt [
        /// Key Error Interrupt
        KE      OFFSET(2) NUMBITS(1) [],
        /// Read/Write Error Interrupt
        RWE     OFFSET(1) NUMBITS(1) [],
        /// Computation Complete Interrupt
        CCI     OFFSET(0) NUMBITS(1) []
    ],

    pub Data [
        DATA OFFSET(0)   NUMBITS(32) []
    ]
];
