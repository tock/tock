//! Implements 802.15.4 MAC device functionality as an abstraction layer over
//! the raw radio transceiver hardware. The abstraction difference between a MAC
//! device and a raw radio transceiver is that the MAC devices exposes a
//! frame-oriented interface to its users, whereas the radio transceiver
//! transmits raw byte sequences. There is some abstraction breaking here,
//! though because the following are still implemented at the hardware level:
//! - CSMA-CA backoff
//! - FCS generation and verification
//!
//! TODO: Encryption/decryption
//! TODO: Sending beacon frames
//! TODO: Channel scanning

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use kernel::hil::radio;
use net::ieee802154::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct FrameInfo {
    // These offsets are relative to buf[radio::PSDU_OFFSET..] so that
    // the mac frame length is data_offset + data_len
    mac_payload_offset: usize,
    data_offset: usize,
    data_len: usize,
    security_params: Option<(Security, [u8; 16])>,
}

impl FrameInfo {
    pub fn frame_length(&self) -> usize {
        self.data_offset + self.data_len
    }

    pub fn remaining_data_capacity(&self, buf: &[u8]) -> usize {
        buf.len() - radio::PSDU_OFFSET - radio::MFR_SIZE - self.data_offset - self.data_len
    }

    pub fn append_payload(&mut self, buf: &mut [u8], payload: &[u8]) -> ReturnCode {
        if payload.len() > self.remaining_data_capacity(buf.as_ref()) {
            return ReturnCode::ENOMEM;
        }
        let begin = radio::PSDU_OFFSET + self.data_offset + self.data_len;
        buf[begin..begin + payload.len()].copy_from_slice(payload);
        self.data_len += payload.len();

        ReturnCode::SUCCESS
    }
}

pub trait Mac {
    fn get_address(&self) -> u16; //....... The local 16-bit address
    fn get_address_long(&self) -> [u8; 8]; // 64-bit address
    fn get_pan(&self) -> u16; //........... The 16-bit PAN ID
    fn get_channel(&self) -> u8;
    fn get_tx_power(&self) -> i8;

    fn set_address(&self, addr: u16);
    fn set_address_long(&self, addr: [u8; 8]);
    fn set_pan(&self, id: u16);
    fn set_channel(&self, chan: u8) -> ReturnCode;
    fn set_tx_power(&self, power: i8) -> ReturnCode;

    fn config_commit(&self) -> ReturnCode;

    fn is_on(&self) -> bool;
    fn prepare_data_frame(&self,
                          buf: &mut [u8],
                          dst_pan: PanID,
                          dst_addr: MacAddress,
                          src_pan: PanID,
                          src_addr: MacAddress,
                          security_needed: Option<(SecurityLevel, KeyId)>)
                          -> Result<FrameInfo, ()>;
    fn transmit(&self,
                buf: &'static mut [u8],
                frame_info: FrameInfo)
                -> (ReturnCode, Option<&'static mut [u8]>);
}

pub trait TxClient {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode);
}

pub trait RxClient {
    fn receive<'a>(&self,
                   buf: &'a [u8],
                   header: Header<'a>,
                   data_offset: usize,
                   data_len: usize,
                   result: ReturnCode);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum TxState {
    Idle,
    Encrypting,
    ReadyToTransmit,
}

pub struct MacDevice<'a, R: radio::Radio + 'a> {
    radio: &'a R,
    data_sequence: Cell<u8>,
    config_in_progress: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    tx_info: Cell<Option<FrameInfo>>,
    tx_state: Cell<TxState>,
    tx_client: Cell<Option<&'static TxClient>>,
    rx_client: Cell<Option<&'static RxClient>>,
}

impl<'a, R: radio::Radio + 'a> MacDevice<'a, R> {
    pub fn new(radio: &'a R) -> MacDevice<'a, R> {
        MacDevice {
            radio: radio,
            data_sequence: Cell::new(0),
            config_in_progress: Cell::new(false),
            tx_buf: TakeCell::empty(),
            tx_info: Cell::new(None),
            tx_state: Cell::new(TxState::Idle),
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

    fn lookup_key(&self, level: SecurityLevel, key_id: KeyId) -> Option<(Security, [u8; 16])> {
        let security = Security {
            level: level,
            asn_in_nonce: false,
            frame_counter: None,
            key_id: key_id,
        };
        Some((security, [0; 16]))
    }

    fn step_transmit_state(&self) -> (ReturnCode, Option<&'static mut [u8]>) {
        match self.tx_state.get() {
            TxState::Idle => (ReturnCode::SUCCESS, None),
            TxState::Encrypting => {
                // let frame_info = self.tx_info.get().unwrap();
                let buf = self.tx_buf.take().unwrap();
                // TODO: implement encryption
                self.tx_state.set(TxState::Idle);
                (ReturnCode::ENOSUPPORT, Some(buf))
            }
            TxState::ReadyToTransmit => {
                if self.config_in_progress.get() {
                    // We will continue when the configuration is done.
                    (ReturnCode::SUCCESS, None)
                } else {
                    let frame_info = self.tx_info.get().unwrap();
                    let buf = self.tx_buf.take().unwrap();
                    self.tx_state.set(TxState::Idle);
                    self.radio.transmit(buf, frame_info.frame_length())
                }
            }
        }
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
        let rval = if !self.config_in_progress.get() {
            self.radio.config_commit()
        } else {
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
                          buf: &mut [u8],
                          dst_pan: PanID,
                          dst_addr: MacAddress,
                          src_pan: PanID,
                          src_addr: MacAddress,
                          security_needed: Option<(SecurityLevel, KeyId)>)
                          -> Result<FrameInfo, ()> {
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

        header.encode(&mut buf[radio::PSDU_OFFSET..], true)
            .done()
            .map(|(data_offset, mac_payload_offset)| {
                FrameInfo {
                    mac_payload_offset: mac_payload_offset,
                    data_offset: data_offset,
                    data_len: 0,
                    security_params: security_params,
                }
            })
            .ok_or(())
    }

    fn transmit(&self,
                buf: &'static mut [u8],
                frame_info: FrameInfo)
                -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.tx_state.get() != TxState::Idle {
            return (ReturnCode::EBUSY, Some(buf));
        }

        self.tx_buf.replace(buf);
        self.tx_info.set(Some(frame_info));
        match frame_info.security_params {
            Some(_) => self.tx_state.set(TxState::Encrypting),
            None => self.tx_state.set(TxState::ReadyToTransmit),
        }
        self.step_transmit_state()
    }
}

impl<'a, R: radio::Radio + 'a> radio::TxClient for MacDevice<'a, R> {
    fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.data_sequence.set(self.data_sequence.get() + 1);
        self.tx_info.set(None);
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
            let (rval, buf) = self.step_transmit_state();
            if let Some(buf) = buf {
                // Return the buffer to the transmit client
                self.tx_client.get().map(move |client| { client.send_done(buf, false, rval); });
            }
        }
    }
}
