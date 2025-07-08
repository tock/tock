// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for sending and receiving IEEE 802.15.4 packets.
//!
//! Hardware independent interface for an 802.15.4 radio. Note that
//! configuration commands are asynchronous and must be committed with a call to
//! config_commit. For example, calling set_address will change the source
//! address of packets but does not change the address stored in hardware used
//! for address recognition. This must be committed to hardware with a call to
//! config_commit. Please see the relevant TRD for more details.

use crate::ErrorCode;

/// Client trait for when sending a packet is finished.
pub trait TxClient {
    /// Send is complete or an error occurred during transmission.
    ///
    /// ## Arguments
    ///
    /// - `buf`: Buffer of the transmitted packet.
    /// - `acked`: Set to true if the sender received an acknowledgement after
    ///   transmitting. Note, this is only set on a confirmed ACK; if the
    ///   transmission did not request an ACK this will be set to false.
    /// - `result`: Status of the transmission. `Ok(())` when the packet was
    ///   sent successfully. On `Err()`, valid errors are:
    ///   - `ErrorCode::BUSY`: The channel was never clear and we could not
    ///     transmit.
    ///   - `ErrorCode::FAIL`: Internal TX error occurred.
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: Result<(), ErrorCode>);
}

/// Client for receiving packets.
pub trait RxClient {
    /// Packet was received.
    ///
    /// ## Arguments
    ///
    /// - `buf`: Buffer containing the packet. This is the buffer provided via
    ///   `RadioData::set_receive_buffer()`. The structure of this buffer is the
    ///   same as in the TX case, as described in this HIL. That is, the first
    ///   byte is reserved, and the full 802.15.4 starts with the PHR in the
    ///   second byte.
    /// - `frame_len`: Length of the received frame, excluding the MAC footer.
    ///   In other words, this length is PHR-MFR_SIZE. Note, this length does
    ///   _not_ correspond to the length of data from the start of the buffer.
    ///   This length is from the third byte in the buffer (i.e.,
    ///   `buf[2:2+frame_length]`).
    /// - `lqi`: The Link Quality Indicator as measured by the receiver during
    ///   the packet reception. This is on the scale as specified in the IEEE
    ///   802.15.4 specification (section 6.9.8), with value 0 being the lowest
    ///   detectable signal and value 0xff as the highest quality detectable
    ///   signal.
    /// - `crc_valid`: Whether the CRC check matched the received frame. Note,
    ///   the MFR bytes are not required to be stored in `buf` so using this
    ///   argument is the only reliable method for checking the CRC.
    /// - `result`: Status of the reception. `Ok(())` when the packet was
    ///   received normally. On `Err()`, valid errors are:
    ///   - `ErrorCode::NOMEM`: Ack was requested, but there was no buffer
    ///     available to transmit an ACK.
    ///   - `ErrorCode::FAIL`: Internal error occurred.
    fn receive(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        lqi: u8,
        crc_valid: bool,
        result: Result<(), ErrorCode>,
    );
}

/// Client for callbacks after the radio is configured.
pub trait ConfigClient {
    /// Configuring the radio has finished.
    ///
    /// ## Arguments
    ///
    /// - `result`: Status of the configuration procedure. `Ok(())` if all
    ///   options were set as expected. On `Err()`, valid errors are:
    ///   - `ErrorCode::FAIL`: Internal error occurred.
    fn config_done(&self, result: Result<(), ErrorCode>);
}

/// Client for callbacks when the radio's power state changes.
pub trait PowerClient {
    /// The power state of the radio changed. This is called when the radio has
    /// turned on or off.
    ///
    /// ## Arguments
    ///
    /// - `on`: True if the radio is now on. False otherwise.
    fn changed(&self, on: bool);
}

// These constants are used for interacting with the SPI buffer, which contains
// a 1-byte SPI command, a 1-byte PHY header, and then the 802.15.4 frame. In
// theory, the number of extra bytes in front of the frame can depend on the
// particular method used to communicate with the radio, but we leave this as a
// constant in this generic trait for now.
//
// Furthermore, the minimum MHR size assumes that
//
// - The source PAN ID is omitted
// - There is no auxiliary security header
// - There are no IEs
//
// ```text
// +---------+-----+--------+-------------+-----+-----+
// | SPI com | PHR | MHR    | MAC payload | MFR | LQI |
// +---------+-----+--------+-------------+-----+-----+
// \______ Static buffer for implementation __________/
//                 \_________ PSDU _____________/
// \___ 2 bytes ___/                            \1byte/
// ```

