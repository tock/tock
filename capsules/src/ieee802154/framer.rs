//! Implements IEEE 802.15.4 MAC device abstraction over a 802.15.4 MAC interface.
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
//! `capsules::ieee802154::mac::Mac`. Suppose we have such an implementation of type
//! `XMacDevice`.
//!
//! ```rust
//! let xmac: &XMacDevice = /* ... */;
//! let mac_device = static_init!(
//!     capsules::ieee802154::mac::Framer<'static, XMacDevice>,
//!     capsules::ieee802154::mac::Framer::new(xmac));
//! xmac.set_transmit_client(mac_device);
//! xmac.set_receive_client(mac_device, &mut MAC_RX_BUF);
//! xmac.set_config_client(mac_device);
//! ```
//!
//! The `mac_device` device is now set up. Users of the MAC device can now
//! configure the underlying radio, prepare and send frames:
//!
//! ```rust
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
//!         if rval == ReturnCode::SUCCESS {
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
//! ```rust
//! let radio_capsule = static_init!(
//!     capsules::ieee802154::RadioDriver<'static>,
//!     capsules::ieee802154::RadioDriver::new(mac_device, kernel::Grant::create(), &mut RADIO_BUF));
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
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::hil::radio;
use kernel::hil::symmetric_encryption::{CCMClient, AES128CCM};
use kernel::ReturnCode;

/// A `Frame` wraps a static mutable byte slice and keeps just enough
/// information about its header contents to expose a restricted interface for
/// modifying its payload. This enables the user to abdicate any concerns about
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
        self.buf.len() - radio::PSDU_OFFSET - radio::MFR_SIZE - self.info.secured_length()
    }

    /// Appends payload bytes into the frame if possible
    pub fn append_payload(&mut self, payload: &[u8]) -> ReturnCode {
        if payload.len() > self.remaining_data_capacity() {
            return ReturnCode::ENOMEM;
        }
        let begin = radio::PSDU_OFFSET + self.info.unsecured_length();
        self.buf[begin..begin + payload.len()].copy_from_slice(payload);
        self.info.data_len += payload.len();

        ReturnCode::SUCCESS
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
            .map_or(false, |(level, _, _)| level.encryption_needed());
        if !encryption_needed {
            // If only integrity is need, a data is the whole frame
            (self.unsecured_length(), 0)
        } else {
            // Otherwise, a data is the header and the open payload, and
            // m data is the private payload field
            (
                private_payload_offset,
                self.unsecured_length() | private_payload_offset,
            )
        }
    }
}

