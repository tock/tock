# Unsafe `core` API Safety Requirements

This document surveys every function and method from Rust's `core` crate
that Tock calls from inside an `unsafe` block, anywhere in the repository
except `tools/` (i.e. `arch/`, `boards/`, `capsules/`, `chips/`, `kernel/`,
and `libraries/`). Tock is `no_std` and does not use dynamic allocation, so
all links below point at the `core` documentation, never `std` or `alloc`.

For each API this lists:

- the API name,
- a link to its official documentation,
- the verbatim `# Safety` section from that documentation (copied exactly
  as rendered, not paraphrased), and
- a short, Tock-specific summary of what the caller must uphold.

This was produced by scanning the codebase (via a `syn`-based tool; see
[`tools/debugging-and-development/unsafe-requirements/`](tools/debugging-and-development/unsafe-requirements/))
for every call and method call inside an `unsafe { ... }` block, then
manually cross-referencing the results against the actual `core` crate
documentation. It does **not** catch unsafe operations hidden inside a
macro invocation's arguments (e.g. a raw pointer dereference written
inside `assert_eq!(...)`) or inside the body of a `macro_rules!` macro
definition, since neither is visible as a structured expression to a
syntax-only scan; a manual pass over the "no automatically recognized
operation" blocks the tool reported did not turn up any additional `core`
APIs beyond the ones listed here, only instances of that same blind spot
and of inline assembly (`asm!` used as a statement, which is a Rust
language construct rather than a named API with its own doc page, so it is
intentionally not included here).

This document is a reference, not a safety verifier: it says nothing about
whether any particular call site in Tock actually upholds these
requirements, only what Rust itself requires.

## Contents

