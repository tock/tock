//! Interfaces necessary to use the Rubble BLE stack.
//!
//! These interfaces provide abstractions in Tock to use the Rubble BLE stack in
//! the kernel. For example, these interfaces would be used to create a capsule
//! that provides userspace with access to BLE via Rubble.

use core::fmt;

use crate::hil::time;
use crate::ReturnCode;

use super::types::{DeviceAddress, Duration, Instant, NextUpdate};

/// The base trait representing for using the Rubble stack. The interface is
/// based off of the types publicly exposed by Rubble which we use, and is
/// otherwise fairly arbitrary.
pub trait RubbleStack<'a, A: time::Alarm<'a> + ?Sized> {
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

pub trait RubbleBleRadio<'a, A, R>
where
    A: time::Alarm<'a> + ?Sized,
    R: RubbleStack<'a, A> + ?Sized,
{
    fn accept_cmd(&mut self, cmd: <R::Cmd as RubbleCmd>::RadioCmd);
}

pub trait RubbleLinkLayer<'a, A, R>
where
    A: time::Alarm<'a> + ?Sized,
    R: RubbleStack<'a, A> + ?Sized,
{
    type Error: fmt::Debug + fmt::Display;
    fn new(device_address: DeviceAddress, alarm: &'a A) -> Self;

    /// Starts advertising this device, optionally sending data along with the
    /// advertising PDU.
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

    /// Returns whether the Link-Layer is currently broadcasting advertisement
    /// packets.
    fn is_advertising(&self) -> bool;

    /// Update the Link-Layer state after the timer expires.
    ///
    /// This should be called whenever the timer set by the last returned `Cmd`
    /// has expired.
    ///
    /// # Parameters
    ///
    /// * `tx`: A `Transmitter` for sending packets.
    fn update_timer(&mut self, tx: &mut R::BleRadio) -> R::Cmd;
}

pub trait RubbleResponder<'a, A, R>
where
    A: time::Alarm<'a> + ?Sized,
    R: RubbleStack<'a, A> + ?Sized,
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

/// A packet queue implementation that can be split into producer and consumer.
///
/// This queue is fairly limited in that it must be stored in a static variable.
/// Rubble's idea of a queue isn't necessarily static, but it usually needs
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