fn get_ccm_nonce(device_addr: [u8; 8], frame_counter: u32, level: SecurityLevel) -> [u8; 13] {
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
/// Trait to be implemented by an upper layer that manages the list of 802.15.4
/// key descriptors. This trait interface enables the lookup procedure to be
/// implemented either explicitly (managing a list of KeyDescriptors) or
/// implicitly with some equivalent logic.
pub trait KeyProcedure {
    /// Lookup the KeyDescriptor matching the provided security level and key ID
    /// mode and return the key associatied with it.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<([u8; 16])>;
}

/// IEEE 802.15.4-2015, 9.2.5, DeviceDescriptor lookup procedure.
/// Trait to be implemented by an upper layer that manages the list of 802.15.4
/// device descriptors. This trait interface enables the lookup procedure to be
/// implemented either explicitly (managing a list of DeviceDescriptors) or
/// implicitly with some equivalent logic.
pub trait DeviceProcedure {
    /// Look up the extended MAC address of a device given either its short or
    /// long address. As defined in the IEEE 802.15.4 spec, even if the provided
    /// address is already long, a long address should be returned only if the
    /// given address matches a known DeviceDescriptor.
    fn lookup_addr_long(&self, addr: MacAddress) -> Option<([u8; 8])>;
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
    ReadyToDecrypt(FrameInfo, &'static mut [u8]),
    /// A secured frame is currently being decrypted by the decryption facility.
    #[allow(dead_code)]
    Decrypting(FrameInfo),
    /// There is an unsecured frame that needs to be re-parsed and exposed to
    /// the client.
    #[allow(dead_code)]
    ReadyToYield(FrameInfo, &'static mut [u8]),
    /// The buffer containing the frame needs to be returned to the radio.
    ReadyToReturn(&'static mut [u8]),
}

/// This struct wraps an IEEE 802.15.4 radio device `kernel::hil::radio::Radio`
/// and exposes IEEE 802.15.4 MAC device functionality as the trait
/// `capsules::mac::Mac`. It hides header preparation, transmission and
/// processing logic from the user by essentially maintaining multiple state
/// machines corresponding to the transmission, reception and
/// encryption/decryption pipelines. See the documentation in
/// `capsules/src/mac.rs` for more details.
pub struct Framer<'a, M: Mac, A: AES128CCM<'a>> {
    mac: &'a M,
    aes_ccm: &'a A,
    data_sequence: Cell<u8>,

    /// KeyDescriptor lookup procedure
    key_procedure: OptionalCell<&'a KeyProcedure>,
    /// DeviceDescriptor lookup procedure
    device_procedure: OptionalCell<&'a DeviceProcedure>,

    /// Transmision pipeline state. This should never be `None`, except when
    /// transitioning between states. That is, any method that consumes the
    /// current state should always remember to replace it along with the
    /// associated state information.
    tx_state: MapCell<TxState>,
    tx_client: OptionalCell<&'a TxClient>,

    /// Reception pipeline state. Similar to the above, this should never be
    /// `None`, except when transitioning between states.
    rx_state: MapCell<RxState>,
    rx_client: OptionalCell<&'a RxClient>,
}

impl<M: Mac, A: AES128CCM<'a>> Framer<'a, M, A> {
    pub fn new(mac: &'a M, aes_ccm: &'a A) -> Framer<'a, M, A> {
        Framer {
            mac: mac,
            aes_ccm: aes_ccm,
            data_sequence: Cell::new(0),
            key_procedure: OptionalCell::empty(),
            device_procedure: OptionalCell::empty(),
            tx_state: MapCell::new(TxState::Idle),
            tx_client: OptionalCell::empty(),
            rx_state: MapCell::new(RxState::Idle),
            rx_client: OptionalCell::empty(),
        }
    }

    /// Sets the IEEE 802.15.4 key lookup procedure to be used.
    pub fn set_key_procedure(&self, key_procedure: &'a KeyProcedure) {
        self.key_procedure.set(key_procedure);
    }

    /// Sets the IEEE 802.15.4 key lookup procedure to be used.
    pub fn set_device_procedure(&self, device_procedure: &'a DeviceProcedure) {
        self.device_procedure.set(device_procedure);
    }

    /// Look up the key using the IEEE 802.15.4 KeyDescriptor lookup prodecure
    /// implemented elsewhere.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<([u8; 16])> {
        self.key_procedure
            .and_then(|key_procedure| key_procedure.lookup_key(level, key_id))
    }

    /// Look up the extended address of a device using the IEEE 802.15.4
    /// DeviceDescriptor lookup prodecure implemented elsewhere.
    fn lookup_addr_long(&self, src_addr: Option<MacAddress>) -> Option<([u8; 8])> {
        src_addr.and_then(|addr| {
            self.device_procedure
                .and_then(|device_procedure| device_procedure.lookup_addr_long(addr))
        })
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
    fn incoming_frame_security(&self, buf: &'static mut [u8], frame_len: usize) -> RxState {
        // Try to decode the MAC header. Three possible results can occur:
        // 1) The frame should be dropped and the buffer returned to the radio
        // 2) The frame is unsecured. We immediately expose the frame to the
        //    user and queue the buffer for returning to the radio.
        // 3) The frame needs to be unsecured.
        let result = Header::decode(&buf[radio::PSDU_OFFSET..], false)
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
                        let device_addr = match self.lookup_addr_long(header.src_addr) {
                            Some(addr) => addr,
                            None => {
                                return None;
                            }
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
                        let nonce = get_ccm_nonce(device_addr, frame_counter, security.level);

                        Some(FrameInfo {
                            frame_type: header.frame_type,
                            mac_payload_offset: mac_payload_offset,
                            data_offset: data_offset,
                            data_len: data_len,
                            mic_len: mic_len,
                            security_params: Some((security.level, key, nonce)),
                        })
                    }
                } else {
                    // No security needed, can yield the frame immediately
                    self.rx_client.map(|client| {
                        client.receive(&buf, header, radio::PSDU_OFFSET + data_offset, data_len);
                    });
                    None
                }
            });

        match result {
            None => RxState::ReadyToReturn(buf),
            Some(frame_info) => RxState::ReadyToDecrypt(frame_info, buf),
        }
    }

    /// Advances the transmission pipeline if it can be advanced.
    fn step_transmit_state(&self) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.tx_state
            .take()
            .map_or((ReturnCode::FAIL, None), |state| {
                // This mechanism is a little more clunky, but makes it
                // difficult to forget to replace `tx_state`.
                let (next_state, result) = match state {
                    TxState::Idle => (TxState::Idle, (ReturnCode::SUCCESS, None)),
                    TxState::ReadyToEncrypt(info, buf) => {
                        match info.security_params {
                            None => {
                                // `ReadyToEncrypt` should only be entered when
                                // `security_params` is not `None`.
                                (TxState::Idle, (ReturnCode::FAIL, Some(buf)))
                            }
                            Some((level, key, nonce)) => {
                                let (m_off, m_len) = info.ccm_encrypt_ranges();
                                let (a_off, m_off) =
                                    (radio::PSDU_OFFSET, radio::PSDU_OFFSET + m_off);

                                if self.aes_ccm.set_key(&key) != ReturnCode::SUCCESS
                                    || self.aes_ccm.set_nonce(&nonce) != ReturnCode::SUCCESS
                                {
                                    (TxState::Idle, (ReturnCode::FAIL, Some(buf)))
                                } else {
                                    let (res, opt_buf) = self.aes_ccm.crypt(
                                        buf,
                                        a_off,
                                        m_off,
                                        m_len,
                                        info.mic_len,
                                        level.encryption_needed(),
                                        true,
                                    );
                                    match res {
                                        ReturnCode::SUCCESS => {
                                            (TxState::Encrypting(info), (res, None))
                                        }
                                        ReturnCode::EBUSY => {
                                            let buf = match opt_buf {
                                                Some(buf) => buf,
                                                None => panic!("aes_ccm did not return the buffer"),
                                            };
                                            (
                                                TxState::ReadyToEncrypt(info, buf),
                                                (ReturnCode::SUCCESS, None),
                                            )
                                        }
                                        _ => (TxState::Idle, (res, opt_buf)),
                                    }
                                }
                            }
                        }
                    }
                    TxState::Encrypting(info) => {
                        // This state should be advanced only by the hardware
                        // encryption callback.
                        (TxState::Encrypting(info), (ReturnCode::SUCCESS, None))
                    }
                    TxState::ReadyToTransmit(info, buf) => {
                        let (rval, buf) = self.mac.transmit(buf, info.secured_length());
                        match rval {
                            // If the radio is busy, just wait for either a
                            // transmit_done or config_done callback to trigger
                            // this state transition again
                            ReturnCode::EBUSY => {
                                match buf {
                                    None => {
                                        // The radio forgot to return the buffer.
                                        (TxState::Idle, (ReturnCode::FAIL, None))
                                    }
                                    Some(buf) => (
                                        TxState::ReadyToTransmit(info, buf),
                                        (ReturnCode::SUCCESS, None),
                                    ),
                                }
                            }
                            _ => (TxState::Idle, (rval, buf)),
                        }
                    }
                };
                self.tx_state.replace(next_state);
                result
            })
    }

    /// Advances the reception pipeline if it can be advanced.
    fn step_receive_state(&self) {
        self.rx_state.take().map(|state| {
            let (next_state, buf) = match state {
                RxState::Idle => (RxState::Idle, None),
                RxState::ReadyToDecrypt(info, buf) => {
                    match info.security_params {
                        None => {
                            // `ReadyToDecrypt` should only be entered when
                            // `security_params` is not `None`.
                            (RxState::Idle, Some(buf))
                        }
                        Some((level, key, nonce)) => {
                            let (m_off, m_len) = info.ccm_encrypt_ranges();
                            let (a_off, m_off) = (radio::PSDU_OFFSET, radio::PSDU_OFFSET + m_off);

                            if self.aes_ccm.set_key(&key) != ReturnCode::SUCCESS
                                || self.aes_ccm.set_nonce(&nonce) != ReturnCode::SUCCESS
                            {
                                (RxState::Idle, Some(buf))
                            } else {
                                let (res, opt_buf) = self.aes_ccm.crypt(
                                    buf,
                                    a_off,
                                    m_off,
                                    m_len,
                                    info.mic_len,
                                    level.encryption_needed(),
                                    true,
                                );
                                match res {
                                    ReturnCode::SUCCESS => (RxState::Decrypting(info), None),
                                    ReturnCode::EBUSY => {
                                        let buf = match opt_buf {
                                            Some(buf) => buf,
                                            None => panic!("aes_ccm did not return the buffer"),
                                        };
                                        (RxState::ReadyToDecrypt(info, buf), None)
                                    }
                                    _ => (RxState::Idle, opt_buf),
                                }
                            }
                        }
                    }
                }
                RxState::Decrypting(info) => {
                    // This state should be advanced only by the hardware
                    // encryption callback.
                    (RxState::Decrypting(info), None)
                }
                RxState::ReadyToYield(info, buf) => {
                    // Between the secured and unsecured frames, the
                    // unsecured frame length remains constant but the data
                    // offsets may change due to the presence of PayloadIEs.
                    // Hence, we can only use the unsecured length from the
                    // frame info, but not the offsets.
                    let frame_len = info.unsecured_length();
                    if let Some((data_offset, (header, _))) =
                        Header::decode(&buf[radio::PSDU_OFFSET..], true).done()
                    {
                        // IEEE 802.15.4-2015 specifies that unsecured
                        // frames do not have auxiliary security headers,
                        // but we do not remove the auxiliary security
                        // header before returning the frame to the client.
                        // This is so that it is possible to tell if the
                        // frame was secured or unsecured, while still
                        // always receiving the frame payload in plaintext.
                        self.rx_client.map(|client| {
                            client.receive(
                                &buf,
                                header,
                                radio::PSDU_OFFSET + data_offset,
                                frame_len - data_offset,
                            );
                        });
                    }
                    (RxState::Idle, Some(buf))
                }
                RxState::ReadyToReturn(buf) => (RxState::Idle, Some(buf)),
            };
            self.rx_state.replace(next_state);

            // Return the buffer to the radio if we are done with it.
            if let Some(buf) = buf {
                self.mac.set_receive_buffer(buf);
            }
        });
    }
}

