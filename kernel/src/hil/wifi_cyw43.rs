// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Interfaces for CYW43439 WiFi devices

use crate::ErrorCode;

/// Client trait for when the access point is up
pub trait AccessPointClient {
    /// Access point is up and can accept incoming connections
    ///
    /// ## Arguments
    ///
    /// - `err`: Status of the initialisation:
    ///     - `Ok(())` if the device is up as an access point
    ///     - `Err(ErrorCode::FAIL)` if an error occured during setup
    fn started_ap(&self, err: Result<(), ErrorCode>);
}

/// Client trait for when the WiFi is initialized and the device
/// returns the MAC address
pub trait WifiCtrlClient {
    /// WiFi device initialized
    ///
    /// ## Arguments
    ///
    /// - `err`: Status of the initialisation:
    ///     - `Ok([u8; 6])`if the operation was successful. The buffer
    ///     provided contains the MAC address of the device
    ///     - `Err(ErrorCode::FAIL)` if an error occured during initialisation
    fn init_done(&self, err: Result<[u8; 6], ErrorCode>);
}

/// Client trait for when the station completes joining a network
pub trait StationClient {
    /// Join completed successfully or an error occured
    ///
    /// ## Arguments
    ///
    /// - `err`: Status of the join operation:
    ///     - `Ok(())`if the device joined the requested network
    ///     - `Err(ErrorCode::FAIL)` if an error occured while joining the network
    fn join_done(&self, err: Result<(), ErrorCode>);
}

/// Security method
#[derive(Copy, Clone, Debug, Default)]
pub enum Security {
    Wpa,
    Wpa2,
    Wpa3,
    #[default]
    Wpa2Wpa3,
}

/// Maximum SSID length
pub const SSID_SIZE: usize = 32;
/// Maximum passphrase length
pub const PS_SIZE: usize = 63;

/// WiFi SSID
#[derive(Clone, Copy, Debug, Default)]
pub struct Ssid {
    pub buf: [u8; SSID_SIZE],
    pub len: u8,
}

/// WiFi passphrase
#[derive(Clone, Copy, Debug)]
pub struct Passphrase {
    pub buf: [u8; PS_SIZE],
    pub len: u8,
}

impl Default for Passphrase {
    fn default() -> Self {
        Self {
            buf: [0u8; 63],
            len: Default::default(),
        }
    }
}

/// Join or leave a network with WiFi device configured as a station
pub trait Station<'a> {
    /// Set station client
    fn set_client(&self, client: &'a dyn StationClient);

    /// Join a network, either open or protected
    ///
    /// ## Arguments
    ///
    /// - `ssid`: WiFi network SSID
    /// - `security`: Security method to use in order to join the network:
    ///     - `None` if the network is open
    ///     - tuple of the security method and the passphrase buffer
    fn join(&self, ssid: Ssid, security: Option<(Security, Passphrase)>) -> Result<(), ErrorCode>;

    /// Disconnect from the current network
    fn leave(&self) -> Result<(), ErrorCode>;
}

/// Scan networks in range
pub trait Scanner<'a> {
    /// Set scanner client
    fn set_client(&self, client: &'a dyn ScannerClient);

    /// Start scanning the available WiFi networks
    fn start_scan(&self) -> Result<(), ErrorCode>;

    /// Stop scanning
    fn stop_scan(&self) -> Result<(), ErrorCode>;
}

/// Client trait for when the scanner found a network
/// or when the scanning is over
pub trait ScannerClient {
    /// Scanner found a network
    ///
    /// ## Arguments
    ///
    /// - `ssid`: SSID of found network
    fn scanned_network(&self, ssid: Ssid);

    /// Scanner is done
    ///
    /// ## Arguments
    ///
    /// - `err`: Status of the scanning operation:
    ///     - `Ok(())` if the scanning operation finished successfully
    ///     - `Err(ErrorCode::FAIL)` if an error occured while scanning
    fn scan_done(&self, err: Result<(), ErrorCode>);
}

/// Configure and enable/disable the WiFi device as an access point
pub trait AccessPoint<'a> {
    /// Set AP client
    fn set_client(&self, client: &'a dyn AccessPointClient);

    /// Configure and start access point
    ///
    /// ## Arguments
    ///
    /// - `ssid`: The SSID of the configured network
    /// - `security`: Security method used by the network:
    ///     - `None` if the network is open
    ///     - tuple of the security method and the passphrase buffer
    /// - `channel`: 802.11 WLAN channel number
    fn start_ap(
        &self,
        ssid: Ssid,
        security: Option<(Security, Passphrase)>,
        channel: u8,
    ) -> Result<(), ErrorCode>;

    /// Stop access point and return to station mode
    fn stop_ap(&self) -> Result<(), ErrorCode>;
}

/// Initialize the WiFi device and request the MAC address
pub trait WifiCtrl<'a> {
    /// Initialize the device
    fn init(&self) -> Result<(), ErrorCode>;
}
