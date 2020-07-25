use crate::common::cells::OptionalCell;
use crate::ReturnCode;

use core::cell::Cell;

/// Enum for configuring any pull-up or pull-down resistors on the GPIO pin.
#[derive(Clone, Copy, Debug)]
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
    /// Cannot be read or written or used; effectively inactive.
    LowPower,
    /// Calls to the `Input` trait are valid.
    Input,
    /// Calls to the `Output` trait are valid.
    Output,
    /// Calls to both the `Input` and `Output` traits are valid.
    InputOutput,
    /// Chip-specific, requires chip-specific API for more detail,
    Function,
    /// In a state not covered by other values.
    Other,
}

/// Some GPIOs can be semantically active or not.
/// For example:
/// - a LED is active when emitting light,
/// - a button GPIO is active when pressed.
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

/// The Pin trait allows a pin to be used as either input
/// or output and to be configured.
pub trait Pin: Input + Output + Configure {}

/// The InterruptPin trait allows a pin to be used as either
/// input or output and also to source interrupts.
pub trait InterruptPin<'a>: Pin + Interrupt<'a> {}

/// The InterruptValuePin trait allows a pin to be used as
/// either input or output and also to source interrupts which
/// pass a value.
pub trait InterruptValuePin<'a>: Pin + InterruptWithValue<'a> {}
/// Control and configure a GPIO pin.
pub trait Configure {
    /// Return the current pin configuration.
    fn configuration(&self) -> Configuration;

    /// Make the pin an output, returning the current configuration,
    /// which should be either `Configuration::Output` or
    /// `Configuration::InputOutput`.
    fn make_output(&self) -> Configuration;
    /// Disable the pin as an output, returning the current configuration.
    fn disable_output(&self) -> Configuration;

    /// Make the pin an input, returning the current configuration,
    /// which should be ither `Configuration::Input` or
    /// `Configuration::InputOutput`.
    fn make_input(&self) -> Configuration;
    /// Disable the pin as an input, returning the current configuration.
    fn disable_input(&self) -> Configuration;

    /// Put a pin into its lowest power state, with no guarantees on
    /// if it is enabled or not. Implementations are free to use any
    /// state (e.g. input, output, disable, etc.) the hardware pin
    /// supports to ensure the pin is as low power as possible.
    /// Re-enabling the pin requires reconfiguring it (i.e. the state
    /// of its enabled configuration is not stored).
    fn deactivate_to_low_power(&self);

    /// Set the floating state of the pin.
    fn set_floating_state(&self, state: FloatingState);
    /// Return the current floating state of the pin.
    fn floating_state(&self) -> FloatingState;

    /// Return whether the pin is an input (reading from
    /// the Input trait will return valid results). Returns
    /// true if the pin is in Configuration::Input or
    /// Configuration::InputOutput.
    fn is_input(&self) -> bool {
        match self.configuration() {
            Configuration::Input | Configuration::InputOutput => true,
            _ => false,
        }
    }

    /// Return whether the pin is an output (writing to
    /// the Output trait will change the output of the pin).
    /// Returns true if the pin is in Configuration::Output or
    /// Configuration::InputOutput.
    fn is_output(&self) -> bool {
        match self.configuration() {
            Configuration::Output | Configuration::InputOutput => true,
            _ => false,
        }
    }
}

/// Configuration trait for pins that can be simultaneously
/// input and output. Having this trait allows an implementation
/// to statically verify this is possible.
pub trait ConfigureInputOutput: Configure {
    /// Make the pin a simultaneously input and output; should always
    /// return `Configuration::InputOutput`.
    fn make_input_output(&self) -> Configuration;
    fn is_input_output(&self) -> bool;
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

    /// Activate or deactivate a GPIO pin, for a given activation mode.
    fn write_activation(&self, state: ActivationState, mode: ActivationMode) {
        match (state, mode) {
            (ActivationState::Active, ActivationMode::ActiveHigh)
            | (ActivationState::Inactive, ActivationMode::ActiveLow) => {
                self.set();
            }
            (ActivationState::Active, ActivationMode::ActiveLow)
            | (ActivationState::Inactive, ActivationMode::ActiveHigh) => {
                self.clear();
            }
        }
    }
}

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

pub trait Interrupt<'a>: Input {
    /// Set the client for interrupt events.
    fn set_client(&self, client: &'a dyn Client);

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
pub trait InterruptWithValue<'a>: Input {
    /// Set the client for interrupt events.
    fn set_client(&self, client: &'a dyn ClientWithValue);

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
pub struct InterruptValueWrapper<'a, IP: InterruptPin<'a>> {
    value: Cell<u32>,
    client: OptionalCell<&'a dyn ClientWithValue>,
    source: &'a IP,
}

impl<'a, IP: InterruptPin<'a>> InterruptValueWrapper<'a, IP> {
    pub fn new(pin: &'a IP) -> Self {
        Self {
            value: Cell::new(0),
            client: OptionalCell::empty(),
            source: pin,
        }
    }

    pub fn finalize(&'static self) -> &'static Self {
        self.source.set_client(self);
        self
    }
}

impl<'a, IP: InterruptPin<'a>> InterruptWithValue<'a> for InterruptValueWrapper<'a, IP> {
    fn set_value(&self, value: u32) {
        self.value.set(value);
    }

    fn value(&self) -> u32 {
        self.value.get()
    }

    fn set_client(&self, client: &'a dyn ClientWithValue) {
        self.client.replace(client);
    }

    fn is_pending(&self) -> bool {
        self.source.is_pending()
    }

    fn enable_interrupts(&self, edge: InterruptEdge) -> ReturnCode {
        self.source.enable_interrupts(edge);
        ReturnCode::SUCCESS
    }

    fn disable_interrupts(&self) {
        self.source.disable_interrupts();
    }
}

impl<'a, IP: InterruptPin<'a>> Input for InterruptValueWrapper<'a, IP> {
    fn read(&self) -> bool {
        self.source.read()
    }
}

impl<'a, IP: InterruptPin<'a>> Configure for InterruptValueWrapper<'a, IP> {
    fn configuration(&self) -> Configuration {
        self.source.configuration()
    }

    fn make_output(&self) -> Configuration {
        self.source.make_output()
    }

    fn disable_output(&self) -> Configuration {
        self.source.disable_output()
    }

    fn make_input(&self) -> Configuration {
        self.source.make_input()
    }

    fn disable_input(&self) -> Configuration {
        self.source.disable_input()
    }

    fn deactivate_to_low_power(&self) {
        self.source.deactivate_to_low_power();
    }

    fn set_floating_state(&self, state: FloatingState) {
        self.source.set_floating_state(state);
    }

    fn floating_state(&self) -> FloatingState {
        self.source.floating_state()
    }

    fn is_input(&self) -> bool {
        self.source.is_input()
    }

    fn is_output(&self) -> bool {
        self.source.is_input()
    }
}

impl<'a, IP: InterruptPin<'a>> Output for InterruptValueWrapper<'a, IP> {
    fn set(&self) {
        self.source.set();
    }

    fn clear(&self) {
        self.source.clear();
    }

    fn toggle(&self) -> bool {
        self.source.toggle()
    }
}

impl<'a, IP: InterruptPin<'a>> InterruptValuePin<'a> for InterruptValueWrapper<'a, IP> {}
impl<'a, IP: InterruptPin<'a>> Pin for InterruptValueWrapper<'a, IP> {}

impl<'a, IP: InterruptPin<'a>> Client for InterruptValueWrapper<'a, IP> {
    fn fired(&self) {
        self.client.map(|c| c.fired(self.value()));
    }
}
