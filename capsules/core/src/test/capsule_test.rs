// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Interface for running capsule tests.
//!
//! As Tock capsules are asynchronous, it is difficult for a test runner to
//! determine when a test when a test has finished. This interface provides a
//! `done()` callback to notify when the test is done.
//!
//! A simple example of a test capsule using this interface:
//!
//! ```rust
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

/// Client for receiving test done events.
pub trait CapsuleTestClient {
    /// Called when the test is finished. If test was successful, `result` is
    /// `Ok(())`. If test failed, `result` is `Err(())`.
    fn done(&'static self, result: Result<(), ()>);
}

/// Identify a test as a capsule test. This is only used for setting the client
/// for test complete callbacks.
pub trait CapsuleTest {
    /// Set the client for the done callback.
    fn set_client(&self, client: &'static dyn CapsuleTestClient);
}
