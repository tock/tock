//! Bluetooth Low Energy Advertising Driver
//!
//! A system call driver that exposes the Bluetooth Low Energy advertising
//! channel. The driver generates a unique static address for each process,
//! allowing each process to act as its own device and send or scan for
//! advertisements. Timing of advertising or scanning events is handled by the
//! driver but processes can request an advertising or scanning interval.
//! Processes can also control the TX power used for their advertisements.
//!
//! Data payloads are limited to 31 bytes since the maximum advertising channel
//! protocol data unit (PDU) is 37 bytes and includes a 6-byte header.
//!
//! ### Allow system call
//! The allow systems calls are used for buffers from allocated by userland
//!
//!
//! There are three different buffers:
//!
//! * Bluetooth Low Energy Gap Types
//! * Passive Scanner
//! * Advertisement
//!
//!
//! The following allow numbers are supported:
//!
//! * 1: «Flags»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.3
//! * 2: «Incomplete List of 16-bit Service Class UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.1
//! * 4: «Incomplete List of 32-bit Service Class UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.1
//! * 5: «Complete List of 32-bit Service Class UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.1
//! * 6: «Incomplete List of 128-bit Service Class UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.1
//! * 7: «Complete List of 128-bit Service Class UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.1
//! * 8: «Shortened Local Name»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.2
//! * 9: «Complete Local Name»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.2
//! * 10`: «Tx Power Level»
//! Bluetooth Core Specification:Vol. 3, Part C, section 8.1.5
//! * 16: «Device ID» Device ID Profile v1.3 or later
//! * 18`: «Slave Connection Interval Range»
//! Bluetooth Core Specification:Vol. 3, Part C, sections 11.1.8 and 18.8
//! * 20: «List of 16-bit Service Solicitation UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, sections 11.1.9 and 18.9
//! * 21: «List of 128-bit Service Solicitation UUIDs»
//! Bluetooth Core Specification:Vol. 3, Part C, sections 11.1.9 and 18.9
//! * 22: «Service Data»
//! Bluetooth Core Specification:Vol. 3, Part C, sections 11.1.10 and 18.10
//! * 25: «Appearance»
//! Bluetooth Core Specification:Core Specification Supplement, Part A, section 1.12
//! * 26: «Advertising Interval»
//! Bluetooth Core Specification:Core Specification Supplement, Part A, section 1.15
//! * 49: Passive Scanning
//! * 50: Advertising
//! * 255: «Manufacturer Specific Data» Bluetooth Core Specification:Vol. 3, Part C, section 8.1.4
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * SUCCESS: The buffer has successfully been filled
//! * ENOSUPPORT: Invalid allow_num
//! * ENOMEM: No sufficient memory available
//! * EINVAL: Invalid address of the buffer or other error
//! * EBUSY: The driver is currently busy with other tasks
//! * ENOSUPPORT: The operation is not supported
//!
//! ### Subscribe system call
//!  The 'subscribe' system call supports two arguments `subscribe_num' and 'callback'.
//! 'subscribe' is used to specify the specific operation, currently:
//!
//! * 0: provides a callback user-space when a device scanning for advertisements
//!      and the callback is used to invoke user-space processes.
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * ENOMEM:    Not sufficient amount memory
//! * EINVAL:    Invalid operation
//!
//! ### Command system call
//! The `command` system call supports two arguments `cmd` and 'sub_cmd'.
//! 'cmd' is used to specify the specific operation, currently
//! the following cmd's are supported:
//!
//! * 0: start advertisement
//! * 1: stop advertisement
//! * 2: configure tx power
//! * 3: configure advertisement interval
//! * 4: clear the advertisement payload
//! * 5: start scanning
//! * 6: initialize driver
//!
//! The possible return codes from the 'command' system call indicate the following:
//!
//! * SUCCESS:      The command was successful
//! * EBUSY:        The driver is currently busy with other tasks
//! * ENOSUPPORT:   The operation is not supported
//!
//! Usage
//! -----
//! ```
//! Advertisement:
//!
//!           +-------------------------------+
//!           | Initilize Advertisement Buffer|
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Request BLE Address           |
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Configure  ADV_TYPE           |
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Start Advertising             |
//!           +-------------------------------+
//!                          |
//!           +-------------------------------+
//!           | Configure Alarm               |------------|
//!           +-------------------------------+            |
//!                          |                             |
//!           +-------------------------------+            |
//!           | Send Packet                   |------------|
//!           +-------------------------------+
//!
//! Client
//!           +-------------------------------+
//!           | Packet Sent or Error          |------------|
//!           +-------------------------------+            |
//!                         |                              |
//!           +-------------------------------+            |
//!           | Notify BLE Driver             |------------|
//!           +-------------------------------+
//!
//! ```
//!
//! ```
//! Passive Scanning:
//!
//!           +-----------------------+
//!           | Configure Callback    |
//!           +-----------------------+
//!                      |
//!           +-----------------------+
//!           | Initilize Scan Buffer |
//!           +-----------------------+
//!                      |
//!           +-----------------------+
//!           | Start Passive Scanning|
//!           +-----------------------+
//!                      |
//!           +-----------------------+
//!           | Configure Alarm       |--------------|
//!           +-----------------------+              |
//!                      |                           |
//!           +-----------------------+              |
//!           | Receive Packet        |--------------|
//!           +-----------------------+
//!
//! Client
//!           +-------------------------------+
//!           | Packet Received or Error      |------------|
//!           +-------------------------------+            |
//!                         |                              |
//!           +-------------------------------+            |
//!           | Notify BLE Driver             |------------|
//!           +-------------------------------+
//! ```
//!
//! You need a device that provides the `nrf5x::ble_advertising_hil::BleAdvertisementDriver` trait
//! along with a virtual timer to perform events and not block the entire kernel
//!
//! ```rust
//!     let ble_radio = static_init!(
//!     nrf5x::ble_advertising_driver::BLE
//!     <'static, nrf52::radio::Radio, VirtualMuxAlarm<'static, Rtc>>,
//!     nrf5x::ble_advertising_driver::BLE::new(
//!         &mut nrf52::radio::RADIO,
//!     kernel::Grant::create(),
//!         &mut nrf5x::ble_advertising_driver::BUF,
//!         ble_radio_virtual_alarm));
//!    nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_rx_client(&nrf52::radio::RADIO,
//!                                                                      ble_radio);
//!    nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_tx_client(&nrf52::radio::RADIO,
//!                                                                      ble_radio);
//!    ble_radio_virtual_alarm.set_client(ble_radio);
//! ```
//!
//! ### Authors
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: June 22, 2017

