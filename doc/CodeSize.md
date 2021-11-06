# Minimizing Tock code size

Many embedded applications are ultimately limited by the flash space available on
the board in use. This document provides tips on how to write Rust code such that
it does not require an undue amount of flash, and highlights some options which
can be used to reduce the size required for a particular image.

## Code Style: tips for keeping Rust code small

#### When to use generic types with trait bounds versus trait objects (`dyn`)

Polymorphic structs and functions are one of the biggest sources of bloat in Rust
binaries -- use of generic types can lead to bloat from monomorphization, while
use of trait objects introduces vtables into the binary and limits opportunities
for inlining.

Use `dyn` when the function in question will be called with multiple concrete types;
otherwise code size is increased for every concrete type used (monomorphization).

```rust
fn set_gpio_client(&dyn GpioClientTrait) -> Self {//...}

// elsewhere
let radio: Radio = Radio::new();
set_gpio_client(&radio);

let button: Button = Button::new();
set_gpio_client(&button);
```

Use generics with trait bounds when the function is only ever called with a single public type
per board; this reduces code size and run time cost. This increases source code
complexity for decreased image size and decreased clock cycles used.

```rust
// On a given chip, there is only a single FlashController. We use generics so
// that there can be a shared interface by all FlashController's on different chips,
// but in a given binary this function will never be called with multiple types.
impl<'a, F: FlashController> StorageDriverBackend<'a, F> {
    pub fn new(
        storage: &'a StorageController<'a, F>,
    ) -> Self { ... }

```

Similarly, only use const generics when there will not be monomorphization, or if the body of the
method which would be monomorphized is sufficiently small that it will be inlined anyways.

#### Non-generic-inner-functions

