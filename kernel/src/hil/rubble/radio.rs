//! Interface between Rubble and Radio HW
//!
//! This HIL file specifies the additional interface required between the Rubble
//! stack and the radio hardware. The Rubble stack also uses the BLE interfaces
//! separately specified as HILs.
//!
//! Eventually, it may make sense to remove this interface and instead have a
//! more generic BLE data interface that both Rubble and other BLE stacks could
//! use.

use crate::hil::ble_advertising;

use super::types::DeviceAddress;

/// The primary interface between the rubble stack and the radio.
///
/// This is based off of a combination of rubble's `RadioCmd` interface for
/// configuring radio receiving, rubble's `Transmitter` trait for data
/// transmission, and Tock's existing `BleAdvertisementDriver`.
///
/// Event notifications well be sent to the [`RxClient`] and [`TxClient`] set
/// using [`ble_advertising::BleAdvertisementDriver`] methods. This allows
/// radios supporting both advertisement and data connections to keep track
/// of one set of event clients rather than two for different transmission
/// types.
///
/// [`RxClient`]: crate::hil::ble_advertising::RxClient
/// [`TxClient`]: crate::hil::ble_advertising::TxClient
pub trait RubbleData<'a>: ble_advertising::BleAdvertisementDriver<'a> {
    /// Return the BLE device address.
    fn get_device_address() -> DeviceAddress;

    /// Transmit a Data Channel PDU.
    ///
    /// The implementor is expected to send the preamble and assemble the rest
    /// of the packet, and must apply data whitening and do the CRC calculation.
    ///
    /// # Parameters
    ///
    /// * `buf`: The data to send, including the Data Channel PDU Header as the
    ///   first two bytes.
    /// * `access_address`: The Access Address of the Link-Layer packet.
    /// * `crc_iv`: CRC calculation initial value (`CRC_PRESET` for advertising
    ///   channel).
    /// * `channel`: Data Channel Index to transmit on. Must be in 0..=36.
    fn transmit_data(
        &self,
        buf: &'static mut [u8],
        access_address: u32,
        crc_iv: u32,
        channel: ble_advertising::RadioChannel,
    );

    /// Configure the radio to receive data.
    ///
    /// # Parameters
    ///
    /// * `channel`: The data channel to listen on.
    /// * `access_address`: The Access Address to listen for.
    ///
    ///    Packets with a different Access Address must not be passed to the
    ///    to the `RxClient`. You may be able to use your Radio's hardware
    ///    address matching for this.
    /// * `crc_init`: Initialization value of the CRC-24 calculation.
    ///
    ///    Only the least 24 bits are relevant.
    fn receive_data(
        &self,
        channel: ble_advertising::RadioChannel,
        access_address: u32,
        crc_init: u32,
    );
}
