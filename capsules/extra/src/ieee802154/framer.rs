// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implements IEEE 802.15.4 MAC device abstraction over a 802.15.4 MAC interface.
//!
//! Allows its users to prepare and send frames in plaintext, handling 802.15.4
//! encoding and security procedures (in the future) transparently.
//!
//! However, certain IEEE 802.15.4 MAC device concepts are not implemented in
//! this layer of abstraction and instead handled in hardware for performance
//! purposes. These include CSMA-CA backoff, FCS generation and authentication,
//! and automatic acknowledgement. Radio power management and channel selection
//! is also passed down to the MAC control layer.
//!
//! Usage
//! -----
//!
//! To use this capsule, we need an implementation of a hardware
//! `capsules_extra::ieee802154::mac::Mac`. Suppose we have such an implementation of type
//! `XMacDevice`.
//!
//! ```rust,ignore
//! let xmac: &XMacDevice = /* ... */;
//! let mac_device = static_init!(
//!     capsules_extra::ieee802154::mac::Framer<'static, XMacDevice>,
//!     capsules_extra::ieee802154::mac::Framer::new(xmac));
//! xmac.set_transmit_client(mac_device);
//! xmac.set_receive_client(mac_device, &mut MAC_RX_BUF);
//! xmac.set_config_client(mac_device);
//! ```
//!
//! The `mac_device` device is now set up. Users of the MAC device can now
//! configure the underlying radio, prepare and send frames:
//!
//! ```rust,ignore
//! mac_device.set_pan(0xABCD);
//! mac_device.set_address(0x1008);
//! mac_device.config_commit();
//!
//! let frame = mac_device
//!     .prepare_data_frame(&mut STATIC_BUFFER,
//!                         0xABCD, MacAddress::Short(0x1008),
//!                         0xABCD, MacAddress::Short(0x1009),
//!                         None)
//!     .ok()
//!     .map(|frame| {
//!         let rval = frame.append_payload(&mut SOME_DATA[..10]);
//!         if rval == Ok(()) {
//!             let (rval, _) = mac_device.transmit(frame);
//!             rval
//!         } else {
//!             rval
//!         }
//!     });
//! ```
//!
//! You should also be able to set up the userspace driver for receiving/sending
//! 802.15.4 frames:
//!
//! ```rust,ignore
//! use kernel::static_init;
//!
//! let radio_capsule = static_init!(
//!     capsules_extra::ieee802154::RadioDriver<'static>,
//!     capsules_extra::ieee802154::RadioDriver::new(mac_device, board_kernel.create_grant(&grant_cap), &mut RADIO_BUF));
//! mac_device.set_key_procedure(radio_capsule);
//! mac_device.set_device_procedure(radio_capsule);
//! mac_device.set_transmit_client(radio_capsule);
//! mac_device.set_receive_client(radio_capsule);
//! ```

//
// TODO: Encryption/decryption
// TODO: Sending beacon frames
// TODO: Channel scanning
//

use crate::ieee802154::device::{MacDevice, RxClient, TxClient};
use crate::ieee802154::mac::Mac;
use crate::net::ieee802154::{
    FrameType, FrameVersion, Header, KeyId, MacAddress, PanID, Security, SecurityLevel,
};
use crate::net::stream::SResult;
use crate::net::stream::{encode_bytes, encode_u32, encode_u8};

use core::cell::Cell;

use kernel::hil::radio::{self, LQI_SIZE};
use kernel::hil::symmetric_encryption::{CCMClient, AES128CCM};
use kernel::processbuffer::ReadableProcessSlice;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

/// Wraps a static mutable byte slice along with header information
/// for a payload.
///
/// This enables the user to abdicate any concerns about
/// where the payload should be placed in the buffer.
#[derive(Eq, PartialEq, Debug)]
pub struct Frame {
    buf: &'static mut [u8],
    info: FrameInfo,
}

/// This contains just enough information about a frame to determine
///
/// 1. How to encode it once its payload has been finalized
/// 2. The sizes of the mac header, payload and MIC tag length to be added
///
/// These offsets are relative to the PSDU or `buf[radio::PSDU_OFFSET..]` so
/// that the mac frame length is `data_offset + data_len`
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct FrameInfo {
    frame_type: FrameType,

    // The MAC payload, including Payload IEs
    mac_payload_offset: usize,
    // The data payload, not including Payload IEs
    data_offset: usize,
    // The length of the data payload, not including MIC and FCS
    data_len: usize,
    // The length of the MIC
    mic_len: usize,

    // Security level, key, and nonce
    security_params: Option<(SecurityLevel, [u8; 16], [u8; 13])>,
}

