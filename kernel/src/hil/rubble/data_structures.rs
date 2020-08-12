//! Simple structs which are relatively directly copied from the `rubble`
//! codebase, but belong in here for interface reasons.
//!
//! All of these structures are represented identically to their `rubble`
//! counterparts. However, most methods are removed: these structures only have
//! most basic methods needed to create and access their internal data, and
//! offer fewer guarantees than their rubble counterparts because of that.
//!
//! These facilitate communication with the interfaces defined in the
//! [`crate::hil::rubble`] module.
use core::convert::{TryFrom, TryInto};

use crate::hil::time::{Frequency, Time};

/// One of 37 data channels on which data channel PDUs are sent between connected devices.
///
/// Copied from `rubble::phy::DataChannel`.
///
/// (channel indices 0..=36)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DataChannel(u8);

impl DataChannel {
    /// Creates a `DataChannel` from a raw index.
    ///
    /// # Errors
    ///
    /// Returns `None` if channel index is not within `0..=36`.
    pub fn new(index: u8) -> Option<Self> {
        match index {
            0..=36 => Some(DataChannel(index)),
            _ => None,
        }
    }

    /// Returns the data channel index.
    ///
    /// The returned value is always in range 0..=36.
    pub fn index(&self) -> u8 {
        self.0
    }
}

/// One of the three advertising channels (channel indices 37, 38 or 39).
///
/// Clone of `rubble::phy::AdvertisingChannel`.
#[derive(Copy, Clone, Debug)]
pub struct AdvertisingChannel(u8);

impl AdvertisingChannel {
    /// Creates an advertising from the given channel index.
    ///
    /// # Errors
    ///
    /// Returns `None` if channel index is not `37`, `38` or `39`.
    pub fn new(idx: u8) -> Option<Self> {
        match idx {
            37..=39 => Some(AdvertisingChannel(idx)),
            _ => None,
        }
    }

    /// Returns the channel index.
    ///
    /// Channels 37, 38 and 39 are used for advertising.
    pub fn channel(&self) -> u8 {
        self.0
    }
}

/// Specifies whether a device address is randomly generated or a LAN MAC address.
///
/// Clone of `rubble::link::device_address::AddressKind`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AddressKind {
    /// Publicly registered IEEE 802-2001 LAN MAC address.
    Public,
    /// Randomly generated address.
    Random,
}

/// A Bluetooth device address.
///
/// Clone of `rubble::link::device_address::DeviceAddress`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DeviceAddress {
    pub bytes: [u8; 6],
    pub kind: AddressKind,
}

/// Specifies when the Link Layer's `update` method should be called the next time.
///
/// Clone of `rubble::link::NextUpdate`.
#[derive(Debug, Clone)]
pub enum NextUpdate {
    /// Disable timer and do not call `update`.
    Disable,

    /// Keep the previously configured time.
    Keep,

    /// Call `update` at the given time.
    ///
    /// If time is in the past, this is a bug and the implementation may panic.
    At(Instant),
}

/// Values of the LLID field in `DataHeader`.
///
/// Clone of `rubble::link::data::Llid`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Llid {
    /// Reserved for future use.
    Reserved = 0b00,

    /// Continuation of L2CAP message, or empty PDU.
    DataCont = 0b01,

    /// Start of L2CAP message.
    DataStart = 0b10,

    /// LL control PDU.
    Control = 0b11,
}

/// 16-bit Advertising Channel PDU header preceding the Payload.
///
/// The header looks like this:
///
/// ```notrust
/// LSB                                                                     MSB
/// +------------+------------+---------+---------+--------------+------------+
/// |  PDU Type  |     -      |  TxAdd  |  RxAdd  |    Length    |     -      |
/// |  (4 bits)  |  (2 bits)  | (1 bit) | (1 bit) |   (6 bits)   |  (2 bits)  |
/// +------------+------------+---------+---------+--------------+------------+
/// ```
///
/// The `TxAdd` and `RxAdd` field are only used for some payloads, for all others, they should be
/// set to 0.
///
/// Length may be in range 6 to 37 (inclusive). With the 2-Byte header this is exactly the max.
/// on-air packet size.
///
/// Clone of `rubble::link::advertising::Header`.
#[derive(Copy, Clone)]
pub struct AdvertisingHeader(u16);

impl AdvertisingHeader {
    /// Creates an [`AdvertisingHeader`] containing the given bytes.
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        AdvertisingHeader(u16::from_le_bytes(bytes))
    }

    /// Returns the raw representation of the header.
    pub fn to_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }
}

/// 16-bit data channel header preceding the payload.
///
/// Clone of `rubble::link::data::Header`. See that structure for in-depth documentation.
#[derive(Copy, Clone)]
pub struct DataHeader(u16);

impl DataHeader {
    /// Creates a [`DataHeader`] from the given

