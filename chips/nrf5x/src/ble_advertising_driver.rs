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
use ble_advertising_hil::RadioChannel;
//use ble_advertising_hil::{RadioChannel, DeviceAddress};
use core::cell::Cell;
use core::cmp;
use core::fmt;
use kernel;
use kernel::hil::time::Frequency;
use kernel::returncode::ReturnCode;

/// Syscall Number
pub const DRIVER_NUM: usize = 0x03_00_00;

pub static mut BUF: [u8; PACKET_LENGTH] = [0; PACKET_LENGTH];

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


#[allow(unused)]
struct LLData {
    aa: [u8; 4],
    crc_init: [u8; 3],
    win_size: u8,
    win_offset: u16,
    interval: u16,
    latency: u16,
    timeout: u16,
    chm: [u8; 5],
    hop_and_sca: u8 // hops 5 bits, sca 3 bits
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub struct DeviceAddress(pub [u8; 6]);

impl DeviceAddress {
    pub fn new(slice: &[u8]) -> DeviceAddress {
        let mut address : [u8; 6] = Default::default();
        address.copy_from_slice(slice);
        DeviceAddress(address)
    }
}

impl fmt::Debug for DeviceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}",
               self.0[5], self.0[4], self.0[3],
               self.0[2], self.0[1], self.0[0])
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum BusyState {
    Free,
    Busy(kernel::AppId) // AppId of the App currently using the radio
}

#[derive(Debug)]
pub enum BLEPduType<'a> {
    ConnectUndirected(DeviceAddress, &'a [u8]),
    ConnectDirected(DeviceAddress, DeviceAddress),
    NonConnectUndirected(DeviceAddress, &'a [u8]),
    ScanUndirected(DeviceAddress, &'a [u8]),
    ScanRequest(DeviceAddress, DeviceAddress),
    ScanResponse(DeviceAddress, &'a [u8]),
    ConnectRequest(DeviceAddress, DeviceAddress, &'a [u8])
}

impl <'a> BLEPduType<'a> {
    pub fn from_buffer(pdu_type: BLEAdvertisementType, buf: &[u8]) -> BLEPduType {
        match pdu_type {
            BLEAdvertisementType::ConnectUndirected => BLEPduType::ConnectUndirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), &buf[PACKET_PAYLOAD_START..]),
            BLEAdvertisementType::ConnectDirected => BLEPduType::ConnectDirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14])),
            BLEAdvertisementType::NonConnectUndirected => BLEPduType::NonConnectUndirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), &buf[PACKET_PAYLOAD_START..]),
            BLEAdvertisementType::ScanUndirected => BLEPduType::ScanUndirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), &buf[PACKET_PAYLOAD_START..]),
            BLEAdvertisementType::ScanRequest => BLEPduType::ScanRequest(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14])),
            BLEAdvertisementType::ScanResponse => BLEPduType::ScanResponse(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), &buf[PACKET_PAYLOAD_START..(buf[PACKET_HDR_LEN] + PACKET_PAYLOAD_START as u8 - 6) as usize]),
            BLEAdvertisementType::ConnectRequest => BLEPduType::ConnectRequest(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END+1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14]), &buf[14..]),
        }
    }

    pub fn address(&self) -> DeviceAddress {
       match *self {
           BLEPduType::ConnectUndirected(a, _) => a,
           BLEPduType::ConnectDirected(a, _) => a,
           BLEPduType::NonConnectUndirected(a, _) => a,
           BLEPduType::ScanUndirected(a, _) => a,
           BLEPduType::ScanRequest(a, _) => a,
           BLEPduType::ScanResponse(a, _) => a,
           BLEPduType::ConnectRequest(a, _, _) => a,
       }
    }
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
#[allow(unused)]
#[repr(u8)]
#[derive(Debug)]
pub enum BLEAdvertisementType {
    ConnectUndirected = 0x00,
    ConnectDirected = 0x01,
    NonConnectUndirected = 0x02,
    ScanRequest = 0x03,
    ScanResponse = 0x04,
    ConnectRequest = 0x05,
    ScanUndirected = 0x06,
}

