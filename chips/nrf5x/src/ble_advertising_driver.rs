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
use ble_connection::ConnectionData;
use core::cell::Cell;
use core::cmp;
use core::fmt;
use kernel;
use kernel::hil::time::Frequency;
use kernel::returncode::ReturnCode;
use ble_event_handler::BLEEventHandler;
use ble_event_handler::BLESender;

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

macro_rules! set_hop_and_sca {
	($hop:expr, $sca:expr) => {{
		if $hop >= 0 && $hop < 0b100000 && $sca >= 0 && $sca < 0b1000{
			Some($hop | ($sca << 5))
		} else {
			None
		}
	}};
}


pub struct LLData {
    aa: [u8; 4],
    crc_init: [u8; 3],
    pub win_size: u8,
    pub win_offset: u16,
    pub interval: u16,
    pub latency: u16,
    pub timeout: u16,
    pub chm: [u8; 5],
    pub hop_and_sca: u8 // hops 5 bits, sca 3 bits
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
            hop_and_sca: (1 << 5) | 15  // = 0010 1111
        }
    }

    fn write_to_buffer(&self, buffer: &mut [u8]) {

        buffer[PACKET_ADDR_START + 12] = self.aa[3]; //aa
        buffer[PACKET_ADDR_START + 13] = self.aa[2]; //aa
        buffer[PACKET_ADDR_START + 14] = self.aa[1]; //aa
        buffer[PACKET_ADDR_START + 15] = self.aa[0]; //aa
        buffer[PACKET_ADDR_START + 16] = self.crc_init[2]; //crcinit
        buffer[PACKET_ADDR_START + 17] = self.crc_init[1]; //crcinit
        buffer[PACKET_ADDR_START + 18] = self.crc_init[0]; //crcinit
        buffer[PACKET_ADDR_START + 19] = self.win_size; //winsize
        buffer[PACKET_ADDR_START + 20] = ((self.win_offset & 0xFF00) >> 8) as u8; //winoffset
        buffer[PACKET_ADDR_START + 21] = (self.win_offset & 0x00FF) as u8; //winoffset
        buffer[PACKET_ADDR_START + 22] = ((self.interval & 0xFF00) >> 8) as u8; //interval
        buffer[PACKET_ADDR_START + 23] = (self.interval & 0x00FF) as u8; //interval
        buffer[PACKET_ADDR_START + 24] = ((self.latency & 0xFF00) >> 8) as u8; //latency
        buffer[PACKET_ADDR_START + 25] = (self.latency & 0x00FF) as u8; //latency
        buffer[PACKET_ADDR_START + 26] = ((self.timeout & 0xFF00) >> 8) as u8; //timeout
        buffer[PACKET_ADDR_START + 27] = (self.timeout & 0x00FF) as u8; //timeout
        buffer[PACKET_ADDR_START + 28] = self.chm[0]; //chm
        buffer[PACKET_ADDR_START + 29] = self.chm[1]; //chm
        buffer[PACKET_ADDR_START + 30] = self.chm[2]; //chm
        buffer[PACKET_ADDR_START + 31] = self.chm[3]; //chm
        buffer[PACKET_ADDR_START + 32] = self.chm[4]; //chm
        buffer[PACKET_ADDR_START + 33] = self.hop_and_sca; //hop, sca

    }


    fn read_from_buffer(buffer: &[u8]) -> LLData {
        LLData {
            aa: [buffer[PACKET_ADDR_START + 15],buffer[PACKET_ADDR_START + 14], buffer[PACKET_ADDR_START + 13], buffer[PACKET_ADDR_START + 12] ],
            crc_init: [buffer[PACKET_ADDR_START + 18], buffer[PACKET_ADDR_START + 17], buffer[PACKET_ADDR_START + 16]],
            win_size: buffer[PACKET_ADDR_START + 19],
            win_offset: (buffer[PACKET_ADDR_START + 20] as u16) << 8 | buffer[PACKET_ADDR_START + 21] as u16,
            interval: (buffer[PACKET_ADDR_START + 22] as u16) << 8 | buffer[PACKET_ADDR_START + 23] as u16,
            latency: (buffer[PACKET_ADDR_START + 24] as u16) << 8 | buffer[PACKET_ADDR_START + 25] as u16,
            timeout: (buffer[PACKET_ADDR_START + 26] as u16) << 8 | buffer[PACKET_ADDR_START + 27] as u16,
            chm: [buffer[PACKET_ADDR_START + 28], buffer[PACKET_ADDR_START + 29], buffer[PACKET_ADDR_START + 30], buffer[PACKET_ADDR_START + 31], buffer[PACKET_ADDR_START + 32]],
            hop_and_sca: buffer[PACKET_ADDR_START + 33]
        }
    }
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
pub enum BusyState {
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
    ConnectRequest(DeviceAddress, DeviceAddress, LLData)
}

