Kernel General Purpose I/O (GPIO) HIL
========================================

**TRD:** 103 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Amit Levy, Philip Levis <br/>
**Draft-Created:** Feb 05, 2017<br/>
**Draft-Modified:** April 09, 2021<br/>
**Draft-Version:** 3<br/>
**Draft-Discuss:** devel@lists.tockos.org</br>

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

The GPIO HIL is the kernel crate, in module hil::gpio. It provides the following traits:

  * `kernel::hil::gpio::Output` controls an output pin.
  * `kernel::hil::gpio::Input` controls an input pin.
  * `kernel::hil::gpio::Configure` configures a pin.
  * `kernel::hil::gpio::ConfigureInputOutput` configures a pin that can simultaneously be an
     input and an output (some hardware supports this. It depends on `Configure`).
  * `kernel::hil::gpio::Interrupt` controls an interrupt pin. It depends on `Input`.
  * `kernel::hil::gpio::Client` handles callbacks from pin interrupts.
  * `kernel::hil::gpio::InterruptWithValue` controls an interrupt pin that provides a value in
    its callbacks. It depends on `Input`.
  * `kernel::hil::gpio::ClientWithValue` handles callbacks from pin interrupts that provide
    a value (`InterruptWithValue`). 
  * `kernel::hil::gpio::Pin depends on `Input`, `Output`, and `Configure`.
  * `kernel::hil::gpio::InterruptPin depends on `Pin` and `Interrupt`.
  * `kernel::hil::gpio::InterruptValuePin depends on `Pin` and `InterruptWithValue`.

The rest of this document discusses each in turn.

2 Output 
========================================

The `Output` trait controls a pin that is an output. It has
four methods:

```rust
pub trait Output {
    /// Set the GPIO pin high. If the pin is not an output or
    /// input/output, this call is ignored.
    fn set(&self);

    /// Set the GPIO pin low. If the pin is not an output or
    /// input/output, this call is ignored.
    fn clear(&self);

    /// Toggle the GPIO pin. If the pin was high, set it low. If
    /// the pin was low, set it high. If the pin is not an output or
    /// input/output, this call is ignored. Return the new value
    /// of the pin (false is cleared, true is set).
    fn toggle(&self) -> bool;

    /// Activate or deactivate a GPIO pin, for a given activation mode.
    fn write_activation(&self, state: ActivationState, mode: ActivationMode);
}
```

The `write_activation` method has a default implementation. This method
allows software to interact with a GPIO using logical, rather than physical
behavior. For example, consider a button which is "active" when it is
pushed. If the button is connected to ground and a pull-up input pin,
then it is active when the pin is low; if it is connected to Vdd and
a pull-down input pin, it is active when the pin is high. Similarly, 
an LED may be connected through an PNP transistor, whose base is 
controlled by a GPIO pin, such that setting the pin low turns on the
LED and setting the pin high turns it off. Rather than keeping track
of these polarities, software can use `ActivationState` to specify
whether the device should be active or inactive, and `ActivationMode`
specifies the polarity.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivationState {
    Inactive = 0,
    Active = 1,
}

/// Whether a GPIO is in the `ActivationState::Active` when the signal is high
/// or low.
#[derive(Clone, Copy)]
pub enum ActivationMode {
    ActiveHigh,
    ActiveLow,
}
```

3 Input
========================================

The `Input` trait controls an input pin. It has two methods:

```rust
pub trait Input {
    /// Get the current state of an input GPIO pin. For an output
    /// pin, return the output; for an input pin, return the input;
    /// for disabled or function pins the value is undefined.
    fn read(&self) -> bool;

    /// Get the current state of a GPIO pin, for a given activation mode.
    fn read_activation(&self, mode: ActivationMode) -> ActivationState {
        let value = self.read();
        match (mode, value) {
            (ActivationMode::ActiveHigh, true) | (ActivationMode::ActiveLow, false) => {
                ActivationState::Active
            }
            (ActivationMode::ActiveLow, true) | (ActivationMode::ActiveHigh, false) => {
                ActivationState::Inactive
            }
        }
    }
}
```

