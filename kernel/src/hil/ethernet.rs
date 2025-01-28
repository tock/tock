// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2023.
// Copyright Tock Contributors 2023.

//! Raw Ethernet Adapter HIL for devices transporting IEEE 802.3 Ethernet
//! frames.
//!
//! This HIL currently only encompasses the raw datapath for IEEE 802.3 Ethernet
//! frames. It expects frames to be fully formed with an Ethernet header
//! containing source and destination address. Frames should not have the FCS
//! (Frame Check Sequence) trailer.
//!
//! This HIL is not stable and will be extended by subsequent contributions
//! building out a more fully-featured in-kernel network stack. However, it is
//! sufficient to bridge Ethernet MACs / adapters into userspace, where another
//! network stack can be used.

use crate::ErrorCode;

/// Ethernet adapter datapath client HIL
pub trait EthernetAdapterClient {
    /// An Ethernet frame was transmitted, or an error occurred during
    /// transmission.
    ///
    /// Arguments:
    ///
    /// 1. `err`: `Ok(())` if no error occurred, `Err(e)` otherwise.
    ///
    ///    Possible error codes:
    ///    - [`ErrorCode::FAIL`]: an internal error occurred. The frame may or
    ///      may not have been sent, and the Ethernet MAC may or may not be able
    ///      to send further frames.
    ///
    ///    - [`ErrorCode::BUSY`]: the Ethernet MAC is current busy processing
    ///      another operation and could not enqueue this frame. A client may
    ///      try again later.
    ///
    ///    - [`ErrorCode::OFF`]: the Ethernet MAC is not enabled or initialized
    ///      and cannot send this frame.
    ///
    ///    - [`ErrorCode::NODEVICE`]: the Ethernet MAC does not have an active
    ///      link and cannot send this frame.
    ///
    /// 2. `frame_buffer`: the buffer initially supplied to `transmit`. Ethernet
    ///    MACs will retain the sent frame data (from the start of this buffer,
    ///    up to `len`) in the buffer for inspection by the client.
    ///
    /// 3. `len`: the length of the raw frame that was transmitted, including
    ///    the Ethernet header and excluding the FCS trailer. This value must be
    ///    identical to the one supplied in [`EthernetAdapter::transmit_frame`].
    ///
    /// 4. `transmission_identifier`: an opaque identifier of this transmission
    ///    operation. This value will be identical to the one supplied in the
    ///    call to [`EthernetAdapter::transmit_frame`].
    ///
    /// 5. `timestamp`: optional timestamp of the transmission time of this
    ///    frame, if frame timestamping is enabled (such as for IEEE 1588 PTP).
    ///    The value of this field is opaque, users of this interface must refer
    ///    to the [`EthernetAdapter`] MAC implementation to interpret this value
    ///    and convert it to a proper timestamp.
    fn transmit_frame_done(
        &self,
        err: Result<(), ErrorCode>,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
        timestamp: Option<u64>,
    );

    /// An Ethernet frame was received.
    ///
    /// Arguments:
    ///
    /// 1. `frame`: a buffer containing the frame data, including the Ethernet
    ///    header and excluding the FCS trailer.
    ///
    /// 2. `timestamp`: optional timestamp of the reception time of this frame,
    ///    if frame timestamping is enabled (such as for IEEE 1588 PTP).  The
    ///    value of this field is opaque, users of this interface must refer to
    ///    the [`EthernetAdapter`] MAC implementation to interpret this value
    ///    and convert it to a proper timestamp.
    fn received_frame(&self, frame: &[u8], timestamp: Option<u64>);
}

/// Ethernet adapter datapath HIL
pub trait EthernetAdapter<'a> {
    /// Set the Ethernet adapter client for this peripheral.
    fn set_client(&self, client: &'a dyn EthernetAdapterClient);

    /// Transmit an Ethernet frame
    ///
    /// Arguments:
    ///
    /// 1. `frame`: buffer holding the raw Ethernet frame to be transmitted. The
    ///    frame must be located at offset `0` in this buffer, including the
    ///    Ethernet header with source and destination address set, but
    ///    excluding the FCS trailer. The buffer may be larger than the Ethernet
    ///    frame. The frame length is set in the `len` argument. The
    ///    [`EthernetAdapter`] implementation will return this buffer with its
    ///    original length in a call to
    ///    [`EthernetAdapterClient::transmit_frame_done`], or in the return
    ///    value of this function.
    ///
    /// 2. `len`: the length of the raw frame, including the Ethernet header and
    ///    excluding the FCS trailer.
    ///
    /// 3. `transmission_identifier`: an opaque identifier of this transmission
    ///    operation. This value will be identical to the one supplied in the
    ///    subsequent call to [`EthernetAdapterClient::transmit_frame_done`],
    ///    which will be issued once this frame has been transmitted or an
    ///    asynchronous error occurred during transmission.
    ///
    /// Return value: This function will return with `Ok(())` when a frame has
    /// successfully been enqueued for transmission. In this case, the currently
    /// registered client will receive a call to
    /// [`EthernetAdapterClient::transmit_frame_done`] containing this function
    /// call's `transmission_identifier`. In case of a synchronous error when
    /// enqueueing a frame for transmission, the following errors may be
    /// returned alongside the passed `frame_buffer`:
    ///
    /// - [`ErrorCode::FAIL`]: an internal error occurred. The frame may or may
    ///   not have been sent, and the Ethernet MAC may or may not be able to
    ///   send further frames.
    ///
    /// - [`ErrorCode::BUSY`]: the Ethernet MAC is current busy processing
    ///   another operation and could not enqueue this frame. A client may try
    ///   again later.
    ///
    /// - [`ErrorCode::OFF`]: the Ethernet MAC is not enabled or initialized and
    ///   cannot send this frame.
    ///
    /// - [`ErrorCode::NODEVICE`]: the Ethernet MAC does not have an active link
    ///   and cannot send this frame.
    fn transmit_frame(
        &self,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}
