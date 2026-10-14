// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Verbatim copies of the "Safety" section of the official `core`/`alloc`
//! documentation for the well-known unsafe functions/methods in
//! `crate::urls`, keyed by the same final-path-segment name.
//!
//! Each entry's first line is the section's own opening sentence (used as
//! the headline next to the function's name); every following line,
//! including blank ones, is copied as rendered, indentation included, from
//! the page's "Safety" heading onward -- see
//! `tools/debugging-and-development/unsafe-requirements/README.md` for how
//! `crate::annotate` turns this back into a comment block.
//!
//! A function with no entry here either has no dedicated "Safety" section
//! in its documentation, or isn't curated yet; callers should treat `None`
//! as "unknown", not as "this operation has no safety requirements".

/// Returns the verbatim "Safety" section text for a well-known `core`/`alloc`
/// function or method, keyed by its final path segment (e.g. `"add"`,
/// `"transmute_copy"`). Returns `None` when that function's documentation
/// has no dedicated "Safety" section to copy.
pub fn text_for(name: &str) -> Option<&'static str> {
    Some(match name {
        "transmute_copy" => r#"- Requires size_of_val::<Src>(src) >= size_of::<Dst>()
- The first size_of::<Dst>() bytes behind src must be readable
- The first size_of::<Dst>() bytes behind src must be valid when interpreted as a Dst.

    On top of that, remember that most types have additional invariants beyond merely being considered initialized at the type level. For example, a 1-initialized Vec<T> is considered initialized (under the current implementation; this does not constitute a stable guarantee) because the only requirement the compiler knows about it is that the data pointer must be non-null. Creating such a Vec<T> does not cause immediate undefined behavior, but will cause undefined behavior with most safe operations (including dropping it)."#,
        "copy_nonoverlapping" => r#"Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
- dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
- Both src and dst must be properly aligned.
- The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.

    Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.

    Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned."#,
        "copy" => r#"Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
- dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0, and dst must remain valid even when src is read for count * size_of::<T>() bytes. (This means if the memory ranges overlap, the dst pointer must not be invalidated by src reads.)
- Both src and dst must be properly aligned.

    Like read, copy creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.

    Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned."#,
        "read" => r#"Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads or T must be a ZST.
- src must be properly aligned. Use read_unaligned if this is not the case.
- src must point to a properly initialized value of type T.

    Note that even if T has size 0, the pointer must be properly aligned."#,
        "read_unaligned" => r#"Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads.
- src must point to a properly initialized value of type T.

    Like read, read_unaligned creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the returned value and the value at *src can violate memory safety.

On packed structs

Attempting to create a raw pointer to an unaligned struct field with an expression such as &packed.unaligned as *const FieldType creates an intermediate unaligned reference before converting that to a raw pointer. That this reference is temporary and immediately cast is inconsequential as the compiler always expects references to be properly aligned. As a result, using &packed.unaligned as *const FieldType causes immediate undefined behavior in your program.

Instead you must use the &raw const syntax to create the pointer. You may use that constructed pointer together with this function.

An example of what not to do and how this relates to read_unaligned is:

#[repr(packed, C)]
struct Packed {
    _padding: u8,
    unaligned: u32,
}

let packed = Packed {
    _padding: 0x00,
    unaligned: 0x01020304,
};

// Take the address of a 32-bit integer which is not aligned.
// In contrast to `&packed.unaligned as *const _`, this has no undefined behavior.
let unaligned = &raw const packed.unaligned;

let v = unsafe { std::ptr::read_unaligned(unaligned) };
assert_eq!(v, 0x01020304);

Accessing unaligned fields directly with e.g. packed.unaligned is safe however."#,
        "read_volatile" => r#"Like read, read_volatile creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the returned value and the value at *src can violate memory safety. However, storing non-Copy types in volatile memory is almost certainly incorrect.

Behavior is undefined if any of the following conditions are violated:

- src must be either valid for reads, or T must be a ZST, or src must point to memory outside of all Rust allocations and reading from that memory must:

    - not trap, and
    - not cause any memory inside a Rust allocation to be modified.
- src must be properly aligned.
- Reading from src must produce a properly initialized value of type T.

    Note that even if T has size 0, the pointer must be properly aligned."#,
        "write" => r#"Behavior is undefined if any of the following conditions are violated:

- dst must be valid for writes or T must be a ZST.
- dst must be properly aligned. Use write_unaligned if this is not the case.

    Note that even if T has size 0, the pointer must be properly aligned."#,
        "write_unaligned" => r#"Behavior is undefined if any of the following conditions are violated:

- dst must be valid for writes.

On packed structs

Attempting to create a raw pointer to an unaligned struct field with an expression such as &packed.unaligned as *const FieldType creates an intermediate unaligned reference before converting that to a raw pointer. That this reference is temporary and immediately cast is inconsequential as the compiler always expects references to be properly aligned. As a result, using &packed.unaligned as *const FieldType causes immediate undefined behavior in your program.

Instead, you must use the &raw mut syntax to create the pointer. You may use that constructed pointer together with this function.

An example of how to do it and how this relates to write_unaligned is:

#[repr(packed, C)]
struct Packed {
    _padding: u8,
    unaligned: u32,
}

let mut packed: Packed = unsafe { std::mem::zeroed() };

// Take the address of a 32-bit integer which is not aligned.
// In contrast to `&packed.unaligned as *mut _`, this has no undefined behavior.
let unaligned = &raw mut packed.unaligned;

unsafe { std::ptr::write_unaligned(unaligned, 42) };

assert_eq!({packed.unaligned}, 42); // `{...}` forces copying the field instead of creating a reference.

Accessing unaligned fields directly with e.g. packed.unaligned is safe however (as can be seen in the assert_eq! above)."#,
        "write_volatile" => r#"Behavior is undefined if any of the following conditions are violated:

- dst must be either valid for writes, or T must be a ZST, or dst must point to memory outside of all Rust allocations and writing to that memory must:

    - not trap, and
    - not cause any memory inside a Rust allocation to be modified.
- dst must be properly aligned.

    Note that even if T has size 0, the pointer must be properly aligned."#,
        "write_bytes" => r#"Behavior is undefined if any of the following conditions are violated:

- dst must be valid for writes of count * size_of::<T>() bytes.
- dst must be properly aligned.

    Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointer must be properly aligned.

    Additionally, note that changing *dst in this way can easily lead to undefined behavior (UB) later if the written bytes are not a valid representation of some T. For instance, the following is an incorrect use of this function:

unsafe {
    let mut value: u8 = 0;
    let ptr: *mut bool = &mut value as *mut u8 as *mut bool;
    let _bool = ptr.read(); // This is fine, `ptr` points to a valid `bool`.
    ptr.write_bytes(42u8, 1); // This function itself does not cause UB...
    let _bool = ptr.read(); // ...but it makes this operation UB! ⚠️
}"#,
        "from_raw_parts" => r#"Behavior is undefined if any of the following conditions are violated:

- data must be non-null, valid for reads for len * size_of::<T>() many bytes, and it must be properly aligned. This means in particular:

    - The entire memory range of this slice must be contained within a single allocation! Slices can never span across multiple allocations. See below for an example incorrectly not taking this into account.
    - data must be non-null and aligned even for zero-length slices or slices of ZSTs. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
- data must point to len consecutive properly initialized values of type T.
- The memory referenced by the returned slice must not be mutated for the duration of lifetime 'a, except inside an UnsafeCell.
- The total size len * size_of::<T>() of the slice must be no larger than isize::MAX, and adding that size to data must not “wrap around” the address space. See the safety documentation of pointer::offset."#,
        "from_raw_parts_mut" => r#"Behavior is undefined if any of the following conditions are violated:

- data must be non-null, valid for both reads and writes for len * size_of::<T>() many bytes, and it must be properly aligned. This means in particular:

    - The entire memory range of this slice must be contained within a single allocation! Slices can never span across multiple allocations.
    - data must be non-null and aligned even for zero-length slices or slices of ZSTs. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
- data must point to len consecutive properly initialized values of type T.
- The memory referenced by the returned slice must not be accessed through any other pointer (not derived from the return value) for the duration of lifetime 'a. Both read and write accesses are forbidden.
- The total size len * size_of::<T>() of the slice must be no larger than isize::MAX, and adding that size to data must not “wrap around” the address space. See the safety documentation of pointer::offset."#,
        "get_unchecked" => r#"Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.

You can think of this like .get(index).unwrap_unchecked(). It’s UB to call .get_unchecked(len), even if you immediately convert to a pointer. And it’s UB to call .get_unchecked(..len + 1), .get_unchecked(..=len), or similar."#,
        "get_unchecked_mut" => r#"Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.

You can think of this like .get_mut(index).unwrap_unchecked(). It’s UB to call .get_unchecked_mut(len), even if you immediately convert to a pointer. And it’s UB to call .get_unchecked_mut(..len + 1), .get_unchecked_mut(..=len), or similar."#,
        "assume_init" => r#"It is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state, i.e., a state that is considered “valid” for type T. Calling this when the content is not yet fully initialized causes immediate undefined behavior. The type-level documentation contains more information about this initialization invariant.

It is a common mistake to assume that this function is safe to call on integers because they can hold all bit patterns. It is also a common mistake to think that calling this function is UB if any byte is uninitialized. Both of these assumptions are wrong. If that is surprising to you, please read the type-level documentation.

On top of that, remember that most types have additional invariants beyond merely being considered initialized at the type level. For example, a 1-initialized Vec<T> is considered initialized (under the current implementation; this does not constitute a stable guarantee) because the only requirement the compiler knows about it is that the data pointer must be non-null. Creating such a Vec<T> does not cause immediate undefined behavior, but will cause undefined behavior with most safe operations (including dropping it)."#,
        "assume_init_ref" => r#"Calling this when the content is not yet fully initialized causes undefined behavior: it is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state."#,
        "assume_init_mut" => r#"Calling this when the content is not yet fully initialized causes undefined behavior: it is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state. For instance, .assume_init_mut() cannot be used to initialize a MaybeUninit."#,
        "unreachable_unchecked" => r#"Reaching this function is Undefined Behavior.

As the compiler assumes that all forms of Undefined Behavior can never happen, it will eliminate all branches in the surrounding code that it can determine will invariably lead to a call to unreachable_unchecked().

If the assumptions embedded in using this function turn out to be wrong - that is, if the site which is calling unreachable_unchecked() is actually reachable at runtime - the compiler may have generated nonsensical machine instructions for this situation, including in seemingly unrelated code, causing difficult-to-debug problems.

Use this function sparingly. Consider using the unreachable! macro, which may prevent some optimizations but will safely panic in case it is actually reached at runtime. Benchmark your code to find out if using unreachable_unchecked() comes with a performance benefit."#,
        "from_utf8_unchecked" => r#"The bytes passed in must be valid UTF-8."#,
        "set_len" => r#"- new_len must be less than or equal to capacity().
- The elements at old_len..new_len must be initialized."#,
        "add" => r#"If any of the following conditions are violated, the result is Undefined Behavior:

- The offset in bytes, count * size_of::<T>(), computed on mathematical integers (without “wrapping around”), must fit in an isize.
- Let result be self.addr() + count * size_of::<T>(), computed on mathematical integers. This must fit in a usize.
- If the computed offset is non-zero, then self must be derived from a pointer to some allocation, and the entire memory range between self and result (i.e., self.addr()..result) must be in bounds of that allocation.

    Allocations can never be larger than isize::MAX bytes and they can only contain addresses representable by usize, so technically the last condition implies the first two. This implies, for instance, that vec.as_ptr().add(vec.len()) (for vec: Vec<T>) is always safe.

    Consider using wrapping_add instead if these constraints are difficult to satisfy. The only advantage of this method is that it enables more aggressive compiler optimizations."#,
        "sub" => r#"If any of the following conditions are violated, the result is Undefined Behavior:

- The offset in bytes, count * size_of::<T>(), computed on mathematical integers (without “wrapping around”), must fit in an isize.
- Let result be self.addr() - count * size_of::<T>(), computed on mathematical integers. This must fit in a usize.
- If the computed offset is non-zero, then self must be derived from a pointer to some allocation, and the entire memory range between self and result (i.e., result..self.addr()) must be in bounds of that allocation.

    Allocations can never be larger than isize::MAX bytes and they can only contain addresses representable by usize, so technically the last condition implies the first two.

    Consider using wrapping_sub instead if these constraints are difficult to satisfy. The only advantage of this method is that it enables more aggressive compiler optimizations."#,
        "offset" => r#"If any of the following conditions are violated, the result is Undefined Behavior:

- The offset in bytes, count * size_of::<T>(), computed on mathematical integers (without “wrapping around”), must fit in an isize.
- Let result be self.addr() + count * size_of::<T>(), computed on mathematical integers. This must fit in a usize.
- If the computed offset is non-zero, then self must be derived from a pointer to some allocation, and the entire memory range between self and result (i.e., min(self.addr(), result)..max(self.addr(), result)) must be in bounds of that allocation.

    Allocations can never be larger than isize::MAX bytes and they can only contain addresses representable by usize, so technically the last condition implies the first two. This implies, for instance, that vec.as_ptr().offset(vec.len() as isize) (for vec: Vec<T>) is always safe.

    Consider using wrapping_offset instead if these constraints are difficult to satisfy. The only advantage of this method is that it enables more aggressive compiler optimizations."#,
        "offset_from" => r#"If any of the following conditions are violated, the result is Undefined Behavior:

- self and origin must either

    - point to the same address, or
    - both be derived from a pointer to the same allocation, and the memory range between the two pointers must be in bounds of that object. (See below for an example.)
- The distance between the pointers, in bytes, must be an exact multiple of the size of T.

    As a consequence, the absolute distance between the pointers, in bytes, computed on mathematical integers (without “wrapping around”), cannot overflow an isize. This is implied by the in-bounds requirement, and the fact that no allocation can be larger than isize::MAX bytes.

    The requirement for pointers to be derived from the same allocation is primarily needed for const-compatibility: the distance between pointers into different allocated objects is not known at compile-time. However, the requirement also exists at runtime and may be exploited by optimizations. If you wish to compute the difference between pointers that are not guaranteed to be from the same allocation, use (self.addr() as isize - origin.addr() as isize) / size_of::<T>()."#,
        _ => return None,
    })
}
