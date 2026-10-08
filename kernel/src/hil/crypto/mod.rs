// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! # Tock Cryptography Interface
//!
//! This module provides HIL traits for standard cryptographic algorithms. Because of the wide
//! variety of hardware- and software-based implementations as well as the risky and unforgiving
//! nature of cryptography, these traits employ some unique design patterns to ensure maximum
//! compatibility while minimizing the risk of misuse.
//!
//! ## Trait Organization
//!
//! This module is organized with exactly one trait per major cryptographic algorithm (e.g. AES,
//! RSA, etc.). Things like different parameters, cipher modes, padding schemes, are represented as
//! arguments or different operations/callbacks within the same trait. If a particular algorithm or
//! combination of paramers is not supported by a given implementation, then it should return
//! [`ErrorCode::NOSUPPORT`].
//!
//! ## Callback-Oriented Data Movement
//!
//! Crypto HIL traits use a callback-oriented approach for moving parameters and data between the
//! client and the driver. This is in contrast to other HIL traits, wherein parameters and input
//! data are typically passed as arguments to the driver's entrypoint function (often with one or
//! more `&'static mut [u8]` buffers floating around).
//! The callback pattern leads to greater flexibility for both the caller and callee to decide how
//! data is stored and when it is moved. It also tends to reduce the kernel's memory footprint as
//! well as the number of times data needs to be copied between buffers.
//!
//! The general flow for a client to perform an operation using a crypto HIL is as follows:
//!
//! 1. Client must ensure it is correctly registered to receive callbacks by calling `set_client()`.
//! 2. Client calls the desired entrypoint function on the target HIL to initiate the flow.
//! 3. Driver issues a sequence of asynchronous `read_xxx()` callbacks to retrieve parameters and
//!    inputs for the requested operation.
//! 4. Driver performs the requested operation.
//! 5. Driver issues a sequence of asynchronous `write_xxx()` callbacks to return outputs.
//! 6. Driver issues `xxx_done()` callback to signal completion of the operation and its outcome.
//!
//! ## Error Handling
//!
//! Errors may be returned to clients synchronously from entrypoint functions or asynchronously via
//! the corresponding `xxx_done()` callback. The meanings of error codes are the same in either
//! case.
//!
//! The following list describes common error codes returned from crypto HIL traits. Be sure to
//! reference individual trait docs for algorithm-specific error codes and conditions.
//!
//! * `ErrorCode::INVAL` - Invalid combination of parameters
//!   * Ex: Input length not multiple of block size when using a cipher that requires it
//! * `ErrorCode::NOSUPPORT` - The requested operation or combination of parameters is not supported
//!   by the HIL implementation
//!   * Ex: Attempting to use AES-GCM on a device that does not provide such functionality
//! * `ErrorCode::BUSY` - An operation is already in progress
//!
//! All `read_xxx()` and `write_xxx()` callbacks are defined to return a `Result` type, allowing the
//! client to express any errors that may occur while reading or writing data. Whenever such errors
//! are returned, the driver shall immediately abort the in-progress operation and propagate the
//! error back to the client via the appropriate `xxx_done()` callback.
//!
//! ## Example HIL Structure
//!
//! ```
//! trait Foo {
//!     fn set_client(&self, client: &'static dyn CryptoClient);
//!
//!     /// Perform operation FOO with the specified mode and input length
//!     fn foo(&self, len: usize) -> Result<(), ErrorCode>;
//! }
//!
//! trait CryptoClient {
//!     /// Read input data for FOO operation from the client
//!     fn read_input(&self, dst: &mut [u8]) -> Result<usize, ErrorCode>;
//!
//!     /// Write output data from FOO operation
//!     fn write_output(&self, src: &[u8]) -> Result<(), ErrorCode>;
//!
//!    /// Signal completion of the FOO operation
//!     fn foo_done(&self, res: Result<(), ErrorCode>);
//! }
//! ```

pub mod ecc;
pub mod modular_arithmetic;
