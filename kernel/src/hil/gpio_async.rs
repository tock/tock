//! Interface for GPIO pins that require split-phase operation to control.

use crate::hil;
use crate::returncode::ReturnCode;

/// Interface for banks of asynchronous GPIO pins. GPIO pins are asynchronous
/// when there is an asynchronous interface used to control them. The most
/// common example is when using a GPIO extender on an I2C or SPI bus. With
/// asynchronous GPIO functions, every config action results in an eventual
/// callback function that indicates that the configuration has finished
/// (unless the initial function call returns an error code, then no callback
/// will be generated).
///
/// Asynchronous GPIO pins are grouped into ports because it is assumed that
/// the remote entity that is controlling the pins can control multiple pins.
/// Typically, a port will be provided by a particular driver.
///
/// The API for the Port mirrors the synchronous GPIO interface.
pub trait Port {
    /// Try to disable a GPIO pin. This cannot be supported for all devices.
    fn disable(&self, pin: usize) -> ReturnCode;

    /// Configure a pin as an ouput GPIO.
    fn make_output(&self, pin: usize) -> ReturnCode;

    /// Configure a pin as an input GPIO. Not all InputMode settings may
    /// be supported by a given device.
    fn make_input(&self, pin: usize, mode: hil::gpio::InputMode) -> ReturnCode;

    /// Get the state (0 or 1) of an input pin. The value will be returned
    /// via a callback.
    fn read(&self, pin: usize) -> ReturnCode;

    /// Toggle an output GPIO pin.
    fn toggle(&self, pin: usize) -> ReturnCode;

    /// Assert a GPIO pin high.
    fn set(&self, pin: usize) -> ReturnCode;

    /// Clear a GPIO pin low.
    fn clear(&self, pin: usize) -> ReturnCode;

    /// Setup an interrupt on a GPIO input pin. The identifier should be
    /// the port number and will be returned when the interrupt callback
    /// fires.
    fn enable_interrupt(
        &self,
        pin: usize,
        mode: hil::gpio::InterruptMode,
        identifier: usize,
    ) -> ReturnCode;

    /// Disable an interrupt on a GPIO input pin.
    fn disable_interrupt(&self, pin: usize) -> ReturnCode;
}

/// The gpio_async Client interface is used to both receive callbacks
/// when a configuration command finishes and to handle interrupt events
/// from pins with interrupts enabled.
pub trait Client {
    /// Called when an interrupt occurs. The pin that interrupted is included,
    /// and the identifier that was passed with the call to `enable_interrupt`
    /// is also returned.
    fn fired(&self, pin: usize, identifier: usize);

    /// Done is called when a configuration command finishes.
    fn done(&self, value: usize);
}
