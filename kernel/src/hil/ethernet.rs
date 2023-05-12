use crate::ErrorCode;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MacAddress {
    address: [u8; 6],
}

impl MacAddress {
    pub fn new() -> Self {
        Self {
            address: [0; 6],
        }
    }

    pub fn set_address(&mut self, address: u64) {
        let mask: u64 = 0xFF0000000000;
        for index in 0..6 {
            self.address[index] = ((address & (mask >> (index * 8))) >> (40 - 8 * index)) as u8;
        }
    }

    pub fn get_address(&self) -> [u8; 6] {
        self.address
    }
}

impl From<u64> for MacAddress {
    fn from(value: u64) -> Self {
        let mut mac_address = MacAddress::new();
        mac_address.set_address(value);
        mac_address
    }
}

impl From<MacAddress> for u64 {
    fn from(mac_address: MacAddress) -> Self {
        let mut result: u64 = 0;
        for byte in mac_address.get_address() {
            result += byte as u64;
            result <<= 8;
        }

        result >> 8
    }
}

#[derive(PartialEq, Debug)]
pub enum OperationMode {
    HalfDuplex = 0b0,
    FullDuplex = 0b1,
}

#[derive(PartialEq, Debug)]
pub enum EthernetSpeed {
    Speed10Mbs = 0b0,
    Speed100Mbs = 0b1,
}

pub trait Configure {
    fn init(&self) -> Result<(), ErrorCode>;

    fn set_operation_mode(&self, _operation_mode: OperationMode) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_operation_mode(&self) -> OperationMode;

    fn set_speed(&self, _speed: EthernetSpeed) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_speed(&self) -> EthernetSpeed;

    fn set_loopback_mode(&self, _enable: bool) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn is_loopback_mode_enabled(&self) -> bool;

    fn set_mac_address(&self, _mac_address: MacAddress) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn get_mac_address(&self) -> MacAddress;
    
    fn start_transmitter(&self) -> Result<(), ErrorCode>;

    fn stop_transmitter(&self) -> Result<(), ErrorCode>;

    fn is_transmitter_up(&self) -> bool;

    fn start_receiver(&self) -> Result<(), ErrorCode>;

    fn stop_receiver(&self) -> Result<(), ErrorCode>;

    fn is_receiver_up(&self) -> bool;

    fn start(&self) -> Result<(), ErrorCode> {
        self.start_transmitter()?;
        self.start_receiver()
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.stop_transmitter()?;
        self.stop_receiver()
    }

    fn is_up(&self) -> bool {
        self.is_transmitter_up() && self.is_receiver_up()
    }
}

pub trait Transmit<'a> {
    fn set_transmit_client(&self, transmit_client: &'a dyn TransmitClient);
    fn transmit_data(&self, destination_address: MacAddress, data: &[u8]) -> Result<(), ErrorCode>;
}

pub trait Receive<'a> {
    fn set_receive_client(&self, receive_client: &'a dyn ReceiveClient);
    fn receive_frame(&self, buffer: &mut [u8]) -> Result<(), ErrorCode>;
}

pub trait TransmitClient {
    fn transmitted_frame(&self, transmit_status: Result<(), ErrorCode>);
}

pub trait ReceiveClient {
    fn received_frame(&self,
        receive_status: Result<(), ErrorCode>,
        received_frame_length: usize
    );
}
