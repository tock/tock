// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interface for a Bluetooth HCI transport carrying H4 packets.
//!
//! An HCI transport connects a Bluetooth host to a Bluetooth controller. In
//! this HIL, [`HciTransport`] is implemented by the BLE controller, and
//! [`HciTransportClient`] is implemented by the BLE host.
//!
//! # Packet format
//!
//! Every packet on this interface is an H4 packet. An H4 packet starts with a
//! one-byte packet type. The type byte is followed by one HCI packet of that
//! type. The Bluetooth Core Specification defines the H4 packet types in Vol 4,
//! Part A, Section 2. It defines the HCI packets in Vol 4, Part E, Section 5.4.
//!
//! This interface carries the packet types in [`PacketType`]. The table lists
//! the length of each H4 packet. `p` is the H4 packet.
//!
//! | Type                     | Direction          | Length of the H4 packet                |
//! |--------------------------|--------------------|----------------------------------------|
//! | [`PacketType::Command`]  | host to controller | `4 + p[3]`                             |
//! | [`PacketType::AclData`]  | both               | `5 + u16::from_le_bytes([p[3], p[4]])` |
//! | [`PacketType::Event`]    | controller to host | `3 + p[2]`                             |
//!
//! An H4 packet is well-formed if its type is in this table and its length
//! equals the length in the table. This interface carries packets of at most
//! [`MAX_PACKET_LEN`] bytes.
//!
//! The interface currently does not carry HCI Synchronous Data packets.
//!
//! # States
//!
//! The transport (BLE controller) is stopped after boot. A client must start
//! the transport by calling [`HciTransport::reset`]. The transport is ready
//! after it calls [`HciTransportClient::reset_done`] with `Ok(())`. The
//! transport is stopped again after it calls [`HciTransportClient::stopped`].
//! The client calls `reset` to start it again.
//!
//! Receive is disabled after boot. A user must call
//! [`HciTransport::enable_receive`] to enable RX.
//!
//! # Guarantees
//!
//! The implementation provides these guarantees to the client:
//!
//! - It does not call a client method from within a method of
//!   [`HciTransport`] (see Tock's TRD 3 on HIL design).
//! - It delivers packets in the order in which the controller produced them.
//! - It delivers only well-formed event and ACL data packets of at most
//!   [`MAX_PACKET_LEN`] bytes. It discards other packets from the controller.
//! - It calls [`HciTransportClient::transmit_done`] exactly once for every call
//!   to [`HciTransport::transmit`] that returned `Ok(())`.
//! - It does not call [`HciTransportClient::packet_received`] while receive is
//!   disabled.
//! - It does not discard packets while receive is disabled. Only the
//!   controller itself can discard packets when its own buffers are full (see
//!   [`HciTransport::disable_receive`]).

use crate::ErrorCode;

/// Maximum length of an H4 packet on this interface.
///
/// This is the length of the longest command. A command has a one-byte
/// packet type, a three-byte header and at most 255 bytes of parameters. An
/// event is at most 258 bytes long.
pub const MAX_PACKET_LEN: usize = 259;

/// Type of an H4 packet.
///
/// The type is the first byte of an H4 packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// An HCI command from the host to the controller.
    Command = 0x01,
    /// HCI ACL data in either direction.
    AclData = 0x02,
    /// An HCI event from the controller to the host.
    Event = 0x04,
}

impl TryFrom<u8> for PacketType {
    type Error = ErrorCode;

    fn try_from(value: u8) -> Result<Self, ErrorCode> {
        match value {
            0x01 => Ok(PacketType::Command),
            0x02 => Ok(PacketType::AclData),
            0x04 => Ok(PacketType::Event),
            _ => Err(ErrorCode::INVAL),
        }
    }
}