The `read_activation` method is similar to the `write_activation` method in `Output`,
described below, but operates on input rather than output bits.

4 Configure
========================================

The `Configure` trait allows a caller to configure a GPIO pin. It has 10 methods,
two of which have default implementations.

```rust
pub enum Configuration {
    LowPower,    // Cannot be read or written or used; effectively inactive.
    Input,       // Calls to the `Input` trait are valid.
    Output,      // Calls to the `Output` trait are valid.
    InputOutput, // Calls to both the `Input` and `Output` traits are valid.
    Function,    // Chip-specific, requires chip-specific API for more detail,
    Other,       // In a state not covered by other values.
}

pub enum FloatingState {
    PullUp,
    PullDown,
    PullNone,
}

pub trait Configure {
    fn configuration(&self) -> Configuration;
    fn make_output(&self) -> Configuration;
    fn disable_output(&self) -> Configuration;
    fn make_input(&self) -> Configuration;
    fn disable_input(&self) -> Configuration;
    fn deactivate_to_low_power(&self);
    fn set_floating_state(&self, state: FloatingState);
    fn floating_state(&self) -> FloatingState;

    // Have default implementations
    fn is_input(&self) -> bool;
    fn is_output(&self) -> bool;
}
```

The `Configuration` enum describes the current configuration of a pin.
The key property of the enumeration, which prompts its use, is the
fact that some hardware allows a pin to simultaneously be an input
and an output, while in other hardware these states are mutually
exclusive. For example, the Atmel SAM4L GPIO pins are always inputs,
and reading them "indicates the level of the GPIO pins regardless of 
the pins being driven by the GPIO or by an external component". In
contrast, on the nRF52 series, a GPIO pin is either an input or
an output.

The `Configuration` enumeration encapsulates this by reporting
the current configuration after a change. For example, suppose
a pin has `Configuration::Input` and software calls `make_output`
on it. A SAM4L will return `Configuration::InputOutput` while
an nRF52 will return `Configuration::Output`.

If a client requires a pin be both an input and an output, it can
use the `ConfigureInputOutput` trait:

```rust
pub trait ConfigureInputOutput: Configure {
    /// Make the pin a simultaneously input and output; should always
    /// return `Configuration::InputOutput`.
    fn make_input_output(&self) -> Configuration;
    fn is_input_output(&self) -> bool;
}
```

Chips that support simultaneous input/output MAY implement this
trait, while others that do not support simultaneous input/output
MUST NOT implement this trait. Therefore, at compile time, one can
distinguish whether the client can operate properly.

The `Configure::deactivate_to_low_power` method exists because the best
configuration for GPIO pins can depend not only on the chip but
also how they are connected in a system. This method puts the 
pin into whatever state is lowest power and causes it to be
both unreadable and unwritable. E.g., even if the lowest power
state is as a pull-down input, when in this state a client cannot
read the pin. Blocking functionality in this way tries to 
prevent clients making assumptions about the underlying hardware.

5 Interrupt and Client
========================================

The `Interrupt` and `Client` traits are how software can control
and handle interrupts generated from a GPIO pin.

```rust
pub enum InterruptEdge {
    RisingEdge,
    FallingEdge,
    EitherEdge,
}

pub trait Interrupt<'a>: Input {
    fn set_client(&self, client: &'a dyn Client);
    fn enable_interrupts(&self, mode: InterruptEdge);
    fn disable_interrupts(&self);
    fn is_pending(&self) -> bool;
}

pub trait Client {
    fn fired(&self);
}
```

These traits assume that hardware can generate interrupts
on rising, falling, or either edges. They do not support
level (high/low) interrupts. Some hardware does not support
level interrupts. The nRF52 GPIOTE peripheral, for example,
doesn't. Chips or capsules that wish to support level interrupts
can define a new trait that depends on the `Interrupt` trait.