impl Frame {
    /// Consumes the frame and retrieves the buffer it wraps
    pub fn into_buf(self) -> &'static mut [u8] {
        self.buf
    }

    /// Calculates how much more data this frame can hold
    pub fn remaining_data_capacity(&self) -> usize {
        self.buf.len() - self.info.secured_length()
    }

    /// Appends payload bytes into the frame if possible
    pub fn append_payload(&mut self, payload: &[u8]) -> Result<(), ErrorCode> {
        if payload.len() > self.remaining_data_capacity() {
            return Err(ErrorCode::NOMEM);
        }
        let begin = self.info.unsecured_length();
        self.buf[begin..begin + payload.len()].copy_from_slice(payload);
        self.info.data_len += payload.len();

        Ok(())
    }

    /// Appends payload bytes from a process slice into the frame if
    /// possible
    pub fn append_payload_process(
        &mut self,
        payload_buf: &ReadableProcessSlice,
    ) -> Result<(), ErrorCode> {
        if payload_buf.len() > self.remaining_data_capacity() {
            return Err(ErrorCode::NOMEM);
        }
        let begin = self.info.unsecured_length();
        payload_buf.copy_to_slice(&mut self.buf[begin..begin + payload_buf.len()]);
        self.info.data_len += payload_buf.len();

        Ok(())
    }
}

impl FrameInfo {
    /// Current size of the frame, not including the MAC footer or the MIC
    fn unsecured_length(&self) -> usize {
        self.data_offset + self.data_len
    }

    /// Current size of the frame, not including the MAC footer but including
    /// the MIC
    fn secured_length(&self) -> usize {
        self.data_offset + self.data_len + self.mic_len
    }

    /// Compute the offsets in the buffer for the a data and m data fields in
    /// the CCM* authentication and encryption procedures which depends on the
    /// frame type and security levels. Returns the (offset, len) of the m data
    /// fields, not including the MIC. The a data is always the remaining prefix
    /// of the header, so it can be determined implicitly.
    #[allow(dead_code)]
    fn ccm_encrypt_ranges(&self) -> (usize, usize) {
        // IEEE 802.15.4-2015: Table 9-1. Exceptions to Private Payload field
        // The boundary between open and private payload fields depends
        // on the type of frame.
        let private_payload_offset = match self.frame_type {
            FrameType::Beacon => {
                // Beginning of beacon payload field
                unimplemented!()
            }
            FrameType::MACCommand => {
                // Beginning of MAC command content field
                unimplemented!()
            }
            _ => {
                // MAC payload field, which includes payload IEs
                self.mac_payload_offset
            }
        };

        // IEEE 802.15.4-2015: Table 9-3. a data and m data
        let encryption_needed = self
            .security_params
            .is_some_and(|(level, _, _)| level.encryption_needed());
        if !encryption_needed {
            // If only integrity is need, a data is the whole frame
            (self.unsecured_length(), 0)
        } else {
            // Otherwise, a data is the header and the open payload, and
            // m data is the private payload field; unsecured length is the end of
            // private payload, length of private payload is difference between
            // the offset and unsecured length
            (
                private_payload_offset,                           // m_offset
                self.unsecured_length() - private_payload_offset, // m_len
            )
        }
    }
}

/// Generate a 15.4 CCM nonce from the device address, frame counter, and SecurityLevel
pub fn get_ccm_nonce(device_addr: &[u8; 8], frame_counter: u32, level: SecurityLevel) -> [u8; 13] {
    let mut nonce = [0u8; 13];
    let encode_ccm_nonce = |buf: &mut [u8]| {
        let off = enc_consume!(buf; encode_bytes, device_addr.as_ref());
        let off = enc_consume!(buf, off; encode_u32, frame_counter);
        let off = enc_consume!(buf, off; encode_u8, level as u8);
        stream_done!(off);
    };
    match encode_ccm_nonce(&mut nonce).done() {
        None => {
            // This should not be possible
            panic!("Failed to produce ccm nonce");
        }
        Some(_) => nonce,
    }
}

/// The needed buffer size might be bigger than an MTU, because
/// the CCM* authentication procedure
///
/// - adds an extra 16-byte block in front of the a and m data
/// - prefixes the a data with a length encoding and pads the result
/// - pads the m data to 16-byte blocks
pub const CRYPT_BUF_SIZE: usize = radio::MAX_MTU + 3 * 16;

