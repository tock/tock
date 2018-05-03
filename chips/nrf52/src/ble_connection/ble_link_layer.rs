use ble_connection::ble_advertising_driver::{App, AppBLEState, BleLinkLayerState};
use ble_connection::ble_advertising_hil::{RadioChannel, ReadAction, ResponseAction, TxImmediate};
use ble_connection::ble_advertising_hil::ActionAfterTimerExpire;
use ble_connection::ble_connection_driver::ConnectionData;
use ble_connection::ble_pdu_parser::{BLEAdvertisementType, BLEPduType};
use ble_connection::ble_pdu_parser::PACKET_ADDR_START;
use nrf5x::constants;
use core::fmt;

pub type TxNextChannelType = (TxImmediate, Option<(RadioChannel, u32, u32)>);

pub struct LinkLayer;

impl LinkLayer {
    pub fn handle_rx_start(
        &self,
        app: &mut App,
        pdu_type: Option<BLEAdvertisementType>,
    ) -> ReadAction {
        match app.process_status {
            Some(AppBLEState::Advertising) => match pdu_type {
                Some(BLEAdvertisementType::ScanRequest) => ReadAction::ReadFrame,
                Some(BLEAdvertisementType::ConnectRequest) => ReadAction::ReadFrame,
                _ => ReadAction::SkipFrame,
            },
            Some(AppBLEState::Connection(_)) => ReadAction::ReadFrame,
            Some(AppBLEState::Scanning) => ReadAction::ReadFrame,
            Some(AppBLEState::InitiatingConnection) => ReadAction::SkipFrame,
            _ => ReadAction::SkipFrame,
        }
    }

    pub fn handle_rx_end(&self, app: &App, pdu: BLEPduType) -> Option<ResponseAction> {
        match pdu {
            BLEPduType::ScanRequest(_scan_addr, ref adv_addr) => {
                if app.is_my_address(adv_addr) {
                    Some(ResponseAction::ScanResponse)
                } else {
                    None
                }
            }
            BLEPduType::ConnectRequest(_init_addr, adv_addr, lldata) => {
                if app.is_my_address(&adv_addr) {
                    Some(ResponseAction::Connection(ConnectionData::new(lldata)))
                } else {
                    None
                }
            }
            _ => {
                debug!("pdu: {:?}", pdu);
                panic!("Unexpected PDU type\n");
            }
        }
    }

    pub fn handle_event_done(&self, app: &mut App) -> TxNextChannelType {
        match app.process_status {
            Some(AppBLEState::Advertising) => {
                if let Some(channel) = app.channel {
                    if let Some(next_channel) = channel.get_next_advertising_channel() {
                        (
                            TxImmediate::TX,
                            Some((
                                next_channel,
                                constants::ADV_ACCESS_ADDRESS_BLE,
                                constants::RADIO_CRCINIT_BLE,
                            )),
                        )
                    } else {
                        (TxImmediate::GoToSleep, None)
                    }
                } else {
                    panic!("App has no channel");
                }
            }
            Some(AppBLEState::Connection(ref mut conn_data)) => {
                let channel = conn_data.next_channel();
                (
                    TxImmediate::RespondAfterTifs,
                    Some((channel, conn_data.aa, conn_data.crcinit)),
                )
            }
            _ => (TxImmediate::GoToSleep, None),
        }
    }

    pub fn handle_timer_expire(&self, app: &mut App) -> ActionAfterTimerExpire {
        match app.process_status {
            Some(AppBLEState::Advertising) => ActionAfterTimerExpire::ContinueAdvertising,
            Some(AppBLEState::Connection(ref conndata)) => {
                ActionAfterTimerExpire::ContinueConnection(
                    conndata.calculate_conn_supervision_timeout(),
                    conndata.lldata.connection_interval(),
                )
            }
            _ => {
                panic!("Timer expired but app has no state\n");
            }
        }
    }
}

