// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2026.
// Copyright Tock Contributors 2026.

//! DMA fence synchronization primitives for sharing memory with DMA
//! peripherals.

/// Synchronization primitives for safely sharing memory with DMA peripherals.
///
/// An implementation of _acquire_ and _release_ memory fence operations to
/// expose memory reads and writes by Rust and DMA peripherals to each other.
/// These operations are from the perspective of Rust and the Tock kernel: a
/// memory buffer is _released_ to the DMA peripheral, and then after the DMA
/// operation is fully completed, _aquired_ back from the DMA peripheral.
///
/// When starting a DMA operation over a buffer prepared from Rust, it is
/// important that the buffer's current contents are actually observable by the
/// DMA hardware. Similarly, when a DMA operation is finished, we must ensure
/// that Rust can see the latest buffer contents, as written by a DMA
/// peripheral. However, instruction reordering by both the compiler, hardware,
/// and non cache-coherent platforms complicate this story. These optimizations
/// can mean that a write from within Rust may not be visible to a DMA
/// peripheral, or a write performed by a DMA peripheral may not be visible to
/// Rust.
///
/// This trait provides [`acquire`](Self::acquire) and
/// [`release`](Self::release) memory fences that recover these guarantees for
/// DMA buffers in the presence of compiler reordering and, if present on the
/// target platform, hardware reordering or non-coherent caches.
///
/// Ordinarily, we'd use the built-in [`core::sync::atomic::fence`] for this,
/// but that explicitly cannot be used to establish synchronization among
/// non-atomic accesses. Additionally, certain platforms require
/// platform-specific instructions to synchronize memory: for instance, the
/// RISC-V unprivileged spec (version 20250508) states that "\[n\]on-coherent
/// DMA may need additional synchronization (such as cache flush or invalidate
/// mechanisms); currently any such extra synchronization will be
/// device-specific" [1]. Therefore, Tock uses a DMA-specific trait implemented
/// by its target architecture and platform crates.
///
/// [1]: https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
///
/// # Implementations
///
/// Implementations may assume this trait is only used for DMA peripherals,
/// where hardware has access to memory separate from normal load and store
/// instructions executed on the CPU. Implementations do not need to provide
/// any synchronization between loads and stores, for example to support
/// multi-core execution. Implementations must correctly synchronize for all
/// possible DMA operations on the chip.
///
/// Implementations may use any chip-specific DMA synchronization features that
/// may exist on a particular microcontroller.
///
/// # Safety
///
/// This is an `unsafe` trait, as users of it rely on correct
/// [`acquire`](Self::acquire) and [`release`](Self::acquire) implementations to
/// maintain soundness. Specifically, an incorrect [`acquire`](Self::acquire)
/// operation could cause DMA-issued writes to memory to be visible to Rust only
/// *after* a shared or immutable reference to this buffer is made accessible,
/// which effectively violates Rust's no-alias assumptions.
pub unsafe trait DmaFence: core::fmt::Debug + Send + Sync + Copy {
    /// Expose prior writes to in-memory buffers to subsequent DMA operations.
    ///
    /// Specifically, this function must ensure that any writes from Rust to the
    /// buffer described by `ptr` and `len` _before_ this function, are visible
    /// to any DMA operations initiated by an MMIO read or write operation
    /// _after_ this function returns.
    fn release<T>(self, buf: *mut [T]);

    /// Expose prior writes by DMA peripherals to subsequent memory reads.
    ///
    /// Specifically, this function must ensure that any reads from Rust to the
    /// buffer described by `ptr` and `len` _after_ this function returns
    /// reflect all writes made by DMA operations finished _before_ this
    /// function ran. Implementations can assume that this function is called
    /// _after_ the program observed that the DMA operation finished, by reading
    /// a status field through an MMIO or memory read.
    fn acquire<T>(self, buf: *mut [T]);
}