Sometimes, generic monomorphization is unavoidable (much of the code in grant.rs is an example
of this). When generics must be used despite functions being called with multiple different types,
use the non-generic-inner-function method, written about at
https://www.possiblerust.com/pattern/non-generic-inner-functions , and applied in our codebase
(see [here](https://github.com/tock/tock/pull/2648) for an example).

#### Panics
Panics add substantial code size to Tock binaries -- on the order of 50-75 bytes per panic. Returning
errors is much cheaper than panicing, and also produces more dependable code. Whenever possible, return
errors instead of panicing. Often, this will not only mean avoiding explicit panics: many core library
functions panic internally depending on the input.

The most common panics in Tock binaries are from array accesses, but these can often be ergonomically
replaced with result-based error handling:

```rust
// BAD: produces bloat
fn do_stuff(&mut self) -> Result<(), ErrorCode> {
    if self.my_array[4] == 7 {
        self.other_array[3] = false;
        Ok(())
    } else {
        Err(ErrorCode::SIZE)
    }
}
```
```rust
// GOOD
fn do_stuff(&mut self) -> Result<(), ErrorCode> {
    if self.my_array.get(4).ok_or(ErrorCode::FAIL)? == 7 {
        *(self.other_array.get_mut(3).ok_or(ErrorCode::FAIL)?) = false;
        Ok(())
    } else {
        Err(ErrorCode::SIZE)
    }
}
```

Similarly, avoid code that could divide by 0, and avoid signed division which could
divide a types MIN value by -1. Finally, avoid using `unwrap()` / `expect()`,
and make sure to give the compiler enough information that it can guarantee `copy_from_slice()`
is only being called on two slices of equal length.


#### Formatting Overhead
Implementations of `fmt::Debug` and `fmt::Display` are expensive -- the core library functions they
rely on include multiple panics and lots of (size) expensive formatting/unicode code that is unneccessary
for simple use cases. This is well-documented elsewhere: https://jamesmunns.com/blog/fmt-unreasonably-expensive/ . Accordingly, use `#[derive(Debug)]` and `fmt::Display` sparingly. For simple enums, manual `to_string(&self) -> &str` methods can be subtantially cheaper. For example, consider the following enum/use:

```rust
// BAD
#[derive(Debug)]
enum TpmState {
    Idle,
    Ready,
    CommandReception,
    CommandExecutionComplete,
    CommandExecution,
    CommandCompletion,
}

let tpm_state = TpmState::Idle;
debug!("{:?}", tpm_state);
```
```rust
// GOOD
enum TpmState {
    Idle,
    Ready,
    CommandReception,
    CommandExecutionComplete,
    CommandExecution,
    CommandCompletion,
}

impl TpmState {
    fn to_string(&self) -> &str {
        use TpmState::*;
        match self {
            Idle => "Idle",
            Ready => "Ready",
            CommandReception => "CommandReception",
            CommandExecutionComplete => "CommandExecutionComplete",
            CommandExecution => "CommandExecution",
            CommandCompletion => "CommandCompletion",
        }
    }
}

let tpm_state = TpmState::Idle;
debug!("{}", tpm_state.to_string());
```

The latter example is 112 bytes smaller than the former, despite being functionally equivalent.

For structs with runtime values that cannot easily be turned into `&str` representations
this process is not so straightforward, consider whether
the substantial overhead of calling these methods is worth the debugability improvement.

#### 64 bit division

Avoid all 64 bit division/modulus, it adds ~1kB if used, as the software techniques for performing
these are speed oriented. Often bit manipulation approaches will be much cheaper, especially if
one of the operands to the division is a compile-time constant.

#### Global Arrays
For global `const`/`static mut` variables, don’t store collections in arrays unless all
elements of the array are used.

The canonical example of this is GPIO -- if you have 100 GPIO pins, but your binary only uses 3 of them:
```rust
pub const GPIO_PINS: [Pin; 100] = [//...]; //BAD -- UNUSED PINS STILL IN BINARY
```
```rust
// GOOD APPROACH
pub const GPIO_PIN_0: Pin = Pin::new(0);
pub const GPIO_PIN_1: Pin = Pin::new(1);
pub const GPIO_PIN_2: Pin = Pin::new(2);
// ...and so on.
```
The latter approach ensures that the compiler can remove pins which are not used from the binary.

#### Combine register accesses
Combine register accesses into as few volatile operations as possible. E.g.
```rust
regs.dcfg.modify(DevConfig::DEVSPD::FullSpeed1_1);
regs.dcfg.modify(DevConfig::DESCDMA::SET);
regs.dcfg.modify(DevConfig::DEVADDR.val(0));
```
is much more expensive than
```rust
regs.dcfg.modify(
    DevConfig::DEVSPD::FullSpeed1_1 + DevConfig::DESCDMA::SET + DevConfig::DEVADDR.val(0),
);
```
because each individual modify is volatile so the compiler cannot optimize the calls together.

#### Minimize calls to `Grant::enter()`
Grants are fundamental to Tock's architecture, but the internal implementation of Grant's are
relatively complex. Further, Grant's are generic over all types that are stored in Grants, so
multiple copies of many Grant functions end up in the binary. The largest of these is
`Grant::enter()`, which is called often in capsule code. That said, it is often possible to
reduce the number of calls to this method. For example: you can combine calls to apps.enter():
```rust
// BAD -- DONT DO THIS
match command_num {
    0 => self.apps.enter(|app, _| {app.perform_cmd_0()},
    1 => self.apps.enter(|app, _| {app.perform_cmd_1()},
}
```
```rust
// GOOD -- DO THIS
self.apps.enter(|app, _| {
    match command_num {
        0 => app.perform_cmd_0(),
        1 => app.perform_cmd_1(),
    }
})
```
The latter saves ~100 bytes because each additional call to `Grant::enter()` leads to an additional
monomorphized copy of the body of `Grant::enter()`.

#### Scattered additional tips
*   Avoid calling functions in `core::str`, there is lots of overhead here that is not optimized out.
    For example: if you have a space seperated string, using `text.split_ascii_whitespace()`
    costs 150 more bytes than using `text.as_bytes().split(|b| *b == b' ');`.
*   Avoid static mut globals when possible, and favor global constants. static mut variables
    are placed in .relocate, so they consume both flash and RAM, and cannot be optimized as well
    because the compiler cannot make its normal aliasing assumptions.
*   Use const generics to pass sized arrays instead of slices, unless this will lead to monomorphization.
    In addition to removing panics on array accesses, this allows for passing smaller objects
    (references to arrays are just a pointer, slices are pointer + length), and lets the compiler
    make additional optimizations based on the known array length.
*   Test the effect of #[inline(always/never)] directives, sometimes the result will surprise you.
    If the savings are small, it is usually better to leave it up to the compiler, for increased
    resilience to future changes.
*   For functions that will not be inlined, try to keep arguments/returns in registers.
    On RISC-V, this means using <= 8 1-word arguments, no arguments > 2 words, and
    <= 2 words in return value


## Reducing the size of an out-of-tree board

In general, upstream Tock strives to produce small binaries. However, there is often a tension
between code size, debugability, and user friendliness. As a result, upstream Tock does not
always choose the most minimal configuration possible. For out-of-tree boards especially focused
on code size, there are a few steps you can take to further reduce code size:

*   Disable the `debug_panic_info` option in `kernel/src/config.rs` -- this will remove a lot
    of debug information that is provided in panics, but can reduce code size by 8-12 kB.
*   Implement your own peripheral struct that does not include peripherals you do not need.
    Often, the `DefaultPeripherals` struct for a chip may include peripherals not used in your
    project, and the structure of the interrupt handler means that you will pay the code size
    cost of the unused peripherals unless you implement your own Peripheral struct. The option
    to do this was first introduced in https://github.com/tock/tock/pull/2069 and is explained there.
*   Modify your panic handler to not use the `PanicInfo` struct. This will allow LLVM to optimize
    out the paths, panic messages, line numbers, etc. which would otherwise be stored in the binary
    to allow users to backtrace and debug panics.
*   Remove the implementation of `debug!()`: if you really want size savings, and are ok not printing
    anything, you can remove the implementation of `debug!()` and replace it with an empty macro.
    This will remove the code associated with any calls to `debug!()` in the core kernel or chip
    crates that you depend on, as well as any remaining code associated with the fmt machinery.
*   Fine-tune your inline-threshold. This can have a significant impact, but the ideal value
    is dependent on your particular code base, and changes as the compiler does -- update it when
    you update the compiler version! In practice, we have observed that very small values are often
    optimal (e.g., in the range of 2 to 10). This is done by passing `-C inline-threshold=x` to rustc.
*   Try `opt-level=s` instead of `opt-level=z`. In practice, `s` (when combined with a reduced
    inline threshold) often seems to produce smaller binaries. This is worth revisiting periodically,
    given that `z` is supposed to lead to smaller binaries than `s`
