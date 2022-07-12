//! Interface for buzzer use.

use crate::ErrorCode;

// Client for Buzzer implementations
pub trait BuzzerClient {
    // Called when the current sound played by the buzzer is finished.
    fn buzzer_done(&self, status: Result<(), ErrorCode>);
}

// The Buzzer command (frequency and duration of sound)
#[derive(Clone, Copy, PartialEq)]
pub enum BuzzerCommand {
    Buzz {
        frequency_hz: usize,
        duration_ms: usize,
    },
}

pub trait Buzzer<'a> {
    // Play a sound at a chosen frequency and for a chosen duration.
    // Returns an error if it fails.
    fn buzz(&self, command: BuzzerCommand) -> Result<(), ErrorCode>;

    // Stop the sound currenty playing.
    // Returns and error if it fails.
    fn stop(&self) -> Result<(), ErrorCode>;

    // Set the client to be used for callbacks of the Buzzer
    // implementation.
    fn set_client(&self, client: &'a dyn BuzzerClient);
}
