//! Implements IEEE 802.15.4 MAC device abstraction over a raw 802.15.4 radio.
//! Allows its users to prepare and send frames in plaintext, handling 802.15.4
//! encoding and security procedures (in the future) transparently.
//!
//! However, certain IEEE 802.15.4 MAC device concepts are not implemented in
//! this layer of abstraction and instead handled in hardware for performance
//! purposes. These include CSMA-CA backoff, FCS generation and authentication,
//! and automatic acknowledgement.
//!
//! TODO: Encryption/decryption
//! TODO: Sending beacon frames
//! TODO: Channel scanning
//!
//! Usage
//! -----
//!
//! To use this capsule, we need an implementation of a hardware
//! `kernel::hil::radio::Radio`. Suppose we have such an implementation of type
//! `RF233Device`.
//!
//! ```rust
//! let radio: RF233Device = /* ... */;
//! let radio_mac = static_init!(
//!     capsules::mac::MacDevice<'static, RF233Device>,
//!     capsules::mac::MacDevice::new(radio));
//! rf233.set_transmit_client(radio_mac);
//! rf233.set_receive_client(radio_mac, &mut RF233_RX_BUF);
//! rf233.set_config_client(radio_mac);
//! ```
//!
//! The `radio_mac` device is now set up. Users of the MAC device can now
//! configure the underlying radio, prepare and send frames:
//! ```rust
//! radio_mac.set_pan(0xABCD);
//! radio_mac.set_address(0x1008);
//! radio_mac.config_commit();
//!
//! let frame = radio_mac
//!     .prepare_data_frame(&mut STATIC_BUFFER,
//!                         0xABCD, MacAddress::Short(0x1008),
//!                         0xABCD, MacAddress::Short(0x1009),
//!                         None)
//!     .ok()
//!     .map(|frame| {
//!         let rval = frame.append_payload(&mut SOME_DATA[..10]);
//!         if rval == ReturnCode::SUCCESS {
//!             let (rval, _) = radio_mac.transmit(frame);
//!             rval
//!         } else {
//!             rval
//!         }
//!     });
//! ```
//!
//! You should also be able to set up the userspace driver for receiving/sending
//! 802.15.4 frames:
//! ```rust
//! let radio_capsule = static_init!(
//!     capsules::radio::RadioDriver<'static,
//!                                  capsules::mac::MacDevice<'static, RF233Device>>,
//!     capsules::radio::RadioDriver::new(radio_mac));
//! radio_capsule.config_buffer(&mut RADIO_BUF);
//! radio_mac.set_transmit_client(radio_capsule);
//! radio_mac.set_receive_client(radio_capsule);
//! ```

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::MapCell;
use kernel::hil::radio;
use net::ieee802154::*;

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
/// 1. How to encode it once its payload has been finalized
/// 2. The sizes of the mac header, payload and MIC tag length to be added
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct FrameInfo {
    // These offsets are relative to the PSDU or buf[radio::PSDU_OFFSET..] so
    // that the mac frame length is data_offset + data_len
    mac_payload_offset: usize,
    data_offset: usize,
    data_len: usize,
    security_params: Option<(Security, [u8; 16])>,
}

impl Frame {
    /// Consumes the frame and retrieves the buffer it wraps
    pub fn into_buf(self) -> &'static mut [u8] {
        self.buf
    }

    /// Calculates how much more data this frame can hold
    pub fn remaining_data_capacity(&self) -> usize {
        self.buf.len() - radio::PSDU_OFFSET - radio::MFR_SIZE - self.info.length()
    }

    /// Appends payload bytes into the frame if possible
    pub fn append_payload(&mut self, payload: &[u8]) -> ReturnCode {
        if payload.len() > self.remaining_data_capacity() {
            return ReturnCode::ENOMEM;
        }
        let begin = radio::PSDU_OFFSET + self.info.length();
        self.buf[begin..begin + payload.len()].copy_from_slice(payload);
        self.info.data_len += payload.len();

        ReturnCode::SUCCESS
    }
}

impl FrameInfo {
    /// Current size of the frame, not including the MAC footer that is added by
    /// the hardware when it generates the CRC
    pub fn length(&self) -> usize {
        self.data_offset + self.data_len
    }
}

