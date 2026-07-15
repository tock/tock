// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! AES128 driver, nRF5X-family, unsafe code

use kernel::utilities::StaticRef;
use kernel::utilities::cells::MapCell;
use kernel::utilities::dma_slice::DmaSliceMut;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_bitfields};

#[repr(C)]
pub struct AesEcbRegisters {
    /// Start ECB block encrypt
    /// - Address 0x000 - 0x004
    task_startecb: WriteOnly<u32, Task::Register>,
    /// Abort a possible executing ECB operation
    /// - Address: 0x004 - 0x008
    task_stopecb: WriteOnly<u32, Task::Register>,
    /// Reserved
    _reserved1: [u32; 62],
    /// ECB block encrypt complete
    /// - Address: 0x100 - 0x104
    pub event_endecb: ReadWrite<u32, Event::Register>,
    /// ECB block encrypt aborted because of a STOPECB task or due to an error
    /// - Address: 0x104 - 0x108
    pub event_errorecb: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved2: [u32; 127],
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    pub intenset: ReadWrite<u32, Intenset::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30c
    pub intenclr: ReadWrite<u32, Intenclr::Register>,
    /// Reserved
    _reserved3: [u32; 126],
    /// ECB block encrypt memory pointers
    /// - Address: 0x504 - 0x508
    ecbdataptr: ReadWrite<u32, EcbDataPointer::Register>,
}

register_bitfields! [u32,
    /// Start task
    pub Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    pub Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Enabled interrupt
    pub Intenset [
        ENDECB OFFSET(0) NUMBITS(1),
        ERRORECB OFFSET(1) NUMBITS(1)
    ],

    /// Disable interrupt
    pub Intenclr [
        ENDECB OFFSET(0) NUMBITS(1),
        ERRORECB OFFSET(1) NUMBITS(1)
    ],

    /// ECB block encrypt memory pointers
    EcbDataPointer [
        POINTER OFFSET(0) NUMBITS(32)
    ]
];

/// Wrapper for managing MMIO for the AES ECB peripheral.
pub struct AesEcbRegistersManager {
    /// MMIO registers for the AES ECB peripheral.
    pub registers: StaticRef<AesEcbRegisters>,
    /// Holding place for the DMA buffer while DMA is in progress.
    dma_buf: MapCell<DmaSliceMut<'static, u8>>,
}

impl AesEcbRegistersManager {
    /// Create a new AES registers manager.
    ///
    /// # Safety
    ///
    /// This is only valid on an nrf5x-based MCU. This must only be called once
    /// as having multiple interfaces to DMA registers is not safe. This must
    /// be the only way the AES DMA registers are controlled.
    pub unsafe fn new(regs: StaticRef<AesEcbRegisters>) -> Self {
        Self {
            registers: regs,
            dma_buf: MapCell::empty(),
        }
    }

    /// Start an ECB encryption with DMA.
    ///
    /// The buffer must cover the full 48-byte ECB data block:
    /// bytes 0–15 = key, bytes 16–31 = plaintext/counter, bytes 32–47 = ciphertext output.
    ///
    /// # Return
    ///
    /// `Ok(())` on successfully starting the DMA operation. `Err(())` if DMA
    /// is already busy.
    pub fn start_ecb_dma(&self, buf: &'static mut [u8]) -> Result<(), ()> {
        if self.dma_pending() {
            return Err(());
        }

        // To create a DmaFence we must trust the implementation.
        //
        // # Safety
        //
        // The architecture-provided version is correct for the nRF52.
        let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

        // Create DmaSubSliceMut for the ECB data buffer. This ensures that we
        // can soundly share it with the DMA hardware.
        let ecb_dma_slice = DmaSliceMut::new_static(buf, fence);

        // Provide the buffer pointer to the ECB hardware. The hardware expects
        // the pointer to point at byte 0 of the 48-byte data block (the key).
        // The SubSliceMut covers the full ecb_data buffer (never sliced), so
        // as_mut_ptr() correctly points to byte 0.
        self.registers
            .ecbdataptr
            .write(EcbDataPointer::POINTER.val(ecb_dma_slice.as_mut_ptr() as u32));

        // Save the DmaSubSliceMut while the DMA operation executes.
        self.dma_buf.replace(ecb_dma_slice);

        // Clear the end event and start the ECB task.
        self.registers.event_endecb.write(Event::READY::CLEAR);
        self.registers.task_startecb.write(Task::ENABLE::SET);

        Ok(())
    }

    pub fn finish_ecb_dma(&self) -> Option<&'static mut [u8]> {
        // Clear the end event before releasing the buffer.
        self.registers.event_endecb.write(Event::READY::CLEAR);

        self.dma_buf.take().map(|dma_slice| {
            // To create a DmaFence we must trust the implementation.
            //
            // # Safety
            //
            // The architecture-provided version is correct for the nRF52.
            let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

            // # Safety
            //
            // We must ensure that the DMA hardware no longer has any access to
            // this buffer. We ensure that by clearing `event_endecb` above;
            // the hardware sets this event only after completing the operation.
            unsafe { dma_slice.take(fence) }
        })
    }

    pub fn dma_pending(&self) -> bool {
        self.dma_buf.is_some()
    }
}