- [Raw pointer read/write operations](#raw-pointer-readwrite-operations)
  - [`core::ptr::read`](#coreptrread)
  - [`core::ptr::read_volatile`](#coreptrread_volatile)
  - [`core::ptr::write`](#coreptrwrite)
  - [`core::ptr::write_volatile`](#coreptrwrite_volatile)
  - [`core::ptr::write_bytes`](#coreptrwrite_bytes)
  - [`core::ptr::copy`](#coreptrcopy)
  - [`core::ptr::copy_nonoverlapping`](#coreptrcopy_nonoverlapping)
  - [`core::ptr::replace`](#coreptrreplace)
  - [`core::ptr::drop_in_place`](#coreptrdrop_in_place)
- [Raw pointer arithmetic](#raw-pointer-arithmetic)
  - [`pointer::add`](#pointeradd)
  - [`pointer::sub`](#pointersub)
  - [`pointer::offset`](#pointeroffset)
  - [`pointer::offset_from`](#pointeroffset_from)
- [`NonNull`](#nonnull)
  - [`NonNull::new_unchecked`](#nonnullnew_unchecked)
  - [`NonNull::as_ref`](#nonnullas_ref)
  - [`NonNull::as_mut`](#nonnullas_mut)
- [Slice construction](#slice-construction)
  - [`core::slice::from_raw_parts`](#coreslicefrom_raw_parts)
  - [`core::slice::from_raw_parts_mut`](#coreslicefrom_raw_parts_mut)
- [`MaybeUninit`](#maybeuninit)
  - [`MaybeUninit::assume_init`](#maybeuninitassume_init)
  - [`MaybeUninit::assume_init_ref`](#maybeuninitassume_init_ref)
  - [`MaybeUninit::assume_init_mut`](#maybeuninitassume_init_mut)
- [Other](#other)
  - [`core::mem::transmute`](#coremem_transmute)
  - [`core::hint::unreachable_unchecked`](#corehintunreachable_unchecked)

---

## Raw pointer read/write operations

### `core::ptr::read`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.read.html>

```text
Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads or T must be a ZST.
- src must be properly aligned. Use read_unaligned if this is not the case.
- src must point to a properly initialized value of type T.

    Note that even if T has size 0, the pointer must be properly aligned.
```

**Summary:** `src` must be a non-dangling, properly aligned pointer to an
already-initialized `T` (unless `T` is zero-sized); calling it on an
unaligned, uninitialized, or otherwise invalid pointer is immediate
undefined behavior.

### `core::ptr::read_volatile`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.read_volatile.html>

```text
Like read, read_volatile creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the returned value and the value at *src can violate memory safety. However, storing non-Copy types in volatile memory is almost certainly incorrect.

Behavior is undefined if any of the following conditions are violated:

- src must be either valid for reads, or T must be a ZST, or src must point to memory outside of all Rust allocations and reading from that memory must:

    - not trap, and
    - not cause any memory inside a Rust allocation to be modified.
- src must be properly aligned.
- Reading from src must produce a properly initialized value of type T.

    Note that even if T has size 0, the pointer must be properly aligned.
```

**Summary:** Same alignment/initialization requirements as `read`, but
`src` is also allowed to point outside of any Rust allocation (e.g. a
memory-mapped register) as long as reading it doesn't trap or mutate other
Rust-owned memory; this is Tock's primary tool for reading MMIO registers.

### `core::ptr::write`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.write.html>

```text
Behavior is undefined if any of the following conditions are violated:

- dst must be valid for writes or T must be a ZST.
- dst must be properly aligned. Use write_unaligned if this is not the case.

    Note that even if T has size 0, the pointer must be properly aligned.
```

**Summary:** `dst` must be a non-dangling, properly aligned, writable
pointer; this overwrites `*dst` without running its destructor, so writing
over a live value that owns resources will leak (or worse) them.

### `core::ptr::write_volatile`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.write_volatile.html>

```text
Behavior is undefined if any of the following conditions are violated:

- dst must be either valid for writes, or T must be a ZST, or dst must point to memory outside of all Rust allocations and writing to that memory must:

    - not trap, and
    - not cause any memory inside a Rust allocation to be modified.
- dst must be properly aligned.

    Note that even if T has size 0, the pointer must be properly aligned.
```

**Summary:** Same as `write`, but `dst` may also point outside any Rust
allocation (e.g. a hardware register) as long as the write doesn't trap or
corrupt other Rust-owned memory; this is Tock's primary tool for writing
MMIO registers.

### `core::ptr::write_bytes`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.write_bytes.html>

```text
Behavior is undefined if any of the following conditions are violated:

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
}
```

**Summary:** `dst` must be writable for `count * size_of::<T>()` bytes and
properly aligned; because this sets raw bytes rather than a value, it's the
caller's job to make sure the resulting bit pattern is actually valid for
`T` before anything reads it again.

### `core::ptr::copy`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.copy.html>

```text
Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
- dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0, and dst must remain valid even when src is read for count * size_of::<T>() bytes. (This means if the memory ranges overlap, the dst pointer must not be invalidated by src reads.)
- Both src and dst must be properly aligned.

    Like read, copy creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.

    Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
```

**Summary:** Both `src` and `dst` must be valid, aligned, and large enough
for `count` elements; unlike `copy_nonoverlapping`, the two regions are
allowed to overlap.

### `core::ptr::copy_nonoverlapping`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.copy_nonoverlapping.html>

```text
Behavior is undefined if any of the following conditions are violated:

- src must be valid for reads of count * size_of::<T>() bytes or that number must be 0.
- dst must be valid for writes of count * size_of::<T>() bytes or that number must be 0.
- Both src and dst must be properly aligned.
- The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.

    Like read, copy_nonoverlapping creates a bitwise copy of T, regardless of whether T is Copy. If T is not Copy, using both the values in the region beginning at *src and the region beginning at *dst can violate memory safety.

    Note that even if the effectively copied size (count * size_of::<T>()) is 0, the pointers must be properly aligned.
```

**Summary:** Same as `copy`, but the caller additionally guarantees the
`src` and `dst` regions do not overlap at all, which is what lets this
compile down to a plain `memcpy`.

### `core::ptr::replace`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.replace.html>

```text
Behavior is undefined if any of the following conditions are violated:

- dst must be valid for both reads and writes or T must be a ZST.
- dst must be properly aligned.
- dst must point to a properly initialized value of type T.

    Note that even if T has size 0, the pointer must be properly aligned.
```

**Summary:** `dst` must be a valid, aligned pointer to an already
-initialized `T`; this atomically swaps in a new value and hands back the
old one without ever leaving `*dst` in an invalid state, which is how Tock
pulls a value out of a cell it only has a raw pointer into.

### `core::ptr::drop_in_place`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/fn.drop_in_place.html>

```text
Behavior is undefined if any of the following conditions are violated:

- to_drop must be valid for both reads and writes.
- to_drop must be properly aligned, even if T has size 0.
- to_drop must be nonnull, even if T has size 0.
- The value to_drop points to must be valid for dropping, which may mean it must uphold additional invariants. These invariants depend on the type of the value being dropped. For instance, when dropping a Box, the box's pointer to the heap must be valid.
- While drop_in_place is executing, the only way to access parts of to_drop is through the &mut self references supplied to the Drop::drop methods that drop_in_place invokes.

    Additionally, if T is not Copy, using the pointed-to value after calling drop_in_place can cause undefined behavior. Note that *to_drop = foo counts as a use because it will cause the value to be dropped again. write() can be used to overwrite data without causing it to be dropped.
```

**Summary:** `to_drop` must be a valid, aligned, non-null pointer to a live
value that is actually safe to tear down; after this call the pointed-to
memory holds a dropped value and must not be read, written, or dropped
again.

---

## Raw pointer arithmetic

The following are inherent methods shared by `*const T` and `*mut T`,
documented together on the primitive pointer type page.

### `pointer::add`

- **Documentation:** <https://doc.rust-lang.org/core/primitive.pointer.html#method.add>

```text
If any of the following conditions are violated, the result is Undefined Behavior:

- The offset in bytes, count * size_of::<T>(), computed on mathematical integers (without "wrapping around"), must fit in an isize.
- Let result be self.addr() + count * size_of::<T>(), computed on mathematical integers. This must fit in a usize.
- If the computed offset is non-zero, then self must be derived from a pointer to some allocation, and the entire memory range between self and result (i.e., self.addr()..result) must be in bounds of that allocation.

    Allocations can never be larger than isize::MAX bytes and they can only contain addresses representable by usize, so technically the last condition implies the first two. This implies, for instance, that vec.as_ptr().add(vec.len()) (for vec: Vec<T>) is always safe.

    Consider using wrapping_add instead if these constraints are difficult to satisfy. The only advantage of this method is that it enables more aggressive compiler optimizations.
```

**Summary:** The resulting address (and every address passed over to get
there) must stay within the bounds of the same allocation `self` came from,
and the byte offset must not overflow `isize`; this is by far the most
heavily used unsafe `core` API in Tock, almost always for walking through
a block of memory one `T` at a time.

### `pointer::sub`

- **Documentation:** <https://doc.rust-lang.org/core/primitive.pointer.html#method.sub>

```text
If any of the following conditions are violated, the result is Undefined Behavior:

- The offset in bytes, count * size_of::<T>(), computed on mathematical integers (without "wrapping around"), must fit in an isize.
- Let result be self.addr() - count * size_of::<T>(), computed on mathematical integers. This must fit in a usize.
- If the computed offset is non-zero, then self must be derived from a pointer to some allocation, and the entire memory range between self and result (i.e., result..self.addr()) must be in bounds of that allocation.

    Allocations can never be larger than isize::MAX bytes and they can only contain addresses representable by usize, so technically the last condition implies the first two.

    Consider using wrapping_sub instead if these constraints are difficult to satisfy. The only advantage of this method is that it enables more aggressive compiler optimizations.
```

**Summary:** The same in-bounds and no-overflow requirements as `add`, just
walking backward through the same allocation instead of forward.

### `pointer::offset`

- **Documentation:** <https://doc.rust-lang.org/core/primitive.pointer.html#method.offset>

```text
If any of the following conditions are violated, the result is Undefined Behavior:

- The offset in bytes, count * size_of::<T>(), computed on mathematical integers (without "wrapping around"), must fit in an isize.
- Let result be self.addr() + count * size_of::<T>(), computed on mathematical integers. This must fit in a usize.
- If the computed offset is non-zero, then self must be derived from a pointer to some allocation, and the entire memory range between self and result (i.e., min(self.addr(), result)..max(self.addr(), result)) must be in bounds of that allocation.

    Allocations can never be larger than isize::MAX bytes and they can only contain addresses representable by usize, so technically the last condition implies the first two. This implies, for instance, that vec.as_ptr().offset(vec.len() as isize) (for vec: Vec<T>) is always safe.

    Consider using wrapping_offset instead if these constraints are difficult to satisfy. The only advantage of this method is that it enables more aggressive compiler optimizations.
```

**Summary:** Identical bounds/overflow requirements as `add`/`sub`, but
takes a signed `count` so it can move in either direction in one call.

### `pointer::offset_from`

- **Documentation:** <https://doc.rust-lang.org/core/primitive.pointer.html#method.offset_from>

```text
If any of the following conditions are violated, the result is Undefined Behavior:

- self and origin must either

    - point to the same address, or
    - both be derived from a pointer to the same allocation, and the memory range between the two pointers must be in bounds of that object. (See below for an example.)
- The distance between the pointers, in bytes, must be an exact multiple of the size of T.

    As a consequence, the absolute distance between the pointers, in bytes, computed on mathematical integers (without "wrapping around"), cannot overflow an isize. This is implied by the in-bounds requirement, and the fact that no allocation can be larger than isize::MAX bytes.

    The requirement for pointers to be derived from the same allocation is primarily needed for const-compatibility: the distance between pointers into different allocated objects is not known at compile-time. However, the requirement also exists at runtime and may be exploited by optimizations. If you wish to compute the difference between pointers that are not guaranteed to be from the same allocation, use (self.addr() as isize - origin.addr() as isize) / size_of::<T>().
```

**Summary:** `self` and `origin` must point into the same allocation (or be
equal), and the byte distance between them must be an exact multiple of
`size_of::<T>()`; Tock uses this to compute the size of a memory region
from two pointers into it (e.g. an MPU region's extent).

---

## `NonNull`

Tock's [`StaticRef`](kernel/src/utilities/static_ref.rs) type -- the
standard way every chip crate wraps a peripheral's memory-mapped register
block -- is built on `NonNull`, so `NonNull::new_unchecked` and
`NonNull::as_ref`/`as_mut` sit underneath essentially all MMIO register
access in Tock, even though each is written out explicitly in only one or
two places.

### `NonNull::new_unchecked`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/struct.NonNull.html#method.new_unchecked>

```text
ptr must be non-null.
```

**Summary:** The caller must already know `ptr` is non-null; passing a
null pointer here is immediate undefined behavior (there's no runtime
check, unlike the safe `NonNull::new`).

### `NonNull::as_ref`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/struct.NonNull.html#method.as_ref>

```text
When calling this method, you have to ensure that
the pointer is convertible to a reference.
```

**Summary:** The pointer must meet all the usual requirements for `&T` to
exist at that address -- aligned, pointing to a valid, initialized `T`, and
not aliased by a live `&mut T` -- for as long as the returned reference is
used.

### `NonNull::as_mut`

- **Documentation:** <https://doc.rust-lang.org/core/ptr/struct.NonNull.html#method.as_mut>

```text
When calling this method, you have to ensure that
the pointer is convertible to a reference.
```

**Summary:** Same as `as_ref`, but for `&mut T`: the caller must also
guarantee no other reference (shared or mutable) to the same memory exists
for as long as the returned `&mut T` is live.

---

## Slice construction

### `core::slice::from_raw_parts`

- **Documentation:** <https://doc.rust-lang.org/core/slice/fn.from_raw_parts.html>

```text
Behavior is undefined if any of the following conditions are violated:

- data must be non-null, valid for reads for len * size_of::<T>() many bytes, and it must be properly aligned. This means in particular:

    - The entire memory range of this slice must be contained within a single allocation! Slices can never span across multiple allocations. See below for an example incorrectly not taking this into account.
    - data must be non-null and aligned even for zero-length slices or slices of ZSTs. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
- data must point to len consecutive properly initialized values of type T.
- The memory referenced by the returned slice must not be mutated for the duration of lifetime 'a, except inside an UnsafeCell.
- The total size len * size_of::<T>() of the slice must be no larger than isize::MAX, and adding that size to data must not "wrap around" the address space. See the safety documentation of pointer::offset.
```

**Summary:** `data` must be a non-null, aligned pointer to `len`
already-initialized, contiguous `T`s within one allocation, and nothing
else may mutate that memory for as long as the returned `&[T]` lives.

### `core::slice::from_raw_parts_mut`

- **Documentation:** <https://doc.rust-lang.org/core/slice/fn.from_raw_parts_mut.html>

```text
Behavior is undefined if any of the following conditions are violated:

- data must be non-null, valid for both reads and writes for len * size_of::<T>() many bytes, and it must be properly aligned. This means in particular:

    - The entire memory range of this slice must be contained within a single allocation! Slices can never span across multiple allocations.
    - data must be non-null and aligned even for zero-length slices or slices of ZSTs. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
- data must point to len consecutive properly initialized values of type T.
- The memory referenced by the returned slice must not be accessed through any other pointer (not derived from the return value) for the duration of lifetime 'a. Both read and write accesses are forbidden.
- The total size len * size_of::<T>() of the slice must be no larger than isize::MAX, and adding that size to data must not "wrap around" the address space. See the safety documentation of pointer::offset.
```

**Summary:** Same as `from_raw_parts`, but for `&mut [T]`: the caller must
additionally guarantee *exclusive* access -- no other pointer or reference
into that memory may be used, for reads or writes, while the returned
slice exists.

---

## `MaybeUninit`

### `MaybeUninit::assume_init`

- **Documentation:** <https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init>

```text
It is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state, i.e., a state that is considered "valid" for type T. Calling this when the content is not yet fully initialized causes immediate undefined behavior. The type-level documentation contains more information about this initialization invariant.

It is a common mistake to assume that this function is safe to call on integers because they can hold all bit patterns. It is also a common mistake to think that calling this function is UB if any byte is uninitialized. Both of these assumptions are wrong. If that is surprising to you, please read the type-level documentation.

On top of that, remember that most types have additional invariants beyond merely being considered initialized at the type level. For example, a 1-initialized Vec<T> is considered initialized (under the current implementation; this does not constitute a stable guarantee) because the only requirement the compiler knows about it is that the data pointer must be non-null. Creating such a Vec<T> does not cause immediate undefined behavior, but will cause undefined behavior with most safe operations (including dropping it).
```

**Summary:** The `MaybeUninit<T>` must actually hold a fully-valid `T`
already -- not just "some bits", and not merely "every byte written" if
`T` has further validity invariants beyond that.

### `MaybeUninit::assume_init_ref`

- **Documentation:** <https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init_ref>

```text
Calling this when the content is not yet fully initialized causes undefined behavior: it is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state.
```

**Summary:** Same initialization requirement as `assume_init`, but only
borrows the value as `&T` instead of taking it by value.

### `MaybeUninit::assume_init_mut`

- **Documentation:** <https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#method.assume_init_mut>

```text
Calling this when the content is not yet fully initialized causes undefined behavior: it is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state. For instance, .assume_init_mut() cannot be used to initialize a MaybeUninit.
```

**Summary:** Same as `assume_init_ref`, borrowing `&mut T` instead; it must
already be initialized going in -- this cannot be used to perform the
initialization itself.

---

## Other

### `core::mem::transmute`

- **Documentation:** <https://doc.rust-lang.org/core/mem/fn.transmute.html>

`transmute`'s documentation does not have a dedicated `# Safety` heading;
its safety requirements are instead spread across the function's main
description and an "Alternatives" section. The closest it comes to a single
summary statement is:

```text
This function will unsafely assume the compatibility of the two types: T and U. Both parameters must have the same size. Compiler says "Error: cannot transmute between types of different sizes, or dependently-sized types" if it finds this to not be true.

transmute is semantically equivalent to a bitwise copy of one value into another type. It copies the bits from the source value into the destination value, then forgets the original. It's equivalent to C's memcpy under the hood, just like transmute_copy.
```

**Summary:** `T` and `U` must be the same size, and the resulting bit
pattern must be a valid value of `U` -- there is no implicit alignment,
lifetime, or layout checking at all, which is why Tock reserves it for
exactly the cases nothing else in `core` can express (e.g. reinterpreting
raw words as a register-layout struct).

### `core::hint::unreachable_unchecked`

- **Documentation:** <https://doc.rust-lang.org/core/hint/fn.unreachable_unchecked.html>

```text
Reaching this function is Undefined Behavior.

As the compiler assumes that all forms of Undefined Behavior can never happen, it will eliminate all branches in the surrounding code that it can determine will invariably lead to a call to unreachable_unchecked().

If the assumptions embedded in using this function turn out to be wrong - that is, if the site which is calling unreachable_unchecked() is actually reachable at runtime - the compiler may have generated nonsensical machine instructions for this situation, including in seemingly unrelated code, causing difficult-to-debug problems.

Use this function sparingly. Consider using the unreachable! macro, which may prevent some optimizations but will safely panic in case it is actually reached at runtime. Benchmark your code to find out if using unreachable_unchecked() comes with a performance benefit.
```

**Summary:** Control flow must never actually reach this call; if it ever
does at runtime, the compiler has already optimized away the surrounding
checks on the assumption that it can't, so the resulting behavior is
unpredictable rather than a clean panic.