/// The contract satisfied by an implementation of an IEEE 802.15.4 MAC device.
/// Any IEEE 802.15.4 MAC device should expose the following high-level
/// functionality:
///
/// - Configuration of addresses and transmit power
/// - Preparing frames (data frame, command frames, beacon frames)
/// - Transmitting and receiving frames
///
/// Outlining this in a trait allows other implementations of MAC devices that
/// divide the responsibilities of software and hardware differently. For
/// example, a radio chip might be able to completely inline the frame security
/// procedure in hardware, as opposed to requiring a software implementation.
pub trait Mac {
    /// The short 16-bit address of the MAC device
    fn get_address(&self) -> u16;
    /// The long 64-bit address (EUI-64) of the MAC device
    fn get_address_long(&self) -> [u8; 8];
    /// The 16-bit PAN ID of the MAC device
    fn get_pan(&self) -> u16;
    /// The 802.15.4 channel ID of the MAC device
    fn get_channel(&self) -> u8;
    /// The transmission power of the MAC device, in dBm
    fn get_tx_power(&self) -> i8;

    /// Set the short 16-bit address of the MAC device
    fn set_address(&self, addr: u16);
    /// Set the long 64-bit address (EUI-64) of the MAC device
    fn set_address_long(&self, addr: [u8; 8]);
    /// Set the 16-bit PAN ID of the MAC device
    fn set_pan(&self, id: u16);
    /// Set the 802.15.4 channel of the MAC device. `channel` should be a valid
    /// channel `11 <= channel <= 26`, otherwise EINVAL will be returned
    fn set_channel(&self, chan: u8) -> ReturnCode;
    /// Set the transmission power of the MAC device, in dBm. `power` should
    /// satisfy `-17 <= power <= 4`, otherwise EINVAL will be returned
    fn set_tx_power(&self, power: i8) -> ReturnCode;

    /// This method must be called after one or more calls to `set_*`. If
    /// `set_*` is called without calling `config_commit`, there is no guarantee
    /// that the underlying hardware configuration (addresses, pan ID) is in
    /// line with this MAC device implementation.
    fn config_commit(&self) -> ReturnCode;

    /// Returns if the MAC device is currently on.
    fn is_on(&self) -> bool;

    /// Prepares a mutable buffer slice as an 802.15.4 frame by writing the appropriate
    /// header bytes into the buffer. This needs to be done before adding the
    /// payload because the length of the header is not fixed.
    ///
    /// - `buf`: The mutable buffer slice to use
    /// - `dst_pan`: The destination PAN ID
    /// - `dst_addr`: The destination MAC address
    /// - `src_pan`: The source PAN ID
    /// - `src_addr`: The source MAC address
    /// - `security_needed`: Whether or not this frame should be secured. This
    /// needs to be specified beforehand so that the auxiliary security header
    /// can be pre-inserted.
    ///
    /// Returns either a Frame that is ready to have payload appended to it, or
    /// the mutable buffer if the frame cannot be prepared for any reason
    fn prepare_data_frame(&self,
                          buf: &'static mut [u8],
                          dst_pan: PanID,
                          dst_addr: MacAddress,
                          src_pan: PanID,
                          src_addr: MacAddress,
                          security_needed: Option<(SecurityLevel, KeyId)>)
                          -> Result<Frame, &'static mut [u8]>;

    /// Transmits a frame that has been prepared by the above process. If the
    /// transmission process fails, the buffer inside the frame is returned so
    /// that it can be re-used.
    fn transmit(&self, frame: Frame) -> (ReturnCode, Option<&'static mut [u8]>);
}

/// Trait to be implemented by any user of the IEEE 802.15.4 device that
/// transmits frames. Contains a callback through which the static mutable
/// reference to the frame buffer is returned to the client.
pub trait TxClient {
    /// When transmission is complete or fails, return the buffer used for
    /// transmission to the client. `result` indicates whether or not
    /// the transmission was successful.
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode);
}

/// Trait to be implemented by users of the IEEE 802.15.4 device that wish to
/// receive frames. The callback is triggered whenever a valid frame is
/// received, verified and unsecured (via the IEEE 802.15.4 security procedure)
/// successfully.
pub trait RxClient {
    /// When a frame is received, this callback is triggered. The client only
    /// receives an immutable borrow of the buffer. Only completely valid,
    /// unsecured frames that have passed the incoming security procedure are
    /// exposed to the client.
    fn receive<'a>(&self,
                   buf: &'a [u8],
                   header: Header<'a>,
                   data_offset: usize,
                   data_len: usize,
                   result: ReturnCode);
}

/// This state enum describes the state of the transmission pipeline.
/// Conditionally-present state is also included as fields in the enum variants.
/// We can view the transmission process as a state machine driven by the
/// following events:
/// - calls to `Mac#transmit`
/// - `send_done` callbacks from the underlying radio
/// - `config_done` callbacks from the underlying radio (if, for example,
/// configuration was in progress when a transmission was requested)
/// - TODO: callbacks from the encryption facility
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

