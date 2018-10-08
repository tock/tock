use core::cell::Cell;
use helium::{virtual_rfcore, device};
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::hil::radio_client;
use kernel::ReturnCode;
use msg;

/// A `Frame` wraps a static mutable byte slice and keeps just enough
/// information about its header contents to expose a restricted interface for
/// modifying its payload. This enables the user to abdicate any concerns about
/// where the payload should be placed in the buffer.
#[derive(Eq, PartialEq, Debug)]
pub struct Frame {
    buf: &'static mut [u8],
    info: FrameInfo,
    max_frame_size: usize,
}

pub struct Header {
    frame_type: FrameType,
    id: Option<u32>,
    seq: Option<u8>,
}
impl Frame {

    pub fn into_buf(self) -> &'static mut [u8] {
        self.buf
    }

    pub fn append_payload(&mut self, payload: &[u8]) -> ReturnCode {
        if payload.len() > self.max_frame_size {
            return ReturnCode::ENOMEM;
        }
        self.buf.copy_from_slice(payload);
        self.info.data_len += payload.len();
        
        ReturnCode::SUCCESS
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct FrameInfo {
    frame_type: FrameType,

    fec_type: Option<FecType>,

    data_len: usize,
}

pub const FEC_TYPE_MASK: u8 = 0b111;
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FecType {
    None = 0b00,
    LdpcTc128 = 0b001,
    LdpcTc256 = 0b010,
    LdpcTc512 = 0b011,
}

impl FecType {
    pub fn from_slice(ft: u8) -> Option<FecType> {
        match ft & FEC_TYPE_MASK {
            0b00 => Some(FecType::None),
            0b01 => Some(FecType::LdpcTc128),
            0b10 => Some(FecType::LdpcTc256),
            0b11 => Some(FecType::LdpcTc512),
            _ => None,
        }
    }
}

pub const FRAME_TYPE_MASK: u16 = 0b111;
#[repr(u16)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FrameType {
    // Reserved = 0b100,
    Beacon = 0b000,
    Data = 0b001,
    Acknowledgement = 0b010,
    MACCommand = 0b011,
    Multipurpose = 0b101,
    Fragment = 0b110,
    Extended = 0b111,
}

