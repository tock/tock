// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! Interface for hardware-accelerated modular arithmetic.
//!
//! A client starts an operation with [`ModularArithmetic::start_computation`],
//! supplies the modulus and operands through the [`MathClient`] read callbacks,
//! and receives the result through [`MathClient::write_number`], followed by
//! exactly one [`MathClient::computation_completed`] callback.
//!
//! # Data format
//!
//! All numbers (the modulus, the operands and the result) are unsigned
//! big-endian integers: the most significant byte comes first.
//!
//! Every number is exactly `modulus_len` bytes long, left-padded with zeros.
//! Leading zero bytes must not be stripped, including from the modulus.
//!
//! # Chunked transfer
//!
//! Numbers may be exchanged in several chunks, so that drivers do not need a
//! buffer large enough to hold a whole number. The driver chooses the chunk
//! sizes, which need not be equal. For each number:
//!
//! - chunks are transferred in order, starting from the most significant end,
//!   so concatenating them yields the full big-endian number;
//! - every byte is transferred exactly once, so the chunks of one number add up
//!   to exactly `modulus_len` bytes.
//!
//! Transfers of different numbers may be interleaved, so a client must track a
//! separate position for the modulus, each operand and the result.
//!
//! # Operations
//!
//! | Operation      | Operands read | Result            |
//! |----------------|---------------|-------------------|
//! | Addition       | `a`, `b`      | `(a + b) mod m`   |
//! | Subtraction    | `a`, `b`      | `(a - b) mod m`   |
//! | Multiplication | `a`, `b`      | `(a * b) mod m`   |
//! | Division       | `a`, `b`      | `(a * b⁻¹) mod m` |
//! | Inverse        | `a`           | `a⁻¹ mod m`       |
//! | Modulo         | `a`           | `a mod m`         |
//!
//! `m` is the modulus, read with [`MathClient::read_modulus`]; `a` is read with
//! [`MathClient::read_number`] and `b` with [`MathClient::read_second_number`].
//! Every result is fully reduced, in the range `0 <= r < m`.
//!
//! For the modulo operation, `a` may be any `modulus_len`-byte value. For all
//! other operations, operands must be less than `m`; a driver may reject larger
//! operands or produce an unspecified result. Inverse and division fail if the
//! value to invert has no inverse modulo `m`.

use crate::ErrorCode;

/// Constructor for the addition operation of a driver-defined operation type.
///
/// The operation traits let generic code name an operation without knowing the
/// driver's operation type. A driver's type implements the traits for the
/// operations it supports.
pub trait OpAddition {
    fn addition() -> Self;
}

/// Constructor for the subtraction operation. See [`OpAddition`].
pub trait OpSubtraction {
    fn subtraction() -> Self;
}

/// Constructor for the multiplication operation. See [`OpAddition`].
pub trait OpMultiplication {
    fn multiplication() -> Self;
}

/// Constructor for the division operation. See [`OpAddition`].
pub trait OpDivision {
    fn division() -> Self;
}

/// Constructor for the inverse operation. See [`OpAddition`].
pub trait OpInverse {
    fn inverse() -> Self;
}

/// Constructor for the modulo (reduction) operation. See [`OpAddition`].
pub trait OpModulo {
    fn modulo() -> Self;
}

/// Client callback interface for the [`ModularArithmetic`] trait.
pub trait MathClient<Op> {
    /// Retrieve modulus.
    ///
    /// The client fills all of `modulus` with the next `modulus.len()` bytes of
    /// the modulus. The driver may call this several times, but requests exactly
    /// `modulus_len` bytes in total.
    ///
    /// Returning an error aborts the operation; see
    /// [`computation_completed`](Self::computation_completed).
    fn read_modulus(&self, modulus: &mut [u8]) -> Result<(), ErrorCode>;

    /// Retrieve first operand.
    ///
    /// The client fills all of `num` with the next `num.len()` bytes of `a`. The
    /// driver may call this several times, but requests exactly `modulus_len`
    /// bytes in total.
    fn read_number(&self, num: &mut [u8]);

    /// Retrieve second operand.
    ///
    /// Only called for operations that take two operands. The client fills all
    /// of `num` with the next `num.len()` bytes of `b`. The driver may call this
    /// several times, but requests exactly `modulus_len` bytes in total.
    fn read_second_number(&self, num: &mut [u8]);

    /// Return operation result.
    ///
    /// Only called if the computation succeeded. The driver may call this
    /// several times, and delivers exactly `modulus_len` bytes in total before
    /// calling [`computation_completed`](Self::computation_completed) with
    /// `Ok(())`.
    ///
    /// Returning an error aborts the operation; see
    /// [`computation_completed`](Self::computation_completed).
    fn write_number(&self, num: &[u8]) -> Result<(), ErrorCode>;

    /// Signal completion of an arithmetic operation.
    ///
    /// Called exactly once for each call to
    /// [`ModularArithmetic::start_computation`] that returned `Ok(())`, after
    /// all data callbacks for that operation.
    ///
    /// `Ok(())` means the complete result has been delivered through
    /// [`write_number`](Self::write_number). On `Err`, any result bytes already
    /// delivered are invalid and must be discarded.
    ///
    /// If [`read_modulus`](Self::read_modulus) or
    /// [`write_number`](Self::write_number) returns an error, the driver issues
    /// no further data callbacks for that operation and passes that same error
    /// here.
    fn computation_completed(&self, result: Result<(), ErrorCode>);
}

/// Hardware-accelerated modular arithmetic.
///
/// See the [module documentation](self) for the data format and the supported
/// operations.
pub trait ModularArithmetic<'a, Op> {
    /// Set the `Client` client to be called on completion.
    fn set_client(&self, client: &'a dyn MathClient<Op>);

    /// Start an operation on numbers of `modulus_len` bytes.
    ///
    /// The modulus and operands are read through [`MathClient`] callbacks. No
    /// callback is issued from within this call.
    ///
    /// On `Ok(())`, exactly one [`MathClient::computation_completed`] callback
    /// follows. If the operation succeeds, the result is delivered first as
    /// exactly `modulus_len` bytes through [`MathClient::write_number`].
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::BUSY`]: another operation is in progress.
    /// - [`ErrorCode::SIZE`]: `modulus_len` is zero or not supported by the
    ///   driver.
    /// - [`ErrorCode::NOSUPPORT`]: `operation` is not supported by the driver.
    ///
    /// On `Err`, no callbacks occur for this request.
    fn start_computation(&self, modulus_len: usize, operation: Op) -> Result<(), ErrorCode>;
}