    /// Creates a [`DataHeader`] containing the given bytes.
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        DataHeader(u16::from_le_bytes(bytes))
    }

    /// Returns the raw representation of the header.
    pub fn to_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }
}

/// Specifies if and how the radio should listen for transmissions.
///
/// Returned by the Link-Layer update and processing methods to reconfigure the radio as needed.
///
/// Clone of `rubble::link::RadioCmd`.
#[derive(Debug, Clone)]
pub enum RadioCmd {
    /// Turn the radio off and don't call `LinkLayer::process_*` methods.
    ///
    /// `LinkLayer::update` must still be called according to `Cmd`'s `next_update` field.
    Off,

    /// Listen on an advertising channel. If a packet is received, pass it to
    /// `LinkLayer::process_adv_packet`.
    ListenAdvertising {
        /// The advertising channel to listen on.
        channel: AdvertisingChannel,
    },

    /// Listen on a data channel. If a matching packet is received, pass it to
    /// `LinkLayer::process_data_packet`.
    ListenData {
        /// The data channel to listen on.
        channel: DataChannel,

        /// The Access Address to listen for.
        ///
        /// Packets with a different Access Address must not be passed to the Link-Layer. You may be
        /// able to use your Radio's hardware address matching for this.
        access_address: u32,

        /// Initialization value of the CRC-24 calculation.
        ///
        /// Only the least significant 24 bits are relevant.
        crc_init: u32,

        /// Flag to indicate if the last connection event timed out.
        timeout: bool,
    },
}

/// A point in time, relative to an unspecfied epoch, specified in microseconds.
///
/// This has microsecond resolution and may wrap around after >1 hour. Apart from the wraparound, it
/// is monotonic.
///
/// Clone of `rubble::timer::Instant`.
#[derive(Debug, Copy, Clone)]
pub struct Instant {
    pub microseconds: u32,
}

impl Instant {
    /// Creates an `Instant` from raw microseconds since an arbitrary implementation-defined
    /// reference point.
    pub fn from_raw_micros(microseconds: u32) -> Self {
        Instant { microseconds }
    }

    pub fn from_alarm_time<A: Time>(raw: u32) -> Self {
        // Frequency::frequency() returns NOW_UNIT / second, and we want
        // microseconds. `now / frequency` gives us seconds, so
        // `now * 1000_000 / frequency` is microseconds

        // multiply before dividing to be as accurate as possible, and use u64 to
        // overflow.
        Instant {
            microseconds: ((raw as u64 * 1000_000u64) / A::Frequency::frequency() as u64)
                .try_into()
                .unwrap(),
        }
    }

    pub fn to_alarm_time<A: Time>(&self, alarm: &A) -> u32 {
        // instant.raw_micros() is microseconds, and we want NOW_UNIT.
        // Frequency::frequency() returns NOW_UNIT / second, so `raw_micros * frequency` gives us
        // `NOW_UNIT * microseconds / seconds`. `microseconds = 1000_000 seconds`,
        // so `raw_micros * frequency / 1000_000` is NOW_UNIT.
        u32::try_from(self.microseconds as u64 * A::Frequency::frequency() as u64 / 1000_000u64)
            .unwrap()
            % alarm.max_tics()
    }
}
/// A duration with microsecond resolution.
///
/// This can represent a maximum duration of about 1 hour. Overflows will result in a panic, but
/// shouldn't happen since the BLE stack doesn't deal with durations that large.
///
/// Clone of `rubble::timer::Duration`
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(u32);

impl Duration {
    /// Creates a `Duration` from a number of microseconds.
    pub fn from_micros(micros: u32) -> Self {
        Duration(micros)
    }

    /// Creates a `Duration` representing the given number of milliseconds.
    pub fn from_millis(millis: u16) -> Self {
        Duration(u32::from(millis) * 1_000)
    }

    /// Creates a `Duration` representing a number of seconds.
    pub fn from_secs(secs: u16) -> Self {
        Duration(u32::from(secs) * 1_000_000)
    }

    /// Returns the number of whole seconds that fit in `self`.
    pub fn whole_secs(&self) -> u32 {
        self.0 / 1_000_000
    }

    /// Returns the number of whole milliseconds that fit in `self`.
    pub fn whole_millis(&self) -> u32 {
        self.0 / 1_000
    }

    /// Returns the number of microseconds represented by `self`.
    pub fn as_micros(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use kernel::hil::time::Freq32KHz;

    struct VAlarm;
    impl Time for VAlarm {
        type Frequency = Freq32KHz;
        fn now(&self) -> u32 {
            panic!()
        }
        fn max_tics(&self) -> u32 {
            !0u32
        }
    }

    #[test]
    fn time_roundtrip() {
        for &start in &[0, 3120, 10000, 22500, 9514094] {
            let rubble = Instant::from_alarm_time::<VAlarm>(start);
            let end = rubble.to_alarm_time(&VAlarm);
            assert!((start as i32 - end as i32).abs() < 10);
        }
    }
}