use ble_advertising_hil;
use ble_advertising_hil::{RadioChannel, ReadAction};
use ble_connection::ConnectionData;
use ble_event_handler::BLESender;
use core::cell::Cell;
use core::cmp;
use kernel;
use kernel::hil::time::Frequency;
use kernel::returncode::ReturnCode;
use ble_advertising_hil::PhyTransition;
use ble_advertising_hil::TxImmediate;
use ble_link_layer::LinkLayer;
use ble_advertising_hil::ResponseAction;
use ble_link_layer::TxNextChannelType;
use constants;
use ble_pdu_parser::BLEAdvertisementType;
use ble_pdu_parser::BLEPduType;
use ble_pdu_parser::DeviceAddress;

/// Syscall Number
pub const DRIVER_NUM: usize = 0x03_00_00;

pub static mut BUF: [u8; PACKET_LENGTH] = [0; PACKET_LENGTH];
const TRANSMIT_WINDOW_DELAY_CONN_IND: u32 = 1000 * 5 / 4; // 1.25ms in us

use ble_pdu_parser::PACKET_START;
use ble_pdu_parser::PACKET_HDR_PDU;
use ble_pdu_parser::PACKET_HDR_LEN;
use ble_pdu_parser::PACKET_ADDR_START;
use ble_pdu_parser::PACKET_ADDR_END;
use ble_pdu_parser::PACKET_PAYLOAD_START;
use ble_pdu_parser::PACKET_LENGTH;


#[allow(unused)]
struct BLEGap(BLEGapType);

enum AllowType {
    BLEGap(BLEGapType),
    PassiveScanning,
    InitAdvertisementBuffer,
}

impl AllowType {
    fn from_usize(n: usize) -> Option<AllowType> {
        match n {
            0x01 => Some(AllowType::BLEGap(BLEGapType::Flags)),
            0x02 => Some(AllowType::BLEGap(BLEGapType::IncompleteList16BitServiceIDs)),
            0x03 => Some(AllowType::BLEGap(BLEGapType::CompleteList16BitServiceIDs)),
            0x04 => Some(AllowType::BLEGap(BLEGapType::IncompleteList32BitServiceIDs)),
            0x05 => Some(AllowType::BLEGap(BLEGapType::CompleteList32BitServiceIDs)),
            0x06 => Some(AllowType::BLEGap(
                BLEGapType::IncompleteList128BitServiceIDs,
            )),
            0x07 => Some(AllowType::BLEGap(BLEGapType::CompleteList128BitServiceIDs)),
            0x08 => Some(AllowType::BLEGap(BLEGapType::ShortedLocalName)),
            0x09 => Some(AllowType::BLEGap(BLEGapType::CompleteLocalName)),
            0x0A => Some(AllowType::BLEGap(BLEGapType::TxPowerLevel)),
            0x10 => Some(AllowType::BLEGap(BLEGapType::DeviceId)),
            0x12 => Some(AllowType::BLEGap(BLEGapType::SlaveConnectionIntervalRange)),
            0x14 => Some(AllowType::BLEGap(BLEGapType::List16BitSolicitationIDs)),
            0x15 => Some(AllowType::BLEGap(BLEGapType::List128BitSolicitationIDs)),
            0x16 => Some(AllowType::BLEGap(BLEGapType::ServiceData)),
            0x19 => Some(AllowType::BLEGap(BLEGapType::Appearance)),
            0x1A => Some(AllowType::BLEGap(BLEGapType::AdvertisingInterval)),
            0x31 => Some(AllowType::PassiveScanning),
            0x32 => Some(AllowType::InitAdvertisementBuffer),
            0xFF => Some(AllowType::BLEGap(BLEGapType::ManufacturerSpecificData)),
            _ => None,
        }
    }
}