/// Length of the physical layer header. This is the Frame length field.
pub const PHR_SIZE: usize = 1;
/// Length of the Frame Control field in the MAC header.
pub const MHR_FC_SIZE: usize = 2;
/// Length of the MAC footer. Contains the CRC.
pub const MFR_SIZE: usize = 2;
/// Maximum length of a MAC frame.
pub const MAX_MTU: usize = 127;
/// Minimum length of the MAC frame (except for acknowledgements). This is
/// explained in Table 21 of the specification.
pub const MIN_FRAME_SIZE: usize = 9;
/// Maximum length of a MAC frame.
pub const MAX_FRAME_SIZE: usize = MAX_MTU;

/// Location in the buffer of the physical layer header. This is the location of
/// the Frame length byte.
pub const PHR_OFFSET: usize = 1;
/// Location in the buffer of the PSDU. This is equivalent to the start of the
/// MAC payload.
pub const PSDU_OFFSET: usize = 2;
/// Length of the reserved space in the buffer for a SPI command.
pub const SPI_HEADER_SIZE: usize = 1;
/// Length of the reserved space in the buffer for LQI information.
pub const LQI_SIZE: usize = 1;
/// Required buffer size for implementations of this HIL.
pub const MAX_BUF_SIZE: usize = SPI_HEADER_SIZE + PHR_SIZE + MAX_MTU + LQI_SIZE;

/// General Radio trait that supports configuration and TX/RX.
pub trait Radio<'a>: RadioConfig<'a> + RadioData<'a> {}
// Provide blanket implementations for trait group
impl<'a, T: RadioConfig<'a> + RadioData<'a>> Radio<'a> for T {}