An important aspect of these traits is that they cannot fail.
For example, `enable_interrupts` does not return anything,
so there is no way to signal failure. Because interrupts
are an extremely low-level aspect of the kernel, these traits
preclude there being complex conditional logic that might cause
them to fail (e.g., some form of dynamic allocation or
mapping). Interrupt implementations that can fail at runtime
should define and use alternative traits.


5 InterruptWithValue and ClientWithValue 
========================================

The `InterruptWithValue` and `ClientWithValue` traits extend
interrupt handling to pass a value with an interrupt. This
is useful when a single method needs to handle callbacks
from multiple pins. Each pin's interrupt can have a different
value, and the callback function can determine which pin
the interrupt is from based on the value passed. This
is used, for example, in the GPIO capsule that allows
userspace to handle interrupts from multiple interrupt pins.
If there weren't a `ClientWithValue` trait, the capsule would
have to define N different callback methods for N pins. These
would likely each then call a helper function with a parameter
indicating which one was invoked: `ClientWithValue` provides
this mechanism automatically.

```rust
pub trait InterruptWithValue<'a>: Input {
    fn set_client(&self, client: &'a dyn ClientWithValue);
    fn enable_interrupts(&self, mode: InterruptEdge) -> Result<(), ErrorCode>;
    fn disable_interrupts(&self);
    fn is_pending(&self) -> bool;

    fn set_value(&self, value: u32);
    fn value(&self) -> u32;
}

pub trait ClientWithValue {
    fn fired(&self, value: u32);
}
```

The `InterruptWithValue` trait does not depend on the
`Interrupt` trait because its client has a different type.
Supporting both types of clients would require case logic
within the GPIO implementation, whose cost (increased storage
for the variably-typed reference, increased code for handling
the cases) is not worth the benefit (being able to pass a 
`Client` to an `InterruptWithValue`.

The GPIO HIL provides a standard implementation of a wrapper
that implements `InterruptWithValue`. It wraps around an
implementation of `Interrupt`, defining itself as a `Client`
and using `Client:callback` to invoke `ClientWithValue::callback`.

```rust
impl<'a, IP: InterruptPin<'a>> InterruptValueWrapper<'a, IP> {
    pub fn new(pin: &'a IP) -> Self {...}
```

`InterruptValueWrapper` implements `InterruptWithValue`, `Client`, 
`Input`, `Output`, and `Configure`.

6 Composite Traits: Pin, InterruptPin, InterruptValuePin
========================================

The GPIO HIL uses fine-grained traits in order to follow the security
principle of least privilege. For example, something that needs to be able to read 
a GPIO pin should not necessarily be able to reconfigure or write to it.
However, because handling multiple small traits at once can be
cumbersome, the GPIO HIL defines several standard composite traits:

```rust
pub trait Pin: Input + Output + Configure {}
pub trait InterruptPin<'a>: Pin + Interrupt<'a> {}
pub trait InterruptValuePin<'a>: Pin + InterruptWithValue<'a> {}
```

6 Example Implementation
========================================

As of this writing (April 2021; Tock v1.6 and v2.0), there are example implementations of the GPIO HIL for the Atmel
SAM4L, lowRISC, nrf5x, sifive, stm32f303xc, stm32f4xx, imxrt10xx,
apollo3, and msp432 chips. The lowrisc, sam4l, and sifive chips
support `Configuration::InputOutput` mode, while the others
support only input or output mode.

7 Authors' Address
========================================
```
Philip Levis
414 Gates Hall
Stanford University
Stanford, CA 94305
email: Philip Levis <pal@cs.stanford.edu>
phone: +1 650 725 9046

Amit Levy
email: Amit Levy <aalevy@cs.princeton.edu>
```

[TRD1]: trd1-trds.md
