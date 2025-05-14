# Kernel Async Library

This library provides the primitives that allow the usage of `async` Rust in drivers. 

This serves two purposes:
- allow downstream users to use drivers from [`crates.io`](https://crates.io) written on to of
  the [`embedded-hal-async`](https://docs.rs/embedded-hal-async/latest/embedded_hal_async/) traits and warp them into Tock drivers
- allow upstream Tock to write drivers using `async/.await`
