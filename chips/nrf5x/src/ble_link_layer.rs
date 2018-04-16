use ble_advertising_driver::{App, BLEAdvertisementType, AppBLEState, BLEPduType};
use ble_advertising_hil::{ReadAction, ResponseAction, TxImmediate, RadioChannel};
use ble_connection::ConnectionData;
use constants;

pub type TxNextChannelType = (TxImmediate, Option<(RadioChannel, u32, u32)>);

pub struct LinkLayer;

impl Default for LinkLayer {
    fn default() -> LinkLayer {
        LinkLayer
    }
}

impl LinkLayer {
    pub fn handle_rx_start(&self, app: &mut App, pdu_type: Option<BLEAdvertisementType>) -> ReadAction {
        match app.process_status {
            Some(AppBLEState::Advertising) => {
                match pdu_type {
                    Some(BLEAdvertisementType::ScanRequest) => ReadAction::ReadFrameAndMoveToTX,
                    Some(BLEAdvertisementType::ConnectRequest) => ReadAction::ReadFrameAndStayRX,
                    _ => ReadAction::SkipFrame,
                }
            },
            Some(AppBLEState::Connection(_)) => {
                ReadAction::ReadFrameAndMoveToTX
            },
            Some(AppBLEState::Scanning) => ReadAction::ReadFrameAndStayRX,
            Some(AppBLEState::InitiatingConnection) => ReadAction::SkipFrame,
            _ => ReadAction::SkipFrame
        }
    }

    pub fn handle_rx_end(&self, app: &mut App, pdu: BLEPduType) -> Option<ResponseAction> {
        match pdu {
            BLEPduType::ScanRequest(_scan_addr, ref adv_addr) => {
                if app.is_my_address(adv_addr) {
                    Some(ResponseAction::ScanResponse)
                } else {
                    None
                }
            }
            BLEPduType::ConnectRequest(_init_addr, ref adv_addr, ref lldata) => {
                if app.is_my_address(adv_addr) {
                    Some(ResponseAction::Connection(ConnectionData::new(lldata)))
                } else {
                    None
                }
            }
            _ => {
                debug!("pdu: {:?}", pdu);
                panic!("Unexpected PDU type\n");
            },
        }
    }

    pub fn handle_event_done(&self, app: &mut App) -> TxNextChannelType {
        match app.process_status {
            Some(AppBLEState::Advertising) => {
                if let Some(channel) = app.channel {
                    if let Some(next_channel) = channel.get_next_advertising_channel() {
                        (TxImmediate::TX, Some((next_channel, constants::ADV_ACCESS_ADDRESS_BLE, constants::RADIO_CRCINIT_BLE)))
                    } else {
                        (TxImmediate::GoToSleep, None)
                    }
                } else {
                    panic!("App has no channel");
                }
            },
            Some(AppBLEState::Connection(ref mut conn_data)) => {
                let channel = conn_data.next_channel();
                (TxImmediate::RespondAfterTifs, Some((channel, conn_data.aa, conn_data.crcinit)))
            }
            _ => {
                (TxImmediate::GoToSleep, None)
            }
        }
    }
}