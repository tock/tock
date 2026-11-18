// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Core low-level operations.

use crate::csr::{CSR, mstatus::mstatus};

#[cfg(riscv)]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    use core::arch::asm;
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(riscv)]
#[inline(always)]
/// Wait For Interrupt (WFI) instruction.
pub unsafe fn wfi() {
    use core::arch::asm;
    asm!("wfi", options(nomem, nostack));
}

/// Single-core critical section operation
pub fn with_interrupts_disabled<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Read the mstatus MIE field and disable machine mode interrupts
    // atomically
    //
    // The result will be the original value of [`mstatus::mie`],
    // shifted to the proper position in [`mstatus`].
    let original_mie: usize = CSR
        .mstatus
        .read_and_clear_bits(mstatus::mie.mask << mstatus::mie.shift)
        & mstatus::mie.mask << mstatus::mie.shift;

    // Machine mode interrupts are disabled, execute the (uninterruptible)
    // function
    let res = f();

    // If [`mstatus::mie`] was set before, set it again. Otherwise,
    // this function will be a nop.
    CSR.mstatus.read_and_set_bits(original_mie);

    res
}

// Mock implementations for tests on Travis-CI.
#[cfg(not(riscv))]
pub fn nop() {
    unimplemented!()
}

#[cfg(not(riscv))]
pub unsafe fn wfi() {
    unimplemented!()
}

#[cfg(riscv)]
// Hidden macro declaration, publicly re-exported as a proper module-hierarchy
// item through the `pub use ... as linker_region_slice_ptr` below.
#[macro_export]
#[doc(hidden)]
macro_rules! __linker_region_slice_ptr {
    ($start_sym:literal, $end_sym:literal) => {{
        // We force-assign to a `&'static str` const to ensure that we reject
        // any non-string literals:
        const _START_SYM: &'static str = $start_sym;
        const _END_SYM: &'static str = $end_sym;

        // The below assembly loads the symbol addresses of `$start_sym` and
        // `$end_sym` into `start` and `end` respectively.
        let start: *const ();
        let end: usize;

        // SAFETY: this RISC-V assembly is guaranteed to run only on RISC-V
        // targets, loads two constants into two output registers, and has no
        // other side effects. It cannot fault at runtime; at worst it results
        // in a link-time error if the specified symbols cannot be resolved.
        //
        // This assembly is allowed to use linker relaxation, and it may rely on
        // `gp` being initialized correctly. It can only be called from Rust,
        // and the surrounding Rust relies on this invariant anyways.
        //
        // Can't use `#[cfg(riscv)]` here as this macro is expanded in the
        // caller's crate context.
        unsafe {
            ::core::arch::asm!(
                ::core::concat!("la {start_reg}, ", $start_sym),
                ::core::concat!("la {end_reg}, ", $end_sym),
                start_reg = out(reg) start,
                end_reg = out(reg) end,
                options(pure, nomem, nostack, preserves_flags),
            );
        }

        // Convert the `start` and (exclusive) `end` address into a raw slice
        // pointer. We use a checked subtraction to avoid silent wraparounds.
        if let ::core::option::Option::Some(len) = end.checked_sub(start.addr()) {
            // `start` comes from `asm!`, which Rust treats as opaque code:
            // the returned pointer may carry any exposed provenance. The
            // range between `$start_sym` (inclusive) and `$end_sym`
            // (exclusive) is assumed to be memory outside the Rust abstract
            // machine, which the `ptr` docs (quoted in this macro's
            // documentation) say is always considered exposed. Therefore,
            // `start` may carry valid provenance for the whole region.
            ::core::option::Option::Some(
                ::core::ptr::slice_from_raw_parts_mut(start, len)
            )
        } else {
            ::core::option::Option::None
        }
    }}
}

// On not riscv just don't do anything.
//
// This enables running `cargo check` on host to succeed.
#[cfg(not(riscv))]
#[macro_export]
#[doc(hidden)]
macro_rules! __linker_region_slice_ptr {
        // We force-assign to a `&'static str` const to ensure that we reject
        // any non-string literals:
        const _START_SYM: &'static str = $start_sym;
        const _END_SYM: &'static str = $end_sym;
    ($start_sym:literal, $end_sym:literal) => {{ ::core::option::Option::None }};
}

/// Create a raw slice pointer (`*mut [u8]`) from a memory region described by
/// start and end symbols.
///
/// This macro returns a raw slice pointer from a start symbol, describing the
/// first address of the memory range, and an end symbol at the first address
/// _past_ the memory range. The start symbol is passed as the first macro
/// argument, the end symbol is passed as the second macro argument.
///
/// The macro internally uses RISC-V assembly to load the integer addresses of
/// both symbols *without* creating any intermediate dereferenceable Rust
/// objects over any parts of the memory region. It then computes the slice
/// length through a checked subtraction of `end - start`. It returns `None` if
/// this subtraction underflows, and `Some(*mut [u8])` if it does not.
///
/// This macro is useful to interact with memory outside of the control of the
/// Rust abstract machine, such as to reference application flash or RAM, MMIO
/// regions, shared ring buffers, etc. From Rust's [documentation
/// of the `ptr` module][ptr-doc]:
///
/// > Memory which is outside the control of the Rust abstract machine (MMIO
/// > registers, for example) is always considered to be exposed, so long as
/// > this memory is disjoint from memory that will be used by the abstract
/// > machine such as the stack, heap, and statics.
///
/// # Usage
///
/// ```should_panic
/// # use riscv::__linker_region_slice_ptr as linker_region_slice_ptr;
/// let apps_flash: *mut [u8] = linker_region_slice_ptr!("_sapps", "_eapps")
///     .expect("App flash region is invalid");
/// ```
///
/// # Safety Considerations
///
/// This macro is safe to invoke, and has no strict safety invariants that
/// callers must discharge.
///
/// On RISC-V targets, non-existent symbols will result in a link-time error.
///
/// On non-RISC-V targets, this just returns `None`.
///
/// This macro does **not** guarantee that the resulting slice's length is `<=
/// isize::MAX`. Callers must establish this invariant themselves before
/// creating any Rust references over (a subset of) the memory described by the
/// resulting raw slice pointer, or performing other unsafe offset-based pointer
/// arithmetic.
///
/// The macro may rely on the `gp` register to be initialized for linker
/// relaxations. It is only callable from Rust code, and all other Rust code may
/// equivalently rely on the `gp` register to be initialized.
///
/// [ptr-doc]: https://doc.rust-lang.org/stable/core/ptr/index.html#exposed-provenance
pub use crate::__linker_region_slice_ptr as linker_region_slice_ptr;
