`tock_registers::peripheral!` Macro and Trait Interface Design
==============================================================

## An oversimplified explanation

`peripheral!` turns a register definition like:

```rust
peripheral! {
    pub foo {
        0x00 => ctrl: u32 { read + write }
        0x04 => received: u8 { read }
    }
}
```

into a module, which contains an `Accessor` trait and a `Registers` struct:

```rust
pub mod foo {
    trait Accessor: /* To be explained */ {}

    struct Registers<A: Accessor> {
        pub ctrl: tock_registers::Register<0x00, Self, A>,
        pub received: tock_registers::Register<0x04, Self, A>,
    }

    /* Plus trait impls that I will explain later */
}
```

## What is this `Accessor` trait?

`Accessor` exists to facilitate unit testing. An `Accessor` provides access to a
peripheral: either the real peripheral (e.g. via MMIO access) or a mock/fake
version. Operations performed on the members of `foo::Registers` forward to the
accessor; if you call `foo.ctrl.get()`, it will invoke a method on an instance
of `A`.

## Where is this `A` instance?

Each `tock_registers::Register` instance needs access to an instance of `A`, so
they must contain it:

```rust
// In tock_registers
pub struct Registers<const REL_ADDR: usize, Peripheral, Accessor> {
    pub accessor: A,
    /* ... */
}
```

Now the implementation of `foo.ctrl.get()` can call the accessor:

```rust
impl<Peripheral, A: ...> /*Mystery trait*/ for Register<..., Peripheral, A> {
    fn get(&self) -> ... {
        self.accessor.get::<...>()
    }
}
```

## What is this mystery trait?

`Register` is defined inside `tock_registers`, but some of the operations are
not. For example, RISC-V CSR traits are defined in the `riscv-csr` crate, which
depends on `tock_registers`. Therefore, the methods on `Register` cannot be
inherent impls. Instead, the 
