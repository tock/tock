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
pub struct Network<'a> {
    ssid: [u8; 32],
    rssi: u32,
    security: Option<WiFiSecurity>,
}

pub trait Controller {
    fn connect(&self, network: WiFiNetwork) -> Result<(), ErrorCode>;
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
    fn scan_done(&self, networks: &'a [Network], status: Result<(), ErrorCode>);
}