/// IEEE 802.15.4-2015, 9.2.2, KeyDescriptor lookup procedure.
///
/// Trait to be implemented by an upper layer that manages the list of 802.15.4
/// key descriptors. This trait interface enables the lookup procedure to be
/// implemented either explicitly (managing a list of KeyDescriptors) or
/// implicitly with some equivalent logic.
pub trait KeyProcedure {
    /// Lookup the KeyDescriptor matching the provided security level and key ID
    /// mode and return the key associated with it.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<[u8; 16]>;
}

/// IEEE 802.15.4-2015, 9.2.5, DeviceDescriptor lookup procedure.
///
/// Trait to be implemented by an upper layer that manages the list of 802.15.4
/// device descriptors. This trait interface enables the lookup procedure to be
/// implemented either explicitly (managing a list of DeviceDescriptors) or
/// implicitly with some equivalent logic.
pub trait DeviceProcedure {
    /// Look up the extended MAC address of a device given either its short or
    /// long address. As defined in the IEEE 802.15.4 spec, even if the provided
    /// address is already long, a long address should be returned only if the
    /// given address matches a known DeviceDescriptor.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<[u8; 8]>;
}

/// This state enum describes the state of the transmission pipeline.
/// Conditionally-present state is also included as fields in the enum variants.
/// We can view the transmission process as a state machine driven by the
/// following events:
///
/// - calls to `Mac#transmit`
/// - `send_done` callbacks from the underlying radio
/// - `config_done` callbacks from the underlying radio (if, for example,
///   configuration was in progress when a transmission was requested)
#[derive(Eq, PartialEq, Debug)]
enum TxState {
    /// There is no frame to be transmitted.
    Idle,
    /// There is a valid frame that needs to be secured before transmission.
    ReadyToEncrypt(FrameInfo, &'static mut [u8]),
    /// There is currently a frame being encrypted by the encryption facility.
    #[allow(dead_code)]
    Encrypting(FrameInfo),
    /// There is a frame that is completely secured or does not require
    /// security, and is waiting to be passed to the radio.
    ReadyToTransmit(FrameInfo, &'static mut [u8]),
}

#[derive(Eq, PartialEq, Debug)]
enum RxState {
    /// There is no frame that has been received.
    Idle,
    /// There is a secured frame that needs to be decrypted.
    /// ReadyToDecrypt(FrameInfo, buf, lqi)
    ReadyToDecrypt(FrameInfo, &'static mut [u8], u8),
    /// A secured frame is currently being decrypted by the decryption facility.
    /// Decrypting(FrameInfo, lqi)
    #[allow(dead_code)]
    Decrypting(FrameInfo, u8),
    /// There is an unsecured frame that needs to be re-parsed and exposed to
    /// the client. ReadyToYield(FrameInfo, buf, lqi)
    #[allow(dead_code)]
    ReadyToYield(FrameInfo, &'static mut [u8], u8),
}

/// Wraps an IEEE 802.15.4 [kernel::hil::radio::Radio]
/// and exposes [`capsules_extra::ieee802154::mac::Mac`](crate::ieee802154::mac::Mac) functionality.
///
/// It hides header preparation, transmission and processing logic
/// from the user by essentially maintaining multiple state machines
/// corresponding to the transmission, reception and
/// encryption/decryption pipelines. See the documentation in
/// `capsules/extra/src/ieee802154/mac.rs` for more details.
pub struct Framer<'a, M: Mac<'a>, A: AES128CCM<'a>> {
    mac: &'a M,
    aes_ccm: &'a A,
    data_sequence: Cell<u8>,

    /// KeyDescriptor lookup procedure
    key_procedure: OptionalCell<&'a dyn KeyProcedure>,
    /// DeviceDescriptor lookup procedure
    device_procedure: OptionalCell<&'a dyn DeviceProcedure>,

    /// Transmission pipeline state. This should never be `None`, except when
    /// transitioning between states. That is, any method that consumes the
    /// current state should always remember to replace it along with the
    /// associated state information.
    tx_state: MapCell<TxState>,
    tx_client: OptionalCell<&'a dyn TxClient>,

    /// Reception pipeline state. Similar to the above, this should never be
    /// `None`, except when transitioning between states.
    rx_state: MapCell<RxState>,
    rx_client: OptionalCell<&'a dyn RxClient>,
    crypt_buf: MapCell<SubSliceMut<'static, u8>>,
}

impl<'a, M: Mac<'a>, A: AES128CCM<'a>> Framer<'a, M, A> {
    pub fn new(
        mac: &'a M,
        aes_ccm: &'a A,
        crypt_buf: SubSliceMut<'static, u8>,
    ) -> Framer<'a, M, A> {
        Framer {
            mac,
            aes_ccm,
            data_sequence: Cell::new(0),
            key_procedure: OptionalCell::empty(),
            device_procedure: OptionalCell::empty(),
            tx_state: MapCell::new(TxState::Idle),
            tx_client: OptionalCell::empty(),
            rx_state: MapCell::new(RxState::Idle),
            rx_client: OptionalCell::empty(),
            crypt_buf: MapCell::new(crypt_buf),
        }
    }

    /// Sets the IEEE 802.15.4 key lookup procedure to be used.
    pub fn set_key_procedure(&self, key_procedure: &'a dyn KeyProcedure) {
        self.key_procedure.set(key_procedure);
    }

    /// Sets the IEEE 802.15.4 key lookup procedure to be used.
    pub fn set_device_procedure(&self, device_procedure: &'a dyn DeviceProcedure) {
        self.device_procedure.set(device_procedure);
    }

    /// Look up the key using the IEEE 802.15.4 KeyDescriptor lookup procedure
    /// implemented elsewhere.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<[u8; 16]> {
        self.key_procedure
            .and_then(|key_procedure| key_procedure.lookup_key(level, key_id))
    }

    /// IEEE 802.15.4-2015, 9.2.1, outgoing frame security procedure
    /// Performs the first checks in the security procedure. The rest of the
    /// steps are performed as part of the transmission pipeline.
    /// Returns the next `TxState` to enter.
    fn outgoing_frame_security(&self, buf: &'static mut [u8], frame_info: FrameInfo) -> TxState {
        // IEEE 802.15.4-2015: 9.2.1, outgoing frame security
        // Steps a-e have already been performed in the frame preparation step,
        // so we only need to dispatch on the security parameters in the frame info
        match frame_info.security_params {
            Some((level, _, _)) => {
                if level == SecurityLevel::None {
                    // This case should never occur if the FrameInfo was
                    // prepared by prepare_data_frame
                    TxState::ReadyToTransmit(frame_info, buf)
                } else {
                    TxState::ReadyToEncrypt(frame_info, buf)
                }
            }
            None => TxState::ReadyToTransmit(frame_info, buf),
        }
    }

    /// IEEE 802.15.4-2015, 9.2.3, incoming frame security procedure
    fn incoming_frame_security(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        lqi: u8,
    ) -> RxState {
        // Try to decode the MAC header. Three possible results can occur:
        // 1) The frame should be dropped and the buffer returned to the radio
        // 2) The frame is unsecured. We immediately expose the frame to the
        //    user and queue the buffer for returning to the radio.
        // 3) The frame needs to be unsecured.

        // The buffer containing the 15.4 packet also contains the PSDU bytes and an LQI
        // byte. We only pass the 15.4 packet up the stack and slice buf accordingly.
        let frame_buffer = &buf[radio::PSDU_OFFSET..(buf.len() - LQI_SIZE)];

        let result = Header::decode(frame_buffer, false)
            .done()
            .and_then(|(data_offset, (header, mac_payload_offset))| {
                // Note: there is a complication here regarding the offsets.
                // When the received frame has security enabled, the payload
                // (including the payload IEs) is encrypted, and hence the data
                // payload field includes the encrypted payload IEs too.
                // However, when the frame is not encrypted, the data payload
                // field does not include the payload IEs.
                //
                // This is fine because we re-parse the unsecured frame before
                // exposing it to the user. At that time, the data payload field
                // will not include the payload IEs.
                let mic_len = header.security.map_or(0, |sec| sec.level.mic_len());
                let data_len = frame_len - data_offset - mic_len;
                if let Some(security) = header.security {
                    // IEEE 802.15.4-2015: 9.2.3, incoming frame security procedure
                    // for security-enabled headers
                    if header.version == FrameVersion::V2003 {
                        None
                    } else {
                        // Step e: Lookup the key.
                        let key = match self.lookup_key(security.level, security.key_id) {
                            Some(key) => key,
                            None => {
                                return None;
                            }
                        };

                        // Step f: Obtain the extended source address
                        // TODO: For Thread, when the frame's security header
                        // specifies `KeyIdMode::Source4Index`, the source
                        // address used for the nonce is actually a constant
                        // defined in their spec
                        let device_addr = match header.src_addr {
                            Some(mac) => match mac {
                                MacAddress::Long(val) => val,
                                MacAddress::Short(_) => {
                                    kernel::debug!("[15.4] DROPPED PACKET - error only short address provided on encrypted packet.");
                                    return None
                                },
                            },
                            None => {
                                kernel::debug!("[15.4] DROPPED PACKET - Malformed, no src address provided.");
                                return None
                            },
                        };

                        // Step g, h: Check frame counter
                        let frame_counter = match security.frame_counter {
                            Some(frame_counter) => {
                                if frame_counter == 0xffffffff {
                                    // Counter error
                                    return None;
                                }
                                // TODO: Check frame counter against source device
                                frame_counter
                            }
                            // TSCH mode, where ASN is used instead, not supported
                            None => {
                                return None;
                            }
                        };

                        // Compute ccm nonce
                        let nonce = get_ccm_nonce(&device_addr, frame_counter, security.level);

                        Some(FrameInfo {
                            frame_type: header.frame_type,
                            mac_payload_offset,
                            data_offset,
                            data_len,
                            mic_len,
                            security_params: Some((security.level, key, nonce)),
                        })
                    }
                } else {
                    // No security needed, can yield the frame immediately

                    // The buffer containing the 15.4 packet also contains the PSDU bytes and an LQI
                    // byte. We only pass the 15.4 packet up the stack and slice buf accordingly.
                    let frame_buffer = &buf[radio::PSDU_OFFSET..(buf.len() - LQI_SIZE)];
                    self.rx_client.map(|client| {
                        client.receive(frame_buffer, header, lqi, data_offset, data_len);
                    });
                    None
                }
            });

        match result {
            None => {
                // The packet was not encrypted, we completed the 15.4 framer procedure, and passed the packet to the
                // client. We can now return the recv buffer to the radio driver and enter framer's idle state.
                self.mac.set_receive_buffer(buf);
                RxState::Idle
            }
            Some(frame_info) => RxState::ReadyToDecrypt(frame_info, buf, lqi),
        }
    }

    /// Advances the transmission pipeline if it can be advanced.
    fn step_transmit_state(&self) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.tx_state.take().map_or_else(
            || panic!("missing tx_state"),
            |state| {
                // This mechanism is a little more clunky, but makes it
                // difficult to forget to replace `tx_state`.
                let (next_state, result) = match state {
                    TxState::Idle => (TxState::Idle, Ok(())),
                    TxState::ReadyToEncrypt(info, buf) => {
                        match info.security_params {
                            None => {
                                // `ReadyToEncrypt` should only be entered when
                                // `security_params` is not `None`.
                                (TxState::Idle, Err((ErrorCode::FAIL, buf)))
                            }
                            Some((level, key, nonce)) => {
                                let (m_off, m_len) = info.ccm_encrypt_ranges();
                                let (a_off, m_off) =
                                    (radio::PSDU_OFFSET, radio::PSDU_OFFSET + m_off);

                                // Crypto setup failed; fail sending packet and return to idle
                                if self.aes_ccm.set_key(&key) != Ok(())
                                    || self.aes_ccm.set_nonce(&nonce) != Ok(())
                                {
                                    (TxState::Idle, Err((ErrorCode::FAIL, buf)))
                                } else {
                                    let res = self.aes_ccm.crypt(
                                        buf,
                                        a_off,
                                        m_off,
                                        m_len,
                                        info.mic_len,
                                        level.encryption_needed(),
                                        true,
                                    );
                                    match res {
                                        Ok(()) => (TxState::Encrypting(info), Ok(())),
                                        Err((ErrorCode::BUSY, buf)) => {
                                            (TxState::ReadyToEncrypt(info, buf), Ok(()))
                                        }
                                        Err((ecode, buf)) => (TxState::Idle, Err((ecode, buf))),
                                    }
                                }
                            }
                        }
                    }
                    TxState::Encrypting(info) => {
                        // This state should be advanced only by the hardware
                        // encryption callback.
                        (TxState::Encrypting(info), Ok(()))
                    }
                    TxState::ReadyToTransmit(info, buf) => {
                        let res = self.mac.transmit(buf, info.secured_length());
                        match res {
                            // If the radio is busy, just wait for either a
                            // transmit_done or config_done callback to trigger
                            // this state transition again
                            Err((ErrorCode::BUSY, buf)) => {
                                (TxState::ReadyToTransmit(info, buf), Ok(()))
                            }
                            Ok(()) => (TxState::Idle, Ok(())),
                            Err((ecode, buf)) => (TxState::Idle, Err((ecode, buf))),
                        }
                    }
                };
                self.tx_state.replace(next_state);
                result
            },
        )
    }

