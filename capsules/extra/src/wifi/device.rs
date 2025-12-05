// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Hardware-independent interface for WiFi devices.
//!
//! This interface provides high-level functionalities: scanning for networks, joining a network,
//! configure as access point, get MAC address.

use core::num::NonZeroU8;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::ErrorCode;

/// Maximum lengths for buffers
pub mod len {
    pub const SSID: usize = 32;

    pub const WPA_PASSPHRASE_MIN: usize = 8;
    pub const WPA_PASSPHRASE_MAX: usize = 63;
}

enum_from_primitive!(
    /// Security method
    #[derive(Copy, Clone, Debug)]
    #[non_exhaustive]
    pub enum Security {
        WpaPsk = 1,
        Wpa2Psk = 2,
        Wpa2PskWpa3Sae = 3,
        Wpa3Sae = 4,
    }
);

pub type Ssid = Credential<{ len::SSID }>;
pub type Passphrase = Credential<{ len::WPA_PASSPHRASE_MAX }>;

#[derive(Clone, Copy, Debug)]
pub struct Credential<const LEN: usize> {
    pub len: NonZeroU8,
    pub buf: [u8; LEN],
}

impl<const LEN: usize> Credential<LEN> {
    pub fn try_new(len: u8) -> Result<Self, ErrorCode> {
        if len as usize > LEN {
            return Err(ErrorCode::INVAL);
        }
        let len = NonZeroU8::new(len).ok_or(ErrorCode::INVAL)?;

        Ok(Self {
            len,
            buf: [0u8; LEN],
        })
    }
}

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
        security: Option<(Security, Passphrase)>,
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
    fn join(&self, ssid: Ssid, security: Option<(Security, Passphrase)>) -> Result<(), ErrorCode>;

    /// Disconnect from the current network
    fn leave(&self) -> Result<(), ErrorCode>;

    /// Start scanning the available WiFi networks
    fn scan(&self) -> Result<(), ErrorCode>;

    /// Try to force the device to stop scanning
    fn stop_scan(&self) -> Result<(), ErrorCode>;
}