/// Client of an [`HciTransport`].
pub trait HciTransportClient {
    /// The transport has finished with the packet from
    /// [`HciTransport::transmit`].
    ///
    /// `Ok(())` means that the transport has passed the packet to the
    /// controller. The client can submit the next packet.
    ///
    /// # Errors
    ///
    /// - `CANCEL`: The client called [`HciTransport::reset`] before the
    ///   transport passed the packet to the controller.
    /// - `FAIL`: The transport stopped before it passed the packet to the
    ///   controller.
    fn transmit_done(&self, buffer: &'static mut [u8], result: Result<(), ErrorCode>);

    /// The controller sent a packet to the host.
    ///
    /// `packet` holds one well-formed H4 packet. Its type is
    /// [`PacketType::Event`] or [`PacketType::AclData`].
    fn packet_received(&self, packet: &[u8]);

    /// The reset that [`HciTransport::reset`] started has finished.
    ///
    /// After `Ok(())`, the controller is in its power-on state. The transport
    /// does not deliver any packet that the controller produced before the
    /// reset.
    ///
    /// # Errors
    ///
    /// - `FAIL`: The controller did not start. The transport is stopped.
    fn reset_done(&self, result: Result<(), ErrorCode>);

    /// The transport has stopped because the controller or the connection to it
    /// failed.
    ///
    /// The transport calls `transmit_done` for an outstanding packet before it
    /// calls this method. It does not deliver packets after this call. It stays
    /// stopped until the client calls [`HciTransport::reset`].
    fn stopped(&self);
}

/// Bluetooth HCI transport that carries H4 packets.
pub trait HciTransport<'a> {
    /// Sets the client of this transport.
    fn set_client(&self, client: &'a dyn HciTransportClient);

    /// Resets the controller and starts the transport.
    ///
    /// The transport discards all packets that are queued in either direction.
    /// It returns the controller to its power-on state. It can restart the
    /// controller to do this. The client can call this method in any state.
    ///
    /// If a packet is outstanding, the transport calls `transmit_done` with
    /// `CANCEL` for it before it calls `reset_done`. The transport does not
    /// deliver packets between this call and `reset_done`. This method does not
    /// change whether receive is enabled.
    ///
    /// # Errors
    ///
    /// - `BUSY`: A reset is already in progress. The transport calls
    ///   `reset_done` when that reset finishes.
    fn reset(&self) -> Result<(), ErrorCode>;

    /// Sends the H4 packet in `buffer[..len]` to the controller.
    ///
    /// The packet type must be [`PacketType::Command`] or
    /// [`PacketType::AclData`]. The transport accepts one packet at a time. It
    /// returns `buffer` in `transmit_done`.
    ///
    /// # Errors
    ///
    /// - `OFF`: The transport is stopped, or a reset is in progress.
    /// - `BUSY`: Another packet is outstanding.
    /// - `SIZE`: `len` is larger than `buffer.len()` or larger than
    ///   [`MAX_PACKET_LEN`].
    /// - `INVAL`: The packet is not well-formed, or its type is not
    ///   [`PacketType::Command`] or [`PacketType::AclData`].
    fn transmit(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Enables delivery of packets to the client.
    ///
    /// The transport delivers the packets that it held back (if any).
    fn enable_receive(&self);

    /// Disables delivery of packets to the client.
    ///
    /// The transport holds back packets from the controller until the client
    /// calls `enable_receive`. This is important for rate limiting, to avoid
    /// dropping BLE packets because of contention on the BLE host side.
    ///
    /// The client may call this method from within `packet_received`. In that
    /// case, the transport does not deliver any further packet from the same
    /// interrupt or deferred call after the current packet.
    ///
    ///  As the controller has control over link-layer acknowledgements, it is
    /// responsible for implementing flow-control. For example, implementations
    /// of this HIL may hold packets in a queue and, once those are full, apply
    /// hardware-level flow control where the protocol allows it (e.g., by not
    /// acknowledging link-layer data PDUs, so that the peer retransmits them)
    /// or drop packets that are safe to drop (e.g., advertising reports).
    fn disable_receive(&self);
}