// Gap Types only the ones that are defined in libtock are defined here
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(usize)]
enum BLEGapType {
    Flags = 0x01,
    IncompleteList16BitServiceIDs = 0x02,
    CompleteList16BitServiceIDs = 0x03,
    IncompleteList32BitServiceIDs = 0x04,
    CompleteList32BitServiceIDs = 0x05,
    IncompleteList128BitServiceIDs = 0x06,
    CompleteList128BitServiceIDs = 0x07,
    ShortedLocalName = 0x08,
    CompleteLocalName = 0x09,
    TxPowerLevel = 0x0A,
    DeviceId = 0x10,
    SlaveConnectionIntervalRange = 0x12,
    List16BitSolicitationIDs = 0x14,
    List128BitSolicitationIDs = 0x15,
    ServiceData = 0x16,
    Appearance = 0x19,
    AdvertisingInterval = 0x1A,
    ManufacturerSpecificData = 0xFF,
}


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BusyState {
    Free,
    Busy(kernel::AppId), // AppId of the App currently using the radio
}

// ConnectUndirected (ADV_IND): connectable undirected advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.1
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | AdvData      |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
// ConnectDirected (ADV_DIRECT_IND): connectable directed advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.2
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | InitA        |
//           | (6 bytes) |      | (6 bytes)    |
//           +-----------+      +--------------+
//
// NonConnectUndirected (ADV_NONCONN_IND): non-connectable undirected advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.3
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | AdvData      |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
//
// ScanUndirected (ADV_SCAN_IND): scannable undirected advertising event
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.1.4
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | AdvData      |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
// ScanRequest (SCAN_REQ)
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.2.1
//
//   PDU     +-----------+      +--------------+
//           | ScanA     |  -   | AdvA        |
//           | (6 bytes) |      | (6 bytes) |
//           +-----------+      +--------------+
//
// ScanResponse (SCAN_RSP)
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.2.2
//
//   PDU     +-----------+      +--------------+
//           | AdvA      |  -   | ScanRspData  |
//           | (6 bytes) |      | (0-31 bytes) |
//           +-----------+      +--------------+
//
// ConnectRequest (CON_REQ)
// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.3.1
//
//   PDU     +-----------+      +--------------+     +--------------+
//           | InitA     |  -   | AdvA         |  -  | LLData       |
//           | (6 bytes) |      | 6 bytes      |     | 22 bytes     |
//           +-----------+      +--------------+     +--------------+
//

#[derive(PartialEq, Debug)]
pub enum AppBLEState {
    NotInitialized,
    Initialized,
    Scanning,
    Advertising,
    InitiatingConnection,
    Connection(ConnectionData),
}

#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
struct AlarmData {
    t0: u32,
    expiration: Expiration,
}

impl AlarmData {
    fn new() -> AlarmData {
        AlarmData {
            t0: 0,
            expiration: Expiration::Disabled,
        }
    }
}

#[derive(PartialEq)]
enum BleLinkLayerState {
    RespondingToScanRequest,
    WaitingForConnection,
    EndOfConnectionEvent,
}

pub struct App {
    advertising_address: Option<DeviceAddress>,
    advertisement_buf: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_write: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_read: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
    idx: usize,
    pub process_status: Option<AppBLEState>,
    advertisement_interval_ms: u32,
    alarm_data: AlarmData,
    tx_power: u8,
    state: Option<BleLinkLayerState>,
    pub channel: Option<RadioChannel>,
    /// The state of an app-specific pseudo random number.
    ///
    /// For example, it can be used for the pseudo-random `advDelay` parameter.
    /// It should be read using the `random_number` method, which updates it as
    /// well.
    random_nonce: u32,
}

impl Default for App {
    fn default() -> App {
        App {
            advertising_address: None,
            advertisement_buf: None,
            alarm_data: AlarmData::new(),
            app_write: None,
            app_read: None,
            scan_callback: None,
            idx: PACKET_PAYLOAD_START,
            process_status: Some(AppBLEState::NotInitialized),
            tx_power: 0,
            state: None,
            channel: None,
            advertisement_interval_ms: 200,
            // Just use any non-zero starting value by default
            random_nonce: 0xdeadbeef,
        }
    }
}

