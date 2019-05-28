use crate::common::cells::OptionalCell;
use crate::ReturnCode;

use core::cell::Cell;

/// Enum for configuring any pull-up or pull-down resistors on the GPIO pin.
#[derive(Debug)]
pub enum FloatingState {
    PullUp,
    PullDown,
    PullNone,
}

/// Enum for selecting which edge to trigger interrupts on.
#[derive(Debug)]
pub enum InterruptEdge {
    RisingEdge,
    FallingEdge,
    EitherEdge,
}

/// Enum for which state the pin is in. Some MCUs can support Input/Output pins,
/// so this is a valid option. `Function` means the pin has been configured to
/// a special function. Determining which function it outside the scope of the HIL,
/// and should instead use a chip-specific API.
#[derive(Debug)]
pub enum Configuration {
    LowPower,
    Input,
    Output,
    InputOutput,
    Function, // Chip-specific, requires chip-specific API for more detail
    Unknown,
}

/// The Pin trait allows a pin to be used as either input
/// or output and to be configured.
pub trait Pin: Input + Output + Configure {}

/// The InterruptPin trait allows a pin to be used as either
/// input or output and also to source interrupts.
pub trait InterruptPin: Pin + Interrupt {}

/// The InterruptValuePin trait allows a pin to be used as
/// either input or output and also to source interrupts which
/// pass a value.
pub trait InterruptValuePin: Pin + InterruptWithValue {}

pub trait Configure {
    fn configuration(&self) -> Configuration;
    fn make_output(&self) -> Configuration;
    fn disable_output(&self) -> Configuration;
    fn make_input(&self) -> Configuration;
    fn disable_input(&self) -> Configuration;

    // Disable the pin and put it into its lowest power state.
    // Re-enabling the pin requires reconfiguring it (state of
    // its enabled configuration is not stored).
    fn low_power(&self);

    fn set_floating_state(&self, state: FloatingState);
    fn floating_state(&self) -> FloatingState;

    fn is_input(&self) -> bool;
    fn is_output(&self) -> bool;
}

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
    /// of the pin.
    fn toggle(&self) -> bool;
}

pub trait Input {
    /// Get the current state of an input GPIO pin. For an output
    /// pin, return the output; for an input pin, return the input;
    /// for disabled or function pins the value is undefined.
    fn read(&self) -> bool;
}

pub trait Interrupt: Input {
    /// Set the client for interrupt events.
    fn set_client(&self, client: &'static Client);

    /// Enable an interrupt on the GPIO pin. This does not
    /// configure the pin except to enable an interrupt: it
    /// should be separately configured as an input, etc.
    fn enable_interrupts(&self, mode: InterruptEdge);

    /// Disable interrupts for the GPIO pin.
    fn disable_interrupts(&self);

    /// Return whether this interrupt is pending
    fn is_pending(&self) -> bool;
}

/// Interface for users of synchronous GPIO interrupts. In order
/// to receive interrupts, the user must implement
/// this `Client` interface.
pub trait Client {
    /// Called when an interrupt occurs. The `identifier` will
    /// be the same value that was passed to `enable_interrupt()`
    /// when the interrupt was configured.
    fn fired(&self);
}

/// Interface that wraps an interrupt to pass a value when it
/// triggers. The standard use case for this trait is when several
/// interrupts call the same callback function and it needs to
/// distinguish which one is calling it by giving each one a unique
/// value.
pub trait InterruptWithValue: Input {
    /// Set the client for interrupt events.
    fn set_client(&self, client: &'static ClientWithValue);

    /// Set the underlying interrupt source.
    fn set_source(&'static self, source: &'static InterruptPin);

    /// Enable an interrupt on the GPIO pin. This does not
    /// configure the pin except to enable an interrupt: it
    /// should be separately configured as an input, etc.
    /// Returns:
    ///    SUCCESS - the interrupt was set up properly
    ///    FAIL    - the interrupt was not set up properly; this is due to
    ///              not having an underlying interrupt source yet, i.e.
    ///              the struct is not yet fully initialized.
    fn enable_interrupts(&self, mode: InterruptEdge) -> ReturnCode;

    /// Disable interrupts for the GPIO pin.
    fn disable_interrupts(&self);

    /// Return whether this interrupt is pending
    fn is_pending(&self) -> bool;

    /// Set the value that will be passed to clients on an
    /// interrupt.
    fn set_value(&self, value: u32);

    /// Return the value that is passed to clients on an
    /// interrupt.
    fn value(&self) -> u32;
}

/// Interfaces for users of GPIO interrupts who handle many interrupts
/// with the same function. The value passed in the callback allows the
/// callback to distinguish which interrupt fired.
pub trait ClientWithValue {
    fn fired(&self, value: u32);
}

/// Standard implementation of InterruptWithValue: handles an
/// `gpio::Client::fired` and passes it up as a
/// `gpio::ClientWithValue::fired`.
pub struct InterruptValueWrapper {
    value: Cell<u32>,
    client: OptionalCell<&'static ClientWithValue>,
    source: OptionalCell<&'static InterruptPin>,
}

impl InterruptValueWrapper {
    pub fn new() -> InterruptValueWrapper {
        InterruptValueWrapper {
            value: Cell::new(0),
            client: OptionalCell::empty(),
            source: OptionalCell::empty(),
        }
    }
}

impl InterruptWithValue for InterruptValueWrapper {
    fn set_value(&self, value: u32) {
        self.value.set(value);
    }

