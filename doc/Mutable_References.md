# Mutable References in Tock - Memory Containers

Borrows are a critical part of the Rust language that help provide its
safety guarantees. However, they can complicate event-driven code without
a heap (no dynamic allocation). Tock uses memory containers
such as the TakeCell abstraction to allow simple code to keep
the safety properties Rust provides.

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

But what does this mean for Tock? As the Tock kernel is single
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

## The TakeCell abstraction

Tock solves this issue of uniquely sharing memory with a memory
container abstraction, TakeCell.
From `tock/kernel/src/common/take_cell.rs`:

> A `TakeCell` is a potential reference to mutable memory. Borrow rules are
> enforced by forcing clients to either move the memory out of the cell or
> operate on a borrow within a closure.

A TakeCell can be full or empty: it is like a safe pointer that can be
null. If code wants to operate on the data contained in the TakeCell,
it must either move the data out of the TakeCell (making it empty), or
it must do so within a closure with a `map` call. Using `map` passes a
block of code for the TakeCell to execute.  Using a closure allows
code to modify the contents of the TakeCell inline, without any danger
of a control path accidentally not replacing the value. However,
because it is a closure, a reference to the contents of the TakeCell
cannot escape.

TakeCell allows code to modify its contents when it has a normal
(non-mutable) reference. This in turn means that if a structure
stores its state in TakeCells, then code which has a regular
(non-mutable) reference to the structure can change the contents
of the TakeCell and therefore modify the structure. Therefore,
it is possible for multiple callbacks to have references to
the structure and modify its state.

### Example use of `take` and `replace`

When `TakeCell.take()` is called, ownership of a location in memory
moves out of the cell. It can then be freely used by whoever took it
(as they own it) and then put back with `TakeCell.put()` or
`TakeCell.replace()`.

For example, this piece of code from `chips/nrf51/src/clock.rs`
sets the callback client for a hardware clock:

```rust
pub fn set_client(&self, client: &'static ClockClient) {
    self.client.replace(client);
}
```

If there is a current client, it's replaced with `client`. If
`self.client` is empty, then it's filled with `client`.

This piece of code from `chips/sam4l/src/dma.rs` cancels a
current direct memory access (DMA) operation, removing the
buffer in the current transaction from the TakeCell with a
call to `take`:

```rust
pub fn abort_xfer(&self) -> Option<&'static mut [u8]> {
    let registers: &DMARegisters = unsafe { &*self.registers };
    registers.interrupt_disable.set(!0);
    // Reset counter
    registers.transfer_counter.set(0);
    self.buffer.take()
}
```


### Example use of `map`

Although the contents of a TakeCell can be directly accessed through
a combination of `take` and `replace`, Tock code typically uses
`TakeCell.map()`, which wraps the provided closure between a
`TakeCell.take()` and `TakeCell.replace()`. This approach has the
advantage that a bug in control flow can't that doesn't correctly
`replace` won't accidentally leave the TakeCell empty.

Here is a simple use of `map`, taken from `chips/sam4l/src/dma.rs`:

```rust
pub fn disable(&self) {
    let regs: &SpiRegisters = unsafe { &*self.registers };

    self.dma_read.map(|read| read.disable());
    self.dma_write.map(|write| write.disable());
    regs.cr.set(0b10);
}
```

Both `dma_read` and `dma_write` are of type `TakeCell<&'static mut DMAChannel>`,
that is, a TakeCell for a mutable reference to a DMA channel. By calling `map`,
the function can access the reference and call the `disable` function. If
the TakeCell has no reference (it is empty), then `map` does nothing.

Here is a more complex example use of `map`, taken from `chips/sam4l/src/spi.rs`:

```rust
self.client.map(|cb| {
    txbuf.map(|txbuf| {
        cb.read_write_done(txbuf, rxbuf, len);
    });
});
```

In this example, `client` is a `TakeCell<&'static SpiMasterClient>`.
The closure passed to `map` has a single argument, the value which the
TakeCell contains. So in this case, `cb` is the reference to an
`SpiMasterClient`. Note that the closure passed to `client.map` then
itself contains a closure, which uses `cb` to invoke a callback passing
`txbuf`.


#### `map` variants

`TakeCell.map()` provides a convenient method for interacting with a
`TakeCell`'s stored contents, but it also hides the case when the `TakeCell` is
empty by simply not executing the closure. To allow for handling the cases when
the `TakeCell` is empty, rust (and by extension Tock) provides additional
functions.

The first is `.map_or()`. This is useful for returning a value both when the
`TakeCell` is empty and when it has a contained value. For example, rather than:

```rust
let return = if txbuf.is_some() {
    txbuf.map(|txbuf| {
        write_done(txbuf);
    });
    ReturnCode::SUCCESS
} else {
    ReturnCode::ERESERVE
};
```

`.map_or()` allows us to do this instead:

```rust
let return = txbuf.map_or(ReturnCode::ERESERVE, |txbuf| {
    write_done(txbuf);
    ReturnCode::SUCCESS
});
```

If the `TakeCell` is empty, the first argument (the error code) is returned,
otherwise the closure is executed and `SUCCESS` is returned.

Sometimes we may want to execute different code based on whether the `TakeCell`
is empty or not. Again, we could do this:

```rust
if txbuf.is_some() {
    txbuf.map(|txbuf| {
        write_done(txbuf);
    });
} else {
    write_done_failure();
};
```

Instead, however, we can use the `.map_or_else()` function. This allows us to
pass in two closures, one for if the `TakeCell` is empty, and one for if it has
contents:

```rust
txbuf.map_or_else(|| {
    write_done_failure();
}, |txbuf| {
    write_done(txbuf);
});
```

Not in both the `.map_or()` and `.map_or_else()` cases, the first argument
corresponds to when the `TakeCell` is empty.


## `MapCell` Version

