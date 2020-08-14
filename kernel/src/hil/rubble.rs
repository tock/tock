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

use super::{
    ble_advertising::{BleAdvertisementDriver, RadioChannel},
    time::Alarm,
};
use crate::ReturnCode;

mod data_structures;

pub use data_structures::*;

/// The base trait representing a single implementation of the Rubble stack. The
/// interface is based off of the types publicly exposed by `rubble` which we
/// use, and is otherwise fairly arbitrary.
///
/// This is intended to be implemented on something which requires the
/// `BleHardware` trait, but `RubbleImplementation` itself never exposes any
/// references to `BleHardware` - they act as two separate interface layers.
pub trait RubbleImplementation<'a, A: Alarm<'a> + ?Sized> {
    type BleRadio: RubbleBleRadio<'a, A, Self>;
    type LinkLayer: RubbleLinkLayer<'a, A, Self>;
    type Responder: RubbleResponder<'a, A, Self>;
    type Cmd: fmt::Debug + RubbleCmd;
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

    /// Handle a receive event from the BLE stack. This is here as it needs
    /// access to the BleRadio and LinkLayer structs simultaneously.
    fn transmit_event(
        radio: &mut Self::BleRadio,
        ll: &mut Self::LinkLayer,
        rx_end: Instant,
        buf: &'static mut [u8],
        result: ReturnCode,
    );

    /// Handle a receive event from the BLE stack. This is here as it needs
    /// access to the BleRadio and LinkLayer structs simultaneously.
    fn receive_event(
        radio: &mut Self::BleRadio,
        ll: &mut Self::LinkLayer,
        rx_end: Instant,
        buf: &'static mut [u8],
        len: u8,
        result: ReturnCode,
    ) -> Self::Cmd;
}

/// A packet queue implementation that can be split into producer and consumer.
///
/// This queue is fairly limited in that it must be stored in a static variable.
/// `rubble`'s idea of a queue isn't necessarily static - but it usually needs
/// to be to be shared everywhere. And to allow for non-static queues, we would
/// need to declare `RubbleImplementation::Queue` as
///
/// ```ignore
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

pub trait RubbleCmd {
    type RadioCmd;
    fn next_update(&self) -> NextUpdate;
    fn queued_work(&self) -> bool;
    fn into_radio_cmd(self) -> Self::RadioCmd;
}

pub trait RubbleBleRadio<'a, A, R>
where
    A: Alarm<'a> + ?Sized,
    R: RubbleImplementation<'a, A> + ?Sized,
{
    fn accept_cmd(&mut self, cmd: <R::Cmd as RubbleCmd>::RadioCmd);
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
///
/// This is based off of a combination of rubble's `RadioCmd` interface for
/// configuring radio receiving, rubble's `Transmitter` trait for data
/// transmission, and Tock's `BleAdvertisementDriver`.
///
/// Event notifications well be sent to the [`RxClient`] and [`TxClient`] set
/// using [`BleAdvertisementDriver`] methods. This allows radios supporting both
/// advertisement and data connections to keep track of one set of event clients
/// rather than two for different transmission types.
///
/// [`RxClient`]: crate::hil::ble_advertisement::RxClient
/// [`TxClient`]: crate::hil::ble_advertisement::TxClient
pub trait RubbleDataDriver<'a>: BleAdvertisementDriver<'a> {
    /// Return the `DeviceAddress`, which is pre-programmed in the device FICR
    /// (Factory information configuration registers).
    fn get_device_address() -> DeviceAddress;

    /// Transmit a Data Channel PDU.
    ///
    /// The implementor is expected to send the preamble and assemble the rest of the packet, and
    /// must apply data whitening and do the CRC calculation.
    ///
    /// # Parameters
    ///
    /// * `buf`: The data to send, including the Data Channel PDU Header as the
    ///   first two bytes.
    /// * `access_address`: The Access Address of the Link-Layer packet.
    /// * `crc_iv`: CRC calculation initial value (`CRC_PRESET` for advertising channel).
    /// * `channel`: Data Channel Index to transmit on. Must be in 0..=36.
    fn transmit_data(
        &self,
        buf: &'static mut [u8],
        access_address: u32,
        crc_iv: u32,
        channel: RadioChannel,
    );

    /// Configure the radio to receive data.
    ///
    /// # Parameters
    ///
    /// * `channel`: The data channel to listen on.
    /// * `access_address`: The Access Address to listen for.
    ///
    ///   Packets with a different Access Address must not be passed to the
    ///   to the `RxClient`. You may be able to use your Radio's hardware
    ///   address matching for this.
    /// * `crc_init`: Initialization value of the CRC-24 calculation.
    ///
    ///   Only the least 24 bits are relevant.
    fn receive_data(&self, channel: RadioChannel, access_address: u32, crc_init: u32);
}
