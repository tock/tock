//! Interfaces for Pulse Width Modulation output.

use returncode::ReturnCode;

/// PWM control for a single pin.
pub trait Pwm {
    /// The chip-dependent type of a PWM pin.
    type Pin;

    /// Generate a PWM single on the given pin at the given frequency and duty
    /// cycle.
    ///
    /// - `frequency_hz` is specified in Hertz.
    /// - `duty_cycle` is specified as a portion of the max duty cycle supported
    ///   by the chip. Clients should call `get_maximum_duty_cycle()` to get the
    ///   value that corresponds to 100% duty cycle, and divide that
    ///   appropriately to get the desired duty cycle value. For example, a 25%
    ///   duty cycle would be `PWM0.get_maximum_duty_cycle() / 4`.
    fn start(&self, pin: &Self::Pin, frequency_hz: usize, duty_cycle: usize) -> ReturnCode;

    /// Stop a PWM pin output.
    fn stop(&self, pin: &Self::Pin) -> ReturnCode;

    /// Return the maximum PWM frequency supported by the PWM implementation.
    /// The frequency will be specified in Hertz.
    fn get_maximum_frequency_hz(&self) -> usize;

    /// Return an opaque number that represents a 100% duty cycle. This value
    /// will be hardware specific, and essentially represents the precision
    /// of the underlying PWM hardware.
    ///
    /// Users of this HIL should divide this number to calculate a duty cycle
    /// value suitable for calling `start()`. For example, to generate a 50%
    /// duty cycle:
    ///
    /// ```ignore
    /// let max = PWM0.get_maximum_duty_cycle();
    /// let dc  = max / 2;
    /// PWM0.start(pin, freq, dc);
    /// ```
    fn get_maximum_duty_cycle(&self) -> usize;
}

/// Higher-level PWM interface that restricts the user to a specific PWM pin.
/// This is particularly useful for passing to capsules that need to control
/// only a specific pin.
pub trait PwmPin {
    /// Start a PWM output. Same as the `start` function in the `Pwm` trait.
    fn start(&self, frequency_hz: usize, duty_cycle: usize) -> ReturnCode;

    /// Stop a PWM output. Same as the `stop` function in the `Pwm` trait.
    fn stop(&self) -> ReturnCode;

    /// Return the maximum PWM frequency supported by the PWM implementation.
    /// Same as the `get_maximum_frequency_hz` function in the `Pwm` trait.
    fn get_maximum_frequency_hz(&self) -> usize;

    /// Return an opaque number that represents a 100% duty cycle. This value
    /// Same as the `get_maximum_duty_cycle` function in the `Pwm` trait.
    fn get_maximum_duty_cycle(&self) -> usize;
}