    /// Advances the reception pipeline if it can be advanced.
    fn step_receive_state(&self) {
        self.rx_state.take().map(|state| {
            let next_state = match state {
                RxState::Idle => RxState::Idle,
                RxState::ReadyToDecrypt(info, buf, lqi) => {
                    match info.security_params {
                        None => {
                            // `ReadyToDecrypt` should only be entered when
                            // `security_params` is not `None`.
                            RxState::Idle
                        }
                        Some((level, key, nonce)) => {
                            let (m_off, m_len) = info.ccm_encrypt_ranges();
                            let (a_off, m_off) = (radio::PSDU_OFFSET, radio::PSDU_OFFSET + m_off);

                            // Crypto setup failed; fail receiving packet and return to idle
                            if self.aes_ccm.set_key(&key) != Ok(())
                                || self.aes_ccm.set_nonce(&nonce) != Ok(())
                            {
                                // No error is returned for the receive function because recv occurs implicitly
                                // Log debug statement here so that this error does not occur silently
                                kernel::debug!(
                                    "[15.4 RECV FAIL] - Failed setting crypto key/nonce."
                                );
                                self.mac.set_receive_buffer(buf);
                                RxState::Idle
                            } else {
                                // The crypto operation requires multiple steps through the receiving pipeline and
                                // an unknown quanitity of time to perform decryption. Holding the 15.4 radio's
                                // receive buffer for this period of time is suboptimal as packets will be dropped.
                                // The radio driver assumes the mac.set_receive_buffer(...) function is called prior
                                // to returning from the framer. These constraints necessitate the creation of a seperate
                                // crypto buffer for the radio framer so that the framer can return the radio driver's
                                // receive buffer and then perform decryption using the copied packet in the crypto buffer.
                                let res = self.crypt_buf.take().map(|mut crypt_buf| {
                                    crypt_buf[0..buf.len()].copy_from_slice(buf);
                                    crypt_buf.slice(0..buf.len());

                                    self.aes_ccm.crypt(
                                        crypt_buf.take(),
                                        a_off,
                                        m_off,
                                        m_len,
                                        info.mic_len,
                                        level.encryption_needed(),
                                        true,
                                    )
                                });

                                // The potential scenarios include:
                                // - (1) Successfully transfer packet to crypto buffer and succesfully begin crypto operation
                                // - (2) Succesfully transfer packet to crypto buffer, but the crypto operation aes_ccm.crypt(...)
                                //   is busy so we do not advance the reception pipeline and retry on the next iteration
                                // - (3) Succesfully transfer packet to crypto buffer, but the crypto operation fails for some
                                //   unknown reason (likely due to the crypto buffer's configuration or the offset/len parameters
                                //   passed to the function. It is not possible to decrypt the packet so we drop the packet, return
                                //   the radio drivers recv buffer and return the framer recv state machine to idle
                                // - (4) The crypto buffer is empty (in use elsewhere) and we are unable to copy the received
                                //   packet. This packet is dropped and we must return the buffer to the radio driver. This
                                //   scenario is handled in the None case
                                match res {
                                    // Scenario 1
                                    Some(Ok(())) => {
                                        self.mac.set_receive_buffer(buf);
                                        RxState::Decrypting(info, lqi)
                                    }
                                    // Scenario 2
                                    Some(Err((ErrorCode::BUSY, buf))) => {
                                        RxState::ReadyToDecrypt(info, buf, lqi)
                                    }
                                    // Scenario 3
                                    Some(Err((_, fail_crypt_buf))) => {
                                        self.mac.set_receive_buffer(buf);
                                        self.crypt_buf.replace(SubSliceMut::new(fail_crypt_buf));
                                        RxState::Idle
                                    }
                                    // Scenario 4
                                    None => {
                                        self.mac.set_receive_buffer(buf);
                                        RxState::Idle
                                    }
                                }
                            }
                        }
                    }
                }
                RxState::Decrypting(info, lqi) => {
                    // This state should be advanced only by the hardware
                    // encryption callback.
                    RxState::Decrypting(info, lqi)
                }
                RxState::ReadyToYield(info, buf, lqi) => {
                    // Between the secured and unsecured frames, the
                    // unsecured frame length remains constant but the data
                    // offsets may change due to the presence of PayloadIEs.
                    // Hence, we can only use the unsecured length from the
                    // frame info, but not the offsets.
                    let frame_len = info.unsecured_length();
                    if let Some((data_offset, (header, _))) = Header::decode(
                        &buf[radio::PSDU_OFFSET..(radio::PSDU_OFFSET + radio::MAX_FRAME_SIZE)],
                        true,
                    )
                    .done()
                    {
                        // IEEE 802.15.4-2015 specifies that unsecured
                        // frames do not have auxiliary security headers,
                        // but we do not remove the auxiliary security
                        // header before returning the frame to the client.
                        // This is so that it is possible to tell if the
                        // frame was secured or unsecured, while still
                        // always receiving the frame payload in plaintext.
                        //
                        // The buffer containing the 15.4 packet also contains
                        // the PSDU bytes and an LQI byte. We only pass the
                        // 15.4 packet up the stack and slice buf accordingly.
                        let frame_buffer = &buf[radio::PSDU_OFFSET..(buf.len() - LQI_SIZE)];
                        self.rx_client.map(|client| {
                            client.receive(
                                frame_buffer,
                                header,
                                lqi,
                                data_offset,
                                frame_len - data_offset,
                            );
                        });
                        self.crypt_buf.replace(SubSliceMut::new(buf));
                    }
                    RxState::Idle
                }
            };
            self.rx_state.replace(next_state);
        });
    }
}

