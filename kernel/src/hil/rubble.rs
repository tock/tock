use rubble::link::{DeviceAddress, RadioCmd, Transmitter};

pub trait BleRadio {
    type Transmitter: Transmitter;

    // TODO: is this something most chips will have?
    // TODO: do we need this?
    fn get_device_address() -> DeviceAddress;

    fn radio_accept_cmd(radio: &mut Self::Transmitter, cmd: RadioCmd);
}

// TODO: integrate or removed the commented code below:

// /// Trait for Link Layer packet transmission.
// ///
// /// The specifics of sending a Link-Layer packet depend on the underlying hardware. The `link`
// /// module provides building blocks that enable implementations without any BLE hardware support,
// /// just a compatible radio is needed.
// pub trait AsyncTransmitter {
//     /// Get a reference to the Transmitter's PDU payload buffer.
//     ///
//     /// The buffer must hold at least 37 Bytes, as that is the maximum length of advertising channel
//     /// payloads. While data channel payloads can be up to 251 Bytes in length (resulting in a
//     /// "length" field of 255 with the MIC), devices are allowed to use smaller buffers and report
//     /// the supported payload length.
//     ///
//     /// Both advertising and data channel packets also use an additional 2-Byte header preceding
//     /// this payload.
//     ///
//     /// This buffer must not be changed. The BLE stack relies on the buffer to retain its old
//     /// contents after transmitting a packet. A separate buffer must be used for received packets.
//     fn tx_payload_buf(&mut self) -> &mut [u8];

//     /// Transmit an Advertising Channel PDU.
//     ///
//     /// For Advertising Channel PDUs, the CRC initialization value is always `CRC_PRESET`, and the
//     /// Access Address is always `ADVERTISING_ADDRESS`.
//     ///
//     /// The implementor is expected to send the preamble and access address, and assemble the rest
//     /// of the packet, and must apply data whitening and do the CRC calculation. The inter-frame
//     /// spacing also has to be upheld by the implementor (`T_IFS`).
//     ///
//     /// # Parameters
//     ///
//     /// * `header`: Advertising Channel PDU Header to prepend to the Payload in `payload_buf()`.
//     /// * `channel`: Advertising Channel Index to transmit on.
//     fn transmit_advertising(&mut self, header: advertising::Header, channel: AdvertisingChannel);

//     /// Transmit a Data Channel PDU.
//     ///
//     /// The implementor is expected to send the preamble and assemble the rest of the packet, and
//     /// must apply data whitening and do the CRC calculation.
//     ///
//     /// # Parameters
//     ///
//     /// * `access_address`: The Access Address of the Link-Layer packet.
//     /// * `crc_iv`: CRC calculation initial value (`CRC_PRESET` for advertising channel).
//     /// * `header`: Data Channel PDU Header to be prepended to the Payload in `payload_buf()`.
//     /// * `channel`: Data Channel Index to transmit on.
//     fn transmit_data(
//         &mut self,
//         access_address: u32,
//         crc_iv: u32,
//         header: data::Header,
//         channel: DataChannel,
//     );
// }

// pub trait BleRadio {
//     /// Sets the channel on which to transmit or receive packets.
//     ///
//     /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
//     /// receiving, otherwise ReturnCode::Success.
//     fn set_channel(&self, channel: RadioChannel) -> ReturnCode;

//     /// Sets the transmit power
//     ///
//     /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
//     /// receiving, otherwise ReturnCode::Success.
//     fn set_tx_power(&self, power: u8) -> ReturnCode;

//     /// Transmits a packet over the radio
//     ///
//     /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
//     /// receiving, otherwise ReturnCode::Success.
//     fn transmit_packet(&self, buf: &'static mut [u8], disable: bool) -> ReturnCode;

//     /// Receives a packet of at most `buf.len()` size
//     ///
//     /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
//     /// receiving, otherwise ReturnCode::Success.
//     fn receive_packet(&self, buf: &'static mut [u8]) -> ReturnCode;

//     /// Aborts an ongoing transmision
//     ///
//     /// Returns None if no transmission was ongoing, or the buffer that was
//     /// being transmitted.
//     fn abort_tx(&self) -> Option<&'static mut [u8]>;

//     /// Aborts an ongoing reception
//     ///
//     /// Returns None if no transmission was ongoing, or the buffer that was
//     /// being received into. The returned buffer may or may not have some populated
//     /// bytes.
//     fn abort_rx(&self) -> Option<&'static mut [u8]>;

//     /// Disable periodic advertisements
//     ///
//     /// Returns always ReturnCode::SUCCESS because it does not respect whether
//     /// the driver is actively advertising or not
//     fn disable(&self) -> ReturnCode;
// }

// pub trait RxClient {
//     fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
// }

// pub trait TxClient {
//     fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode);
// }
