//! Interface for buzzer use.

use crate::ErrorCode;

pub trait BuzzerClient {
    /// Called when the current sound played by the buzzer has finished
    /// or it was stopped.
    fn buzzer_done(&self, status: Result<(), ErrorCode>);
}

pub trait Buzzer<'a> {
    /// Play a sound at a chosen frequency and for a chosen duration.
    /// After the buzzer starts playing, an alarm will be set and once
    /// it fires after the set duration, the `buzzer_done()` callback
    /// is called.
    /// Return values:
    ///
    /// - `Ok(())`: The attempt at starting the buzzer was successful.
    /// - `BUSY`: The buzzer is already in use.
    fn buzz(&self, frequency_hz: usize, duration_ms: usize) -> Result<(), ErrorCode>;

    /// Stop the sound currenty playing.
    /// After the alarm is disarmed and the buzzer is successfully
    /// stopped, the `buzzer_done()` is called.
    ///
    /// Return values:
    ///
    /// - `Ok(())`: The attempt at disarming the alarm was successful.
    /// - `FAIL`: The alarm could not be disarmed.
    fn stop(&self) -> Result<(), ErrorCode>;

    /// Set the client to be used for callbacks of the Buzzer
    /// implementation.
    fn set_client(&self, client: &'a dyn BuzzerClient);
}