impl<'a, M: Mac<'a>, A: AES128CCM<'a>> MacDevice<'a> for Framer<'a, M, A> {
    fn set_transmit_client(&self, client: &'a dyn TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'a dyn RxClient) {
        self.rx_client.set(client);
    }

    fn get_address(&self) -> u16 {
        self.mac.get_address()
    }

    fn get_address_long(&self) -> [u8; 8] {
        self.mac.get_address_long()
    }

    fn get_pan(&self) -> u16 {
        self.mac.get_pan()
    }

    fn set_address(&self, addr: u16) {
        self.mac.set_address(addr)
    }

    fn set_address_long(&self, addr: [u8; 8]) {
        self.mac.set_address_long(addr)
    }

    fn set_pan(&self, id: u16) {
        self.mac.set_pan(id)
    }

    fn config_commit(&self) {
        self.mac.config_commit()
    }

    fn is_on(&self) -> bool {
        self.mac.is_on()
    }

    fn start(&self) -> Result<(), ErrorCode> {
        self.mac.start()
    }

    fn prepare_data_frame(
        &self,
        buf: &'static mut [u8],
        dst_pan: PanID,
        dst_addr: MacAddress,
        src_pan: PanID,
        src_addr: MacAddress,
        security_needed: Option<(SecurityLevel, KeyId)>,
    ) -> Result<Frame, &'static mut [u8]> {
        // IEEE 802.15.4-2015: 9.2.1, outgoing frame security
        // Steps a-e of the security procedure are implemented here.

        // TODO: For Thread, in the case of `KeyIdMode::Source4Index`, the source
        // address should instead be some constant defined in their
        // specification.

        let security_desc = security_needed.and_then(|(level, key_id)| {
            // To decrypt the packet, we need the long addr.
            // Without the long addr, we are unable to proceed
            // and return None
            let src_addr_long = match src_addr {
                MacAddress::Long(addr) => addr,
                MacAddress::Short(_) => return None,
            };

            self.lookup_key(level, key_id).map(|key| {
                // TODO: lookup frame counter for device
                let frame_counter = 0;
                let nonce = get_ccm_nonce(&src_addr_long, frame_counter, level);
                (
                    Security {
                        level,
                        asn_in_nonce: false,
                        frame_counter: Some(frame_counter),
                        key_id,
                    },
                    key,
                    nonce,
                )
            })
        });
        if security_needed.is_some() && security_desc.is_none() {
            // If security was requested, fail when desired key was not found.
            return Err(buf);
        }

        // Construct MAC header
        let security = security_desc.map(|(sec, _, _)| sec);
        let mic_len = security.map_or(0, |sec| sec.level.mic_len());
        let header = Header {
            frame_type: FrameType::Data,
            /* TODO: determine this by looking at queue, and also set it in
             * hardware so that ACKs set this flag to the right value. */
            frame_pending: false,
            // Unicast data frames request acknowledgement
            ack_requested: true,
            version: FrameVersion::V2006,
            seq: Some(self.data_sequence.get()),
            dst_pan: Some(dst_pan),
            dst_addr: Some(dst_addr),
            src_pan: Some(src_pan),
            src_addr: Some(src_addr),
            security,
            header_ies: Default::default(),
            header_ies_len: 0,
            payload_ies: Default::default(),
            payload_ies_len: 0,
        };

        match header.encode(buf, true).done() {
            Some((data_offset, mac_payload_offset)) => Ok(Frame {
                buf,
                info: FrameInfo {
                    frame_type: FrameType::Data,
                    mac_payload_offset,
                    data_offset,
                    data_len: 0,
                    mic_len,
                    security_params: security_desc.map(|(sec, key, nonce)| (sec.level, key, nonce)),
                },
            }),
            None => Err(buf),
        }
    }

    fn transmit(&self, frame: Frame) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let Frame { buf, info } = frame;
        let state = match self.tx_state.take() {
            None => {
                return Err((ErrorCode::FAIL, buf));
            }
            Some(state) => state,
        };
        match state {
            TxState::Idle => {
                let next_state = self.outgoing_frame_security(buf, info);
                self.tx_state.replace(next_state);
                self.step_transmit_state()
            }
            other_state => {
                self.tx_state.replace(other_state);
                Err((ErrorCode::BUSY, buf))
            }
        }
    }
}

