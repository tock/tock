Kernel General Purpose I/O (GPIO) HIL
========================================

**TRD:** 103 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Amit Levy <br/>
**Draft-Created:** Feb 05, 2017<br/>
**Draft-Modified:** Feb 05, 2017<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL) for
General Purpose Input/Output (GPIO) in the Tock operating system kernel.  It
describes the Rust traits and other definitions for this service as well as the
reasoning behind them. This document is in full compliance with [TRD1].

1 Introduction
========================================

General Purpose Input/Output (GPIO) controls generic pins. User code can control
the output level on the pin (high or low), read the externally drive logic level
and often configure pull-up or pull-down resistence. Typically, microcontrollers
expose pins in groups called ports however Tock's GPIO HIL exposes pins
individually since ports often do not group pins as they are actually used on a
board. Software that wishes to control a whole port (e.g. for efficiency) should
use the per-chip implementation, which may export this feature.

The GPIO HIL is the kernel crate, in module hil::gpio. It provides three traits:

  * `kernel::hil::gpio::Pin`: Controls and reads output level and
    enables/disables interrupts on a single pin.
  * `kernel::hil::gpio::PinCtl`: Controlls the input mode on a single pin.
  * `kernel::hil::gpio::Client`: handles the callback when a GPIO interrupt is
    fired.

The rest of this document discusses each in turn.

2 `Pin` trait
========================================

The `Pin` trait is for a GPIO Pin. It has
the following functions:

```rust
pub trait Pin {
    /// Configure the GPIO pin as an output pin.
    fn make_output(&self);

    /// Configure the GPIO pin as an input pin.
    fn make_input(&self);

    /// Disable the GPIO pin and put it into its lowest power
    /// mode.
    fn disable(&self);

    /// Set the GPIO pin high. It must be an output.
    fn set(&self);

    /// Set the GPIO pin low. It must be an output.
    fn clear(&self);

    /// Toggle the GPIO pin. It must be an output.
    fn toggle(&self);

    /// Get the current state of an input GPIO pin.
    fn read(&self) -> bool;

    /// Enable an interrupt on the GPIO pin. It must
    /// be configured as an interrupt. The `identifier`
    /// can be any value and will be returned to you
    /// when the interrupt on this pin fires.
    fn enable_interrupt(&self, identifier: usize, mode: InterruptMode);

    /// Disable the interrupt for the GPIO pin.
    fn disable_interrupt(&self);

    /// Return PinID as ReturnCode::SuccessWithValue
    /// return ENOSUPORT if not implmented
    fn pin_id(&self) -> ReturnCode;
}
```

Either the `make_output` or `make_input` methods MUST be called at least once
before any other methods are called. The `make_output` method MUST put the pin
into output mode and ensure that `set`, `clear` and `toggle` are effective. The
`make_input` method MUST put the pin into input mode and ensure that `read`
correctly returns the logic level of the pin. It is undefined whether input
methods are supported in output mode and whether output methods are supported
in input mode.

The `set` method asserts the pin's output level. The `clear` method de-asserts
the pin's output level. The `toggle` method asserts the pin's output level if it
is currently de-asserted and de-asserts the pin's output level if it is
currently asserted.

The `read` method returns `true` if the current input level is asserted and
`false` if it is de-asserted. If the pin's logic level is floating the return
value is undefined.

The `enable_interrupt` method sets up an interrupt for the given
`InterruptMode`, which is defined as follows:

```rust
pub enum InterruptMode {
    RisingEdge,
    FallingEdge,
    EitherEdge,
}
```

`RisingEdge` will generate an interrupt when the pin's input goes from
de-asserted to asserted. `FallingEdge` will generate an interrupt when the pin's
input goes from asserted to de-asserted. `EitherEdge` will generate an interrupt
if either the pin's input goes from asserted to de-asserted or from de-asserted
to asserted. Implementations SHOULD ensure that edges triggers are not missed
in `EitherEdge` mode.

The `identifier` argument passed to `enable_interrupt` is user-defined and MUST
be returned in corresponding calls to the `Client` trait's `fired` method
(below). For example, users MAY use this to differentiate between interrupts
from different pins.

The `disable_interrupts` method disables interrupts on the pin. Once
`disable_interrupts` is called, the implementation MUST NOT deliver interrupts
to the user via the `Client` trait until `enable_interrupts` is called.

3 PinCtl
========================================

The `PinCtl` trait is for controlling the input mode of a particular pin. It is
OPTIONAL and shoul only be implemented on microcontrollers that provide this
control.

```rust
pub trait PinCtl {
    /// Configure whether the pin should have a pull-up or pull-down resistor or
    /// neither.
    fn set_input_mode(&self, InputMode);
}

pub enum InputMode {
    PullUp,
    PullDown,
    PullNone,
}
```

The `set_input_mode` configures whether the microcontroller should apply pull-up
or pull-down resistence to the pin.

4 Client
========================================

The `Client` trait is how a caller provides a callback to the `Pin`
implementation. Using a function defined outside the `Pin` trait, it registers a
reference implementing the `Client` trait with the `Pin` implementation.

```rust
pub trait Client {
    fn fired(&self, identifier: usize);
}
```

Whenever an interrupt occurs on the pin it invokes the `fired`
method. `identifier` MUST contain the value passed to the `Pin` trait's `enable_interrupts` method.


5 Example Implementation
========================================

6 Authors' Address
========================================
```
email - amit@amitlevy.com
```