impl<M: Mac, A: AES128CCM<'a>> MacDevice<'a> for Framer<'a, M, A> {
    fn set_transmit_client(&self, client: &'a TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'a RxClient) {
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
        let src_addr_long = self.get_address_long();
        let security_desc = security_needed.and_then(|(level, key_id)| {
            self.lookup_key(level, key_id).map(|key| {
                // TODO: lookup frame counter for device
                let frame_counter = 0;
                let nonce = get_ccm_nonce(src_addr_long, frame_counter, level);
                (
                    Security {
                        level: level,
                        asn_in_nonce: false,
                        frame_counter: Some(frame_counter),
                        key_id: key_id,
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
            security: security,
            header_ies: Default::default(),
            header_ies_len: 0,
            payload_ies: Default::default(),
            payload_ies_len: 0,
        };

        match header.encode(&mut buf[radio::PSDU_OFFSET..], true).done() {
            Some((data_offset, mac_payload_offset)) => Ok(Frame {
                buf: buf,
                info: FrameInfo {
                    frame_type: FrameType::Data,
                    mac_payload_offset: mac_payload_offset,
                    data_offset: data_offset,
                    data_len: 0,
                    mic_len: mic_len,
                    security_params: security_desc.map(|(sec, key, nonce)| (sec.level, key, nonce)),
                },
            }),
            None => Err(buf),
        }
    }

    fn transmit(&self, frame: Frame) -> (ReturnCode, Option<&'static mut [u8]>) {
        let Frame { buf, info } = frame;
        let state = match self.tx_state.take() {
            None => {
                return (ReturnCode::FAIL, Some(buf));
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
                (ReturnCode::EBUSY, Some(buf))
            }
        }
    }
}

impl<M: Mac, A: AES128CCM<'a>> radio::TxClient for Framer<'a, M, A> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.data_sequence.set(self.data_sequence.get() + 1);
        self.tx_client.map(move |client| {
            client.send_done(buf, acked, result);
        });
    }
}

impl<M: Mac, A: AES128CCM<'a>> radio::RxClient for Framer<'a, M, A> {
    fn receive(&self, buf: &'static mut [u8], frame_len: usize, crc_valid: bool, _: ReturnCode) {
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
                    self.incoming_frame_security(buf, frame_len)
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
            self.step_receive_state();
        });
    }
}

