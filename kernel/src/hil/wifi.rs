use crate::ErrorCode;

#[derive(Copy, Clone)]
pub enum Security {
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
}

pub enum Status {
    Connected(Network),
    Connecting(Network),
    Disconnected,
    Disconnecting,
}

#[derive(Copy, Clone)]
pub struct Network {
    pub ssid: [u8; 32],
    pub rssi: u32,
    pub security: Option<Security>,
}

pub trait Controller {
    fn connect(&self, network: Network) -> Result<(), ErrorCode>;
    fn disconnect(&self) -> Result<(), ErrorCode>;

    fn get_status(&self) -> Status;
}

pub trait Scanner<'a> {
    fn scan(&self) -> Result<(), (ErrorCode, &'a [Network])>;
}

pub trait ControllerClient {
    fn command_complete(&self, network: Network, status: Result<(), ErrorCode>);
}

pub trait ScannerClient {
    fn scan_done(&self, networks: &[Network], len: usize, status: Result<(), ErrorCode>);
}