impl BLEAdvertisementType {
    pub fn from_u8(pdu_type: u8) -> Option<BLEAdvertisementType> {
        match pdu_type {
            0x00 => Some(BLEAdvertisementType::ConnectUndirected),
            0x01 => Some(BLEAdvertisementType::ConnectDirected),
            0x02 => Some(BLEAdvertisementType::NonConnectUndirected),
            0x03 => Some(BLEAdvertisementType::ScanRequest),
            0x04 => Some(BLEAdvertisementType::ScanResponse),
            0x05 => Some(BLEAdvertisementType::ConnectRequest),
            0x06 => Some(BLEAdvertisementType::ScanUndirected),
            _ => None
        }
    }
}

const PACKET_START: usize = 0;
const PACKET_HDR_PDU: usize = 0;
const PACKET_HDR_LEN: usize = 1;
const PACKET_ADDR_START: usize = 2;
const PACKET_ADDR_END: usize = 7;
const PACKET_PAYLOAD_START: usize = 8;
const PACKET_LENGTH: usize = 39;

#[derive(PartialEq, Debug)]
enum BLEState {
    NotInitialized,
    Initialized,
    ScanningIdle,
    Scanning(RadioChannel),
    Listening(RadioChannel),
    Requesting(RadioChannel),
    Responding(RadioChannel),
    AdvertisingIdle,
    Advertising(RadioChannel),
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

pub struct App {
    advertising_address: Option<DeviceAddress>,
    advertisement_buf: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_write: Option<kernel::AppSlice<kernel::Shared, u8>>,
    app_read: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
    idx: usize,
    process_status: Option<BLEState>,
    advertisement_interval_ms: u32,
    scan_timeout_ms: u32,
    alarm_data: AlarmData,
    tx_power: u8,
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
            process_status: Some(BLEState::NotInitialized),
            tx_power: 0,
            advertisement_interval_ms: 200,
            scan_timeout_ms: 10,
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

        let random_address: [u8; 6] = [0xf0, 0x11, 0x11, ((appid.idx() << 16) as u8 & 0xff), ((appid.idx() << 24) as u8 & 0xff), 0xf0];
        //let random_address: [u8; 6] = [0xf0, 0x0f, 0x0f, ((appid.idx() << 16) as u8 & 0xff), ((appid.idx() << 24) as u8 & 0xff), 0xf0];
        self.advertising_address = Some(DeviceAddress::new(&random_address));

        self.advertisement_buf
            .as_mut()
            .map_or(ReturnCode::ESIZE,|data| {
                data.as_mut()[PACKET_HDR_LEN] = 6;
                for i in 0..6 {
                    data.as_mut()[PACKET_ADDR_START + i] = random_address[i];
                }
                ReturnCode::SUCCESS
            })

        /*
        self.scan_response_buf
            .as_mut()
            .map_or(ReturnCode::ESIZE,|data| {
                data.as_mut()[PACKET_HDR_LEN] = 6;
                for i in 0..6 {
                    data.as_mut()[PACKET_ADDR_START + i] = random_address[i];
                }
                ReturnCode::SUCCESS
            })
         */

    }