impl<'a, M: Mac<'a>, A: AES128CCM<'a>> radio::TxClient for Framer<'a, M, A> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: Result<(), ErrorCode>) {
        self.data_sequence.set(self.data_sequence.get() + 1);
        self.tx_client.map(move |client| {
            client.send_done(buf, acked, result);
        });
    }
}

impl<'a, M: Mac<'a>, A: AES128CCM<'a>> radio::RxClient for Framer<'a, M, A> {
    fn receive(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        lqi: u8,
        crc_valid: bool,
        _: Result<(), ErrorCode>,
    ) {
        // Drop all frames with invalid CRC
        if !crc_valid {
            self.mac.set_receive_buffer(buf);
            return;
        }

        self.rx_state.take().map(move |state| {
            let next_state = match state {
                RxState::Idle => {
                    // We can start processing a new received frame only if
                    // the reception pipeline is free
                    self.incoming_frame_security(buf, frame_len, lqi)
                }
                other_state => {
                    // This should never occur unless something other than
                    // this MAC layer provided a receive buffer to the
                    // radio, but if this occurs then we have no choice but
                    // to drop the frame.
                    self.mac.set_receive_buffer(buf);
                    other_state
                }
            };
            self.rx_state.replace(next_state);
            self.step_receive_state()
        });
    }
}

