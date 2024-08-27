Parse
==============

The configurator and generator are built on top of the common crate that defines
the components which are being used in the process.

The exported functionality is used mostly in order to make the interfaces
implementations easier for the supported chips.

The `component` module contains the definition of traits used in the **generator**. The `Component` (used for defining the dependencies of an object and the expression needed for initialization) and `Ident` (used for defining an
unique identifier for the variable) traits define the objects that will be used in the main's setup function. These objects will be used in a dependency graph that will assure that initialization expressions will be written in the correct order.

The `parse` crate both uses and exports procedural macros. The `component` and `peripheral` procedural macros
have mostly the same attributes, but the way they implement the `Ident` trait differs. As the peripherals
in most cases are fields in a default peripherals-like struct, they don't have an unique identifier
for themselves, but instead use the unique identifier of the peripherals struct they are fields of,
defined in the `constants` module.

```rust
// Output from cargo expand.
impl parse::component::Ident for Uart {
    fn ident(&self) -> Result<String, parse::error::Error> {
        Ok(PERIPHERALS.clone() + &String::from(".nrf52.uarte0"))
    }
}
```

The `config` module defines the configuration that will be exposed to the **configurator**. Since one of the
goals was to separate the user from Tock's internals, this configuration only exposes the peripherals and other
agnostic features of the capsules, leaving the building of virtualizers and other Tock-specific components
as the job of the *context builder*. This is defined as a struct, in the `context` module, and is used to parse the configuration extracted from the configurator into the `Platform` that will be the root of our dependency graph.

The `peripherals` module contains the definition of virtual components and peripheral traits: `Uart`, `Timer`, etc.
These traits are mostly marker-like and used in trait-bounding the types of the `DefaultPeripherals`
trait. The `capsules` module contains the definition of capsules and their relation to the peripheral
they're dependent of, e.g.:

```rust
pub struct Console<U: uart::Uart> {
    pub(crate) mux_uart: Rc<uart::MuxUart<U>>,
}
```

The generator-purpose implementations use the `quote` crate for the usage of its macros in order to easily generate
the TokenStream outputs.