impl App {
    fn initialize_advertisement_buffer(&mut self) -> ReturnCode {
        self.advertisement_buf
            .as_mut()
            .map(|buf| {
                for i in buf.as_mut()[PACKET_START..PACKET_LENGTH].iter_mut() {
                    *i = 0x00;
                }
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| ReturnCode::EINVAL)
    }

    // Bluetooth Core Specification:Vol. 6, Part B, section 1.3.2.1 Static Device Address
    //
    // A static address is a 48-bit randomly generated address and shall meet the following
    // requirements:
    // • The two most significant bits of the address shall be equal to 1
    // • At least one bit of the random part of the address shall be 0
    // • At least one bit of the random part of the address shall be 1
    //
    // Note that endianness is a potential problem here as this is suppose to be platform
    // independent therefore use 0xf0 as both byte 1 and byte 6 i.e., the two most significant bits
    // are equal to one regardless of endianness
    //
    // Byte 1            0xf0
    // Byte 2-5          random
    // Byte 6            0xf0
    // FIXME: For now use AppId as "randomness"
    fn generate_random_address(&mut self, appid: kernel::AppId) -> ReturnCode {
        /*let random_address: [u8; 6] = [
            0xf0,
            0x11,
            0x11,
            ((appid.idx() << 16) as u8 & 0xff),
            ((appid.idx() << 24) as u8 & 0xff),
            0xf0,
        ];*/
        let random_address: [u8; 6] = [0xf0, 0x0f, 0x0f, ((appid.idx() << 16) as u8 & 0xff), ((appid.idx() << 24) as u8 & 0xff), 0xf0];
        self.advertising_address = Some(DeviceAddress::new(&random_address));

        debug!("random address!, {:?}", self.advertising_address);

        self.advertisement_buf
            .as_mut()
            .map_or(ReturnCode::ESIZE, |data| {
                data.as_mut()[PACKET_HDR_LEN] = 6;
                for i in 0..6 {
                    data.as_mut()[PACKET_ADDR_START + i] = random_address[i];
                }
                ReturnCode::SUCCESS
            })
    }

    pub fn make_adv_pdu(&self, buffer: &mut [u8], header: &mut u8) -> u8 {
        self.advertisement_buf.as_ref().map(|data| {
            for i in 0..PACKET_LENGTH {
                buffer[i] = data.as_ref()[PACKET_ADDR_START + i];
            }
        });

        *header = (0x04 << 4) | (BLEAdvertisementType::ConnectUndirected as u8);

        self.idx as u8
    }

    fn reset_payload(&mut self) -> ReturnCode {
        match self.process_status {
            Some(AppBLEState::Advertising) | Some(AppBLEState::Scanning) => ReturnCode::EBUSY,
            _ => {
                let res = self.advertisement_buf
                    .as_mut()
                    .map(|data| {
                        for byte in data.as_mut()[PACKET_PAYLOAD_START..PACKET_LENGTH].iter_mut() {
                            *byte = 0x00;
                        }
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|| ReturnCode::EINVAL);
                if res == ReturnCode::SUCCESS {
                    self.idx = PACKET_PAYLOAD_START;
                }
                res
            }
        }
    }

    // Hard-coded to ADV_NONCONN_IND
    fn configure_advertisement_pdu(&mut self) -> ReturnCode {
        self.advertisement_buf
            .as_mut()
            .map(|slice| {
                slice.as_mut()[PACKET_HDR_PDU] =
                    (0x04 << 4) | (BLEAdvertisementType::ConnectUndirected as u8);
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| ReturnCode::ESIZE)
    }

    fn set_gap_data(&mut self, gap_type: BLEGapType) -> ReturnCode {
        self.app_write
            .take()
            .as_ref()
            .map(|slice| {
                let idx = self.idx;
                let end = idx + slice.len() + 2;

                if end <= PACKET_LENGTH {
                    let result = self.advertisement_buf
                        .as_mut()
                        .map(|data| {
                            // set header and length
                            data.as_mut()[idx] = (slice.len() + 1) as u8;
                            data.as_mut()[idx + 1] = gap_type as u8;

                            // update total packet size
                            data.as_mut()[PACKET_HDR_LEN] = (end - 2) as u8;

                            // set data
                            for (dst, src) in data.as_mut()[idx + 2..end]
                                .iter_mut()
                                .zip(slice.as_ref()[0..slice.len()].iter())
                                {
                                    *dst = *src;
                                }
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|| ReturnCode::EINVAL);

                    // If the operation was successful => update idx
                    if result == ReturnCode::SUCCESS {
                        self.idx = end;
                    }
                    result
                } else {
                    ReturnCode::ESIZE
                }
            })
            .unwrap_or_else(|| ReturnCode::EINVAL)
    }

    fn prepare_advertisement(
        &mut self,
        ble: &BLESender,
        advertisement_type: BLEAdvertisementType,
    ) -> ReturnCode {
        self.state = None;

        self.advertisement_buf
            .as_ref()
            .map_or(ReturnCode::EINVAL, |slice| {
                ble.replace_buffer(&|data: &mut [u8]| {
                    for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                        .iter_mut()
                        .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                    data.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (advertisement_type as u8);
                });
                ReturnCode::SUCCESS
            })
    }

    fn prepare_scan_response<'a, B, A>(&mut self, ble: &BLE<'a, B, A>) -> ReturnCode
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {
        self.state = Some(BleLinkLayerState::RespondingToScanRequest);

        self.advertisement_buf
            .as_ref()
            .map(|slice| {
                ble.replace_buffer(&|data: &mut [u8]| {
                    for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                        .iter_mut()
                        .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                    data.as_mut()[PACKET_HDR_PDU] =
                        (0x04 << 4) | (BLEAdvertisementType::ScanResponse as u8);
                });

                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| ReturnCode::EINVAL)
    }


    fn set_empty_conn_pdu<'a, B, A>(&mut self, ble: &BLE<'a, B, A>, transmit_sequence_number: u8, next_expected_sequence_number: u8) -> ReturnCode
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {
        // debug!("Sending ConnectRequest to {:?} on channel {:?}", adv_addr, channel);

        self.advertisement_buf
            .as_ref()
            .map(|_| {
                ble.replace_buffer(&|data: &mut [u8]| {

                    // LLID == 0x01 Empty PDU
                    data.as_mut()[PACKET_HDR_PDU] = 0x01 |
                        (next_expected_sequence_number & 0b1) << 2 |
                        (transmit_sequence_number & 0b1) << 3;

                    data.as_mut()[PACKET_HDR_LEN] = 0;
                });

                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| ReturnCode::EINVAL)
    }

    // Returns a new pseudo-random number and updates the randomness state.
    //
    // Uses the [Xorshift](https://en.wikipedia.org/wiki/Xorshift) algorithm to
    // produce pseudo-random numbers. Uses the `random_nonce` field to keep
    // state.
    fn random_nonce(&mut self) -> u32 {
        let mut next_nonce = ::core::num::Wrapping(self.random_nonce);
        next_nonce ^= next_nonce << 13;
        next_nonce ^= next_nonce >> 17;
        next_nonce ^= next_nonce << 5;
        self.random_nonce = next_nonce.0;
        self.random_nonce
    }

    // Set the next alarm for this app using the period and provided start time.
    fn set_next_alarm<F: Frequency>(&mut self, now: u32) {
        self.alarm_data.t0 = now;
        let nonce = self.random_nonce() % 10;

        let period_ms = (self.advertisement_interval_ms + nonce) * F::frequency() / 1000;

        self.alarm_data.expiration = Expiration::Abs(now.wrapping_add(period_ms));
    }

    pub fn is_my_address(&self, address: &DeviceAddress) -> bool {
        self.advertising_address == Some(*address)
    }
}

pub struct BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    radio: &'a B,
    busy: Cell<BusyState>,
    app: kernel::Grant<App>,
    kernel_tx: kernel::common::take_cell::TakeCell<'static, [u8]>,
    alarm: &'a A,
    sending_app: Cell<Option<kernel::AppId>>,
    receiving_app: Cell<Option<kernel::AppId>>,
    link_layer: LinkLayer,
}

impl<'a, B, A> BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    pub fn new(
        radio: &'a B,
        container: kernel::Grant<App>,
        tx_buf: &'static mut [u8],
        alarm: &'a A,
    ) -> BLE<'a, B, A> {
        BLE {
            radio: radio,
            busy: Cell::new(BusyState::Free),
            app: container,
            kernel_tx: kernel::common::take_cell::TakeCell::new(tx_buf),
            alarm: alarm,
            sending_app: Cell::new(None),
            receiving_app: Cell::new(None),
            link_layer: LinkLayer::default(),
        }
    }