    fn reset_payload(&mut self) -> ReturnCode {
        match self.process_status {
            Some(BLEState::Advertising(_)) | Some(BLEState::Scanning(_)) => ReturnCode::EBUSY,
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
                slice.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (BLEAdvertisementType::ConnectUndirected as u8);   //<-- vill sätta Tx om vi advertisar med random address
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

    fn send_buffer<'a, B, A>(&self, ble: &BLE<'a, B, A>, slice: &'static mut [u8],  advertisement_type: BLEAdvertisementType, channel: RadioChannel) -> ReturnCode
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {
        slice.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (advertisement_type as u8);

        let result = ble.radio.transmit_advertisement(slice, PACKET_LENGTH, channel);
        ble.kernel_tx.replace(result);
        ReturnCode::SUCCESS
    }


    fn send_advertisement<'a, B, A>(&self, ble: &BLE<'a, B, A>, channel: RadioChannel) -> ReturnCode
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
    {

        debug!("Sending advertisement");

        self.advertisement_buf
            .as_ref()
            .map(|slice| {
                ble.kernel_tx
                    .take()
                    .map(|data| {
                        for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                            .iter_mut()
                            .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                        self.send_buffer(ble, data, BLEAdvertisementType::ConnectUndirected, channel)
                    })
                    .unwrap_or_else(|| ReturnCode::EINVAL)
            })
            .unwrap_or_else(|| ReturnCode::EINVAL)
    }

    fn send_scan_response<'a, B, A>(&self, ble: &BLE<'a, B, A>, channel: RadioChannel) -> ReturnCode
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {
        self.advertisement_buf
            .as_ref()
            .map_or(ReturnCode::EINVAL,|slice| {
                ble.kernel_tx
                    .take()
                    .map_or(ReturnCode::EINVAL,|data| {
                        for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                            .iter_mut()
                            .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                            {
                                *out = *inp;
                            }
                        self.send_buffer(ble, data, BLEAdvertisementType::ScanResponse, channel)
                    })
            })
    }

