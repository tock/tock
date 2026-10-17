// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for extendable-output functions (XOFs).
//!
//! An XOF absorbs a message of arbitrary length and produces an output whose length is chosen by
//! the caller, as specified in FIPS 202. This module separates the XOF operation from the
//! algorithm used to perform it. Algorithm marker types identify the function, while [`Xof`] and
//! [`XofClient`] define the asynchronous data flow shared by XOF implementations.
//!
//! The output length is fixed when the operation begins. After finalization, the driver
//! delivers the output through several [`XofClient::write_output`] transfers, in order, until
//! all of it has been produced. The client cannot request additional output once the operation
//! has begun, so incremental squeezing is not supported.
//!
//! # Operation flow
//!
//! An XOF operation proceeds as follows:
//!
//! 1. [`Xof::begin`] starts a new operation and fixes the output length.
//! 2. [`Xof::update`] absorbs input, which the driver pulls through [`XofClient::read_input`].
//!    Each accepted update completes with [`XofClient::update_done`]. Updates may be repeated
//!    any number of times, including zero.
//! 3. [`Xof::finalize`] completes the computation. The driver delivers the output through one or
//!    more [`XofClient::write_output`] callbacks and then signals completion through
//!    [`XofClient::xof_done`].
//!
//! [`Xof::cancel`] may be called at any point after [`Xof::begin`] to abandon the operation. At
//! most one asynchronous request ([`Xof::update`] or [`Xof::finalize`]) may be outstanding at a
//! time.

use crate::ErrorCode;

/// Marker trait for an extendable-output function.
///
/// XOFs have no fixed output length; the length is chosen per operation through
/// [`Xof::begin`].
pub trait XofAlgorithm {}

macro_rules! xof_algorithm {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name;

        impl XofAlgorithm for $name {}
    };
}

xof_algorithm!(Shake128);
xof_algorithm!(Shake256);

/// Computes an extendable-output function using algorithm `A`.
pub trait Xof<A: XofAlgorithm> {
    /// Start a new XOF operation producing `output_len` bytes of output.
    ///
    /// Initializes the state for algorithm `A` and records the output length. This call
    /// completes synchronously and does not cause a callback. On success, the driver accepts
    /// [`Self::update`], [`Self::finalize`], and [`Self::cancel`] for this operation.
    ///
    /// # Errors
    ///
    /// If this call returns an error, no operation is started.
    ///
    /// - [`ErrorCode::BUSY`]: an operation is already in progress.
    /// - [`ErrorCode::INVAL`]: `output_len` is zero.
    /// - [`ErrorCode::SIZE`]: `output_len` exceeds the maximum output length supported by the
    ///   implementation.
    fn begin(&self, output_len: usize) -> Result<(), ErrorCode>;

    /// Absorb `len` bytes of input.
    ///
    /// The driver obtains the input through one or more [`XofClient::read_input`] callbacks
    /// totaling exactly `len` bytes, and then signals completion through
    /// [`XofClient::update_done`].
    ///
    /// Multiple updates are equivalent to a single update over the concatenated input.
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and the operation state is unchanged.
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized, or `len` is zero and no absorption can happen.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet completed.
    /// - [`ErrorCode::SIZE`]: absorbing `len` more bytes would exceed the maximum message length
    ///   supported by the implementation.
    fn update(&self, len: usize) -> Result<(), ErrorCode>;

    /// Complete the XOF computation.
    ///
    /// The driver delivers `output_len` bytes of output, computed over all absorbed input,
    /// through one or more [`XofClient::write_output`] callbacks, and then signals completion
    /// through [`XofClient::xof_done`]. Finalizing without any prior [`Self::update`] produces
    /// the output for the empty message.
    ///
    /// After [`XofClient::xof_done`], the operation is over regardless of the result, and a new
    /// operation must be started with [`Self::begin`].
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and the operation state is unchanged.
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet completed.
    fn finalize(&self) -> Result<(), ErrorCode>;

    /// Abandon the operation in progress.
    ///
    /// Discards all absorbed chunks of data and returns the driver to idle, so a new operation can be
    /// started with [`Self::begin`].
    ///
    /// If an [`Self::update`] or [`Self::finalize`] request is outstanding, the driver completes
    /// it with [`ErrorCode::CANCEL`] through callback [`XofClient::xof_done`]. The driver issues no further
    /// data callbacks for the operation; in particular, no further [`XofClient::write_output`]
    /// callbacks occur, even if part of the output has already been delivered. If no request is
    /// outstanding, no callback occurs.
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress.
    fn cancel(&self) -> Result<(), ErrorCode>;

    /// Set the client that will receive callbacks for XOF operations.
    fn set_client(&self, client: &'static dyn XofClient<A>);
}

/// Client callbacks for [`Xof`] operations.
pub trait XofClient<A: XofAlgorithm> {
    /// Retrieve input to absorb.
    ///
    /// The client must write input into `input` and return the number of bytes written. A driver
    /// may issue this callback multiple times, but each destination must fit within the remaining
    /// input. The total number of bytes consumed must equal the `len` passed to [`Xof::update`].
    /// Follow the [crypto data movement contract](super#callback-oriented-data-movement):
    /// return between one byte and `input.len()` bytes for each nonempty request. A zero or
    /// oversized count terminates the operation with [`ErrorCode::SIZE`]. Empty updates do not
    /// cause input callbacks.
    fn read_input(&self, input: &mut [u8]) -> Result<usize, ErrorCode>;

    /// Signal completion of an [`Xof::update`] request.
    ///
    /// This callback occurs exactly once after a request accepted by [`Xof::update`].
    ///
    /// On success, all `len` bytes have been absorbed and the driver accepts another
    /// [`Xof::update`] or [`Xof::finalize`].
    ///
    /// On error, the operation has been aborted and its state discarded; the client must start a
    /// new operation with [`Xof::begin`]. If [`Self::read_input`] returned an error, that error
    /// is reported here.
    fn update_done(&self, result: Result<(), ErrorCode>);

    /// Deliver a chunk of output.
    ///
    /// After [`Xof::finalize`], the driver calls this one or more times, passing consecutive,
    /// nonempty chunks of the output in order. The chunk size is chosen by the driver, and the
    /// total length of all chunks equals the `output_len` passed to [`Xof::begin`]. A driver
    /// able to produce the whole output at once may deliver it in a single call.
    ///
    /// If this callback returns an error, the driver issues no
    /// further output callbacks and reports that error through [`Self::xof_done`].
    fn write_output(&self, output: &[u8]) -> Result<(), ErrorCode>;

    /// Signal completion of an XOF operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`Xof::finalize`]. On
    /// success, the full output has been delivered through [`Self::write_output`]. If a data
    /// callback returns an error, the driver must immediately abort the operation and report that
    /// error through `result`; any output already delivered must be treated as incomplete.
    /// [`ErrorCode::CANCEL`] indicates the request was abandoned by [`Xof::cancel`]. After this
    /// callback, the driver is idle and accepts a new [`Xof::begin`].
    fn xof_done(&self, result: Result<(), ErrorCode>);
}
