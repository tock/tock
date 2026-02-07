// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2026.
// Copyright Tock Contributors 2026.

use kernel::platform::dma_fence::DmaFence;

/// An implementation of [`DmaFence`] for x86 CPUs.
///
/// This implementation assumes that all DMA peripherals are cache-coherent
/// w.r.t. the CPU.
///
/// The provided `release` and `acquire` methods use opaque assembly blocks to
/// make prior writes to DMA buffers visible to DMA devices, and DMA writes
/// visible to prior memory reads, respectively.
///
/// This implementation is insufficient when used for write-combining (WC)
/// memory (e.g., for video buffers or specific non-coherent regions mapped via
/// PAT).
#[derive(Debug, Copy, Clone)]
pub struct X86DmaFence {
    _private: (),
}

impl X86DmaFence {
    /// Construct a new [`X86DmaFence`].
    ///
    /// Refer to the [type-level documentation](Self) and the documentation of
    /// the [`DmaFence` trait](DmaFence) and [its implementation for
    /// `X86DmaFence`](<X86DmaFence as DmaFence>) for more details.
    ///
    /// # Safety
    ///
    /// This [`X86DmaFence`] implementation is insufficient when used for
    /// write-combining (WC) memory. By using `unsafe`, callers of this function
    /// promise that the resulting instance is not used to fence DMA memory
    /// accesses over write-combining memory.
    pub unsafe fn new() -> Self {
        X86DmaFence { _private: () }
    }
}

unsafe impl DmaFence for X86DmaFence {
    /// Expose prior writes to in-memory buffers to subsequent DMA operations.
    ///
    /// This assembly block ensures that neither the compiler, not the CPU
    /// reorder any memory writes beyond the point at which a subsequent memory
    /// or I/O access is made (e.g., to start a DMA transaction).
    ///
    /// Conventionally, we'd use the built-in `core::sync::atomic::fence` for
    /// this, but that explicitly cannot be used to establish synchronization
    /// among non-atomic accesses.
    ///
    /// Instead, to deal with any potential compiler re-ordering, we use an
    /// `asm!()` that does **not** have the `nomem` clobber set. This block is
    /// opaque to the compiler. We further explicitly pass in a pointer
    /// originating from, and thus carrying provenance of our DMA buffer. This
    /// should be sufficient to make the compiler assume that this function may
    /// read the entire DMA buffer, and thus cause it to commit all pending
    /// writes before this `asm!()` block.
    ///
    /// We expect that x86 CPUs do not perform any observable hardware
    /// re-ordering, and that all DMA access are coherent with CPU instructions
    /// accessing memory.
    #[inline(always)]
    fn release<T>(self, buf: *mut [T]) {
        if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
            unsafe {
                core::arch::asm!(
                    "
                        // This block is opaque to the compiler; it must assume
                        // that it could read the entire buffer from which the
                        // pointer stored in {dma_buffer_ptr_reg} was derived.
                    ",
                    dma_buffer_ptr_reg = in(reg) buf as *mut T,
                );
            }
        } else {
            // When building for another architecture, such as for tests or CI:
            unimplemented!("X86DmaFence can only be used on x86 targets");
        }
    }

    /// Expose prior writes by DMA peripherals to subsequent memory reads.
    ///
    /// This assembly block ensures that neither the compiler, not the CPU
    /// reorder any memory reads before the point at which a subsequent
    /// memory or I/O access is made (e.g., to start a DMA transaction).
    ///
    /// Conventionally, we'd use the built-in `core::sync::atomic::fence` for
    /// this, but that explicitly cannot be used to establish synchronization
    /// among non-atomic accesses.
    ///
    /// Instead, to deal with any potential compiler re-ordering, we use an
    /// `asm!()` that does **not** have the `nomem` clobber set. This block
    /// is opaque to the compiler. We further explicitly pass in a pointer
    /// originating from, and thus carrying provenance of our DMA
    /// buffer. This should be sufficient to make the compiler assume that
    /// this function may write to the entire DMA buffer, and thus prevent it
    /// from moving reads to before this `asm!()` block.
    ///
    /// We expect that x86 CPUs do not perform any observable hardware
    /// re-ordering, and that all DMA access are coherent with CPU instructions
    /// accessing memory.
    #[inline(always)]
    fn acquire<T>(self, buf: *mut [T]) {
        if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
            unsafe {
                core::arch::asm!(
                    "
                        // This block is opaque to the compiler; it must assume
                        // that it could write to the entire buffer from which
                        // the pointer stored in {dma_buffer_ptr_reg} was
                        // derived.
                    ",
                    dma_buffer_ptr_reg = in(reg) buf as *mut T,
                );
            }
        } else {
            // When building for another architecture, such as for tests or CI:
            unimplemented!("X86DmaFence can only be used on x86 targets");
        }
    }
}
