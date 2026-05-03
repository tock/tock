// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Cortex-M synchronization implementation for Rust when using DMA.

use kernel::platform::dma_fence::DmaFence;

#[derive(Debug, Copy, Clone)]
pub struct CortexMDmaFence {
    _private: (),
}

/// An implementation of [DmaFence] for ARM Cortex-M systems.
///
/// The provided `release` and `acquire` methods use opaque assembly
/// blocks and the THUMB `DMB` instructions to make prior writes to
/// shared buffers visible to DMA devices, and DMA writes visible
/// subsequent memory reads, as specified in the ARM Cortex-M
/// Programming Guide to Memory Barrier Instructions [1].
///
/// [1]: https://developer.arm.com/documentation/dai0321/a/
impl CortexMDmaFence {
    /// Construct a new [CortexMDmaFence].
    ///
    /// # Safety
    ///
    /// Users of this function guarantee that this fence is an appropriate
    /// implementation of [`DmaFence`] for the platform on which this code is
    /// running. In practice, this means that users must assert to be running on
    /// an ARM Cortex-M (ARM-v6m / ARM-v7m) CPU.
    pub unsafe fn new() -> Self {
        Self { _private: () }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl DmaFence for CortexMDmaFence {
    fn release<T>(self, slice_ptr: *mut [T]) {
        let slice_start_ptr: *mut T = slice_ptr.cast();
        unsafe {
            core::arch::asm!(
                "
    // This block is opaque to the compiler; the compiler must assume
    // that the block could read to the entire buffer from which the
    // pointer stored in {dma_buffer_ptr_reg} was derived.
    //
    // Do not reorder prior memory reads or writes over subsequent
    // I/O reads or writes.
    dmb
                ",
                dma_buffer_ptr_reg = in(reg) slice_start_ptr,
            );
        }
    }

    fn acquire<T>(self, slice_ptr: *mut [T]) {
        let slice_start_ptr: *mut T = slice_ptr.cast();
        unsafe {
            core::arch::asm!(
                "
    // This block is opaque to the compiler; the compiler must assume
    // that the block could write to the entire buffer from which the
    // pointer stored in {dma_buffer_ptr_reg} was derived.
    //
    // Do not reorder prior I/O reads or writes over subsequent
    // memory reads or writes.
    dmb
                ",
                dma_buffer_ptr_reg = in(reg) slice_start_ptr,
            );
        }
    }
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
unsafe impl DmaFence for CortexMDmaFence {
    fn release<T>(self, _buf: *mut [T]) {
        // When building for another architecture, such as for tests or CI:
        unimplemented!("CortexMDmaFence can only be used on cortex-m targets");
    }

    fn acquire<T>(self, _buf: *mut [T]) {
        // When building for another architecture, such as for tests or CI:
        unimplemented!("CortexMDmaFence can only be used on cortex-m targets");
    }
}
