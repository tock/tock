//! Interfaces necessary to use the Rubble BLE stack.
//!
//! Contains two interfaces.
//!
//! First, an interface for the kernel to access the Rubble stack, which
//! copies various structs from rubble and provides trait-based interfaces to
//! Rubble's implementation.
//!
//! Second, an interface for Rubble to access hardware, which will be
//! implemented by chip crates.
use core::fmt;

use super::time::Alarm;

mod data_structures;

pub use data_structures::*;

/// The base trait representing a single implementation of the Rubble stack. The
/// interface is based off of the types publicly exposed by `rubble` which we
/// use, and is otherwise fairly arbitrary.
///
/// This is intended to be implemented on something which requires the
/// `BleHardware` trait, but `RubbleImplementation` itself never exposes any
/// references to `BleHardware` - they act as two separate interface layers.
pub trait RubbleImplementation<'a, A: Alarm<'a> + ?Sized>
where
    Self::Cmd: RubbleCmd<Self::RadioCmd>,
{
    type BleRadio: RubbleBleRadio<'a, A, Self>;
    type LinkLayer: RubbleLinkLayer<'a, A, Self>;
    type Responder: RubbleResponder<'a, A, Self>;
    type Cmd: fmt::Debug;
    type RadioCmd: fmt::Debug;
    /// The packet queue that this implementation uses.
    type PacketQueue: RubblePacketQueue + 'static;

    /// Retrieves this device's address.
    fn get_device_address() -> DeviceAddress;

    /// Retrieve a statically-allocated packet queue for communication from the
    /// hardware to the LinkLayer.
    fn rx_packet_queue() -> &'static Self::PacketQueue;

    /// Retrieve a statically-allocated packet queue for communication from the
    /// LinkLayer to the hardware.
    fn tx_packet_queue() -> &'static Self::PacketQueue;
}