    // Determines which app timer will expire next and sets the underlying alarm
    // to it.
    //
    // This method iterates through all grants so it should be used somewhat
    // sparringly. Moreover, it should _not_ be called from within a grant,
    // since any open grant will not be iterated over and the wrong timer will
    // likely be chosen.
    fn reset_active_alarm(&self) {
        let now = self.alarm.now();
        let mut next_alarm = u32::max_value();
        let mut next_dist = u32::max_value();
        for app in self.app.iter() {
            app.enter(|app, _| match app.alarm_data.expiration {
                Expiration::Abs(exp) => {
                    let t_dist = exp.wrapping_sub(now);
                    if next_dist > t_dist {
                        next_alarm = exp;
                        next_dist = t_dist;
                    }
                }
                Expiration::Disabled => {}
            });
        }
        if next_alarm != u32::max_value() {
            self.alarm.set_alarm(next_alarm);
        }
    }
}

impl<'a, B, A> BLESender for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    fn transmit_buffer(&self, appid: kernel::AppId) {
        self.sending_app.set(Some(appid));
        self.kernel_tx.take().map(|buf| {
            let res = self.radio.transmit_advertisement(buf, PACKET_LENGTH);
            self.kernel_tx.replace(res);
        });
    }

    fn transmit_buffer_edit(
        &self,
        len: usize,
        appid: kernel::AppId,
        edit_buffer: &Fn(&mut [u8]) -> (),
    ) {
        self.kernel_tx.map(|buffer| {
            edit_buffer(buffer);
        });

        self.transmit_buffer(appid);
    }

    fn replace_buffer(&self, edit_buffer: &Fn(&mut [u8]) -> ()) {
        self.kernel_tx.take().map(|buffer| {
            edit_buffer(buffer);
            let res = self.radio.set_advertisement_data(buffer, PACKET_LENGTH);
            self.kernel_tx.replace(res);
        });
    }

    fn receive_buffer(&self, appid: kernel::AppId) {
        self.receiving_app.set(Some(appid));
        self.radio.receive_advertisement();
    }
    fn set_tx_power(&self, power: u8) -> ReturnCode {
        self.radio.set_tx_power(power)
    }

    fn set_busy(&self, state: BusyState) {
        self.busy.set(state);
    }
    fn alarm_now(&self) -> u32 {
        self.alarm.now()
    }
    fn set_access_address(&self, address: u32) {
        self.radio.set_access_address(address)
    }
}