impl <'a> BLEPduType<'a> {
    pub fn from_buffer(pdu_type: BLEAdvertisementType, buf: &[u8]) -> Option<BLEPduType> {
        if buf[PACKET_HDR_LEN] < 6 {
            //debug!("This is the buffer {:?}", buf);

            None
        } else {
            let s = match pdu_type {
                BLEAdvertisementType::ConnectUndirected => BLEPduType::ConnectUndirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), &buf[PACKET_PAYLOAD_START..]),
                BLEAdvertisementType::ConnectDirected => BLEPduType::ConnectDirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14])),
                BLEAdvertisementType::NonConnectUndirected => BLEPduType::NonConnectUndirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), &buf[PACKET_PAYLOAD_START..]),
                BLEAdvertisementType::ScanUndirected => BLEPduType::ScanUndirected(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), &buf[PACKET_PAYLOAD_START..]),
                BLEAdvertisementType::ScanRequest => BLEPduType::ScanRequest(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14])),
                BLEAdvertisementType::ScanResponse => BLEPduType::ScanResponse(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), &[]), //  &buf[PACKET_PAYLOAD_START..(buf[PACKET_HDR_LEN] + PACKET_PAYLOAD_START as u8 - 6) as usize]),
                //BLEAdvertisementType::ConnectRequest => BLEPduType::ConnectRequest(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14]), &buf[14..]),
                BLEAdvertisementType::ConnectRequest => BLEPduType::ConnectRequest(DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]), DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14]), LLData::read_from_buffer(&buf[..])),
            };


            Some(s)
        }
    }

    pub fn address(&self) -> DeviceAddress {
       match *self {
           BLEPduType::ConnectUndirected(a, _) => a,
           BLEPduType::ConnectDirected(a, _) => a,
           BLEPduType::NonConnectUndirected(a, _) => a,
           BLEPduType::ScanUndirected(a, _) => a,
           BLEPduType::ScanRequest(_, a) => a,
           BLEPduType::ScanResponse(a, _) => a,
           BLEPduType::ConnectRequest(_, a, _) => a,
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
#[derive(Debug, PartialEq)]
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

const NBR_PACKETS: usize = 2;

#[derive(PartialEq, Debug, Copy, Clone)]
enum BLEState {
    NotInitialized,
    Initialized,
    Scanning(BLEScanningState),
    Advertising(BLEAdvertisingState),
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum BLEAdvertisingState {
    Idle,
    Advertising(RadioChannel),
    Listening(RadioChannel),
    Responding(RadioChannel),
    Connection(RadioChannel, ConnectionData),
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum BLEScanningState {
    Idle,
    Scanning(RadioChannel),
    Requesting(RadioChannel),
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
    timing_array: [(u32, TimestampType); NBR_PACKETS],
    array_ptr: usize,
    log_array: [[u8; PACKET_LENGTH]; NBR_PACKETS],
    //log_array: [u8; NBR_PACKETS],
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
            scan_timeout_ms: 100,
            // Just use any non-zero starting value by default
            random_nonce: 0xdeadbeef,
            timing_array: [(0, TimestampType::BeforeIdle); NBR_PACKETS],
            array_ptr: 0,
            log_array: [[0; PACKET_LENGTH]; NBR_PACKETS],
            //log_array: [0; NBR_PACKETS]
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum TimestampType {
    BeforeAdv(RadioChannel),
    AfterAdv(RadioChannel),
    BeforeList(RadioChannel),
    AfterList(RadioChannel),
    BeforeIdle,
    ScanRequest(RadioChannel),
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

        // debug!("random address!, {:?}", self.advertising_address);

        self.advertisement_buf
            .as_mut()
            .map_or(ReturnCode::ESIZE,|data| {
                data.as_mut()[PACKET_HDR_LEN] = 6;
                for i in 0..6 {
                    data.as_mut()[PACKET_ADDR_START + i] = random_address[i];
                }
                ReturnCode::SUCCESS
            })
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


    fn send_advertisement(&self, ble: &BLESender, channel: RadioChannel, appid: kernel::AppId) -> ReturnCode {

        //debug!("\nSending on channel {:?}", channel);

        self.advertisement_buf
            .as_ref()
            .map_or(ReturnCode::EINVAL, |slice| {
                ble.transmit_buffer_edit(PACKET_LENGTH, channel, appid, &|data: &mut [u8]| {
                    for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                        .iter_mut()
                        .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                    data.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (BLEAdvertisementType::ConnectUndirected as u8);
                });

                ReturnCode::SUCCESS
            })
    }

    fn send_scan_response(&self, ble: &BLESender, channel: RadioChannel, appid: kernel::AppId) -> ReturnCode {
        self.advertisement_buf
            .as_ref()
            .map_or(ReturnCode::EINVAL,|slice| {
                ble.transmit_buffer_edit(PACKET_LENGTH, channel, appid, &|data: &mut [u8]| {
                    for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                        .iter_mut()
                        .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                    data.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (BLEAdvertisementType::ScanResponse as u8);
                });

                ReturnCode::SUCCESS
            })
    }

    fn send_scan_request(&mut self, ble: &BLESender, adv_addr: DeviceAddress, channel: RadioChannel, appid: kernel::AppId) -> ReturnCode {

        // debug!("Sending ScanRequest to {:?}", adv_addr);

        self.advertisement_buf
            .as_ref()
            .map(|slice| {
                ble.transmit_buffer_edit(PACKET_LENGTH, channel, appid, &|data: &mut [u8]| {
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
                    data.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (BLEAdvertisementType::ScanRequest as u8);
                });

                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| ReturnCode::EINVAL)
    }

    fn send_connect_request(&mut self, ble: &BLESender, adv_addr: DeviceAddress, channel: RadioChannel, lldata: LLData, appid: kernel::AppId) -> ReturnCode {

        // debug!("Sending ConnectRequest to {:?} on channel {:?}", adv_addr, channel);

        self.advertisement_buf
            .as_ref()
            .map(|slice| {
                ble.transmit_buffer_edit(PACKET_LENGTH, channel, appid, &|data: &mut [u8]| {
                    for (out, inp) in data.as_mut()[PACKET_HDR_PDU..PACKET_LENGTH]
                        .iter_mut()
                        .zip(slice.as_ref()[PACKET_HDR_PDU..PACKET_LENGTH].iter())
                        {
                            *out = *inp;
                        }
                    data.as_mut()[PACKET_HDR_LEN] = 34;
                    for i in 0..6 {
                        data.as_mut()[PACKET_ADDR_START + 6 + i] = adv_addr.0[i];
                    }
                    lldata.write_to_buffer(data);
                    data.as_mut()[PACKET_HDR_PDU] = (0x04 << 4) | (BLEAdvertisementType::ConnectRequest as u8);
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

    fn set_next_adv_scan_timeout<F: Frequency>(&mut self, now: u32) {
        self.alarm_data.t0 = now;

        let period_ms = self.scan_timeout_ms * F::frequency() / 1000;

        self.alarm_data.expiration = Expiration::Abs(now.wrapping_add(period_ms));
    }


    fn handle_timer_event<'a, B, A>(&mut self, ble: &BLE<'a, B, A>, appid: kernel::AppId)
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {
        let new_state = self.process_status.map(|ble_state: BLEState| -> BLEState {
            match ble_state {
                BLEState::Advertising(adv_state) => BLEState::Advertising(Advertiser::handle_timer_event::<A>(adv_state, self, ble, appid)),
                BLEState::Scanning(scan_state) => BLEState::Scanning(Scanner::handle_timer_event::<A>(scan_state, self, ble, appid)),

                state => {
                    debug!("Error! handle_timer_event {:?}", state);

                    state
                }
            }
        });

        self.process_status = new_state;
    }

    /*fn collect_log(&mut self, buf: u8, timestamp: u32) {

        //debug!("{}", pdu);

        if self.array_ptr < NBR_PACKETS {

            self.timing_array[self.array_ptr] = timestamp;

            *//*
            for i in 0..PACKET_LENGTH {
                self.log_array[self.array_ptr][i] = buf[i];
            }
            *//*
            self.log_array[self.array_ptr] = buf;

            self.array_ptr += 1;
        } else {

            for i in 0..self.log_array.len() {
                debug!("\n{:?} {:?}", self.timing_array[i], self.log_array[i]);
            }
            *//*let mut conn_req = true;

            for i in 0..self.log_array.len() {

                let pdu = self.log_array[i];

                if (pdu != 0) && (pdu != 2) {
                    debug!("\n{:?} {}: ", self.timing_array[i], pdu);

                    for k in self.log_array[i].iter() {
                        debug!("{:0>2x} ", k);
                    }
                    conn_req = false;
                }
            }

            if conn_req {
                debug!("\n.");
            }*//*
            self.array_ptr = 0;
        }
    }*/

    fn collect_timestamp(&mut self, time: u32, timestamp_type: TimestampType) {
        if self.array_ptr < self.timing_array.len() {
            self.timing_array[self.array_ptr] = (time, timestamp_type);
            self.array_ptr += 1;
        } else {
            debug!("\n\nTiming values:");
            for i in 0..self.timing_array.len() as u8 {
                debug!("\n{:?}", self.timing_array[i as usize]);
            }
            self.array_ptr = 0;
        }
    }

    fn print_log(&mut self, adv_address: &DeviceAddress, channel: RadioChannel) {
        let mut conn_req = false;

        for i in 0..self.log_array.len() {
            let parsed_type = BLEAdvertisementType::from_u8(self.log_array[i][0] & 0x0f);
            let pdu = parsed_type.and_then(|adv_type| BLEPduType::from_buffer(adv_type, &self.log_array[i]) );

            let parsed_type = BLEAdvertisementType::from_u8(self.log_array[i][0] & 0x0f);

            if let Some(pdu) = pdu {

                /*match parsed_type {
                    Some(BLEAdvertisementType::ScanRequest) |
                    Some(BLEAdvertisementType::ScanResponse) |
                    Some(BLEAdvertisementType::ConnectRequest) => {*/
                        // conn_req = true;
                        // debug!("\n{} {:?} {:?} ", self.timing_array[i], parsed_type, channel);
                       /* for c in self.log_array[i].iter() {
                            debug!("{:0>2x} ", c);
                        }*/
                 /*   },
                    Some(_) | None => {}
                }*/
                if pdu.address() == *adv_address {
                    conn_req = true;
                    debug!("\n{} {:?} {:?} ", self.timing_array[i].0, parsed_type, channel);
                    for c in self.log_array[i].iter() {
                        debug!("{:0>2x} ", c);
                    }
                    // debug!("\n");
                }
            }
        }
        if !conn_req {
            // debug!(".");
        }
        self.array_ptr = 0;
    }

    fn handle_rx_event<'a, B, A>(&mut self, ble: &BLE<'a, B, A>, appid: kernel::AppId, buf: &'static mut [u8])
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {


        let parsed_type = BLEAdvertisementType::from_u8(buf[0] & 0x0f);
        let pdu = parsed_type.and_then(|adv_type| BLEPduType::from_buffer(adv_type, buf) );


        let tmp_adv_addr = DeviceAddress::new(&[0xf0, 0x11, 0x11, 0x00, 0x00, 0xf0]);

        /*if let Some(ref pdu) = pdu {
            self.collect_log(&buf, ble.alarm_now());
        }*/


        /*                if notify_userland {
                            app.scan_callback.map(|mut cb| {
                                cb.schedule(usize::from(result), len as usize, 0);
                            });
                        }*/

        if let Some(pdu) = pdu {
            let new_state = self.process_status.map(|ble_state| {
                match ble_state {
                    BLEState::Advertising(adv_state) => BLEState::Advertising(Advertiser::handle_rx_event::<A>(adv_state, self, ble, appid, &pdu)),
                    BLEState::Scanning(scan_state) => BLEState::Scanning(Scanner::handle_rx_event::<A>(scan_state, self, ble, appid, &pdu)),
                    state => {
                        debug!("Error! handle_rx_event {:?}", state);

                        state
                    }
                }
            });
            self.process_status = new_state;
        }

        match self.process_status {
            Some(BLEState::Scanning(BLEScanningState::Scanning(channel))) | Some(BLEState::Scanning(BLEScanningState::Requesting(channel))) => {
                self.print_log(&tmp_adv_addr, channel);
            },

            Some(_) | None => {}
        }
    }

    fn handle_tx_event<'a, B, A>(&mut self, ble: &BLE<'a, B, A>, appid: kernel::AppId)
        where
            B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
            A: kernel::hil::time::Alarm + 'a,
    {

        let new_state = self.process_status.map(|ble_state| {
            match ble_state {
                BLEState::Advertising(adv_state) => BLEState::Advertising(Advertiser::handle_tx_event::<A>(adv_state, self, ble, appid)),
                BLEState::Scanning(scan_state) => BLEState::Scanning(Scanner::handle_tx_event::<A>(scan_state, self, ble, appid)),
                state => {
                    debug!("Error! handle_tx_event {:?}", state);

                    state
                }
            }
        });
        self.process_status = new_state;
    }
}

#[derive(PartialEq)]
struct Advertiser;

impl BLEEventHandler<BLEAdvertisingState> for Advertiser {
    fn handle_rx_event<A>(state: BLEAdvertisingState, app: &mut App, ble: &BLESender, appid: kernel::AppId, pdu: &BLEPduType) -> BLEAdvertisingState where A: kernel::hil::time::Alarm {

        state
        /*
        match state {
            BLEAdvertisingState::Connection(channel, connection_data) => {
                debug!("Yass!!! We got one! {:?}, {:?}", channel, connection_data);

                state
            }
            BLEAdvertisingState::Listening(channel) => {

                match pdu {
                    &BLEPduType::ConnectRequest(init_addr, adv_addr, ref lldata) => {

                        debug!("\nConnection request! {:?} {:?}", init_addr, adv_addr);

                        let mut new_state = state;

                        app.advertising_address.map(|address| {
                            if address == adv_addr {
                                //panic!("Connection req");
                                debug!("Connection request! {:?} {:?}", init_addr, lldata);

                                app.alarm_data.expiration = Expiration::Disabled;

                                let mut conn_data = ConnectionData::new(lldata);

                                ble.set_access_address(lldata.aa);

                                let next_channel = conn_data.next_channel();

                                new_state = BLEAdvertisingState::Connection(next_channel, conn_data);


                                ble.receive_buffer(next_channel, appid);

                            } else {
                                ble.receive_buffer(channel, appid);
                                //debug!("Connection req not for me: {:?} {:?} {:?}", init_addr, adv_addr, lldata);
                            }

                            address
                        });

                        new_state

                    },
                    &BLEPduType::ScanRequest(scan_addr, adv_addr) => {

                        debug!("\n{:?} {:?}", scan_addr, adv_addr);

                        let mut new_state = state;

                        app.advertising_address.map(|address| {

                            ble.set_tx_power(app.tx_power);

                            if address == adv_addr {
                                app.alarm_data.expiration = Expiration::Disabled;
                                new_state = BLEAdvertisingState::Responding(channel);

                                debug!("Got a ScanReq");

                                app.send_scan_response(ble, channel, appid);

                            } else {
                                ble.receive_buffer(channel, appid);
                            }
                            address
                        });

                        new_state
                    },
                    // Received other packet, try receive again!
                    _ => {
                        ble.receive_buffer(channel, appid);

                        state
                    }

                }
            },
            state @ BLEAdvertisingState::Idle => {

                // ignore if rx during Idle
                state
            }
            state => {
                debug!("Error! Adv::handle_rx_event {:?}", state);

                state
            }
        }*/
    }

    fn handle_tx_event<A>(state: BLEAdvertisingState, app: &mut App, ble: &BLESender, appid: kernel::AppId) -> BLEAdvertisingState where A: kernel::hil::time::Alarm {

        debug!("\ntx");

        match state {

            BLEAdvertisingState::Advertising(channel) => {

                ble.set_tx_power(app.tx_power);
                ble.receive_buffer(channel, appid);

                //app.collect_timestamp(ble, TimestampType::AfterList(channel));

                BLEAdvertisingState::Listening(channel)

            },
            BLEAdvertisingState::Responding(channel) => {
                if let Some(channel) = channel.get_next_advertising_channel() {
                    ble.set_tx_power(app.tx_power);
                    app.send_advertisement(ble, channel, appid);

                    // TODO - check correct timeout
                    app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());

                    BLEAdvertisingState::Advertising(channel)
                } else {
                    ble.set_busy(BusyState::Free);

                    app.set_next_alarm::<A::Frequency>(ble.alarm_now());

                    BLEAdvertisingState::Idle
                }
            },
            BLEAdvertisingState::Idle => {
                debug!("I'm Idle in handle_tx_event, this is unprofessional");
                /*ble.set_busy(BusyState::Busy(appid));
                let channel = RadioChannel::AdvertisingChannel37;
                ble.set_tx_power(app.tx_power);
                app.send_advertisement(ble, RadioChannel::AdvertisingChannel37, appid);

                app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());

                BLEAdvertisingState::Advertising(RadioChannel::AdvertisingChannel37)*/
                state
            }
            state => {
                debug!("Error! Adv::handle_tx_event {:?}", state);

                state
            }
        }
    }

    fn handle_timer_event<A>(state: BLEAdvertisingState, app: &mut App, ble: &BLESender, appid: kernel::AppId) -> BLEAdvertisingState where A: kernel::hil::time::Alarm {

        debug!("\ntimer");

        match state {

                BLEAdvertisingState::Listening(channel) => {

                    if let Some(channel) = channel.get_next_advertising_channel() {

                        ble.set_tx_power(app.tx_power);
                        app.send_advertisement(ble, channel, appid);

                        app.collect_timestamp(ble.alarm_now(), TimestampType::AfterAdv(channel));

                        app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());
                        BLEAdvertisingState::Advertising(channel)


                        /*//Remove the below after debugging
                        ble.set_tx_power(app.tx_power);
                        ble.receive_buffer(channel, appid);

                        //app.collect_timestamp(ble, TimestampType::AfterList(channel));

                        BLEAdvertisingState::Listening(channel)*/


                    } else {

                        app.collect_timestamp(ble.alarm_now(), TimestampType::BeforeIdle);


                        ble.set_busy(BusyState::Free);
                        app.set_next_alarm::<A::Frequency>(ble.alarm_now());

                        BLEAdvertisingState::Idle
                    }
                },


                BLEAdvertisingState::Idle => {

                    //debug!("handle_tx_event, Idle -> 37");
                    ble.set_busy(BusyState::Busy(appid));
                    let channel = RadioChannel::AdvertisingChannel37;


                    ble.set_tx_power(app.tx_power);
                    app.send_advertisement(ble, RadioChannel::AdvertisingChannel37, appid);

                    app.collect_timestamp(ble.alarm_now(), TimestampType::BeforeAdv(channel));

                    app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());
                    BLEAdvertisingState::Advertising(RadioChannel::AdvertisingChannel37)
                },
                _ => {
                    debug!(
                        "app: {:?} \t invalid state {:?}",
                        appid,
                        app.process_status
                    );

                    state
                },
            }
    }
}

#[derive(PartialEq)]
struct Scanner;

impl BLEEventHandler<BLEScanningState> for Scanner {
    fn handle_rx_event<A>(state: BLEScanningState, app: &mut App, ble: &BLESender, appid: kernel::AppId, pdu: &BLEPduType) -> BLEScanningState where A: kernel::hil::time::Alarm {

        match state {
            BLEScanningState::Scanning(channel) => {
                match *pdu {
                    BLEPduType::ConnectUndirected(adv_addr, _) => {

                        let tmp_adv_addr = DeviceAddress::new(&[0xf0, 0x0f, 0x0f, 0x0, 0x0, 0xf0]);
                        //debug!("adv_addr: {:?}", adv_addr);

                        if adv_addr == tmp_adv_addr {
                            app.alarm_data.expiration = Expiration::Disabled;
                            ble.set_tx_power(app.tx_power);

                            debug!("ADV_IND from {:?}", adv_addr);
                            let lldata = LLData::new();
                            app.send_connect_request(ble, tmp_adv_addr, channel, lldata, appid);
                            //app.send_scan_request(ble, tmp_adv_addr, channel, appid);

                            BLEScanningState::Requesting(channel)
                        } else {
                            ble.receive_buffer(channel, appid);

                            state
                        }
                    },
                    BLEPduType::ScanResponse(adv_addr, _) => {
                        let tmp_adv_addr = DeviceAddress::new(&[0xf0, 0x0f, 0x0f, 0x0, 0x0, 0xf0]);
                        if adv_addr == tmp_adv_addr {
                            ble.set_tx_power(app.tx_power);
                            ble.receive_buffer(channel, appid);
                        }

                        state
                    }
                    _ => {
                        ble.receive_buffer(channel, appid);

                        state
                    }
                }
            }
            _ => state
        }
    }

    fn handle_tx_event<A>(state: BLEScanningState, app: &mut App, ble: &BLESender, appid: kernel::AppId) -> BLEScanningState where A: kernel::hil::time::Alarm {
        match state {
            BLEScanningState::Requesting(channel) => {
                ble.set_tx_power(app.tx_power);
                ble.receive_buffer(channel, appid);

                app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());

                BLEScanningState::Scanning(channel)
            }
            _ => state
        }
    }

    fn handle_timer_event<A>(state: BLEScanningState, app: &mut App, ble: &BLESender, appid: kernel::AppId) -> BLEScanningState where A: kernel::hil::time::Alarm {

        match state {
            BLEScanningState::Idle => {


                ble.set_busy(BusyState::Busy(appid));

                let channel = RadioChannel::AdvertisingChannel37;

                ble.set_tx_power(app.tx_power);
                ble.receive_buffer(channel, appid);
                app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());


                BLEScanningState::Scanning(channel)
            }
            BLEScanningState::Scanning(channel) => {
                if let Some(channel) = channel.get_next_advertising_channel() {

                    ble.set_tx_power(app.tx_power);
                    ble.receive_buffer(channel, appid);

                    app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());

                    BLEScanningState::Scanning(channel)
                } else {
                    app.set_next_adv_scan_timeout::<A::Frequency>(ble.alarm_now());
                    ble.set_busy(BusyState::Free);

                    BLEScanningState::Idle
                }
            },
            state => {
                debug!("Error! Scanner::handle_timer_event {:?}", state);

                state
            }
        }
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

impl<'a, B, A> BLESender for BLE<'a, B, A>
    where
        B: ble_advertising_hil::BleAdvertisementDriver + ble_advertising_hil::BleConfig + 'a,
        A: kernel::hil::time::Alarm + 'a,
{
    fn transmit_buffer(
        &self,
        buf: &'static mut [u8],
        _len: usize,
        channel: RadioChannel,
        appid: kernel::AppId
    ) {
        self.sending_app.set(Some(appid));
        let result = self.radio.transmit_advertisement(buf, PACKET_LENGTH, channel);
        self.kernel_tx.replace(result);
    }
    fn transmit_buffer_edit(&self, len: usize, channel: RadioChannel, appid: kernel::AppId, edit_buffer: &Fn(&mut [u8]) -> ()) {
        self.kernel_tx.take().map(|buffer| {
            edit_buffer(buffer);
            buffer
        }).map(|slice| self.transmit_buffer(slice, len, channel, appid));
    }

    fn receive_buffer(&self, channel: RadioChannel, appid: kernel::AppId) {
        self.receiving_app.set(Some(appid));
        self.radio.receive_advertisement(channel);
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
    fn set_access_address(&self, address: [u8; 4]) {
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
                    let appid = app.appid();
                    app.handle_timer_event(&self, appid);
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
            let _ = self.app.enter(appid, move |app, _| {
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
                app.handle_rx_event(self, appid, buf);

            });
            self.reset_active_alarm();
        }
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
    fn transmit_event(&self, _crc_ok: ReturnCode) {
        if let Some(appid) = self.sending_app.get() {
            let _ = self.app.enter(appid, |app, _| {

                //debug!("==transmit_event! {:?}" , app.process_status);

                match app.process_status {
                    Some(BLEState::Advertising(BLEAdvertisingState::Advertising(channel))) | Some(BLEState::Advertising(BLEAdvertisingState::Listening(channel))) |
                        Some(BLEState::Advertising(BLEAdvertisingState::Responding(channel))) => {
                    //Some(BLEState::Advertising(BLEAdvertisingState::Advertising(channel))) => {
                        app.collect_timestamp(self.alarm.now(), TimestampType::BeforeList(channel));
                    },

                    Some(_) | None => {}
                }

                app.handle_tx_event(&self, appid);

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
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::Advertising(BLEAdvertisingState::Idle));
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
                    Some(BLEState::Advertising(BLEAdvertisingState::Idle)) | Some(BLEState::Scanning(BLEScanningState::Idle)) => {
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
                        if app.process_status != Some(BLEState::Scanning(BLEScanningState::Idle))
                            && app.process_status != Some(BLEState::Advertising(BLEAdvertisingState::Idle))
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
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::Scanning(state)) if state != BLEScanningState::Idle =>
                        ReturnCode::EBUSY,

                    Some(BLEState::Advertising(state)) if state != BLEAdvertisingState::Idle =>
                        ReturnCode::EBUSY,
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
                    if let Some(BLEState::Initialized) = app.process_status {
                        app.process_status = Some(BLEState::Scanning(BLEScanningState::Idle));
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