/// A packet queue implementation that can be split into producer and consumer.
///
/// This queue is fairly limited in that it must be stored in a static variable.
/// `rubble`'s idea of a queue isn't necessarily static - but it usually needs
/// to be to be shared everywhere. And to allow for non-static queues, we would
/// need to declare `RubbleImplementation::Queue` as
///
/// ```
/// type Queue<'a>: RubblePacketQueue<'a>;
/// ```
///
/// which would require generic associated types.
pub trait RubblePacketQueue {
    /// A static reference to the producer half of the PacketQueue once split.
    type Producer: 'static;
    /// A static reference to the consumer half of the PacketQueue once split.
    type Consumer: 'static;
    fn split(&'static self) -> (Self::Producer, Self::Consumer);
}

pub trait RubbleCmd<RadioCmd> {
    fn next_update(&self) -> NextUpdate;
    fn queued_work(&self) -> bool;
    fn into_radio_cmd(self) -> RadioCmd;
}

pub trait RubbleBleRadio<'a, A, R>
where
    A: Alarm<'a> + ?Sized,
    R: RubbleImplementation<'a, A> + ?Sized,
{
    fn accept_cmd(&mut self, cmd: R::RadioCmd);
}

pub trait RubbleLinkLayer<'a, A, R>
where
    A: Alarm<'a> + ?Sized,
    R: RubbleImplementation<'a, A> + ?Sized,
{
    type Error: fmt::Debug + fmt::Display;
    fn new(device_address: DeviceAddress, alarm: &'a A) -> Self;

    /// Starts advertising this device, optionally sending data along with the advertising PDU.
    ///
    /// # Errors
    ///
    /// This will error if the advertising data is incorrectly formatted, or if
    /// it's too long.
    fn start_advertise(
        &mut self,
        interval: Duration,
        data: &[u8],
        transmitter: &mut R::BleRadio,
        tx: <R::PacketQueue as RubblePacketQueue>::Consumer,
        rx: <R::PacketQueue as RubblePacketQueue>::Producer,
    ) -> Result<NextUpdate, Self::Error>;

    /// Returns whether the Link-Layer is currently broadcasting advertisement packets.
    fn is_advertising(&self) -> bool;

    /// Update the Link-Layer state after the timer expires.
    ///
    /// This should be called whenever the timer set by the last returned `Cmd` has expired.
    ///
    /// # Parameters
    ///
    /// * `tx`: A `Transmitter` for sending packets.
    fn update_timer(&mut self, tx: &mut R::BleRadio) -> R::Cmd;
}

pub trait RubbleResponder<'a, A, R>
where
    A: Alarm<'a> + ?Sized,
    R: RubbleImplementation<'a, A> + ?Sized,
{
    type Error: fmt::Debug + fmt::Display;

    /// Creates a new packet processor hooked up to data channel packet queues.
    ///
    /// `tx` is the transmitter for the tx_queue - the queue from the LinkLayer
    /// to the hardware.
    ///
    /// `rx` is the receiver for the rx_queue - the queue from the hardware to
    /// LinkLayer.
    fn new(
        tx: <R::PacketQueue as RubblePacketQueue>::Producer,
        rx: <R::PacketQueue as RubblePacketQueue>::Consumer,
    ) -> Self;

    /// Whether this responder has any incoming packets to process.
    fn has_work(&mut self) -> bool;
    /// Processes a single incoming packet. Returns `Err` if `has_work()` is false.
    fn process_one(&mut self) -> Result<(), Self::Error>;
}

/// The primary interface between the rubble stack and the radio.
pub trait BleHardware {
    type Transmitter: Transmitter;
    type RadioCmd;

    fn get_device_address() -> DeviceAddress;

    fn radio_accept_cmd(radio: &mut Self::Transmitter, cmd: Self::RadioCmd);
}

/// Clone of `rubble::link::Transmitter`.
pub trait Transmitter {
    /// Get a reference to the Transmitter's PDU payload buffer.
    ///
    /// The buffer must hold at least 37 Bytes, as that is the maximum length of advertising channel
    /// payloads. While data channel payloads can be up to 251 Bytes in length (resulting in a
    /// "length" field of 255 with the MIC), devices are allowed to use smaller buffers and report
    /// the supported payload length.
    ///
    /// Both advertising and data channel packets also use an additional 2-Byte header preceding
    /// this payload.
    ///
    /// This buffer must not be changed. The BLE stack relies on the buffer to retain its old
    /// contents after transmitting a packet. A separate buffer must be used for received packets.
    fn tx_payload_buf(&mut self) -> &mut [u8];

    /// Transmit an Advertising Channel PDU.
    ///
    /// For Advertising Channel PDUs, the CRC initialization value is always `CRC_PRESET`, and the
    /// Access Address is always `ADVERTISING_ADDRESS`.
    ///
    /// The implementor is expected to send the preamble and access address, and assemble the rest
    /// of the packet, and must apply data whitening and do the CRC calculation. The inter-frame
    /// spacing also has to be upheld by the implementor (`T_IFS`).
    ///
    /// # Parameters
    ///
    /// * `header`: Advertising Channel PDU Header to prepend to the Payload in `payload_buf()`.
    /// * `channel`: Advertising Channel Index to transmit on.
    fn transmit_advertising(&mut self, header: AdvertisingHeader, channel: AdvertisingChannel);

    /// Transmit a Data Channel PDU.
    ///
    /// The implementor is expected to send the preamble and assemble the rest of the packet, and
    /// must apply data whitening and do the CRC calculation.
    ///
    /// # Parameters
    ///
    /// * `access_address`: The Access Address of the Link-Layer packet.
    /// * `crc_iv`: CRC calculation initial value (`CRC_PRESET` for advertising channel).
    /// * `header`: Data Channel PDU Header to be prepended to the Payload in `payload_buf()`.
    /// * `channel`: Data Channel Index to transmit on. Must be in 0..=36.
    fn transmit_data(
        &mut self,
        access_address: u32,
        crc_iv: u32,
        header: DataHeader,
        channel: DataChannel,
    );
}