// Timer alarm
impl<'a, B, A> kernel::hil::time::Client for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    // When an alarm is fired, we find which apps have expired timers. Expired
    // timers indicate a desire to perform some operation (e.g. start an
    // advertising or scanning event). We know which operation based on the
    // current app's state.
    //
    // In case of collision---if there is already an event happening---we'll
    // just delay the operation for next time and hope for the best. Since some
    // randomness is added for each period in an app's timer, collisions should
    // be rare in practice.
    //
    // TODO: perhaps break ties more fairly by prioritizing apps that have least
    // recently performed an operation.
    fn fired(&self) {
        let now = self.alarm.now();
        //debug!("Timer fired!");

        self.app.each(|app| {
            if let Expiration::Abs(exp) = app.alarm_data.expiration {
                let expired =
                    now.wrapping_sub(app.alarm_data.t0) >= exp.wrapping_sub(app.alarm_data.t0);
                if expired {
                    let appid = app.appid();

                    if let BusyState::Busy(busy_app_id) = self.busy.get() {
                        if busy_app_id != appid {
                            // The radio is currently busy, so we won't be able to start the
                            // operation at the appropriate time. Instead, reschedule the
                            // operation for later. This is _kind_ of simulating actual
                            // on-air interference
                            debug!("BLE: operationg delayed for app {:?}", appid);
                            app.set_next_alarm::<A::Frequency>(self.alarm.now());
                            return;
                        }
                    }

                    self.receiving_app.set(Some(appid));
                    self.sending_app.set(Some(appid));
                    self.radio.set_channel(RadioChannel::AdvertisingChannel37, constants::ADV_ACCESS_ADDRESS_BLE, constants::RADIO_CRCINIT_BLE);

                    //TODO - for now, let the advertiser always set MoveToRX, change later
                    app.channel = Some(RadioChannel::AdvertisingChannel37);
                    app.prepare_advertisement(self, BLEAdvertisementType::ConnectUndirected);
                    self.transmit_buffer(appid);
                }
            }
        });
        self.reset_active_alarm();
    }
}


// Callback from the radio once a RX event occur
impl<'a, B, A> ble_advertising_hil::RxClient for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    fn receive_end(&self, buf: &'static mut [u8], len: u8, result: ReturnCode, rx_timestamp: u32) -> PhyTransition {
        let mut transition = PhyTransition::None;

        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                let pdu_type = BLEAdvertisementType::from_u8(buf[0] & 0x0f);


                // Validate PDU type
                // TODO Move into separate module
                let len: u8 = buf[1];

                let mut valid_pkt = false;

                if result == ReturnCode::SUCCESS {
                    match app.process_status {
                        Some(AppBLEState::Advertising) => {
                            valid_pkt = pdu_type.as_ref().map_or(false, |pdu| pdu.validate_pdu(len));
                        }
                        Some(AppBLEState::Connection(_)) => {
                            valid_pkt = true;
                        }
                        _ => {}
                    }
                }
                // End validate PDU type


                // TODO call advertising/scanner/connection/initiating driver