/// This struct wraps an IEEE 802.15.4 radio device `kernel::hil::radio::Radio`
/// and exposes IEEE 802.15.4 MAC device functionality as the trait
/// `capsules::mac::Mac`. It hides header preparation, transmission and
/// processing logic from the user by essentially maintaining multiple state
/// machines corresponding to the transmission, reception and
/// encryption/decryption pipelines. See the documentation in
/// `capsules/src/mac.rs` for more details.
pub struct MacDevice<'a, R: radio::Radio + 'a> {
    radio: &'a R,
    data_sequence: Cell<u8>,
    config_needed: Cell<bool>,
    config_in_progress: Cell<bool>,

    /// Transmision pipeline state. This should never be `None`, except when
    /// transitioning between states. That is, any method that consumes the
    /// current state should always remember to replace it along with the
    /// associated state information.
    tx_state: MapCell<TxState>,
    tx_client: Cell<Option<&'static TxClient>>,

    /// Currently, the receive pathway is stateless, but once
    /// encryption/decryption is implemented, there will be a similar mechanism
    /// to keep track of the state of the reception pipeline.
    rx_client: Cell<Option<&'static RxClient>>,
}

impl<'a, R: radio::Radio + 'a> MacDevice<'a, R> {
    pub fn new(radio: &'a R) -> MacDevice<'a, R> {
        MacDevice {
            radio: radio,
            data_sequence: Cell::new(0),
            config_needed: Cell::new(false),
            config_in_progress: Cell::new(false),
            tx_state: MapCell::new(TxState::Idle),
            tx_client: Cell::new(None),
            rx_client: Cell::new(None),
        }
    }

    pub fn set_transmit_client(&self, client: &'static TxClient) {
        self.tx_client.set(Some(client));
    }

    pub fn set_receive_client(&self, client: &'static RxClient) {
        self.rx_client.set(Some(client));
    }

    /// Looks up the key from the available key descriptors.
    /// TODO: implement a mechanism for an upper layer to provide these keys.
    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<(Security, [u8; 16])> {
        let security = Security {
            level: level,
            asn_in_nonce: false,
            frame_counter: None,
            key_id: key_id,
        };
        Some((security, [0; 16]))
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
                    TxState::ReadyToEncrypt(_info, buf) => {
                        // TODO: implement encryption
                        (TxState::Idle, (ReturnCode::ENOSUPPORT, Some(buf)))
                    }
                    TxState::Encrypting(info) => {
                        // This state should be advanced only by the hardware
                        // encryption callback.
                        (TxState::Encrypting(info), (ReturnCode::SUCCESS, None))
                    }
                    TxState::ReadyToTransmit(info, buf) => {
                        if self.config_in_progress.get() {
                            // We will continue when the configuration is done.
                            (TxState::Idle, (ReturnCode::SUCCESS, None))
                        } else {
                            (TxState::Idle, self.radio.transmit(buf, info.length()))
                        }
                    }
                };
                self.tx_state.replace(next_state);
                result
            })
    }
}

impl<'a, R: radio::Radio + 'a> Mac for MacDevice<'a, R> {
    fn get_address(&self) -> u16 {
        self.radio.get_address()
    }

    fn get_address_long(&self) -> [u8; 8] {
        self.radio.get_address_long()
    }

    fn get_pan(&self) -> u16 {
        self.radio.get_pan()
    }

    fn get_channel(&self) -> u8 {
        self.radio.get_channel()
    }

    fn get_tx_power(&self) -> i8 {
        self.radio.get_tx_power()
    }

    fn set_address(&self, addr: u16) {
        self.radio.set_address(addr)
    }

    fn set_address_long(&self, addr: [u8; 8]) {
        self.radio.set_address_long(addr)
    }

    fn set_pan(&self, id: u16) {
        self.radio.set_pan(id)
    }

    fn set_channel(&self, chan: u8) -> ReturnCode {
        self.radio.set_channel(chan)
    }

    fn set_tx_power(&self, power: i8) -> ReturnCode {
        self.radio.set_tx_power(power)
    }

    fn config_commit(&self) -> ReturnCode {
        // If no configuration is in progress, begin configuration.  Otherwise,
        // ensure that once the current configuration process completes, it is
        // re-triggered. This is to ensure that if the user has made any changes
        // to the MAC device parameters between the two `config_commit` calls,
        // they are eventually propagated to the hardware.
        let rval = if !self.config_in_progress.get() {
            self.config_needed.set(false);
            self.radio.config_commit()
        } else {
            self.config_needed.set(true);
            ReturnCode::EBUSY
        };
        if rval == ReturnCode::SUCCESS {
            self.config_in_progress.set(true)
        }
        rval
    }

    fn is_on(&self) -> bool {
        self.radio.is_on()
    }

    fn prepare_data_frame(&self,
                          buf: &'static mut [u8],
                          dst_pan: PanID,
                          dst_addr: MacAddress,
                          src_pan: PanID,
                          src_addr: MacAddress,
                          security_needed: Option<(SecurityLevel, KeyId)>)
                          -> Result<Frame, &'static mut [u8]> {
        let security_params =
            security_needed.and_then(|(level, key_id)| self.lookup_key(level, key_id));

        // Construct MAC header
        let header = Header {
            frame_type: FrameType::Data,
            /* TODO: determine this by looking at queue */
            frame_pending: false,
            // Unicast data frames request acknowledgement
            ack_requested: true,
            version: FrameVersion::V2015,
            seq: Some(self.data_sequence.get()),
            dst_pan: Some(dst_pan),
            dst_addr: Some(dst_addr),
            src_pan: Some(src_pan),
            src_addr: Some(src_addr),
            security: security_params.map(|(sec, _)| sec),
            header_ies: Default::default(),
            header_ies_len: 0,
            payload_ies: Default::default(),
            payload_ies_len: 0,
        };

        match header.encode(&mut buf[radio::PSDU_OFFSET..], true).done() {
            Some((data_offset, mac_payload_offset)) => {
                Ok(Frame {
                    buf: buf,
                    info: FrameInfo {
                        mac_payload_offset: mac_payload_offset,
                        data_offset: data_offset,
                        data_len: 0,
                        security_params: security_params,
                    },
                })
            }
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
                if info.security_params.is_some() {
                    self.tx_state.replace(TxState::ReadyToEncrypt(info, buf));
                } else {
                    self.tx_state.replace(TxState::ReadyToTransmit(info, buf));
                }
                self.step_transmit_state()
            }
            other_state => {
                self.tx_state.replace(other_state);
                (ReturnCode::EBUSY, Some(buf))
            }
        }
    }
}

