//! Interface for direct control of GPIO pins.

/// Enum for configuring any pull-up or pull-down resistors on the GPIO pin.
#[derive(Debug)]
pub enum InputMode {
    PullUp,
    PullDown,
    PullNone,
}

/// Enum for selecting which edge to trigger interrupts on.
#[derive(Debug)]
pub enum InterruptMode {
    RisingEdge,
    FallingEdge,
    EitherEdge,
}

pub trait PinCtl {
    /// Configure whether the pin should have a pull-up or pull-down resistor or
    /// neither.
    fn set_input_mode(&self, _: InputMode);
}

/// Interface for synchronous GPIO pins.
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
}

/// Interface for users of synchronous GPIO. In order
/// to receive interrupts, the user must implement
/// this `Client` interface.
pub trait Client {
    /// Called when an interrupt occurs. The `identifier` will
    /// be the same value that was passed to `enable_interrupt()`
    /// when the interrupt was configured.
    fn fired(&self, identifier: usize);
}
