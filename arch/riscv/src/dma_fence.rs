// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2026.
// Copyright Tock Contributors 2026.

use kernel::platform::dma_fence::DmaFence;

/// An implementation of [`DmaFence`] for RISC-V systems with cache-coherent DMA
/// memory accesses.
///
/// The provided `release` and `acquire` methods use opaque assembly blocks and
/// RISC-V `FENCE` instructions to make prior writes to DMA buffers visible to
/// DMA devices, and DMA writes visible to prior memory reads, respectively.
///
/// These primitives are sufficient to implement the `release` / `acquire`
/// semantics of [`DmaFence`] for cache-coherent platforms where all memory
/// writes written back are immediately visible to DMA devices, and all DMA
/// writes are immediately visible to CPU fetches.
///
/// For platforms where explicit cache-flush instructions are required, this
/// implementation alone is insufficient and must be extended with the necessary
/// platform-specific instructions.
#[derive(Debug, Copy, Clone)]
pub struct RiscvCoherentDmaFence {
    _private: (),
}

impl RiscvCoherentDmaFence {
    /// Construct a new [`RiscvCoherentDmaFence`].
    ///
    /// Refer to the [type-level documentation](Self) and the documentation of
    /// the [`DmaFence` trait](DmaFence) and [its implementation for
    /// `RiscvCoherentDmaFence`](<RiscvCoherentDmaFence as DmaFence>) for more
    /// details.
    ///
    /// # Safety
    ///
    /// This [`RiscvCoherentDmaFence`] implementation is insufficient for
    /// platforms with non-coherent DMA, where explicit cache-flush instructions
    /// are required. By using `unsafe`, callers of this function promise that
    /// the resulting instance is not used for non-coherent DMA mappings.
    pub unsafe fn new() -> Self {
        RiscvCoherentDmaFence { _private: () }
    }
}

unsafe impl DmaFence for RiscvCoherentDmaFence {
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
    /// To deal with any hardware re-ordering, use manually issue RISC-V fence
    /// instruction with a predecessor set including all memory writes (to make
    /// the buffer contents visible to hardware), and a successor set of all
    /// memory reads and memory writes, and I/O reads or writes. Then, all
    /// updates to the buffer are guaranteed to be written out to memory before
    /// starting a DMA operation by reading or writing an MMIO register. We
    /// include memory reads or writes in the successor set too, in case the
    /// memory containing the MMIO registers is incorrectly not mapped as I/O
    /// memory.
    ///
    /// This is only sufficient for platforms or devices with coherent DMA. As
    /// per the RISC-V unprivileged spec (version 20250508), "\[n\]on-coherent
    /// DMA may need additional synchronization (such as cache flush or
    /// invalidate mechanisms); currently any such extra synchronization will be
    /// device-specific" [1].
    ///
    /// [1]: https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
    #[inline(always)]
    fn release<T>(self, buf: *mut [T]) {
        if cfg!(any(target_arch = "riscv32", target_arch = "riscv64")) {
            unsafe {
                core::arch::asm!(
                    "
                        // This block is opaque to the compiler; it must assume
                        // that it could read the entire buffer from which the
                        // pointer stored in {dma_buffer_ptr_reg} was derived.

                        // Do not reorder prior memory writes over subsequent
                        // I/O or memory reads or writes; see above Rust source
                        // comment for explanation.
                        fence w, iorw
                    ",
                    dma_buffer_ptr_reg = in(reg) buf as *mut T,
                );
            }
        } else {
            // When building for another architecture, such as for tests or CI:
            unimplemented!("RiscvCoherentDmaFence can only be used on RISC-V targets");
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
    /// To deal with any hardware re-ordering, use manually issue RISC-V
    /// fence instruction with a predecessor set including all memory reads
    /// and I/O reads (to ensure that the DMA data is only read _after_ a
    /// prior status register or in-memory descriptor indicates that the DMA
    /// data is ready, and a successor set of all memory reads. This prevents
    /// the CPU from issuing read instructions to the DMA buffer _before_ a
    /// prior read confirmed that the data was ready.
    ///
    /// This is only sufficient for platforms or devices with coherent DMA. As
    /// per the RISC-V unprivileged spec (version 20250508), "\[n\]on-coherent
    /// DMA may need additional synchronization (such as cache flush or
    /// invalidate mechanisms); currently any such extra synchronization will be
    /// device-specific" [1].
    ///
    /// [1]: https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
    #[inline(always)]
    fn acquire<T>(self, buf: *mut [T]) {
        if cfg!(any(target_arch = "riscv32", target_arch = "riscv64")) {
            unsafe {
                core::arch::asm!(
                    "
                        // This block is opaque to the compiler; it must assume
                        // that it could write to the entire buffer from which
                        // the pointer stored in {dma_buffer_ptr_reg} was
                        // derived.

                        // Do not reorder subsequent memory reads over prior I/O
                        // or memory reads; see above Rust source comment for
                        // explanation.
                        fence ir, r
                    ",
                    dma_buffer_ptr_reg = in(reg) buf as *mut T,
                );
            }
        } else {
            // When building for another architecture, such as for tests or CI:
            unimplemented!("RiscvCoherentDmaFence can only be used on RISC-V targets");
        }
    }
}
