//! Traits relted to handling of WiFi using WiFiNINA
//!
//! Devices that provide WiFi functionallity are divided into
//! two categories:
//!  - Station - the device is a client of an Access Point
//!  - AccessPoint - the device is an access point that accepts incoming connections
//!
//! Some devices might allow using the WiFi system both as
//! an AccessPoint and a Station at the same time.
//!

use crate::ErrorCode;

#[derive(Copy, Clone)]
pub enum Security {
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
}

pub enum StationStatus {
    // the device is not a station
    // it might be an access point
    Off,
    // the station is connected to the `Network`
    Connected(Network),
    // the station is in the process of connecting to the `Network`
    Connecting(Network),
    // the station is not connected to any network
    Disconnected,
    // the station is disconnecting from a network
    Disconnecting,
}

pub enum AccessPointStatus {
    // the device is not an access point
    // it might be a station
    Off,
    // the access point SSID and Security
    // type have not been yet configured
    NoConfiguration,
    // the access point is in the process of starting and
    // boradcasting the `Network`
    Starting(Network),
    // the access point is started and is boardcasting the `Network`
    Started(Network),
    // the access point is stopped
    Stopped,
    // the access point it in the process of stopping
    Stopping,
}

#[derive(Copy, Clone, Default)]
pub struct Ssid {
    // The max length of an SSID is 32
    pub value: [u8; 32],

    // the actual length of the SSID
    pub len: u8,
}
#[derive(Copy, Clone)]
pub struct Psk {
    // The max length of an SSID is 32
    pub value: [u8; 63],

    // the actual length of the SSID
    pub len: u8,
}

impl Default for Psk {
    fn default() -> Self {
        Psk {
            value: [0; 63],
            len: 0,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Network {
    pub ssid: Ssid,
    // 802.11 defines RSSI as a value from 0 to 255
    pub rssi: u8,
    pub security: Option<Security>,
}

/// Defines the function used for handling WiFi connections as a station
pub trait Station<'a> {
    // try to initiatie a connection to the `Network`
    fn connect(&self, ssid: Ssid, psk: Option<Psk>) -> Result<(), ErrorCode>;
    // try to disconnect from the network that it is currently connected to
    fn disconnect(&self) -> Result<(), ErrorCode>;

    // return the status
    fn get_status(&self) -> Result<(), ErrorCode>;

    fn set_client(&self, client: &'a dyn StationClient);
}

/// Defines the functions used to get information about existing networks
pub trait Scanner<'a> {
    // start scanning the available WiFi networks
    fn scan(&self) -> Result<(), ErrorCode>;

    fn set_client(&self, client: &'a dyn ScannerClient);
}

/// Defines the function used for handling WiFi connections as an access point
pub trait AccessPoint {
    // Sets the SSID and Security type of the access point.
    //
    // This function should be called only when the access point's status
    // is `Stopped`, otherwise it should return `ErrorCode::INVAL`.
    // A successful return means that the SSID and Security type will be set
    // and a call to `command_complete` will follow.
    fn configure(&self, ssid: Ssid, security: Security) -> Result<(), ErrorCode>;

    // Starts the access point
    //
    // This function should be called only when the access point's status
    // is `Stopped`, otherwise it should return:
    //  - `ErrorCode::OFF` if in `Off`
    //  - `ErrorCode::INVAL` if in `NotConfigured` or `Started(_)`
    //  - `ErrorCode::BUSY` if in `Started(_)` or `Stopped(_)`
    // A successful return means that the access point will try to start and
    // a call to `command_complete` will follow.
    fn start(&self) -> Result<(), ErrorCode>;

    // Stops the access point
    //
    // This function should be called only when the access point's status
    // is `Started(_)`, otherwise it should return:
    //  - `ErrorCode::OFF` if in `Off`
    //  - `ErrorCode::INVAL` if in `NotConfigured`or `Stopped(_)`
    //  - `ErrorCode::BUSY` if in `Starting(_)` or `Stopping(_)`
    // A successful return means that the access point will try to start and
    // a call to `command_complete` will follow.
    fn stop(&self) -> Result<(), ErrorCode>;

    // synchronously get status of the access point
    fn get_status(&self) -> AccessPointStatus;
}

pub trait StationClient {
    fn command_complete(&self, status: Result<StationStatus, ErrorCode>);
}

pub trait ScannerClient {
    fn scan_done<'a>(&self, status: Result<&'a [Network], ErrorCode>);
}

pub trait AccessPointClient {
    fn command_complete(&self, status: Result<Network, ErrorCode>);
}
