// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Hardware-independent interface for WiFi devices.
//!
//! This interface provides high-level functionalities: scanning for networks, joining a network,
//! configure as access point, get MAC address.

use kernel::ErrorCode;

/// Maximum lengths for buffers
pub mod len {
    pub const SSID: usize = 32;
    pub const WPA_PASSPHRASE: usize = 64;
    pub const WPA3_PASSPHRASE: usize = 128;
}

/// Security method
#[derive(Copy, Clone, Debug)]
pub enum Security {
    Wpa(WpaPassphrase),
    Wpa2(WpaPassphrase),
    Wpa2Wpa3(Wpa3Passphrase),
    Wpa3(Wpa3Passphrase),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Ssid {
    pub len: u8,
    pub buf: [u8; len::SSID],
}

#[derive(Clone, Copy, Debug)]
pub struct Passphrase<const LEN: usize> {
    pub len: u8,
    pub buf: [u8; LEN],
}

impl<const LEN: usize> Default for Passphrase<LEN> {
    fn default() -> Self {
        Self {
            len: Default::default(),
            buf: [0u8; LEN],
        }
    }
}

pub type WpaPassphrase = Passphrase<{ len::WPA_PASSPHRASE }>;
pub type Wpa3Passphrase = Passphrase<{ len::WPA3_PASSPHRASE }>;

/// Client trait
pub trait Client {
    /// Command is complete. This is an universal callback method
    /// for all the [`Device`] methods except `set_client` and `mac` which
    /// are expected to be synchronous.
    ///
    /// ## Arguments
    ///
    /// - `rval`: Status of the command. `Ok(())` means that it has been completed
    /// successfully. If the command has failed, it will provide an `ErrorCode` to
    /// signal the motive.
    ///
    /// TODO: This maybe should be split between commands. I haven't figured it out yet
    /// on the CYW4343x but we might be able to retrieve from the event status code
    /// why the join failed, for example.
    fn command_done(&self, rval: Result<(), ErrorCode>);

    /// Scanned a network
    ///
    /// ## Arguments
    ///
    /// - `ssid`: The SSID of the scanned network
    ///
    /// TODO: Also add rssi
    fn scanned_network(&self, ssid: Ssid);

    /// The device finished scanning on its own
    fn scan_done(&self);
}

pub trait Device<'a> {
    /// Set client
    fn set_client(&self, client: &'a dyn Client);

    /// Initialize the device
    fn init(&self) -> Result<(), ErrorCode>;

    /// Return the device's MAC address
    fn mac(&self) -> Result<[u8; 6], ErrorCode>;

    /// Configure the device as access point (AP)
    ///
    /// ## Arguments
    ///
    /// - `ssid`: The SSID of the configured network
    /// - `security`: Security method used by the network:
    ///     - `None` if the network is open
    ///     - tuple of the security method and the passphrase buffer
    /// - `channel`: 802.11 WLAN channel number
    fn access_point(
        &self,
        ssid: Ssid,
        security: Option<Security>,
        channel: u8,
    ) -> Result<(), ErrorCode>;

    /// Configure the device as station (STA)
    fn station(&self) -> Result<(), ErrorCode>;

    /// Join a network, either open or protected
    ///
    /// ## Arguments
    ///
    /// - `ssid`: WiFi network SSID
    /// - `security`: Security method to use in order to join the network:
    ///     - `None` if the network is open
    ///     - tuple of the security method and the passphrase buffer
    fn join(&self, ssid: Ssid, security: Option<Security>) -> Result<(), ErrorCode>;

    /// Disconnect from the current network
    fn leave(&self) -> Result<(), ErrorCode>;

    /// Start scanning the available WiFi networks
    fn scan(&self) -> Result<(), ErrorCode>;

    /// Try to force the device to stop scanning
    fn stop_scan(&self) -> Result<(), ErrorCode>;
}
