// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Bluetooth Low Energy HIL
//!
//! ```text
//! Application
//!
//!           +------------------------------------------------+
//!           | Applications                                   |
//!           +------------------------------------------------+
//!
//! ```
//!
//! ```text
//! Host
//!
//!           +------------------------------------------------+
//!           | Generic Access Profile                         |
//!           +------------------------------------------------+
//!
//!           +------------------------------------------------+
//!           | Generic Attribute Profile                      |
//!           +------------------------------------------------+
//!
//!           +--------------------+      +-------------------+
//!           | Attribute Protocol |      | Security Manager  |
//!           +--------------------+      +-------------------+
//!
//!           +-----------------------------------------------+
//!           | Logical Link and Adaptation Protocol          |
//!           +-----------------------------------------------+
//!
//! ```
//!
//! ```text
//! Controller
//!
//!           +--------------------------------------------+
//!           | Host Controller Interface                  |
//!           +--------------------------------------------+
//!
//!           +------------------+      +------------------+
//!           | Link Layer       |      | Direct Test Mode |
//!           +------------------+      +------------------+
//!
//!           +--------------------------------------------+
//!           | Physical Layer                             |
//!           +--------------------------------------------+
//!
//! ```

use crate::ErrorCode;

pub trait BleAdvertisementDriver<'a> {
    fn transmit_advertisement(&self, buf: &'static mut [u8], len: usize, channel: RadioChannel);
    fn receive_advertisement(&self, channel: RadioChannel);
    fn set_receive_client(&self, client: &'a dyn RxClient);
    fn set_transmit_client(&self, client: &'a dyn TxClient);
}

pub trait BleConfig {
    fn set_tx_power(&self, power: u8) -> Result<(), ErrorCode>;
}

pub trait RxClient {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: Result<(), ErrorCode>);
}

pub trait TxClient {
    fn transmit_event(&self, buf: &'static mut [u8], result: Result<(), ErrorCode>);
}

// Bluetooth Core Specification:Vol. 6. Part B, section 1.4.1 Advertising and Data Channel Indices
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    DataChannel0 = 4,
    DataChannel1 = 6,
    DataChannel2 = 8,
    DataChannel3 = 10,
    DataChannel4 = 12,
    DataChannel5 = 14,
    DataChannel6 = 16,
    DataChannel7 = 18,
    DataChannel8 = 20,
    DataChannel9 = 22,
    DataChannel10 = 24,
    DataChannel11 = 28,
    DataChannel12 = 30,
    DataChannel13 = 32,
    DataChannel14 = 34,
    DataChannel15 = 36,
    DataChannel16 = 38,
    DataChannel17 = 40,
    DataChannel18 = 42,
    DataChannel19 = 44,
    DataChannel20 = 46,
    DataChannel21 = 48,
    DataChannel22 = 50,
    DataChannel23 = 52,
    DataChannel24 = 54,
    DataChannel25 = 56,
    DataChannel26 = 58,
    DataChannel27 = 60,
    DataChannel28 = 62,
    DataChannel29 = 64,
    DataChannel30 = 66,
    DataChannel31 = 68,
    DataChannel32 = 70,
    DataChannel33 = 72,
    DataChannel34 = 74,
    DataChannel35 = 76,
    DataChannel36 = 78,
    AdvertisingChannel37 = 2,
    AdvertisingChannel38 = 26,
    AdvertisingChannel39 = 80,
}

impl RadioChannel {
    pub fn get_channel_index(&self) -> u32 {
        match *self {
            Self::DataChannel0 => 0,
            Self::DataChannel1 => 1,
            Self::DataChannel2 => 2,
            Self::DataChannel3 => 3,
            Self::DataChannel4 => 4,
            Self::DataChannel5 => 5,
            Self::DataChannel6 => 6,
            Self::DataChannel7 => 7,
            Self::DataChannel8 => 8,
            Self::DataChannel9 => 9,
            Self::DataChannel10 => 10,
            Self::DataChannel11 => 11,
            Self::DataChannel12 => 12,
            Self::DataChannel13 => 13,
            Self::DataChannel14 => 14,
            Self::DataChannel15 => 15,
            Self::DataChannel16 => 16,
            Self::DataChannel17 => 17,
            Self::DataChannel18 => 18,
            Self::DataChannel19 => 19,
            Self::DataChannel20 => 20,
            Self::DataChannel21 => 21,
            Self::DataChannel22 => 22,
            Self::DataChannel23 => 23,
            Self::DataChannel24 => 24,
            Self::DataChannel25 => 25,
            Self::DataChannel26 => 26,
            Self::DataChannel27 => 27,
            Self::DataChannel28 => 28,
            Self::DataChannel29 => 29,
            Self::DataChannel30 => 30,
            Self::DataChannel31 => 31,
            Self::DataChannel32 => 32,
            Self::DataChannel33 => 33,
            Self::DataChannel34 => 34,
            Self::DataChannel35 => 35,
            Self::DataChannel36 => 36,
            Self::AdvertisingChannel37 => 37,
            Self::AdvertisingChannel38 => 38,
            Self::AdvertisingChannel39 => 39,
        }
    }
}