impl<'a, M: Mac<'a>, A: AES128CCM<'a>> radio::ConfigClient for Framer<'a, M, A> {
    fn config_done(&self, _: Result<(), ErrorCode>) {
        // The transmission pipeline is the only state machine that
        // waits for the configuration procedure to complete before
        // advancing.
        let _ = self.step_transmit_state().map_err(|(ecode, buf)| {
            // Return the buffer to the transmit client
            self.tx_client.map(move |client| {
                client.send_done(buf, false, Err(ecode));
            });
        });
    }
}

impl<'a, M: Mac<'a>, A: AES128CCM<'a>> CCMClient for Framer<'a, M, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        let mut tx_waiting = false;
        let mut rx_waiting = false;

        // The crypto operation was from the transmission pipeline.
        let opt_buf = if let Some(state) = self.tx_state.take() {
            match state {
                TxState::Encrypting(info) => {
                    let res2 = match res {
                        Err(ecode) => {
                            self.tx_state.replace(TxState::Idle);
                            Err((ecode, buf))
                        }
                        Ok(()) => {
                            self.tx_state.replace(TxState::ReadyToTransmit(info, buf));
                            self.step_transmit_state()
                        }
                    };

                    if let Err((ecode, buf)) = res2 {
                        // Abort the transmission process. Return the buffer to the client.
                        self.tx_client.map(move |client| {
                            client.send_done(buf, false, Err(ecode));
                        });
                    }
                    None
                }
                other_state => {
                    tx_waiting = match other_state {
                        TxState::ReadyToEncrypt(_, _) => true,
                        _ => false,
                    };
                    self.tx_state.replace(other_state);
                    Some(buf)
                }
            }
        } else {
            Some(buf)
        };

        // The crypto operation was from the reception pipeline.
        if let Some(buf) = opt_buf {
            self.rx_state.take().map(|state| {
                match state {
                    RxState::Decrypting(info, lqi) => {
                        let next_state = if tag_is_valid {
                            RxState::ReadyToYield(info, buf, lqi)
                        } else {
                            // The CRC tag is invalid, meaning the packet was corrupted. Drop this packet
                            // and reset reception pipeline
                            self.crypt_buf.replace(SubSliceMut::new(buf));
                            RxState::Idle
                        };
                        self.rx_state.replace(next_state);
                        self.step_receive_state()
                    }
                    other_state => {
                        rx_waiting = match other_state {
                            RxState::ReadyToDecrypt(_, _, _) => true,
                            _ => false,
                        };
                        self.rx_state.replace(other_state);
                    }
                }
            });
        }

        // Now trigger the next crypto operation if one exists.
        if tx_waiting {
            let _ = self.step_transmit_state().map_err(|(ecode, buf)| {
                // Return the buffer to the client.
                self.tx_client.map(move |client| {
                    client.send_done(buf, false, Err(ecode));
                });
            });
        } else if rx_waiting {
            self.step_receive_state()
        }
    }
}