impl FrameType {
    pub fn from_slice(ft: u16) -> Option<FrameType> {
        match ft & FRAME_TYPE_MASK {
            0b000 => Some(FrameType::Beacon),
            0b001 => Some(FrameType::Data),
            0b010 => Some(FrameType::Acknowledgement),
            0b011 => Some(FrameType::MACCommand),
            0b101 => Some(FrameType::Multipurpose),
            0b110 => Some(FrameType::Fragment),
            0b111 => Some(FrameType::Extended),
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

pub struct Framer<'a, V> 
where
    V: virtual_rfcore::RFCore,
{
    vrfc: &'a V,
    seq: Cell<u8>,
    tx_state: MapCell<TxState>,
    tx_client: OptionalCell<&'a device::TxClient>,
    rx_state: MapCell<RxState>,
    rx_client: OptionalCell<&'a device::RxClient>,
}

impl<V> Framer<'a, V> 
where 
    V: virtual_rfcore::RFCore,
{
    pub fn new(vrfc: &'a V) -> Framer<'a, V> {
        Framer {
            vrfc: vrfc,
            seq: Cell::new(0),
            tx_state: MapCell::new(TxState::Idle),
            tx_client: OptionalCell::empty(),
            rx_state: MapCell::new(RxState::Idle),
            rx_client: OptionalCell::empty(),
        }
    }

    fn outgoing_frame(&self, buf: &'static mut [u8], frame_info: FrameInfo) -> TxState {
        match frame_info.fec_type {
            Some(_) => TxState::ReadyToEncode(frame_info, buf),
            None => TxState::ReadyToTransmit(frame_info, buf),
        }
    }

    fn incoming_frame(&self, buf: &'static mut [u8], frame_len: usize) -> RxState {
        // TODO Attempt LDPC decode methods and see if decoding is possible.
        // There are three resulting scenarios here:
        // 1. The receive buffer decodes correctly and the buffer can be returned
        // 2. The receive buffer does not decode correctly and the radio continues to receive and
        //    retry decoding.
        // 3. The receive buffer never decodes and the buffer needs to be flushed and start over
        self.rx_client.map(|client| {
            client.receive_event(&buf, frame_len);
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
                        let (rval, buf) = self.vrfc.transmit(buf, info.data_len);
                        match rval {
                            ReturnCode::EBUSY => {
                                match buf {
                                    None => (TxState::Idle, (ReturnCode::FAIL, None)),
                                    Some(buf) => (TxState::ReadyToTransmit(info, buf), (ReturnCode::SUCCESS, None)),
                                }
                            }
                            _ => (TxState::Idle, (rval, buf)),
                        }
                    }
                    TxState::ReadyToEncode(info, buf) => {
                        // TODO MAC ldpc encode here
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
                    match info.fec_type {
                        Some(FecType::None) => {
                            (RxState::Idle, Some(buf))
                        }
                        Some(FecType::LdpcTc128) => {
                            // Do decode for LDPC TC128 here then return success for fail
                            (RxState::Idle, Some(buf))
                        }
                        Some(FecType::LdpcTc256) => {
                            // Same as above
                            (RxState::Idle, Some(buf))
                        }
                        Some(FecType::LdpcTc512) => {
                            // Same as above
                            (RxState::Idle, Some(buf))
                        }
                        _ => {
                            (RxState::Idle, Some(buf))
                        }
                    }
                }
                RxState::ReadyToYield(info, buf) => {
                    let _frame_len = info.data_len;
                    // Extract data - headers from frame here and return if success
                    (RxState::Idle, Some(buf))
                }
                RxState::ReadyToReturn(buf) => (RxState::Idle, Some(buf)),
            };
            self.rx_state.replace(next_state);

            if let Some(buf) = buf {
                self.vrfc.set_receive_buffer(buf);
            }
        });
    }
}

impl<V> device::Device<'a> for Framer<'a, V> 
where 
    V: virtual_rfcore::RFCore,
{
    fn set_transmit_client(&self, client: &'a device::TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'a device::RxClient) {
        self.rx_client.set(client);
    }

    fn config_commit(&self) {
        self.vrfc.config_commit();
    }

    fn is_on(&self) -> bool {
        self.vrfc.get_radio_status()
    }

    fn prepare_data_frame(&self, buf: &'static mut [u8], _seq: u8, fec_type: Option<FecType>) -> Result<Frame, &'static mut [u8]> {
        let _header = Header {
            frame_type: FrameType::Data,
            id: None,
            seq: Some(self.seq.get()),
        };
        
        // encode header here and return some result
        let frame = Frame {
            buf: buf,
            info: FrameInfo {
                frame_type: FrameType::Data,
                fec_type: fec_type,
                data_len: 0,
            },
            max_frame_size: 128, // This needs to be configurable later
        };
        Ok(frame)
    }

    fn transmit(&self, frame: Frame) -> (ReturnCode, Option<&'static mut [u8]>) {
        let Frame { buf, info, max_frame_size: _ } = frame;
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

impl<V> radio_client::TxClient for Framer<'a, V> 
where
    V: virtual_rfcore::RFCore,
{
    fn transmit_event(&self, buf: &'static mut [u8], _result: ReturnCode) {
        self.seq.set(self.seq.get() + 1);
        self.tx_client.map(move |client| {
            client.transmit_event(buf, _result);
        });
    }
}

impl<V> radio_client::RxClient for Framer<'a, V> 
where 
    V: virtual_rfcore::RFCore,
{
    fn receive_event(&self, buf: &'static mut [u8], frame_len: usize, crc_valid: bool, _result: ReturnCode) {
        if !crc_valid {
            self.vrfc.set_receive_buffer(buf);
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
                    self.vrfc.set_receive_buffer(buf);
                    other_state
                }
            };
            self.rx_state.replace(next_state);
            self.step_receive_state();
        });
    }
}

impl<V> radio_client::ConfigClient for Framer<'a, V> 
where 
    V: virtual_rfcore::RFCore,
{
    fn config_done(&self, _: ReturnCode) {
        let (rval, buf) = self.step_transmit_state();
        if let Some(buf) = buf {
            // Return buf to tx client
            self.tx_client.map(move |client| {
                client.transmit_event(buf, rval);
            });
        }
    }
}