impl<M: Mac, A: AES128CCM<'a>> radio::ConfigClient for Framer<'a, M, A> {
    fn config_done(&self, _: ReturnCode) {
        // The transmission pipeline is the only state machine that
        // waits for the configuration procedure to complete before
        // advancing.
        let (rval, buf) = self.step_transmit_state();
        if let Some(buf) = buf {
            // Return the buffer to the transmit client
            self.tx_client.map(move |client| {
                client.send_done(buf, false, rval);
            });
        }
    }
}

impl<M: Mac, A: AES128CCM<'a>> CCMClient for Framer<'a, M, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: ReturnCode, tag_is_valid: bool) {
        let mut tx_waiting = false;
        let mut rx_waiting = false;

        // The crypto operation was from the transmission pipeline.
        let opt_buf = if let Some(state) = self.tx_state.take() {
            match state {
                TxState::Encrypting(info) => {
                    let (rval, opt_buf) = if res != ReturnCode::SUCCESS {
                        self.tx_state.replace(TxState::Idle);
                        (res, Some(buf))
                    } else {
                        self.tx_state.replace(TxState::ReadyToTransmit(info, buf));
                        self.step_transmit_state()
                    };

                    if let Some(buf) = opt_buf {
                        // Abort the transmission process. Return the buffer to the client.
                        self.tx_client.map(move |client| {
                            client.send_done(buf, false, rval);
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
            self.rx_state.take().map(move |state| {
                match state {
                    RxState::Decrypting(info) => {
                        let next_state = if tag_is_valid {
                            RxState::ReadyToYield(info, buf)
                        } else {
                            RxState::ReadyToReturn(buf)
                        };
                        self.rx_state.replace(next_state);
                        self.step_receive_state();
                    }
                    other_state => {
                        rx_waiting = match other_state {
                            RxState::ReadyToDecrypt(_, _) => true,
                            _ => false,
                        };
                        self.rx_state.replace(other_state);
                    }
                };
            });
        }

        // Now trigger the next crypto operation if one exists.
        if tx_waiting {
            let (rval, opt_buf) = self.step_transmit_state();
            if let Some(buf) = opt_buf {
                // Return the buffer to the client.
                self.tx_client.map(move |client| {
                    client.send_done(buf, false, rval);
                });
            }
        } else if rx_waiting {
            self.step_receive_state();
        }
    }
}
