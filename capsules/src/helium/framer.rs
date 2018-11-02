use cauterize::{Cauterize, Encoder, Vector};
use core::cell::Cell;
use helium::{device, virtual_rfcore};
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::hil::rfcore;
use kernel::ReturnCode;
use msg;

/// A `Frame` wraps a static mutable byte slice and keeps just enough
/// information about its header contents to expose a restricted interface for
/// modifying its payload. This enables the user to abdicate any concerns about
/// where the payload should be placed in the buffer.
#[derive(Eq, PartialEq, Debug)]
pub struct Frame {
    info: FrameInfo,
    buf: &'static mut [u8],
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Header {
    pub id: u16,
    pub address: [u8; 10],
    pub seq: u8,
    pub data_len: usize,
}

impl Frame {
    pub fn into_buf(self) -> &'static mut [u8] {
        self.buf
    }

    pub fn append_payload(&mut self, payload: &[u8]) -> ReturnCode {
        if payload.len() > 200 {
            return ReturnCode::ENOMEM;
        }

        for (i, c) in payload.as_ref()[0..payload.len()].iter().enumerate() {
            self.buf[i] = *c;
        }

        // self.buf.copy_from_slice(payload);
        self.info.header.data_len = payload.len();
        ReturnCode::SUCCESS
    }

