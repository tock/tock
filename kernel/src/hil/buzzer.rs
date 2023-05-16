// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for buzzer use.

use crate::ErrorCode;

pub trait BuzzerClient {
    /// Called when the current sound played by the buzzer has finished
    /// or it was stopped.
    fn buzzer_done(&self, status: Result<(), ErrorCode>);
}

/// The Buzzer HIL is used to play a sound on a buzzer at a fixed frequency and
/// for a certain duration.
pub trait Buzzer<'a> {
    /// Play a sound at a chosen frequency and for a chosen duration.
    /// Once the buzzer finishes buzzing, the `buzzer_done()` callback
    /// is called.
    /// If it is called while the buzzer is playing, the buzzer command will be
    /// overriden with the new frequency and duration values.
    ///
    /// Return values:
    ///
    /// - `Ok(())`: The attempt at starting the buzzer was successful.
    /// - `FAIL`: Cannot start the buzzer.
    fn buzz(&self, frequency_hz: usize, duration_ms: usize) -> Result<(), ErrorCode>;

    /// Stop the sound currenty playing.
    /// After the buzzer is successfully stopped, the `buzzer_done()`
    /// callback is called.
    ///
    /// Return values:
    ///
    /// - `Ok(())`: The attempt at stopping the buzzer was successful.
    /// - `FAIL`: Cannot stop the buzzer.
    /// - `OFF`: The buzzer wasn't playing a sound when the stop command was called.
    fn stop(&self) -> Result<(), ErrorCode>;

    /// Set the client to be used for callbacks of the Buzzer
    /// implementation.
    fn set_client(&self, client: &'a dyn BuzzerClient);
}
