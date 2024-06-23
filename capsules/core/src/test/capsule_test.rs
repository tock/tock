// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Interface for running capsule tests.
//!
//! As Tock capsules are asynchronous, it is difficult for a test runner to
//! determine when a test has finished. This interface provides a `done()`
//! callback used by the test implementation to notify when the test has
//! completed.
//!
//! A simple example of a test capsule using this interface:
//!
//! ```rust,ignore
//! pub struct TestSensorX {
//!     client: OptionalCell<&'static dyn CapsuleTestClient>,
//! }
//!
//! impl TestSensorX {
//!     pub fn new() -> Self {
//!         TestHmacSha256 {
//!             client: OptionalCell::empty(),
//!         }
//!     }
//! }
//!
//! impl CapsuleTest for TestSensorX {
//!     fn set_client(&self, client: &'static dyn CapsuleTestClient) {
//!         self.client.set(client);
//!     }
//! }
//!
//! impl AsyncClient for TestSensorX {
//!     fn operation_complete(&self) {
//!         // Test has finished at this point.
//!         self.client.map(|client| {
//!             client.done(Ok(()));
//!         });
//!     }
//! }
//! ```

use kernel::ErrorCode;

/// Errors for the result of a failed test.
pub enum CapsuleTestError {
    /// The test computed some result (e.g., a checksum or hash) and the result
    /// is not correct (e.g., it doesn't match the intended value, say the
    /// correct checksum or hash).
    IncorrectResult,

    /// An error occurred while running the test, and the resulting `ErrorCode`
    /// is provided.
    ErrorCode(ErrorCode),
}

/// Client for receiving test done events.
pub trait CapsuleTestClient {
    /// Called when the test is finished. If the test was successful, `result`
    /// is `Ok(())`. If the test failed, `result` is `Err()` with a suitable
    /// error.
    fn done(&'static self, result: Result<(), CapsuleTestError>);
}

/// Identify a test as a capsule test. This is only used for setting the client
/// for test complete callbacks.
pub trait CapsuleTest {
    /// Set the client for the done callback.
    fn set_client(&self, client: &'static dyn CapsuleTestClient);
}