    fn value(&self) -> u32 {
        self.value.get()
    }

    fn set_client(&self, client: &'static ClientWithValue) {
        self.client.replace(client);
    }

    fn is_pending(&self) -> bool {
        self.source.map_or(false, |s| s.is_pending())
    }

    fn enable_interrupts(&self, edge: InterruptEdge) -> ReturnCode {
        self.source.map_or(ReturnCode::FAIL, |s| {
            s.enable_interrupts(edge);
            ReturnCode::SUCCESS
        })
    }

    fn disable_interrupts(&self) {
        self.source.map(|s| s.disable_interrupts());
    }

    fn set_source(&'static self, source: &'static InterruptPin) {
        source.set_client(self);
        self.source.replace(source);
    }
}

impl Input for InterruptValueWrapper {
    fn read(&self) -> bool {
        self.source.map_or(false, |s| s.read())
    }
}

impl Configure for InterruptValueWrapper {
    fn configuration(&self) -> Configuration {
        self.source
            .map_or(Configuration::Unknown, |s| s.configuration())
    }

    fn make_output(&self) -> Configuration {
        self.source
            .map_or(Configuration::Unknown, |s| s.make_output())
    }

    fn disable_output(&self) -> Configuration {
        self.source
            .map_or(Configuration::Unknown, |s| s.disable_output())
    }

    fn make_input(&self) -> Configuration {
        self.source
            .map_or(Configuration::Unknown, |s| s.make_input())
    }

    fn disable_input(&self) -> Configuration {
        self.source
            .map_or(Configuration::Unknown, |s| s.disable_input())
    }

    fn low_power(&self) {
        self.source.map(|s| s.low_power());
    }

    fn set_floating_state(&self, state: FloatingState) {
        self.source.map(|s| s.set_floating_state(state));
    }

    fn floating_state(&self) -> FloatingState {
        self.source
            .map_or(FloatingState::PullNone, |s| s.floating_state())
    }

    fn is_input(&self) -> bool {
        self.source.map_or(false, |s| s.is_input())
    }

    fn is_output(&self) -> bool {
        self.source.map_or(false, |s| s.is_input())
    }
}

impl Output for InterruptValueWrapper {
    fn set(&self) {
        self.source.map(|s| s.is_input());
    }

    fn clear(&self) {
        self.source.map(|s| s.clear());
    }

    fn toggle(&self) -> bool {
        self.source.map_or(false, |s| s.toggle())
    }
}

impl InterruptValuePin for InterruptValueWrapper {}
impl Pin for InterruptValueWrapper {}

impl Client for InterruptValueWrapper {
    fn fired(&self) {
        self.client.map(|c| c.fired(self.value()));
    }
}
