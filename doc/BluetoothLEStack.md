Bluetooth Low Energy Design Document
====================================

## System call interface

The system call interface is modeled after the HCI interface defined in the
Bluetooth specification.

### Device address

The kernel assigns the device address. The process may read the device address
using an `allow` system call.

### Advertising

For advertising, the system call interface allows a process to configure an
advertising payload, advertising event type, scan response payload, interval and
tx power. Permissible advertising types include:

  * Connectable undirected

  * Connectable directed

  * Non-connectable undirected

  * Scannable undirected

The driver is _not_ responsible for validating that the payload for these
advertising types follows any particular specification. Advertising event types
that require particular interactions at the link-layer with peer devices (e.g.
scanning or establishing connections) are not permissible:

  * Scan request

  * Scan response

  * Connect request

Scan response are sent automatically if a scan response payload is configured.
Scan request and connection requests are handled by other parts of the system
call interface.

To set up an advertisement:

  1. Configure the advertisement payload, type, interval, tx power and,
     optionally, scan response payload.

     * Advertisement payload `allow`

     * Advertisement type `command`

     * If the advertising type is scannable, you SHOULD configure a scan
       response payload using `allow`

     * Advertisement interval `command`

     * Advertisement tx power `command`

  2. Start periodic advertising using a `command`

Any changes to the configuration while periodic advertising is happening will
take effect in a future advertising event. The kernel will use best effort to
reconfigure advertising in as few events as possible.

To stop advertising

  1. Stop periodic advertising a `command`

### Scanning

### Connection-oriented communication

## Hardware Interface Layer (HIL)

The Bluetooth Low Energy Radio HIL defines a cross-platform interface for
interacting with on-chip BLE radios (i.e. it does not necessarily work for
radios on a dedicated IC connected over a bus).

The goal of this interface is to expose low-level details of the radio that are
common across platforms, except in cases where abstraction is needed for common
cases to meet timing constraints.


```rust
pub trait BleRadio {
    /// Sets the channel on which to transmit or receive packets.
    ///
    /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
    /// receiving, otherwise ReturnCode::Success.
    fn set_channel(&self, channel: RadioChannel) -> ReturnCode;

    /// Sets the transmit power
    ///
    /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
    /// receiving, otherwise ReturnCode::Success.
    fn set_tx_power(&self, power: u8) -> ReturnCode;

    /// Transmits a packet over the radio
    ///
    /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
    /// receiving, otherwise ReturnCode::Success.
    fn transmit_packet(
        &self,
        buf: &'static mut [u8],
        disable: bool) -> ReturnCode;

    /// Receives a packet of at most `buf.len()` size
    ///
    /// Returns ReturnCode::EBUSY if the radio is currently transmitting or
    /// receiving, otherwise ReturnCode::Success.
    fn receive_packet(&self, buf: &'static mut [u8]) -> ReturnCode;

    // Aborts an ongoing transmision
    //
    // Returns None if no transmission was ongoing, or the buffer that was
    // being transmitted.
    fn abort_tx(&self) -> Option<&'static mut [u8]>;

    // Aborts an ongoing reception
    //
    // Returns None if no transmission was ongoing, or the buffer that was //
    // being received into. The returned buffer may or may not have some populated
    // bytes.
    fn abort_rx(&self) -> Option<&'static mut [u8]>;

    // Disable periodic advertisements
    //
    // Returns always ReturnCode::SUCCESS because it does not respect whether
    // the driver is actively advertising or not
    fn disable(&self) -> ReturnCode;
}

pub trait RxClient {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait TxClient {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode);
}

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
```