impl<'a, R: radio::Radio + 'a> radio::TxClient for MacDevice<'a, R> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.data_sequence.set(self.data_sequence.get() + 1);
        self.tx_client.get().map(move |client| { client.send_done(buf, acked, result); });
    }
}

impl<'a, R: radio::Radio + 'a> radio::RxClient for MacDevice<'a, R> {
    fn receive(&self, buf: &'static mut [u8], frame_len: usize, crc_valid: bool, _: ReturnCode) {
        // Drop all frames with invalid CRC
        if !crc_valid {
            self.radio.set_receive_buffer(buf);
            return;
        }

        // Try to read the MAC headers of the frame to determine if decryption is
        // needed. Otherwise, dispatch the parsed headers directly to the client
        let decrypt = if let Some((data_offset, (header, _))) =
            Header::decode(&buf[radio::PSDU_OFFSET..]).done() {
            // 802.15.4 Incoming frame security procedure
            let buf_data_offset = radio::PSDU_OFFSET + data_offset;
            let data_len = frame_len - data_offset;
            if let Some(security) = header.security {
                if header.version == FrameVersion::V2003 || security.level == SecurityLevel::None {
                    // Version must not be 2003 (legacy) and the security level must
                    // not be none, otherwise incoming security is undefined.
                    // Hence, we drop the frame
                    false
                } else {
                    // TODO: Implement decryption
                    self.rx_client.get().map(|client| {
                        client.receive(&buf,
                                       header,
                                       buf_data_offset,
                                       data_len,
                                       ReturnCode::ENOSUPPORT);
                    });
                    false
                }
            } else {
                // No security needed, can yield the frame immediately
                self.rx_client.get().map(|client| {
                    client.receive(&buf,
                                   header,
                                   buf_data_offset,
                                   data_len,
                                   ReturnCode::ENOSUPPORT);
                });
                false
            }
        } else {
            false
        };

        // If decryption is needed, we begin the decryption process, otherwise,
        // we can return the buffer immediately to the radio.
        if decrypt {
            // TODO: Implement decryption
            self.radio.set_receive_buffer(buf);
        } else {
            self.radio.set_receive_buffer(buf);
        }
    }
}

impl<'a, R: radio::Radio + 'a> radio::ConfigClient for MacDevice<'a, R> {
    fn config_done(&self, _: ReturnCode) {
        if self.config_in_progress.get() {
            self.config_in_progress.set(false);

            if self.config_needed.get() {
                // If the user called config_commit while a configuration was in
                // progress, there is no guarantee that the new information was
                // transmitted to the hardware, so we need to call config_commit
                // to start the process again.
                self.config_commit();
            } else {
                let (rval, buf) = self.step_transmit_state();
                if let Some(buf) = buf {
                    // Return the buffer to the transmit client
                    self.tx_client.get().map(move |client| { client.send_done(buf, false, rval); });
                }
            }
        }
    }
}