    pub fn cauterize_payload(&mut self, payload: &[u8]) -> ReturnCode {
        if payload.len() > 180 {
            return ReturnCode::ENOMEM;
        } else {
            self.info.header.data_len = payload.len();
        }

        let mut pkt = msg::Payload::new();

        for elem in payload.iter() {
            pkt.push(*elem);
        }

        let ping = msg::Ping {
            id: self.info.header.id,
            address: msg::Addr(self.info.header.address),
            seq: self.info.header.seq,
            len: self.info.header.data_len as u32,
            data: pkt,
        };

        let pingpong = msg::Pingpong::Ping(ping);

        let mut ectx = Encoder::new(&mut self.buf);

        pingpong
            .encode(&mut ectx)
            .map_err(|e| debug!("Cauterize Error: {:?}", e))
            .ok();

        self.info.header.data_len = ectx.consume();
        debug!("Data len: {:?}", self.info.header.data_len);

        if self.info.header.data_len > 200 {
            return ReturnCode::ENOMEM;
        }

        ReturnCode::SUCCESS
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct FrameInfo {
    pub header: Header,

    caut_type: Option<CauterizeType>,
}

pub const CAUT_TYPE_MASK: u8 = 0b111;
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CauterizeType {
    None = 0b00,
    Standard = 0b001,
    Custom = 0b010,
}

impl CauterizeType {
    pub fn from_slice(ct: u8) -> Option<CauterizeType> {
        match ct & CAUT_TYPE_MASK {
            0b00 => Some(CauterizeType::None),
            0b01 => Some(CauterizeType::Standard),
            0b10 => Some(CauterizeType::Custom),
            _ => None,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum TxState {
    Idle,
    ReadyToEncode(FrameInfo, &'static mut [u8]),
    Encoding(FrameInfo),
    ReadyToTransmit(FrameInfo, &'static mut [u8]),
}

#[allow(unused)]
#[derive(Eq, PartialEq, Debug)]
enum RxState {
    // No received frame
    Idle,
    // Frame is ready to be re-parsed and exposed to client
    ReadyToYield(FrameInfo, &'static mut [u8]),
    // Frame buffer is ready to attempt FEC decode
    ReadyToDecode(FrameInfo, &'static mut [u8]),
    // Buffer with frame is ready to be returned to radio
    ReadyToReturn(&'static mut [u8]),
}

pub struct Framer<'a, D: virtual_rfcore::RFCore> {
    radio_device: &'a D,
    seq: Cell<u8>,
    address: Cell<[u8; 10]>,
    tx_state: MapCell<TxState>,
    tx_client: OptionalCell<&'a device::TxClient>,
    rx_state: MapCell<RxState>,
    rx_client: OptionalCell<&'a device::RxClient>,
}

impl<D: virtual_rfcore::RFCore> Framer<'a, D> {
    pub fn new(radio_device: &'a D) -> Framer<'a, D> {
        Framer {
            radio_device,
            seq: Cell::new(0),
            address: Cell::new([0; 10]),
            tx_state: MapCell::new(TxState::Idle),
            tx_client: OptionalCell::empty(),
            rx_state: MapCell::new(RxState::Idle),
            rx_client: OptionalCell::empty(),
        }
    }

    fn outgoing_frame(&self, buf: &'static mut [u8], frame_info: FrameInfo) -> TxState {
        match frame_info.caut_type {
            Some(CauterizeType::None) => TxState::ReadyToTransmit(frame_info, buf),
            Some(CauterizeType::Custom) => TxState::ReadyToEncode(frame_info, buf),
            Some(CauterizeType::Standard) => TxState::ReadyToTransmit(frame_info, buf),
            None => TxState::ReadyToTransmit(frame_info, buf),
        }
    }

    fn incoming_frame(&self, buf: &'static mut [u8], frame_len: usize) -> RxState {
        // TODO Attempt LDPC decode methods and see if decoding is possible.
        // There are three resulting scenarios here:
        // 1. The receive buffer decodes correctly and the buffer can be returned
        // 2. The receive buffer does not decode correctly and the radio continues to receive and
        //    retry decoding.
        // 3. The receive buffer never decodes and the buffer needs to be flushed and start overi
        let temp_data_offset: usize = 0;
        self.rx_client.map(|client| {
            client.receive_event(&buf, temp_data_offset, frame_len);
        });
        RxState::ReadyToReturn(buf)
    }

    pub fn step_transmit_state(&self) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.tx_state
            .take()
            .map_or((ReturnCode::FAIL, None), |state| {
                let (next_state, result) = match state {
                    TxState::Idle => (TxState::Idle, (ReturnCode::SUCCESS, None)),
                    TxState::ReadyToTransmit(info, buf) => {
                        let (rval, buf) = self.radio_device.transmit(buf, info);
                        match rval {
                            ReturnCode::EBUSY => match buf {
                                None => (TxState::Idle, (ReturnCode::FAIL, None)),
                                Some(buf) => (
                                    TxState::ReadyToTransmit(info, buf),
                                    (ReturnCode::SUCCESS, None),
                                ),
                            },
                            _ => (TxState::Idle, (rval, buf)),
                        }
                    }
                    TxState::ReadyToEncode(info, buf) => {
                        (TxState::Encoding(info), (ReturnCode::SUCCESS, Some(buf)))
                    }
                    TxState::Encoding(info) => {
                        // TODO Should be advanced by encoding callback
                        (TxState::Encoding(info), (ReturnCode::SUCCESS, None))
                    }
                };
                self.tx_state.replace(next_state);
                result
            })
    }

    pub fn step_receive_state(&self) {
        self.rx_state.take().map(|state| {
            let (next_state, buf) = match state {
                RxState::Idle => (RxState::Idle, None),
                RxState::ReadyToDecode(info, buf) => {
                    match info.caut_type {
                        Some(CauterizeType::None) => (RxState::Idle, Some(buf)),
                        Some(CauterizeType::Standard) => {
                            // Do decode for LDPC TC128 here then return success for fail
                            (RxState::Idle, Some(buf))
                        }
                        Some(CauterizeType::Custom) => {
                            // Same as above
                            (RxState::Idle, Some(buf))
                        }
                        _ => (RxState::Idle, Some(buf)),
                    }
                }
                RxState::ReadyToYield(info, buf) => {
                    let _frame_len = info.header.data_len;
                    // Extract data - headers from frame here and return if success
                    (RxState::Idle, Some(buf))
                }
                RxState::ReadyToReturn(buf) => (RxState::Idle, Some(buf)),
            };
            self.rx_state.replace(next_state);

            if let Some(buf) = buf {
                self.radio_device.set_receive_buffer(buf);
            }
        });
    }
}

impl<D: virtual_rfcore::RFCore> device::Device<'a> for Framer<'a, D> {
    fn initialize(&self) -> ReturnCode {
        self.radio_device.initialize()
    }

    fn set_transmit_client(&self, client: &'a device::TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'a device::RxClient) {
        self.rx_client.set(client);
    }

    fn send_stop_command(&self) -> ReturnCode {
        self.radio_device.send_stop_command()
    }

    fn send_kill_command(&self) -> ReturnCode {
        self.radio_device.send_kill_command()
    }

    fn set_device_config(&self) -> ReturnCode {
        self.radio_device.config_commit()
    }

    fn is_on(&self) -> bool {
        self.radio_device.get_radio_status()
    }

    fn set_address_long(&self, address: [u8; 10]) {
        debug!("address set: {:?}", address);
        self.address.set(address);
    }

    fn prepare_data_frame(
        &self,
        buf: &'static mut [u8],
        seq: u8,
        id: u16,
        caut_type: Option<CauterizeType>,
    ) -> Result<Frame, &'static mut [u8]> {
        let header = Header {
            id: id,
            address: self.address.get(),
            seq: seq, //Some(self.seq.get()),
            data_len: buf.len(),
        };

        // encode header here and return some result
        let frame = Frame {
            info: FrameInfo {
                header: header,
                caut_type: caut_type,
            },
            buf: buf,
        };
        Ok(frame)
    }

    fn transmit(&self, frame: Frame) -> (ReturnCode, Option<&'static mut [u8]>) {
        let Frame { info, buf } = frame;
        let state = match self.tx_state.take() {
            None => {
                return (ReturnCode::FAIL, Some(buf));
            }
            Some(state) => state,
        };
        match state {
            TxState::Idle => {
                let next_state = self.outgoing_frame(buf, info);
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

impl<D: virtual_rfcore::RFCore> rfcore::TxClient for Framer<'a, D> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.seq.set(self.seq.get() + 1);
        self.tx_client.map(move |client| {
            client.transmit_event(buf, result);
        });
    }
}

impl<D: virtual_rfcore::RFCore> rfcore::RxClient for Framer<'a, D> {
    fn receive_event(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        _result: ReturnCode,
    ) {
        if !crc_valid {
            self.radio_device.set_receive_buffer(buf);
            return;
        }

        self.rx_state.take().map(move |state| {
            let next_state = match state {
                RxState::Idle => {
                    // Process new frame if RX pipeline is free
                    self.incoming_frame(buf, frame_len)
                }
                other_state => {
                    // Should never occur unless a receive buffer was provided by something outside
                    // of virtual radio layer
                    self.radio_device.set_receive_buffer(buf);
                    other_state
                }
            };
            self.rx_state.replace(next_state);
            self.step_receive_state();
        });
    }
}

impl<D: virtual_rfcore::RFCore> rfcore::ConfigClient for Framer<'a, D> {
    fn config_event(&self, _: ReturnCode) {
        let (rval, buf) = self.step_transmit_state();
        if let Some(buf) = buf {
            // Return buf to tx client
            self.tx_client.map(move |client| {
                client.transmit_event(buf, rval);
            });
        }
    }
}
