# Mutable References in Tock - Memory Containers (Cells)

Borrows are a critical part of the Rust language that help provide its
safety guarantees. However, when there is no dynamic memory allocation
(no heap), event-driven code runs into challenges with Rust's borrow
semantics.  Often multiple structs need to
be able to call (share) a struct based on what events occur. For example,
a struct representing a radio interface needs to handle callbacks
both from the bus it uses as well as handle calls from higher layers of
a networking stack. Both of these callers need to be able to change the
state of the radio struct, but Rust's borrow checker does not allow them
to both have mutable references to the struct.

To solve this problem, Tock builds on the observation that having two
references to a struct that can modify it is safe, as long as no references
to memory inside the struct are leaked (there is no interior mutability).
Tock uses *memory containers*, a set of types that allow mutability
but not interior mutability, to achieve this goal. The Rust standard
library has two memory container types, `Cell` and `RefCell`. Tock uses
`Cell` extensively, but also adds five new memory container types, each
of which is tailored to a specific use common in kernel code.

<!-- npm i -g markdown-toc; markdown-toc -i Mutable_References.md -->

<!-- toc -->

- [Brief Overview of Borrowing in Rust](#brief-overview-of-borrowing-in-rust)
- [Issues with Borrowing in Event-Driven code](#issues-with-borrowing-in-event-driven-code)
- [`Cell`s in Tock](#cells-in-tock)
- [`MapCell`](#mapcell)
- [`OptionalCell`](#optionalcell)
- [`VolatileCell`](#volatilecell)
- [Cell Extensions](#cell-extensions)
  * [`NumericCellExt`](#numericcellext)

<!-- tocstop -->

## Brief Overview of Borrowing in Rust

Ownership and Borrowing are two design features in Rust which
prevent race conditions and make it impossible to write code that produces
dangling pointers.

Borrowing is the Rust mechanism to allow references to
memory. Similar to references in C++ and other languages, borrows make
it possible to efficiently pass large structures by passing pointers
rather than copying the entire structure.  The Rust compiler, however,
limits borrows so that they cannot create race conditions, which are
caused by concurrent writes or concurrent reads and writes to
memory. Rust limits code to either a single mutable (writeable)
reference or any number of read-only references.

If a piece of code has a mutable reference to a piece of memory, it's
also important that other code does not have any references within
that memory. Otherwise, the language is not safe. For example, consider
this case of an `enum` which can be either a pointer or a value:

```rust
enum NumOrPointer {
  Num(u32),
  Pointer(&'static mut u32)
}
```

A Rust `enum` is like a type-safe C union. Suppose that code has both
a mutable reference to a `NumOrPointer` and a read-only reference to
the encapsulated `Pointer`. If the code with the `NumOrPointer`
reference changes it to be a `Num`, it can then set the `Num` to be
any value.  However, the reference to `Pointer` can still access the
memory as a pointer. As these two representations use the same memory,
this means that the reference to `Num` can create any pointer it
wants, breaking Rust's type safety:

```rust
// n.b. illegal example
let external : &mut NumOrPointer;
match external {
  &mut Pointer(ref mut internal) => {
    // This would violate safety and
    // write to memory at 0xdeadbeef
    *external = Num(0xdeadbeef);
    *internal = 12345;
  },
  ...
}
```

As the Tock kernel is single
threaded, it doesn't have race conditions and so in some cases it may
be safe for there to be multiple references, as long as they do not
point inside each other (as in the number/pointer example).  But Rust
doesn't know this, so its rules still hold.  In practice, Rust's rules
cause problems in event-driven code.

## Issues with Borrowing in Event-Driven code

Event-driven code often requires multiple writeable references to
the same object. Consider, for example, an event-driven embedded
application that periodically samples a sensor and receives commands
over a serial port. At any given time, this application can have two
or three event callbacks registered: a timer, sensor data acquisition,
and receiving a command. Each callback is registered with a different
component in the kernel, and each of these components requires a
reference to the object to issue a callback on. That is, the generator
of each callback requires its own writeable reference to the
application. Rust's rules, however, do not allow multiple mutable
references.

## `Cell`s in Tock

Tock uses several [Cell](https://doc.rust-lang.org/core/cell/) types for
different data types. This
table summarizes the various types, and more detail is included below.

| Cell Type      | Best Used For        | Example                                                                                                                                                   | Common Uses                                                                            |
|----------------|----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------|
| `Cell`         | Primitive types      | `Cell<bool>`, [`sched/mod.rs`](../kernel/src/sched.rs)                                                                                                        | State variables (holding an `enum`), true/false flags, integer parameters like length. |
| `MapCell`      | Large static buffers | `MapCell<App>`, [`spi.rs`](../capsules/src/spi.rs)                                                                                                        | Delegating reference to large buffers (e.g. application buffers).                      |
| `OptionalCell` | Optional parameters, small static buffers  | `client: OptionalCell<&'static hil::nonvolatile_storage::NonvolatileStorageClient>`, [`nonvolatile_to_pages.rs`](../capsules/src/nonvolatile_to_pages.rs) | Keeping state that can be uninitialized, like a Client before one is set.              |
| `VolatileCell` | Registers            | `VolatileCell<u32>`                                                                                                                                       | Accessing MMIO registers, used by `tock_registers` crate.                              |

## `MapCell`

A `MapCell` is very similar to a `OptionalCell` in its purpose and interface.
What differs is the underlying implementation. In a `OptionalCell`, when
something `take()`s the contents of the cell, the memory inside is actually
moved. This is a performance problem if the data in a `OptionalCell` is
large, but saves both cycles and memory if the data is small (like a
pointer or slice) because the internal `Option` can be optimized in many cases
and the code operates on registers as opposed to memory. On the flip side,
`MapCell`s introduce some accounting overhead for small types and require a
minimum number of cycles to access.

The [commit that introduced `MapCell`][mapcell] includes some performance
benchmarks, but exact performance will vary based on the usage scenario.
Generally speaking, medium to large sized buffers should prefer `MapCell`s.

[mapcell]: https://github.com/tock/tock/commit/5f7246d4af139864f567cebf15bfc0b49e17b787)


## `OptionalCell`

[`OptionalCell`](https://github.com/tock/tock/blob/master/libraries/tock-cells/src/optional_cell.rs)
is effectively a wrapper for a `Cell` that contains an `Option`, like:
`Cell<Option<T>>`. So instead of `my_optional_cell.get().map(||
{})`, the code can be: `my_optional_cell.map(|| {})`.

`OptionalCell` can hold the same values that `Cell` can, but can also be just
`None` if the value is effectively unset. Using an `OptionalCell` (like a
`NumCell`) makes the code clearer and hides extra tedious function calls.

## `VolatileCell`

A `VolatileCell` is just a helper type for doing volatile reads and writes to a
value. This is mostly used for accessing memory-mapped I/O registers. The
`get()` and `set()` functions are wrappers around `core::ptr::read_volatile()`
and `core::ptr::write_volatile()`.


## Cell Extensions

In addition to custom types, Tock adds [extensions][extension_trait] to some of
the standard cells to enhance and ease usability. The mechanism here is to add
traits to existing data types to enhance their ability. To use extensions,
authors need only `use kernel::common::cells::THE_EXTENSION` to pull the new
traits into scope.

### `NumericCellExt`

[`NumericCellExt`](https://github.com/tock/tock/blob/master/libraries/tock-cells/src/numeric_cell_ext.rs)
extends cells that contain "numeric" types (like `usize` or `i32`) to provide
some convenient functions (`add()` and `subtract()`, for example). This
extension makes for cleaner code when storing numbers that are increased or
decreased. For example, with a typical `Cell`, adding one to the stored value
looks like: `my_cell.set(my_cell.get() + 1)`. With a `NumericCellExt` it is a
little easier to understand: `my_cell.increment()` (or `my_cell.add(1)`).

[extension_trait]: https://github.com/aturon/rfcs/blob/extension-trait-conventions/text/0000-extension-trait-conventions.md
