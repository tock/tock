// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mapping from a category of unsafe operation to a URL in the Rust
//! documentation that explains why that operation requires `unsafe`.
//!
//! The generic anchors below point at the "Unsafety" chapter of the Rust
//! Reference, which enumerates the operations that are only permitted inside
//! an `unsafe` block:
//! <https://doc.rust-lang.org/reference/unsafety.html>

/// Dereferencing a raw pointer.
pub const DEREF: &str = "https://doc.rust-lang.org/reference/unsafety.html#r-safety.unsafe-deref";
/// Reading or writing a mutable (or unsafe extern) static variable.
pub const STATIC: &str = "https://doc.rust-lang.org/reference/unsafety.html#r-safety.unsafe-static";
/// Accessing a field of a `union`.
pub const UNION: &str =
    "https://doc.rust-lang.org/reference/unsafety.html#r-safety.unsafe-union-access";
/// Calling an unsafe function or method, generic fallback when the callee
/// isn't one of the well-known standard library functions below.
pub const CALL: &str = "https://doc.rust-lang.org/reference/unsafety.html#r-safety.unsafe-call";
/// Inline assembly (the `asm!`/`naked_asm!` macros).
pub const ASM: &str = "https://doc.rust-lang.org/reference/inline-assembly.html";

/// Names of commonly used `core`/`std` functions and methods that are
/// `unsafe`, so that calls to them are recognized even though this tool does
/// not parse the standard library's own source.
///
/// This list is a heuristic, matched purely by the final path segment of the
/// call (no type information is available), and is not exhaustive.
const WELL_KNOWN_UNSAFE_CALLABLES: &[&str] = &[
    "transmute",
    "transmute_copy",
    "copy",
    "copy_nonoverlapping",
    "read",
    "read_unaligned",
    "read_volatile",
    "write",
    "write_unaligned",
    "write_volatile",
    "write_bytes",
    "from_raw_parts",
    "from_raw_parts_mut",
    "from_raw",
    "get_unchecked",
    "get_unchecked_mut",
    "assume_init",
    "assume_init_ref",
    "assume_init_mut",
    "assume_init_drop",
    "unreachable_unchecked",
    "from_utf8_unchecked",
    "from_utf8_unchecked_mut",
    "set_len",
    "new_unchecked",
    // Raw pointer arithmetic (`*const T`/`*mut T` inherent methods). Their
    // `wrapping_*` counterparts are intentionally excluded: those are safe.
    "add",
    "sub",
    "offset",
    "byte_add",
    "byte_sub",
    "byte_offset",
    "offset_from",
    "byte_offset_from",
];

/// Whether `name` (the final segment of a call path, or a method name) is one
/// of the well-known unsafe standard library functions/methods above.
pub fn is_known_unsafe_callable(name: &str) -> bool {
    WELL_KNOWN_UNSAFE_CALLABLES.contains(&name)
}

/// A URL pointing directly at the documentation for a specific well-known
/// unsafe function/method, when one is available. Falls back to the generic
/// [`CALL`] anchor when the callee is ambiguous (for example, several types
/// have their own `from_raw`) or unrecognized.
///
/// Tock is `no_std` (no dynamic allocation, no `std`), so these link into
/// `core` wherever the item lives there; the one exception is `Vec::set_len`,
/// which links into `alloc` since `Vec` doesn't exist in `core` at all.
pub fn specific_url_for_callee(name: &str) -> Option<&'static str> {
    Some(match name {
        "transmute" => "https://doc.rust-lang.org/core/mem/fn.transmute.html",
        "transmute_copy" => "https://doc.rust-lang.org/core/mem/fn.transmute_copy.html",
        "copy_nonoverlapping" => "https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html",
        "copy" => "https://doc.rust-lang.org/core/ptr/fn.copy.html",
        "read" => "https://doc.rust-lang.org/core/ptr/fn.read.html",
        "read_unaligned" => "https://doc.rust-lang.org/core/ptr/fn.read_unaligned.html",
        "read_volatile" => "https://doc.rust-lang.org/core/ptr/fn.read_volatile.html",
        "write" => "https://doc.rust-lang.org/core/ptr/fn.write.html",
        "write_unaligned" => "https://doc.rust-lang.org/core/ptr/fn.write_unaligned.html",
        "write_volatile" => "https://doc.rust-lang.org/core/ptr/fn.write_volatile.html",
        "write_bytes" => "https://doc.rust-lang.org/core/ptr/fn.write_bytes.html",
        "from_raw_parts" => "https://doc.rust-lang.org/core/slice/fn.from_raw_parts.html",
        "from_raw_parts_mut" => "https://doc.rust-lang.org/core/slice/fn.from_raw_parts_mut.html",
        "get_unchecked" => {
            "https://doc.rust-lang.org/core/primitive.slice.html#method.get_unchecked"
        }
        "get_unchecked_mut" => {
            "https://doc.rust-lang.org/core/primitive.slice.html#method.get_unchecked_mut"
        }
        "assume_init" => {
            "https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init"
        }
        "assume_init_ref" => {
            "https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init_ref"
        }
        "assume_init_mut" => {
            "https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init_mut"
        }
        "unreachable_unchecked" => {
            "https://doc.rust-lang.org/core/hint/fn.unreachable_unchecked.html"
        }
        "from_utf8_unchecked" => "https://doc.rust-lang.org/core/str/fn.from_utf8_unchecked.html",
        "from_utf8_unchecked_mut" => {
            "https://doc.rust-lang.org/core/str/fn.from_utf8_unchecked_mut.html"
        }
        // `Vec` lives in `alloc`, not `core` -- there is no `core` equivalent
        // to link to.
        "set_len" => "https://doc.rust-lang.org/alloc/vec/struct.Vec.html#method.set_len",
        "add" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.add",
        "sub" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.sub",
        "offset" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.offset",
        "byte_add" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.byte_add",
        "byte_sub" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.byte_sub",
        "byte_offset" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.byte_offset",
        "offset_from" => "https://doc.rust-lang.org/core/primitive.pointer.html#method.offset_from",
        "byte_offset_from" => {
            "https://doc.rust-lang.org/core/primitive.pointer.html#method.byte_offset_from"
        }
        _ => return None,
    })
}