                transition = if valid_pkt {
                    let res = if let Some(AppBLEState::Advertising) = app.process_status {
                        let pdu_type = pdu_type.expect("PDU type should be valid");
                        let pdu = BLEPduType::from_buffer(pdu_type, buf).expect("PDU should be valid");

                        let response_action = self.link_layer.handle_rx_end(app, pdu);

                        match response_action {
                            Some(ResponseAction::ScanResponse) => {
                                app.prepare_scan_response(&self);

                                PhyTransition::MoveToTX(None)
                            }
                            Some(ResponseAction::Connection(mut conndata)) => {
                                let channel = conndata.next_channel();
                                self.radio.set_channel(channel, conndata.aa, conndata.crcinit);

                                // windowOffset is a multiple of 1.25ms, convert to us
                                let transmitWindowOffset = (conndata.lldata.win_offset as u32) * 1000 * 5 / 4;

                                let delay_until_rx = TRANSMIT_WINDOW_DELAY_CONN_IND + transmitWindowOffset;

                                app.process_status = Some(AppBLEState::Connection(conndata));
                                // app.state = Some(BleLinkLayerState::WaitingForConnection);

                                PhyTransition::MoveToRX(Some(delay_until_rx))
                            }
                            _ => PhyTransition::None,
                        }
                    } else if let Some(AppBLEState::Connection(_)) = app.process_status {
                        let (sn, nesn, interval_ended) = if let Some(AppBLEState::Connection(ref mut conndata)) = app.process_status {
                            let (sn, nesn, retransmit) = conndata.next_sequence_number(buf[0]);


                            let _interval_ended = conndata.connection_interval_ended(rx_timestamp, self.alarm.now());


                            (sn, nesn, true)
                        } else {
                            panic!("Process status is not Connection in Connection!");
                        };

                        if interval_ended {
                            app.state = Some(BleLinkLayerState::EndOfConnectionEvent);
                        }

                        app.set_empty_conn_pdu(&self, sn, nesn);

                        PhyTransition::MoveToTX(None)
                    } else {
                        PhyTransition::None
                    };


                    res
                } else {
                    PhyTransition::None
                }
            });
        }

        transition
    }

    fn receive_start(&self, buf: &'static mut [u8], len: u8) -> ReadAction {

        // TODO parse differently when not advertising - move to link layer?
        let pdu_type = BLEAdvertisementType::from_u8(buf[0] & 0x0f);


        let mut read_action = ReadAction::SkipFrame;

        if let Some(appid) = self.receiving_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                read_action = self.link_layer.handle_rx_start(app, pdu_type)
            });
        }

        read_action
    }
}

// Callback from the radio once a TX event occur
impl<'a, B, A> ble_advertising_hil::TxClient for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    // The ReturnCode indicates valid CRC or not, not used yet but could be used for
    // re-tranmissions for invalid CRCs
    fn transmit_end(&self, _crc_ok: ReturnCode) -> PhyTransition {
        let mut transition = PhyTransition::None;

        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                transition = if let Some(AppBLEState::Advertising) = app.process_status {
                    if let Some(BleLinkLayerState::RespondingToScanRequest) = app.state {
                        app.prepare_advertisement(self, BLEAdvertisementType::ConnectUndirected);
                        PhyTransition::MoveToTX(None)
                    } else {
                        PhyTransition::MoveToRX(None)
                    }
                } else if let Some(AppBLEState::Connection(_)) = app.process_status {
                    if let Some(BleLinkLayerState::EndOfConnectionEvent) = app.state {
                        app.state = None;
                        if let Some(AppBLEState::Connection(ref mut conndata)) = app.process_status {
                            let channel = conndata.next_channel();
                            self.radio.set_channel(channel, conndata.aa, conndata.crcinit);
                        }
                    }
                    PhyTransition::MoveToRX(None)
                } else {
                    PhyTransition::None
                };
            });
            self.reset_active_alarm();
        }
        transition
    }
}


impl<'a, B, A> ble_advertising_hil::AdvertisementClient for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    fn advertisement_done(&self) -> TxImmediate {
        let mut result = TxImmediate::GoToSleep;

        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                if app.state == Some(BleLinkLayerState::RespondingToScanRequest) {
                    app.prepare_advertisement(self, BLEAdvertisementType::ConnectUndirected);
                }

                let (tx_immediate, channeltriple): TxNextChannelType = self.link_layer.handle_event_done(app);

                app.channel = if let Some((channel, adv_addr, crcinit)) = channeltriple {
                    self.radio.set_channel(channel, adv_addr, crcinit);

                    Some(channel)
                } else {
                    None
                };

                if tx_immediate == TxImmediate::GoToSleep {
                    // TODO: Shut down radio when sleeping
                    app.set_next_alarm::<A::Frequency>(self.alarm.now());
                }

                result = tx_immediate;
            });
            self.reset_active_alarm();
        }

        result
    }

    fn timer_expired(&self) {
        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                app.prepare_advertisement(self, BLEAdvertisementType::ConnectUndirected);
                self.transmit_buffer(appid);
            });

            self.reset_active_alarm();
        }
    }
}

