use ble_connection::ble_link_layer::LLData;
use core::fmt;

#[derive(Debug)]
pub enum BLEPduType<'a> {
    ConnectUndirected(DeviceAddress, &'a [u8]),
    ConnectDirected(DeviceAddress, DeviceAddress),
    NonConnectUndirected(DeviceAddress, &'a [u8]),
    ScanUndirected(DeviceAddress, &'a [u8]),
    ScanRequest(DeviceAddress, DeviceAddress),
    ScanResponse(DeviceAddress, &'a [u8]),
    ConnectRequest(DeviceAddress, DeviceAddress, LLData),
}

impl<'a> BLEPduType<'a> {
    pub fn from_buffer(pdu_type: BLEAdvertisementType, buf: &[u8]) -> Option<BLEPduType> {
        if buf[PACKET_HDR_LEN] < 6 {
            //debug!("This is the buffer {:?}", buf);

            None
        } else {
            let s = match pdu_type {
                BLEAdvertisementType::ConnectUndirected => BLEPduType::ConnectUndirected(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    &buf[PACKET_PAYLOAD_START..],
                ),
                BLEAdvertisementType::ConnectDirected => BLEPduType::ConnectDirected(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14]),
                ),
                BLEAdvertisementType::NonConnectUndirected => BLEPduType::NonConnectUndirected(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    &buf[PACKET_PAYLOAD_START..],
                ),
                BLEAdvertisementType::ScanUndirected => BLEPduType::ScanUndirected(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    &buf[PACKET_PAYLOAD_START..],
                ),
                BLEAdvertisementType::ScanRequest => BLEPduType::ScanRequest(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14]),
                ),
                BLEAdvertisementType::ScanResponse => BLEPduType::ScanResponse(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    &[],
                ),
                BLEAdvertisementType::ConnectRequest => BLEPduType::ConnectRequest(
                    DeviceAddress::new(&buf[PACKET_ADDR_START..PACKET_ADDR_END + 1]),
                    DeviceAddress::new(&buf[PACKET_PAYLOAD_START..14]),
                    LLData::read_from_buffer(&buf[..]),
                ),
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

const SCAN_REQ_LEN: u8 = 12;
const SCAN_IND_MAX_LEN: u8 = 37;
const DEVICE_ADDRESS_LEN: u8 = 6;
const CONNECT_REQ_LEN: u8 = 34;

pub const PACKET_START: usize = 0;
pub const PACKET_HDR_PDU: usize = 0;
pub const PACKET_HDR_LEN: usize = 1;
pub const PACKET_ADDR_START: usize = 2;
pub const PACKET_ADDR_END: usize = 7;
pub const PACKET_PAYLOAD_START: usize = 8;
pub const PACKET_LENGTH: usize = 39;

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
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
            _ => None,
        }
    }

    pub fn validate_pdu(&self, len: u8) -> bool {
        match self {
            &BLEAdvertisementType::ScanRequest | &BLEAdvertisementType::ConnectDirected => {
                len == SCAN_REQ_LEN
            }

            &BLEAdvertisementType::ScanResponse
            | &BLEAdvertisementType::ConnectUndirected
            | &BLEAdvertisementType::ScanUndirected
            | &BLEAdvertisementType::NonConnectUndirected => {
                len >= DEVICE_ADDRESS_LEN && len <= SCAN_IND_MAX_LEN
            }

            &BLEAdvertisementType::ConnectRequest => len == CONNECT_REQ_LEN,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct DeviceAddress(pub [u8; 6]);

impl DeviceAddress {
    pub fn new(slice: &[u8]) -> DeviceAddress {
        let mut address: [u8; 6] = Default::default();
        address.copy_from_slice(slice);
        DeviceAddress(address)
    }
}

impl fmt::Debug for DeviceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}:{:0>2x}",
            self.0[5], self.0[4], self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}
