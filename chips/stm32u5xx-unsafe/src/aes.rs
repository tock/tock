// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! AES driver, stm32u5xx-family, unsafe code
use cortexm33::dma_fence::CortexMDmaFence;
use kernel::hil::symmetric_encryption::AES_BLOCK_SIZE;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

pub const AES_BASE: StaticRef<AesRegisters> =
    unsafe { StaticRef::new(0x520C0000 as *const AesRegisters) };

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

pub struct DMABuffers {
    pub dma_in_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    pub dma_out_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    pub dma_aad_buff: OptionalCell<[u8; AES_BLOCK_SIZE]>,
    pub dma_message_buff: OptionalCell<[u8; AES_BLOCK_SIZE]>,
}

/// Wrapper for managing MMIO for the AES peripheral.
pub struct AesRegistersManager {
    /// MMIO registers for the AES peripheral.
    pub registers: StaticRef<AesRegisters>,
}

impl AesRegistersManager {
    /// ### Safety
    ///
    /// The caller must ensure that the provided `StaticRef` points to a valid
    /// memory-mapped AES peripheral and that no other part of the system is
    /// conflicting with its register access.
    pub unsafe fn new(regs: StaticRef<AesRegisters>) -> Self {
        Self { registers: regs }
    }
}
impl DMABuffers {
    pub const fn new() -> Self {
        Self {
            dma_in_buf: MapCell::empty(),
            dma_out_buf: MapCell::empty(),
            dma_aad_buff: OptionalCell::empty(),
            dma_message_buff: OptionalCell::empty(),
        }
    }
    /// Helper function to take the dma_in_buf as a normal [u8]. If there is no dma_in_buf,
    /// will return None
    pub fn take_dma_in_buf(&self) -> Option<&'static mut [u8]> {
        self.dma_in_buf.take().map(|s| {
            // ### Safety
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
    pub fn take_dma_out_buf(&self) -> Option<&'static mut [u8]> {
        self.dma_out_buf.take().map(|s| {
            // ### Safety
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
    /// necessary memory barriers for safe DMA transfer.
    pub fn setup_dma_buf(
        buf: &'static mut [u8],
        start: usize,
        len: usize,
    ) -> (DmaSubSliceMut<'static, u8>, u32) {
        let mut subslice = SubSliceMut::new(buf);
        subslice.slice(start..start + len);
        // ### Safety
        //
        // This creates a new DMA fence to ensure that all previous CPU
        // writes to the buffer are visible to the DMA engine before the
        // transfer starts.
        let fence = unsafe { CortexMDmaFence::new() };
        let dma_slice = DmaSubSliceMut::new_static(subslice, fence);
        let ptr = dma_slice.as_mut_ptr() as u32;
        (dma_slice, ptr)
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
