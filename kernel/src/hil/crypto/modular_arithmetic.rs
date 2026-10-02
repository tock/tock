// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026

use crate::ErrorCode;

pub trait OpAddition {
    fn addition() -> Self;
}
pub trait OpSubtraction {
    fn subtraction() -> Self;
}
pub trait OpMultiplication {
    fn multiplication() -> Self;
}
pub trait OpDivision {
    fn division() -> Self;
}
pub trait OpInverse {
    fn inverse() -> Self;
}
pub trait OpModulo {
    fn modulo() -> Self;
}

/// Upcall from the `ModularArithmetic` trait.
pub trait MathClient<Op> {
    /// Retrieve modulus.
    ///
    /// The driver may issue this callback multiple times, but must not request more than
    /// `modulus_len` bytes
    fn read_modulus(&self, modulus: &mut [u8]) -> Result<(), ErrorCode>;
    /// Retrieve first operand.
    ///
    /// The driver may issue this callback multiple times, but must not request more than
    /// `modulus_len` bytes
    fn read_number(&self, num: &mut [u8]);
    /// Retrieve second operand.
    ///
    /// The driver may issue this callback multiple times, but must not request more than
    /// `modulus_len` bytes
    fn read_second_number(&self, num: &mut [u8]);
    /// Return operation result.
    ///
    /// The driver may issue this callback multiple times, but must not provide more than
    /// `modulus_len` bytes
    fn write_number(&self, num: &[u8]) -> Result<(), ErrorCode>;
    /// Signal completion of an arithmetic operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`ModularArithmetic`]. If a data
    /// callback returns an error, the driver must abort and report that error through `result`.
    fn computation_completed(&self, result: Result<(), ErrorCode>);
}

pub trait ModularArithmetic<'a, Op> {
    /// Set the `Client` client to be called on completion.
    fn set_client(&self, client: &'a dyn MathClient<Op>);
    /// Clear any confidential data.
    fn clear_data(&self);
    /// Initiate an arithmetic operation
    ///
    /// The input is retrieved through [`Client`] callbacks. The driver returns
    /// exactly `modulus_len` bytes of output, then issues exactly one [`MathClient::computation_completed`]
    /// callback.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is in progress
    /// On `Ok(())`, a completion callback will occur. On `Err`, no callbacks will occur for this
    /// request.
    fn start_computation(&self, modulus_len: usize, operation: Op) -> Result<(), ErrorCode>;
}
