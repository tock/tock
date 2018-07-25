# Lifetimes

Values in the Tock kernel can be allocated in three ways:

  1. **Static allocation** Statically allocated values are never deallocated.
     These values are represented as Rust "borrows" with a `'static` lifetime.

  2. **Stack allocation** Stack allocated values have a lexically bound
     lifetime. That is, we know by looking at the source code when they will be
     deallocated. When you create a reference to such a value, the Rust type
     system ensures that reference is never used after the value is deallocated
     by assigning a "lifetime" to the reference.

  3. **Grant values** Values allocated from a process's grant region have a
     runtime-dependent lifetime. For example, when they are deallocated depends
     on whether the processes crashes. Since we can't represent
     runtime-dependent lifetimes in Rust's type-system, references to grant
     values in Tock are done through the `Grant` type, which is owned by its
     referrer.

Next we'll discuss how Rust's notion of lifetimes maps to the lifetimes of
values in Tock and how this affects the use of different types of values in the
kernel.

## Rust lifetimes

Each reference (called a _borrow_) in Rust has _lifetime_ associated with its
type that determines in what scope it is valid. The lifetime of a reference
must be more constrained than the value it was borrowed from. The compiler, in
turn, ensures that references cannot escape their valid scope.

As a result, data structures that store a reference must declare the minimal
lifetime of that reference. For example:

```rust
struct Foo<'a> {
  bar: &'a Bar
}
```

defines a data structure `Foo` that contains a reference to another type,
`Bar`. The reference has a lifetime `'a'`, which is a type parameter of `Foo`.
Note that `'a` is an arbitrary choice of name for the lifetime, such as `E` in
a generic `List<E>`.  It is also possible to use the explicit lifetime
`'static` rather than a type parameter when the reference should always live
forever, regardless of how long the containing type (e.g. `Foo`) lives:

```rust
struct Foo {
  bar: &'static Bar
}
```

## Buffer management

Buffers used in asynchronous hardware operations must be static. On the one
hand, we need to guarantee (to the hardware) that the buffer will not be
deallocated before the hardware relinquishes its pointer. On the other hand,
the hardware has no way of telling us (i.e. the Rust compiler) that it will
only access the buffer within a certain lexical bound (because we are using the
hardware asynchronously). To resolve this, buffers passed to hardware should be
allocated statically.

## Circular dependencies

Tock uses circular dependencies to give capsules access to each other.
Specifically, two capsules that depend on each other will each have a field
containing a reference to the other. For example, a client of the timer `Alarm` trait
needs a reference to an instance of the timer in order to start/stop it, while
the instance of timer needs a reference to the client in order to propagate
events. This is handled by the `set_client` function, which allows the platform
definition to connect objects after creation.

```rust
impl Foo<'a> {
  fn set_client(&self, client: &'a Client) {
    self.client.set(client);
  }
}
```

