// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! AES driver, stm32u5xx-family, unsafe code
use cortexm33::dma_fence::CortexMDmaFence;
use kernel::hil::symmetric_encryption::AES_BLOCK_SIZE;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{
    ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

pub const AES_BASE: StaticRef<AesRegisters> =
    unsafe { StaticRef::new(0x520C0000 as *const AesRegisters) };

register_structs! {
    pub AesRegisters {
        // Control register
        (0x0000 => cr: ReadWrite<u32, Control::Register>),

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

pub struct AesDmaBuffers {
    dma_in_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    dma_out_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    dma_aad_buf: OptionalCell<[u8; AES_BLOCK_SIZE]>,
    dma_message_buf: OptionalCell<[u8; AES_BLOCK_SIZE]>,
}

/// Wrapper for managing MMIO for the AES peripheral.
pub struct AesRegistersManager {
    /// MMIO registers for the AES peripheral.
    pub registers: StaticRef<AesRegisters>,
}

impl AesRegistersManager {
    /// # Safety
    ///
    /// The caller must ensure that the provided `StaticRef` points to a valid
    /// memory-mapped AES peripheral and that no other part of the system is
    /// conflicting with its register access.
    pub unsafe fn new(regs: StaticRef<AesRegisters>) -> Self {
        Self { registers: regs }
    }

    pub fn apply_crypto_direction(&self, encrypting: bool) {
        if encrypting {
            self.registers.cr.modify(Control::MODE::Encrypt);
        } else {
            self.registers.cr.modify(Control::MODE::Decrypt);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.registers.cr.any_matching_bits_set(Control::EN::SET)
    }

    pub fn enable(&self) {
        self.registers.cr.modify(Control::EN::SET);
    }

    pub fn set_dma(&self, both: bool) {
        if both {
            self.registers
                .cr
                .modify(Control::DMAINEN::SET + Control::DMAOUTEN::SET);
        } else {
            self.registers.cr.modify(Control::DMAINEN::SET);
        }
    }

    pub fn clear_dma(&self) {
        self.registers
            .cr
            .modify(Control::DMAINEN::CLEAR + Control::DMAOUTEN::CLEAR);
    }

    pub fn disable(&self) {
        self.registers.cr.modify(Control::EN::CLEAR);
    }

    pub fn key_preparation(&self) {
        self.registers
            .cr
            .modify(Control::MODE::KeyDerivation + Control::KMOD::Normal);
    }

    pub fn reset(&self) {
        self.registers.cr.modify(Control::IPRST::SET);
    }

    pub fn set_data_swap_byte(&self) {
        self.registers.cr.modify(Control::DATATYPE::Byte);
    }

    pub fn set_key_len(&self, length: usize) {
        if length == 16 {
            self.registers.cr.modify(Control::KEYSIZE::AES128);
        } else {
            self.registers.cr.modify(Control::KEYSIZE::AES256);
        }
    }

    pub fn set_mode_ecb(&self) {
        self.registers
            .cr
            .modify(Control::CHMOD::ECB + Control::CHMOD_2::CLEAR);
    }

    pub fn set_mode_ctr(&self) {
        self.registers
            .cr
            .modify(Control::CHMOD::CTR + Control::CHMOD_2::CLEAR);
    }

    pub fn set_mode_cbc(&self) {
        self.registers
            .cr
            .modify(Control::CHMOD::CBC + Control::CHMOD_2::CLEAR);
    }

    pub fn set_mode_ccm(&self) {
        self.registers.cr.modify(Control::CHMOD::ECB);
        self.registers.cr.modify(Control::CHMOD_2::SET);
    }

    pub fn set_mode_gcm(&self) {
        self.registers.cr.modify(Control::CHMOD::GCM_CCM);
        self.registers.cr.modify(Control::CHMOD_2::CLEAR);
    }

    pub fn set_init_phase(&self) {
        self.registers.cr.modify(Control::GCMPH::Init);
    }

    pub fn set_header_phase(&self) {
        self.registers.cr.modify(Control::GCMPH::Header);
    }

    pub fn set_payload_phase(&self) {
        self.registers.cr.modify(Control::GCMPH::Payload);
    }

    pub fn set_final_phase(&self) {
        self.registers.cr.modify(Control::GCMPH::Final);
    }

    pub fn clear_phase(&self) {
        self.registers.cr.modify(Control::GCMPH::CLEAR);
    }

    pub fn set_npblb(&self, block_len: usize) {
        self.registers
            .cr
            .modify(Control::NPBLB.val((AES_BLOCK_SIZE - block_len) as u32));
    }
}

impl AesDmaBuffers {
    pub const fn new() -> Self {
        Self {
            dma_in_buf: MapCell::empty(),
            dma_out_buf: MapCell::empty(),
            dma_aad_buf: OptionalCell::empty(),
            dma_message_buf: OptionalCell::empty(),
        }
    }

    pub fn set_aad_buffer(&self, buf: [u8; AES_BLOCK_SIZE]) {
        self.dma_aad_buf.replace(buf);
    }

    pub fn set_message_buffer(&self, buf: [u8; AES_BLOCK_SIZE]) {
        self.dma_message_buf.replace(buf);
    }

    pub fn get_aad_buf(&self) -> Option<[u8; AES_BLOCK_SIZE]> {
        self.dma_aad_buf.take()
    }

    pub fn get_message_buf(&self) -> Option<[u8; AES_BLOCK_SIZE]> {
        self.dma_message_buf.take()
    }

    /// Helper function to take the dma_in_buf as a normal [u8]. If there is no dma_in_buf,
    /// will return None
    pub fn take_dma_in_buf(&self, reg: StaticRef<AesRegisters>) -> Option<&'static mut [u8]> {
        if reg.cr.is_set(Control::DMAOUTEN) {
            reg.cr.modify(Control::DMAOUTEN::CLEAR);
        }
        if reg.cr.is_set(Control::DMAINEN) {
            reg.cr.modify(Control::DMAINEN::CLEAR);
        }
        self.dma_in_buf.take().map(|s| {
            // # Safety
            //
            // This creates a new DMA fence to ensure that all previous DMA
            // transfers have completed and memory is consistent before the
            // CPU accesses the buffer.
            let mut sub = unsafe { s.take(CortexMDmaFence::new()) };
            sub.reset();
            sub.take()
        })
    }

    /// Helper function to take the dma_out_buf as a normal [u8].
    /// If there is no dma_in_buf, will return None
    pub fn take_dma_out_buf(&self, reg: StaticRef<AesRegisters>) -> Option<&'static mut [u8]> {
        if reg.cr.is_set(Control::DMAOUTEN) {
            reg.cr.modify(Control::DMAOUTEN::CLEAR);
        }
        if reg.cr.is_set(Control::DMAINEN) {
            reg.cr.modify(Control::DMAINEN::CLEAR);
        }
        self.dma_out_buf.take().map(|s| {
            // # Safety
            //
            // This creates a new DMA fence to ensure that all previous DMA
            // transfers have completed and memory is consistent before the
            // CPU accesses the buffer.
            let mut sub = unsafe { s.take(CortexMDmaFence::new()) };
            sub.reset();
            sub.take()
        })
    }

    /// Wraps a raw buffer slice into a DmaSubSliceMut, applying the
    /// necessary memory barriers for safe DMA transfer. Stores it in dma_out_buf.
    pub fn setup_dma_out_buf(&self, buf: &'static mut [u8], start: usize, len: usize) -> u32 {
        let mut subslice = SubSliceMut::new(buf);
        subslice.slice(start..start + len);
        let fence = unsafe { CortexMDmaFence::new() };
        let dma_slice = DmaSubSliceMut::new_static(subslice, fence);
        let ptr = dma_slice.as_mut_ptr() as u32;
        self.dma_out_buf.replace(dma_slice);
        ptr
    }

    /// Wraps a raw buffer slice into a DmaSubSliceMut, applying the
    /// necessary memory barriers for safe DMA transfer. Stores it in dma_in_buf.
    pub fn setup_dma_in_buf(&self, buf: &'static mut [u8], start: usize, len: usize) -> u32 {
        let mut subslice = SubSliceMut::new(buf);
        subslice.slice(start..start + len);
        let fence = unsafe { CortexMDmaFence::new() };
        let dma_slice = DmaSubSliceMut::new_static(subslice, fence);
        let ptr = dma_slice.as_mut_ptr() as u32;
        self.dma_in_buf.replace(dma_slice);
        ptr
    }

    /// Helper function designed to calculate the length of the buffer as a multiple of AES_BLOCK_SIZE
    /// and return the remaining bytes inside a 0-padded buffer. If the length of the buffer, beginning
    /// from start is a multiple of AES_BLOCK_SIZE, will return total_len and None
    pub fn extract_dma_padding(
        buf: &[u8],
        start: usize,
        total_len: usize,
    ) -> (usize, Option<[u8; AES_BLOCK_SIZE]>) {
        // check whether the buffer needs 0-padding
        if total_len > 0 && !total_len.is_multiple_of(AES_BLOCK_SIZE) {
            // length multiple of AES_BLOCK_SIZE
            let len = total_len - (total_len % AES_BLOCK_SIZE);
            // remainder of the buffer, padded with 0s
            let mut pad = [0u8; AES_BLOCK_SIZE];
            let rem = total_len - len;
            pad[..rem].copy_from_slice(&buf[start + len..start + total_len]);
            (len, Some(pad))
        } else {
            (total_len, None)
        }
    }
}
