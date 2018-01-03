//! Bluetooth Low Energy HIL


//! ```
//! Application
//!
//!           +------------------------------------------------+
//!           | Applications                                   |
//!           +------------------------------------------------+
//!
//! ```
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
//!           | Attribute Protocol |      | Security Mananger |
//!           +--------------------+      +-------------------+
//!
//!           +-----------------------------------------------+
//!           | Logical Link and Adaptation Protocol          |
//!           +-----------------------------------------------+
//!
//! ```
//!
//! ```
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

use kernel::ReturnCode;

pub trait BleAdvertisementDriver {
    fn transmit_advertisement(&self,
                              buf: &'static mut [u8],
                              len: usize,
                              channel: RadioFrequency)
                              -> &'static mut [u8];
    fn receive_advertisement(&self, channel: RadioFrequency);
    fn set_receive_client(&self, client: &'static RxClient);
    fn set_transmit_client(&self, client: &'static TxClient);
}

pub trait BleConfig {
    fn set_tx_power(&self, power: u8) -> ReturnCode;
}

pub trait RxClient {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait TxClient {
    fn transmit_event(&self, result: ReturnCode);
}

// Bluetooth Core Specification:Vol. 6. Part B, section 1.4.1 Advertising and Data Channel Indices
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioFrequency {
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

impl RadioFrequency {
    pub fn get_channel_index(&self) -> u32 {
        match *self {
            RadioFrequency::DataChannel0 => 0,
            RadioFrequency::DataChannel1 => 1,
            RadioFrequency::DataChannel2 => 2,
            RadioFrequency::DataChannel3 => 3,
            RadioFrequency::DataChannel4 => 4,
            RadioFrequency::DataChannel5 => 5,
            RadioFrequency::DataChannel6 => 6,
            RadioFrequency::DataChannel7 => 7,
            RadioFrequency::DataChannel8 => 8,
            RadioFrequency::DataChannel9 => 9,
            RadioFrequency::DataChannel10 => 10,
            RadioFrequency::DataChannel11 => 11,
            RadioFrequency::DataChannel12 => 12,
            RadioFrequency::DataChannel13 => 13,
            RadioFrequency::DataChannel14 => 14,
            RadioFrequency::DataChannel15 => 15,
            RadioFrequency::DataChannel16 => 16,
            RadioFrequency::DataChannel17 => 17,
            RadioFrequency::DataChannel18 => 18,
            RadioFrequency::DataChannel19 => 19,
            RadioFrequency::DataChannel20 => 20,
            RadioFrequency::DataChannel21 => 21,
            RadioFrequency::DataChannel22 => 22,
            RadioFrequency::DataChannel23 => 23,
            RadioFrequency::DataChannel24 => 24,
            RadioFrequency::DataChannel25 => 25,
            RadioFrequency::DataChannel26 => 26,
            RadioFrequency::DataChannel27 => 27,
            RadioFrequency::DataChannel28 => 28,
            RadioFrequency::DataChannel29 => 29,
            RadioFrequency::DataChannel30 => 30,
            RadioFrequency::DataChannel31 => 31,
            RadioFrequency::DataChannel32 => 32,
            RadioFrequency::DataChannel33 => 33,
            RadioFrequency::DataChannel34 => 34,
            RadioFrequency::DataChannel35 => 35,
            RadioFrequency::DataChannel36 => 36,
            RadioFrequency::AdvertisingChannel37 => 37,
            RadioFrequency::AdvertisingChannel38 => 38,
            RadioFrequency::AdvertisingChannel39 => 39,
        }
    }
}