pub struct LLData {
    pub aa: [u8; 4],
    pub crc_init: [u8; 3],
    win_size: u8,
    win_offset: u16,
    interval: u16,
    pub latency: u16,
    pub timeout: u16,
    pub chm: [u8; 5],
    pub hop_and_sca: u8, // hops 5 bits, sca 3 bits
}

impl fmt::Debug for LLData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LLData {{ aa: {:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}, crc_init: {:0>2x}{:0>2x}{:0>2x}, win_size: {}, win_offset: {:0>4x}, interval: {:0>4x}, latency: {:0>4x}, timeout: {:0>4x}, chm: {:0>2x}{:0>2x}{:0>2x}{:0>2x}{:0>2x}, hop: {}, sca: {:0>3b} }}",
               self.aa[0], self.aa[1], self.aa[2], self.aa[3],
               self.crc_init[0], self.crc_init[1], self.crc_init[2],
               self.win_size,
               self.win_offset,
               self.interval,
               self.latency,
               self.timeout,
               self.chm[0], self.chm[1], self.chm[2], self.chm[3], self.chm[4],
               self.hop_and_sca & 0b11111, // Hop
               (self.hop_and_sca & 0b11100000) >> 5, // sca
        )
    }
}

impl LLData {
    pub fn new() -> LLData {
        LLData {
            aa: [0x33, 0x19, 0x32, 0x66], // TODO Implement with 20 bits of entropy: p. 2564
            crc_init: [0x27, 0x01, 0x11], // TODO Implement with 20 bits of entropy: p. 2578
            win_size: 0x03,
            win_offset: 0x0d00,
            interval: 0x1800,
            latency: 0x0000,
            timeout: 0x4800, // TODO .to_be() or .to_le()
            chm: [0x00, 0xf0, 0x1f, 0x00, 0x18],
            hop_and_sca: (1 << 5) | 15, // = 0010 1111
        }
    }

    pub fn read_from_buffer(buffer: &[u8]) -> LLData {
        LLData {
            aa: [
                buffer[PACKET_ADDR_START + 15],
                buffer[PACKET_ADDR_START + 14],
                buffer[PACKET_ADDR_START + 13],
                buffer[PACKET_ADDR_START + 12],
            ],
            crc_init: [
                buffer[PACKET_ADDR_START + 18],
                buffer[PACKET_ADDR_START + 17],
                buffer[PACKET_ADDR_START + 16],
            ],
            win_size: buffer[PACKET_ADDR_START + 19],
            win_offset: (buffer[PACKET_ADDR_START + 21] as u16) << 8
                | buffer[PACKET_ADDR_START + 20] as u16,
            interval: (buffer[PACKET_ADDR_START + 23] as u16) << 8
                | buffer[PACKET_ADDR_START + 22] as u16,
            latency: (buffer[PACKET_ADDR_START + 25] as u16) << 8
                | buffer[PACKET_ADDR_START + 24] as u16,
            timeout: (buffer[PACKET_ADDR_START + 27] as u16) << 8
                | buffer[PACKET_ADDR_START + 26] as u16,
            chm: [
                buffer[PACKET_ADDR_START + 28],
                buffer[PACKET_ADDR_START + 29],
                buffer[PACKET_ADDR_START + 30],
                buffer[PACKET_ADDR_START + 31],
                buffer[PACKET_ADDR_START + 32],
            ],
            hop_and_sca: buffer[PACKET_ADDR_START + 33],
        }
    }

    #[inline(always)]
    fn msec_to_usec(msec: u32) -> u32 {
        msec * 1000
    }

    #[inline(always)]
    fn msec_to_multiple_of_125(msec: u32) -> u32 {
        Self::msec_to_usec(msec) * 5 / 4
    }

    #[inline(always)]
    pub fn window_offset(&self) -> u32 {
        Self::msec_to_multiple_of_125(self.win_offset as u32)
    }

    #[inline(always)]
    pub fn window_size(&self) -> u32 {
        Self::msec_to_multiple_of_125(self.win_size as u32)
    }

    #[inline(always)]
    pub fn connection_interval(&self) -> u32 {
        Self::msec_to_multiple_of_125(self.interval as u32)
    }
}