// System Call implementation
impl<'a, B, A> kernel::Driver for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    fn command(
        &self,
        command_num: usize,
        data: usize,
        _: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {
            // Start periodic advertisements
            0 => self.app
                .enter(appid, |app, _| {
                    if let Some(AppBLEState::Initialized) = app.process_status {
                        app.process_status =
                            Some(AppBLEState::Advertising);
                        app.channel = Some(RadioChannel::AdvertisingChannel37);
                        app.random_nonce = self.alarm.now();
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        self.reset_active_alarm();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Stop periodic advertisements or passive scanning
            1 => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(AppBLEState::Advertising)
                    | Some(AppBLEState::Scanning) => {
                        app.process_status = Some(AppBLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EBUSY,
                })
                .unwrap_or_else(|err| err.into()),

            // Configure transmitted power
            // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part A], section 3
            //
            // Minimum Output Power:    0.01 mW (-20 dBm)
            // Maximum Output Power:    10 mW (+10 dBm)
            //
            // data - Transmitting power in dBm
            2 => {
                self.app
                    .enter(appid, |app, _| {
                        if app.process_status != Some(AppBLEState::Scanning)
                            && app.process_status
                            != Some(AppBLEState::Advertising)
                            {
                                match data as u8 {
                                    e @ 0...10 | e @ 0xec...0xff => {
                                        app.tx_power = e;
                                        // ask chip if the power level is supported
                                        self.radio.set_tx_power(e)
                                    }
                                    _ => ReturnCode::EINVAL,
                                }
                            } else {
                            ReturnCode::EBUSY
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Configure advertisement interval
            // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 4.4.2.2
            //
            // The advertising interval shall an integer multiple of 0.625ms in the range of
            // 20ms to 10240 ms!
            //
            // data - advertisement interval in ms
            // FIXME: add check that data is a multiple of 0.625
            3 => self.app
                .enter(appid, |app, _| match self.busy.get() {
                    BusyState::Busy(appid) if app.appid() == appid => {
                        ReturnCode::EBUSY
                    }
                    _ => {
                        //app.advertisement_interval_ms = cmp::max(20, cmp::min(10240, data as u32));
                        app.advertisement_interval_ms = cmp::max(20, cmp::min(10240, 280 as u32));
                        ReturnCode::SUCCESS
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Reset payload when the kernel is not actively advertising
            // reset_payload checks whether the current app is correct state or not
            // i.e. if it's ok to reset the payload or not
            4 => self.app
                .enter(appid, |app, _| app.reset_payload())
                .unwrap_or_else(|err| err.into()),

            // Passive scanning mode
            5 => self.app
                .enter(appid, |app, _| {
                    if let Some(AppBLEState::Initialized) = app.process_status {
                        app.process_status = Some(AppBLEState::Scanning);
                        app.channel = Some(RadioChannel::AdvertisingChannel37);
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        self.reset_active_alarm();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Initilize BLE Driver
            // Allow call to allocate the advertisement buffer must be
            // invoked before this
            // Request advertisement address
            6 => self.app
                .enter(appid, |app, _| {
                    if let Some(AppBLEState::Initialized) = app.process_status {
                        let status = app.generate_random_address(appid);
                        if status == ReturnCode::SUCCESS {
                            debug!("Initialize!");
                            app.configure_advertisement_pdu()
                            //app.configure_scan_response_pdu()
                        } else {
                            status
                        }
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        appid: kernel::AppId,
        allow_num: usize,
        slice: kernel::AppSlice<kernel::Shared, u8>,
    ) -> ReturnCode {
        match AllowType::from_usize(allow_num) {
            Some(AllowType::BLEGap(gap_type)) => self.app
                .enter(appid, |app, _| {
                    if app.process_status != Some(AppBLEState::NotInitialized) {
                        app.app_write = Some(slice);
                        app.set_gap_data(gap_type)
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),

            Some(AllowType::PassiveScanning) => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(AppBLEState::NotInitialized) | Some(AppBLEState::Initialized) => {
                        app.app_read = Some(slice);
                        app.process_status = Some(AppBLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EINVAL,
                })
                .unwrap_or_else(|err| err.into()),

            Some(AllowType::InitAdvertisementBuffer) => self.app
                .enter(appid, |app, _| {
                    if let Some(AppBLEState::NotInitialized) = app.process_status {
                        app.advertisement_buf = Some(slice);

                        app.process_status = Some(AppBLEState::Initialized);
                        app.initialize_advertisement_buffer();
                        //app.initialize_scan_response_buffer();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: kernel::Callback) -> ReturnCode {
        match subscribe_num {
            // Callback for scanning
            0 => self.app
                .enter(callback.app_id(), |app, _| match app.process_status {
                    Some(AppBLEState::NotInitialized) | Some(AppBLEState::Initialized) => {
                        app.scan_callback = Some(callback);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EINVAL,
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