    fn send_scan_request<'a, B, A>(&mut self, ble: &BLE<'a, B, A>, adv_addr: DeviceAddress, channel: RadioChannel) -> ReturnCode
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {

        debug!("Sending ScanRequest to {:?} on channel {:?}", adv_addr, channel);

        self.advertisement_buf
            .as_ref()
            .map(|slice| {
                ble.kernel_tx
                    .take()
                    .map(|data| {
                        for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                            .iter_mut()
                            .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                            {
                                *out = *inp;
                            }
                        data.as_mut()[PACKET_HDR_LEN] = 12;
                        for i in 0..6 {
                            data.as_mut()[PACKET_ADDR_START+ 6 + i] = adv_addr.0[i];
                        }
                        self.send_buffer(ble, data, BLEAdvertisementType::ScanRequest, channel)
                    })
                    .unwrap_or_else(|| ReturnCode::EINVAL)
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

    fn set_next_alarm_ms<F: Frequency>(&mut self, now: u32) {
        self.alarm_data.t0 = now;

        let period_ms = self.scan_timeout_ms * F::frequency() / 1000;

        self.alarm_data.expiration = Expiration::Abs(now.wrapping_add(period_ms));
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
                    if let BusyState::Busy(busy_app_id) = self.busy.get() {
                        if busy_app_id != app.appid() {
                            // The radio is currently busy, so we won't be able to start the
                            // operation at the appropriate time. Instead, reschedule the
                            // operation for later. This is _kind_ of simulating actual
                            // on-air interference
                            debug!("BLE: operationg delayed for app {:?}", app.appid());
                            app.set_next_alarm::<A::Frequency>(self.alarm.now());
                            return;
                        }
                    }

                    //debug!("Timer fired! {:?}", app.process_status);

                    match app.process_status {
                        Some(BLEState::Listening(channel)) => { // Listening for SCAN_REQ, if timeout - resume advertising on next channel
                            self.sending_app.set(Some(app.appid()));
                            if let Some(channel) = channel.get_next_advertising_channel() {
                                app.process_status = Some(BLEState::Advertising(channel));
                                self.sending_app.set(Some(app.appid()));
                                self.radio.set_tx_power(app.tx_power);
                                app.send_advertisement(&self, channel);

                                //debug!("Advertisement sent on channel: {:?}", channel);

                                // TODO - check correct timeout
                                app.set_next_alarm_ms::<A::Frequency>(self.alarm.now());
                            } else {
                                self.busy.set(BusyState::Free);
                                app.process_status = Some(BLEState::AdvertisingIdle);
                                app.set_next_alarm::<A::Frequency>(self.alarm.now());

                                //debug!("I'm idle now");
                            }
                        }
                        Some(BLEState::AdvertisingIdle) => {
                            self.busy.set(BusyState::Busy(app.appid()));
                            app.process_status = Some(BLEState::Advertising(RadioChannel::AdvertisingChannel37));
                            self.sending_app.set(Some(app.appid()));
                            self.radio.set_tx_power(app.tx_power);
                            app.send_advertisement(&self, RadioChannel::AdvertisingChannel37);

                            //debug!("Advertisement sent on channel: {:?}", RadioChannel::AdvertisingChannel37);

                            // TODO - check correct timeout
                            app.set_next_alarm_ms::<A::Frequency>(self.alarm.now());
                        }
                        Some(BLEState::ScanningIdle) => {
                            //debug!("Moving to channel {:?}", RadioChannel::AdvertisingChannel37);

                            self.busy.set(BusyState::Busy(app.appid()));
                            app.process_status =
                                Some(BLEState::Scanning(RadioChannel::AdvertisingChannel37));
                            self.receiving_app.set(Some(app.appid()));
                            self.radio.set_tx_power(app.tx_power);
                            self.radio
                                .receive_advertisement(RadioChannel::AdvertisingChannel37);

                            app.set_next_alarm_ms::<A::Frequency>(self.alarm.now());
                        } Some(BLEState::Scanning(channel)) => {

                            if let Some(channel) = channel.get_next_advertising_channel() {
                                //debug!("Moving to channel {:?}", channel);
                                app.process_status =
                                    Some(BLEState::Scanning(channel));
                                self.receiving_app.set(Some(app.appid()));
                                self.radio.set_tx_power(app.tx_power);
                                self.radio.receive_advertisement(channel);

                                app.set_next_alarm_ms::<A::Frequency>(self.alarm.now());
                            } else {
                                //debug!("Moving to idle");
                                app.set_next_alarm::<A::Frequency>(self.alarm.now());
                                self.busy.set(BusyState::Free);
                                app.process_status = Some(BLEState::ScanningIdle);
                            }
                        } Some(BLEState::Requesting(channel)) => {
                            //TODO - shall we handle this case? Can it possible handle?
                            debug!("Fire: I'm in requesting state");
                        }
                        _ => debug!(
                            "app: {:?} \t invalid state {:?}",
                            app.appid(),
                            app.process_status
                        ),
                    }
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
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {

        if let Some(appid) = self.receiving_app.get() {
            let _ = self.app.enter(appid, |app, _| {
                // Validate the received data, because ordinary BLE packets can be bigger than 39
                // bytes we need check for that!
                // Moreover, we use the packet header to find size but the radio reads maximum
                // 39 bytes.
                // Therefore, we ignore payloads with a header size bigger than 39 because the
                // channels 37, 38 and 39 should only be used for advertisements!
                // Packets that are bigger than 39 bytes are likely "Channel PDUs" which should
                // only be sent on the other 37 RadioChannel channels.

                let notify_userland = if len <= PACKET_LENGTH as u8 && app.app_read.is_some()
                    && result == ReturnCode::SUCCESS
                {
                    let dest = app.app_read.as_mut().unwrap();
                    let d = &mut dest.as_mut();
                    // write to buffer in userland
                    for (i, c) in buf[0..len as usize].iter().enumerate() {
                        d[i] = *c;
                    }
                    true
                } else {
                    false
                };

                let parsed_type = BLEAdvertisementType::from_u8(buf[0] & 0x0f);
                let pdu = parsed_type.map(|adv_type| BLEPduType::from_buffer(adv_type, buf) );


/*                if notify_userland {
                    app.scan_callback.map(|mut cb| {
                        cb.schedule(usize::from(result), len as usize, 0);
                    });
                }*/

                //debug!("==receive_event! {:?}", app.process_status);

                match app.process_status {
                    Some(BLEState::Listening(channel)) => {

                        match pdu {
                            Some(BLEPduType::ConnectRequest(init_addr, adv_addr, buf)) => {

                                debug!("Listening on channel {:?} init_addr {:?} adv_addr {:?}", channel, init_addr, adv_addr);

                                app.advertising_address.map(|address| {
                                    if address == adv_addr {
                                        debug!("Connection request! {:?}", buf);
                                    } else {
                                        self.radio.receive_advertisement(channel);
                                        debug!("Connection req not for me: {:?} {:?} {:?}", init_addr, adv_addr, buf);
                                    }
                                });

                            },
                            Some(BLEPduType::ScanRequest(scan_addr, adv_addr)) => {

                                debug!("Listening on channel {:?} scan_addr {:?} adv_addr {:?}", channel, scan_addr, adv_addr);

                                app.advertising_address.map(|address| {


                                    //debug!("address {:?} adv_addr {:?}", address, adv_addr);
                                    //debug!("............... address == adv_addr {}", address == adv_addr);

                                    if address == adv_addr {

                                        app.alarm_data.expiration = Expiration::Disabled;
                                        debug!("ScanRequest for ME! {:?} on channel {:?}", adv_addr, channel);
                                        app.process_status = Some(BLEState::Responding(channel));
                                        self.sending_app.set(Some(app.appid()));
                                        self.radio.set_tx_power(app.tx_power);
                                        app.send_scan_response(&self, channel);
                                    } else {
                                        //app.process_status = Some(BLEState::Listening(channel));
                                        self.receiving_app.set(Some(app.appid()));
                                        self.radio.set_tx_power(app.tx_power);
                                        self.radio.receive_advertisement(channel);
                                        //debug!("Not for me");
                                    }
                                    address
                                });
                            },
                            _ => {
                                //debug!("Listening, not ConnectReq or ScanReq");
                                //app.process_status = Some(BLEState::Listening(channel));
                                self.radio.receive_advertisement(channel);
                                //debug!("Staying on channel {:?}", channel);
                            }

                        }
                    }
                    Some(BLEState::Scanning(channel)) => {

                        if let Some(BLEPduType::ConnectUndirected(adv_addr, _)) = pdu {

                            let tmp_adv_addr = DeviceAddress::new(&[0xf0, 0x0f, 0x0f, 0x0, 0x0, 0xf0]);

                            if adv_addr == tmp_adv_addr {

                                app.alarm_data.expiration = Expiration::Disabled;
                                app.process_status = Some(BLEState::Requesting(channel));
                                self.sending_app.set(Some(app.appid()));
                                self.radio.set_tx_power(app.tx_power);
                                app.send_scan_request(&self, adv_addr, channel);

                                //debug!("adv_addr == tmp_adv_addr: {}", adv_addr == tmp_adv_addr);
                            }
                        } else if let Some(BLEPduType::ScanResponse(adv_addr, _)) = pdu {

                            let tmp_adv_addr = DeviceAddress::new(&[0xf0, 0x0f, 0x0f, 0x0, 0x0, 0xf0]);

                            if adv_addr == tmp_adv_addr {
                                //app.process_status = Some(BLEState::Scanning(channel));
                                //self.sending_app.set(Some(app.appid()));
                                self.receiving_app.set(Some(app.appid()));
                                self.radio.set_tx_power(app.tx_power);
                                //app.send_scan_response(&self, channel);
                                self.radio.receive_advertisement(channel);

                                debug!("Oh, I received a ScanResponse! :D");
                                debug!("Packet: {:?}", pdu);
                            }
                        }
                    }
                    // Invalid state => don't care
                    _ => (),
                }
            });
            self.reset_active_alarm();
        }
    }
    /*
    fn get_address(&self) -> Option<DeviceAddress> {
        self.receiving_app.get().and_then(|appid| self.app.enter(appid, |app, _| app.advertising_address).ok()).and_then(|address| address)
    }
    */
}

// Callback from the radio once a TX event occur
impl<'a, B, A> ble_advertising_hil::TxClient for BLE<'a, B, A>
where
    B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
    A: kernel::hil::time::Alarm + 'a,
{
    // The ReturnCode indicates valid CRC or not, not used yet but could be used for
    // re-tranmissions for invalid CRCs
    fn transmit_event(&self, _crc_ok: ReturnCode) {
        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {

                //debug!("==transmit_event! {:?}" , app.process_status);

                match app.process_status {
                    Some(BLEState::Requesting(channel)) => {
                        app.process_status = Some(BLEState::Scanning(channel));
                        self.receiving_app.set(Some(app.appid()));
                        self.radio.set_tx_power(app.tx_power);
                        self.radio.receive_advertisement(channel);

                        app.set_next_alarm_ms::<A::Frequency>(self.alarm.now());

                        debug!("Request was sent on channel: {:?}", channel);
                    }
                    Some(BLEState::Responding(channel)) => {
                        if let Some(channel) = channel.get_next_advertising_channel() {
                            app.process_status = Some(BLEState::Advertising(channel));
                            self.sending_app.set(Some(app.appid()));
                            self.radio.set_tx_power(app.tx_power);
                            app.send_advertisement(&self, channel);

                            debug!("Sending ScanRespones on channel {:?}", channel);

                            // TODO - check correct timeout
                            app.set_next_alarm_ms::<A::Frequency>(self.alarm.now());
                        } else {
                            self.busy.set(BusyState::Free);
                            app.process_status = Some(BLEState::AdvertisingIdle);

                            app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        }
                    }
                    Some(BLEState::Advertising(ch)) => {

                        app.process_status = Some(BLEState::Listening(ch));
                        self.receiving_app.set(Some(app.appid()));
                        self.radio.set_tx_power(app.tx_power);
                        self.radio.receive_advertisement(ch);

                        debug!("Sent, now listening on channel {:?}", ch)
                    }
                    // Invalid state => don't care
                    _ => {
                        debug!("Invalid state (transmit_event()). State: {:?}", app.process_status);
                    },
                }
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
            // Start periodic advertisments
            0 => self.app
                .enter(appid, |app, _| {
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::AdvertisingIdle);
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
                    Some(BLEState::AdvertisingIdle) | Some(BLEState::ScanningIdle) => {
                        app.process_status = Some(BLEState::Initialized);
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
                        if app.process_status != Some(BLEState::ScanningIdle)
                            && app.process_status != Some(BLEState::AdvertisingIdle)
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
            // The advertisment interval shall an integer multiple of 0.625ms in the range of
            // 20ms to 10240 ms!
            //
            // data - advertisement interval in ms
            // FIXME: add check that data is a multiple of 0.625
            3 => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::Scanning(_)) | Some(BLEState::Advertising(_)) => {
                        ReturnCode::EBUSY
                    }
                    _ => {
                        app.advertisement_interval_ms = cmp::max(20, cmp::min(10240, data as u32));
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
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::ScanningIdle);
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
                    if let Some(BLEState::Initialized) = app.process_status {

                        let status = app.generate_random_address(appid);
                        debug!("random address!");
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
                    if app.process_status != Some(BLEState::NotInitialized) {
                        app.app_write = Some(slice);
                        app.set_gap_data(gap_type);
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),

            Some(AllowType::PassiveScanning) => self.app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::NotInitialized) | Some(BLEState::Initialized) => {
                        app.app_read = Some(slice);
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EINVAL,
                })
                .unwrap_or_else(|err| err.into()),

            Some(AllowType::InitAdvertisementBuffer) => self.app
                .enter(appid, |app, _| {
                    if let Some(BLEState::NotInitialized) = app.process_status {
                        app.advertisement_buf = Some(slice);

                        app.process_status = Some(BLEState::Initialized);
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
                    Some(BLEState::NotInitialized) | Some(BLEState::Initialized) => {
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