/// Configure the 802.15.4 radio.
pub trait RadioConfig<'a> {
    /// Initialize the radio.
    ///
    /// This should perform any needed initialization, but should not turn the
    /// radio on.
    ///
    /// ## Return
    ///
    /// `Ok(())` on success. On `Err()`, valid errors are:
    ///
    /// - `ErrorCode::FAIL`: Internal error occurred.
    fn initialize(&self) -> Result<(), ErrorCode>;

    /// Reset the radio.
    ///
    /// Perform a radio reset which may reset the internal state of the radio.
    ///
    /// ## Return
    ///
    /// `Ok(())` on success. On `Err()`, valid errors are:
    ///
    /// - `ErrorCode::FAIL`: Internal error occurred.
    fn reset(&self) -> Result<(), ErrorCode>;

    /// Start the radio.
    ///
    /// This should put the radio into receive mode.
    ///
    /// ## Return
    ///
    /// `Ok(())` on success. On `Err()`, valid errors are:
    ///
    /// - `ErrorCode::FAIL`: Internal error occurred.
    fn start(&self) -> Result<(), ErrorCode>;

    /// Stop the radio.
    ///
    /// This should turn the radio off, disabling receive mode, and put the
    /// radio into a low power state.
    ///
    /// ## Return
    ///
    /// `Ok(())` on success. On `Err()`, valid errors are:
    ///
    /// - `ErrorCode::FAIL`: Internal error occurred.
    fn stop(&self) -> Result<(), ErrorCode>;

    /// Check if the radio is currently on.
    ///
    /// ## Return
    ///
    /// True if the radio is on, false otherwise.
    fn is_on(&self) -> bool;

    /// Check if the radio is currently busy transmitting or receiving a packet.
    ///
    /// If this returns `true`, the radio is unable to start another operation.
    ///
    /// ## Return
    ///
    /// True if the radio is busy, false otherwise.
    fn busy(&self) -> bool;

    /// Set the client that is called when the radio changes power states.
    fn set_power_client(&self, client: &'a dyn PowerClient);

    /// Commit the configuration calls to the radio.
    ///
    /// This will set the address, PAN ID, TX power, and channel to the
    /// specified values within the radio hardware. When this finishes, this
    /// will issue a callback to the config client when done.
    fn config_commit(&self);

    /// Set the client that is called when configuration finishes.
    fn set_config_client(&self, client: &'a dyn ConfigClient);

    /// Get the 802.15.4 short (16-bit) address for the radio.
    ///
    /// ## Return
    ///
    /// The radio's short address.
    fn get_address(&self) -> u16;

    /// Get the 802.15.4 extended (64-bit) address for the radio.
    ///
    /// ## Return
    ///
    /// The radio's extended address.
    fn get_address_long(&self) -> [u8; 8];

    /// Get the 802.15.4 16-bit PAN ID for the radio.
    ///
    /// ## Return
    ///
    /// The radio's PAN ID.
    fn get_pan(&self) -> u16;

    /// Get the radio's transmit power.
    ///
    /// ## Return
    ///
    /// The transmit power setting used by the radio, in dBm.
    fn get_tx_power(&self) -> i8;

    /// Get the 802.15.4 channel the radio is currently using.
    ///
    /// ## Return
    ///
    /// The channel number.
    fn get_channel(&self) -> u8;

    /// Set the 802.15.4 short (16-bit) address for the radio.
    ///
    /// Note, calling this function configures the software driver, but does not
    /// take effect in the radio hardware. Call `RadioConfig::config_commit()`
    /// to set the configuration settings in the radio hardware.
    ///
    /// ## Argument
    ///
    /// - `addr`: The short address.
    fn set_address(&self, addr: u16);

    /// Set the 802.15.4 extended (64-bit) address for the radio.
    ///
    /// Note, calling this function configures the software driver, but does not
    /// take effect in the radio hardware. Call `RadioConfig::config_commit()`
    /// to set the configuration settings in the radio hardware.
    ///
    /// ## Argument
    ///
    /// - `addr`: The extended address.
    fn set_address_long(&self, addr: [u8; 8]);

    /// Set the 802.15.4 PAN ID (16-bit) for the radio.
    ///
    /// Note, calling this function configures the software driver, but does not
    /// take effect in the radio hardware. Call `RadioConfig::config_commit()`
    /// to set the configuration settings in the radio hardware.
    ///
    /// ## Argument
    ///
    /// - `id`: The PAN ID.
    fn set_pan(&self, id: u16);

    /// Set the radio's transmit power.
    ///
    /// Note, calling this function configures the software driver, but does not
    /// take effect in the radio hardware. Call `RadioConfig::config_commit()`
    /// to set the configuration settings in the radio hardware.
    ///
    /// ## Argument
    ///
    /// - `power`: The transmit power in dBm.
    ///
    /// ## Return
    ///
    /// `Ok(())` on success. On `Err()`, valid errors are:
    ///
    /// - `ErrorCode::INVAL`: The transmit power is above acceptable limits.
    /// - `ErrorCode::NOSUPPORT`: The transmit power is not supported by the
    ///   radio.
    /// - `ErrorCode::FAIL`: Internal error occurred.
    fn set_tx_power(&self, power: i8) -> Result<(), ErrorCode>;

    /// Set the 802.15.4 channel for the radio.
    ///
    /// Note, calling this function configures the software driver, but does not
    /// take effect in the radio hardware. Call `RadioConfig::config_commit()`
    /// to set the configuration settings in the radio hardware.
    ///
    /// ## Argument
    ///
    /// - `chan`: The 802.15.4 channel.
    fn set_channel(&self, chan: RadioChannel);
}

/// Send and receive packets with the 802.15.4 radio.
pub trait RadioData<'a> {
    /// Set the client that will be called when packets are transmitted.
    fn set_transmit_client(&self, client: &'a dyn TxClient);

    /// Set the client that will be called when packets are received.
    fn set_receive_client(&self, client: &'a dyn RxClient);

    /// Set the buffer to receive packets into.
    ///
    /// ## Argument
    ///
    /// - `receive_buffer`: The buffer to receive into. Must be at least
    ///   `MAX_BUF_SIZE` bytes long.
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

    /// Transmit a packet.
    ///
    /// The radio will create and insert the PHR (Frame length) field.
    ///
    /// ## Argument
    ///
    /// - `buf`: Buffer with the MAC layer 802.15.4 frame to be transmitted.
    ///   The buffer must conform to the buffer formatted documented in the HIL.
    ///   That is, the MAC payload (PSDU) must start at the third byte.
    ///   The first byte must be reserved for the radio driver (i.e.
    ///   for a SPI transaction) and the second byte is reserved for the PHR.
    ///   The buffer must be at least `frame_len` + 2 + MFR_SIZE` bytes long.
    /// - `frame_len`: The length of the MAC payload, not including the MFR.
    ///
    /// ## Return
    ///
    /// `Ok(())` on success. On `Err()`, valid errors are:
    ///
    /// - `ErrorCode::OFF`: The radio is off and cannot transmit.
    /// - `ErrorCode::BUSY`: The radio is busy. This is likely to occur because
    ///   the radio is already transmitting a packet.
    /// - `ErrorCode::SIZE`: The buffer does not have room for the MFR (CRC).
    /// - `ErrorCode::FAIL`: Internal error occurred.
    fn transmit(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}

/// IEEE 802.15.4 valid channels.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    Channel11 = 5,
    Channel12 = 10,
    Channel13 = 15,
    Channel14 = 20,
    Channel15 = 25,
    Channel16 = 30,
    Channel17 = 35,
    Channel18 = 40,
    Channel19 = 45,
    Channel20 = 50,
    Channel21 = 55,
    Channel22 = 60,
    Channel23 = 65,
    Channel24 = 70,
    Channel25 = 75,
    Channel26 = 80,
}

impl RadioChannel {
    /// Get the IEEE 802.15.4 channel number for the `RadioChannel`.
    ///
    /// ## Return
    ///
    /// A 1 byte number corresponding to the channel number.
    pub fn get_channel_number(&self) -> u8 {
        match *self {
            RadioChannel::Channel11 => 11,
            RadioChannel::Channel12 => 12,
            RadioChannel::Channel13 => 13,
            RadioChannel::Channel14 => 14,
            RadioChannel::Channel15 => 15,
            RadioChannel::Channel16 => 16,
            RadioChannel::Channel17 => 17,
            RadioChannel::Channel18 => 18,
            RadioChannel::Channel19 => 19,
            RadioChannel::Channel20 => 20,
            RadioChannel::Channel21 => 21,
            RadioChannel::Channel22 => 22,
            RadioChannel::Channel23 => 23,
            RadioChannel::Channel24 => 24,
            RadioChannel::Channel25 => 25,
            RadioChannel::Channel26 => 26,
        }
    }
}

impl TryFrom<u8> for RadioChannel {
    type Error = ();
    /// Try to convert a 1 byte channel number to a `RadioChannel`
    ///
    /// ## Argument
    ///
    /// - `val`: The channel number to convert.
    ///
    /// ## Return
    ///
    /// Returns `Ok(RadioChannel)` if `val` is a valid IEEE 802.15.4 2.4 GHz
    /// channel number. Otherwise, returns `Err(())`.
    fn try_from(val: u8) -> Result<RadioChannel, ()> {
        match val {
            11 => Ok(RadioChannel::Channel11),
            12 => Ok(RadioChannel::Channel12),
            13 => Ok(RadioChannel::Channel13),
            14 => Ok(RadioChannel::Channel14),
            15 => Ok(RadioChannel::Channel15),
            16 => Ok(RadioChannel::Channel16),
            17 => Ok(RadioChannel::Channel17),
            18 => Ok(RadioChannel::Channel18),
            19 => Ok(RadioChannel::Channel19),
            20 => Ok(RadioChannel::Channel20),
            21 => Ok(RadioChannel::Channel21),
            22 => Ok(RadioChannel::Channel22),
            23 => Ok(RadioChannel::Channel23),
            24 => Ok(RadioChannel::Channel24),
            25 => Ok(RadioChannel::Channel25),
            26 => Ok(RadioChannel::Channel26),
            _ => Err(()),
        }
    }
}
