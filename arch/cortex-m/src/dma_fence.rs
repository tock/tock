// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

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
    pub unsafe fn new() -> Self {
        Self { _private: () }
    }
}

unsafe impl DmaFence for CortexMDmaFence {
    fn release<T>(self, buf: *mut [T]) {
        if cfg!(target_arch = "arm") {
            unsafe {
                core::arch::asm!(
                    "
                        /*
                         * This block is opaque to the compiler; the
                         * compiler must assume that the block could
                         * read to the entire buffer from which the
                         * pointer stored in {dma_buffer_ptr_reg} was
                         * derived.
                         *
                         * Do not reorder prior memory reads or writes
                         * over subsequent I/O reads or writes.
                         */
                        dmb
                    ",
                    dma_buffer_ptr_reg = in(reg) buf.cast::<T>(),
                );
            }
        } else {
            // When building for another architecture, such as for tests or CI:
            unimplemented!("CortexMDmaFence can only be used on cortex-m targets");
        }
    }

    fn acquire<T>(self, buf: *mut [T]) {
        if cfg!(target_arch = "arm") {
            unsafe {
                core::arch::asm!(
                    "
                        /*
                         * This block is opaque to the compiler; the
                         * compiler must assume that the block could
                         * write to the entire buffer from which the
                         * pointer stored in {dma_buffer_ptr_reg} was
                         * derived.
                         *
                         * Do not reorder prior I/O reads or writes
                         * over subsequent memory reads or writes.
                         */
                        dmb
                    ",
                    dma_buffer_ptr_reg = in(reg) buf.cast::<T>(),
                );
            }
        } else {
            // When building for another architecture, such as for tests or CI:
            unimplemented!("CortexMDmaFence can only be used on cortex-m targets");
        }
    }
}
