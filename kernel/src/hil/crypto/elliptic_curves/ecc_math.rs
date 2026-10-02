// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026

//! Interface for Math Operations with long numbers modulo some other number
use crate::{ErrorCode, hil::crypto::elliptic_curves::ecc_constants::Curve};

/// Upcall from the `EccCrypto` trait.
pub trait EccClient {
    /// Retrieve scalar.
    ///
    /// The driver may issue this callback multiple times, but must not request more than
    /// `P_SIZE` bytes
    fn read_scalar(&self, scalar: &mut [u8]) -> Result<(), ErrorCode>;
    /// Retrieve the point for the operation.
    ///
    /// The driver may issue this callback multiple times, but must not request more than
    /// 2 * `P_SIZE` bytes
    fn read_point(&self, point: &mut [u8]) -> Result<(), ErrorCode>;
    /// Retrieve the second point for the `point_addition`.
    ///
    /// The driver may issue this callback multiple times, but must not request more than
    /// 2 * `P_SIZE` bytes
    fn read_second_point(&self, point: &mut [u8]) -> Result<(), ErrorCode>;
    /// Return resulting point.
    ///
    /// Across all calls, the driver must return exactly 2 * `P_SIZE` bytes in input order.
    fn write_point(&self, point: &[u8]) -> Result<(), ErrorCode>;
    /// Signal completion of an elliptic curve operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`EccCrypto`]. If a data
    /// callback returns an error, the driver must abort and report that error through `result`.
    fn operation_done(&self, result: Result<(), ErrorCode>);
}

pub trait EccCrypto<'a, const P_SIZE: usize, C: Curve<P_SIZE>> {
    /// Set the `Client` client to be called on completion.
    fn set_client(&self, client: &'a dyn EccClient);
    /// Clear any confidential data.
    fn clear_data(&self);
    /// Initiate a point doubling operation
    ///
    /// The input is retrieved through [`Client`] callbacks if and only if `use_curve_generator` is false.
    /// If `use_curve_generator` is true, the point to be doubled will be the `GENERATOR` constant
    /// in `Curve<P_SIZE>`
    /// The driver returns exactly 2 * `P_SIZE` bytes of output, then issues exactly
    /// one [`EccClient::operation_done`] callback.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is in progress
    /// On `Ok(())`, a completion callback will occur. On `Err`, no callbacks will occur for this
    /// request.
    fn point_doubling(&self, use_curve_generator: bool) -> Result<(), ErrorCode>;
    /// Initiate a point addition operation
    ///
    /// The input is retrieved through [`Client`] callbacks. The driver returns
    /// exactly 2 * `P_SIZE` bytes of output, then issues exactly one [`EccClient::operation_done`]
    /// callback. If `use_curve_generator` is false, the `read_second_point` callback will be used.
    /// If `use_curve_generator` is true, the second point of the addition will be the `GENERATOR` constant
    /// in `Curve<P_SIZE>`.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is in progress
    /// On `Ok(())`, a completion callback will occur. On `Err`, no callbacks will occur for this
    /// request.
    fn point_addition(&self, use_curve_generator: bool) -> Result<(), ErrorCode>;
    /// Initiate a scalar multiplication operation
    ///
    /// The input is retrieved through [`Client`] callbacks. The driver returns
    /// exactly 2 * `P_SIZE` bytes of output, then issues exactly one [`EccClient::operation_done`]
    /// callback. If `use_curve_generator` is false, the `read_point` callback will be used.
    /// If `use_curve_generator` is true, the point to be multiplied by the scalar will be the `GENERATOR`
    /// constant in `Curve<P_SIZE>`
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is in progress
    /// On `Ok(())`, a completion callback will occur. On `Err`, no callbacks will occur for this
    /// request.
    fn scalar_multiplication(&self, use_curve_generator: bool) -> Result<(), ErrorCode>;
}

pub trait VerifyEccPoint<'a, const P_SIZE: usize, C: Curve<P_SIZE>>:
    EccCrypto<'a, P_SIZE, C>
{
    /// Initiate a point verification operation
    ///
    /// The input is retrieved through [`Client`] callbacks. The driver will call
    /// `operation_done`, where result will either be `Ok(())` if the point is on the elliptic
    /// curve, or `ErrorCode::Inval` if it is not.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is in progress
    /// On `Ok(())`, a completion callback will occur. On `Err`, no callbacks will occur for this
    /// request.
    fn verify_point(&self) -> Result<(), ErrorCode>;
}
