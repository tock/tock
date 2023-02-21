`tock_registers::peripheral!` Macro and Trait Interface Design
==============================================================

## Basic terminology

A `peripheral!` invocation looks like the following:

```rust
peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write }
        0x04 => received: u8 { read }
    }
}
```

This invocation specifies two *registers*, `ctrl` and `received`. `ctrl`
implements two *operations*`, `read` and `write`. `received` only implements
`read`.

## Core `tock_registers` structs

`tock_registers` exports the following structs:

```rust
#[repr(transparent)]
pub struct Register<const REL_ADDR: usize, Value, Peripheral, Accessor> {
    pub accessor: Accessor,
    _phantom: core::marker::PhantomData<(Value, Peripheral)>,
}

impl<Accessor> Register<_, _, _, Accessor> {
    fn new(accessor: Accessor) -> Self {
        Self {
            accessor,
            _phantom: Default::default(),
        }
    }
}

pub struct Real { _noconstruct: () }
```

## Operation implementation

An *operation* is implemented as a module that contains three traits:

```rust
pub mod read {
    pub trait Access<const REL_ADDR: usize, Value> {
        fn read(&self) -> Value;
    }

    pub trait Has<const REL_ADDR: usize, Value> {}

    pub trait Register<Value> {
        fn read(&self) -> Value;
    }

    impl<
            const REL_ADDR: usize,
            Value,
            Peripheral: Has<REL_ADDR, Value>,
            Accessor: Access<REL_ADDR, Value>,
        > Register<Value> for crate::tock_registers::Register<REL_ADDR, Peripheral, Accessor>
    {
        fn read(&self) -> Value {
            self.accessor.read()
        }
    }

    impl<const REL_ADDR: usize, Value> Access<REL_ADDR, Value> for tock_registers::Real {
        fn read(&self) -> Value {
            unsafe {
                core::ptr::read_volatile((self as *const Self as usize + REL_ADDR) as *const Value)
            }
        }
    }
}
```

## `peripheral!` expansion

Our example peripheral:

```rust
peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write }
        0x04 => received: u8 { read }
    }
}
```

expands to:

```rust
mod foo {
    trait Accessor: Copy +
                    read::Access<0, u32> +
                    write::Access<0, u32> +
                    read::Access<4, u8> {}

    impl<A: Copy +
            read::Access<0, u32> +
            write::Access<0, u32> +
            read::Access<4, u8>>
    Accessor for A {}

    #[repr(C)]
    struct Registers<Accessor> {
        pub ctrl: tock_registers::Register<0, Self, Accessor>,
        pub received: tock_registers::Register<4, Self, Accessor>,
    }

    impl read::Has<0, u32> for Registers<_> {}
    impl write::Has<0, u32> for Registers<_> {}
    impl read::Has<4, u8> for Registers<_> {}

    impl<Accessor> Registers<Accessor> {
        // Used in unit tests
        pub fn new(accessor: Accessor) -> Self {
            Self {
                ctrl: Register::new(accessor),
                received: Register::new(accessor),
            }
        }
    }
}
```

## How do you use the generated registers?

```rust
peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write }
        0x04 => received: u8 { read }
    }
}

// Called with A == tock_registers::Real in the real kernel, and a fake version
// of the Foo peripheral in unit tests.
fn use_foo<A: foo::Accessor>(instance: &'static foo::Registers<A>) -> u32 {
    use read::Register;

    foo.ctrl.read()
    // foo.ctrl.read() invokes read::Register::<u32>::read(), which calls
    // read::Access<0, u32> on foo.ctrl.accessor. If A is tock_registers::Real,
    // this performs a volatile memory read.
}
```

## Properties of this design

1. Allows unit testing -- a Foo test can implement `read::Access<0, u32>`,
   `write::Access<0, u32>`, and `read::Access<4, u8>` on a fake version of
   `Foo` and use that to test `use_foo`'s functionality.
2. Resolves the unsoundness with pointers pointing into MMIO memory.
   `foo::Registers::<tock_registers::Real>` is a zero-sized type, so a
   reference to it does not point to any data, so the compiler cannot insert
   arbitrary deferences to it.
3. Operations can be defined outside `tock_registers`. This design allows the
   `riscv-csr` crate to define its operations and retain the full functionality
   of `tock_registers` (including unit test functionality).
4. Hideously complex and hard to explain.
