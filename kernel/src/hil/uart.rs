// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for UART communication.

use crate::ErrorCode;

/// Number of stop bits to send after each word.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    /// Include one stop bit after each word.
    One = 1,
    /// Include two stop bits after each word.
    Two = 2,
}

/// Parity bit configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    /// No parity bits.
    None = 0,
    /// Add a parity bit to ensure an odd number of 1 bits in the word.
    Odd = 1,
    /// Add a parity bit to ensure an even number of 1 bits in the word.
    Even = 2,
}

/// Number of bits in each word.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Width {
    /// Six bits per word.
    Six = 6,
    /// Seven bits per word.
    Seven = 7,
    /// Eight bits per word.
    Eight = 8,
}

/// UART parameters for configuring the bus.
#[derive(Copy, Clone, Debug)]
pub struct Parameters {
    /// Baud rate in bit/s.
    pub baud_rate: u32,
    /// Number of bits per word.
    pub width: Width,
    /// Parity bit configuration.
    pub parity: Parity,
    /// Number of stop bits per word.
    pub stop_bits: StopBits,
    /// Whether UART flow control is enabled.
    pub hw_flow_control: bool,
}

/// The type of error encountered during UART transaction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    /// No error occurred and the command completed successfully
    None,

    /// Parity error during receive
    ParityError,

    /// Framing error during receive
    FramingError,

    /// Overrun error during receive
    OverrunError,

    /// Repeat call of transmit or receive before initial command complete
    RepeatCallError,

    /// UART hardware was reset
    ResetError,

    /// UART hardware was disconnected
    BreakError,

    /// Read or write was aborted early
    Aborted,
}

/// Trait for a full UART device.
///
/// This includes configuring the bus, transmitting data, and receiving data.
pub trait Uart<'a>: Configure + Transmit<'a> + Receive<'a> {}

/// Trait for sending and receiving on UART.
///
/// This includes transmitting data and receiving data.
///
/// Capsules can use this to require a UART device that can both send and
/// receive but do not need the ability to configure the bus settings.
pub trait UartData<'a>: Transmit<'a> + Receive<'a> {}

/// Trait for a full advanced UART device.
///
/// This includes configuring the bus, transmitting data, and the advanced
/// reception operations.
pub trait UartAdvanced<'a>: Configure + Transmit<'a> + ReceiveAdvanced<'a> {}

/// Trait for both receive and transmit callbacks.
pub trait Client: ReceiveClient + TransmitClient {}

// Provide blanket implementations for all trait groups
impl<'a, T: Configure + Transmit<'a> + Receive<'a>> Uart<'a> for T {}
impl<'a, T: Transmit<'a> + Receive<'a>> UartData<'a> for T {}
impl<'a, T: Configure + Transmit<'a> + ReceiveAdvanced<'a>> UartAdvanced<'a> for T {}
impl<T: ReceiveClient + TransmitClient> Client for T {}

/// Trait for configuring a UART.
pub trait Configure {
    /// Set the configuration parameters for the UART bus.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: The bus was configured correctly.
    /// - `Err(OFF)`: The underlying hardware is currently not available,
    ///   perhaps because it has not been initialized or in the case of a shared
    ///   hardware USART controller because it is set up for SPI.
    /// - `Err(INVAL)`: Impossible parameters (e.g. a [`Parameters::baud_rate`]
    ///   of 0).
    /// - `Err(ENOSUPPORT)`: The underlying UART cannot satisfy this
    ///   configuration.
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode>;
}

/// Trait for sending data via a UART bus.
pub trait Transmit<'a> {
    /// Set the transmit client, which will be called when transmissions
    /// complete.
    fn set_transmit_client(&self, client: &'a dyn TransmitClient);

    /// Transmit a buffer of data.
    ///
    /// If the transmission is not started successfully, this function will
    /// return `Err()` and no callback will be called.
    ///
    /// Each byte in `tx_buffer` is a UART transfer word of 8 or fewer bits. The
    /// word width is determined by the UART configuration, truncating any more
    /// significant bits. E.g., `0x18f` transmitted in 8N1 will be sent as
    /// `0x8f` and in 7N1 will be sent as `0x0f`. Clients that need to transfer
    /// 9-bit words should use [`Transmit::transmit_word`].
    ///
    /// Calling [`Transmit::transmit_buffer`] while there is an outstanding
    /// [`Transmit::transmit_buffer`] or [`Transmit::transmit_word`] operation
    /// will return [`ErrorCode::BUSY`].
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: The transmission started successfully.
    ///   [`TransmitClient::transmitted_buffer`] will be called.
    /// - `Err(OFF)`: The underlying hardware is not available, perhaps because
    ///   it has not been initialized or in the case of a shared hardware USART
    ///   controller because it is set up for SPI.
    /// - `Err(BUSY)`: the UART is already transmitting and has not made a
    ///   transmission callback yet.
    /// - `Err(SIZE)` : `tx_len` is larger than the passed slice.
    /// - `Err(FAIL)`: some other error.
    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Transmit a single word of data asynchronously.
    ///
    /// The word length is determined by the UART configuration: it can be 6, 7,
    /// 8, or 9 bits long.
    ///
    /// If initiating the transmission failed, this function will return `Err()`
    /// and no callback will be made.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: The transmission started successfully.
    ///   [`TransmitClient::transmitted_word`] will be called.
    /// - `Err(OFF)`: The underlying hardware is not available, perhaps because
    ///   it has not been initialized or in the case of a shared hardware USART
    ///   controller because it is set up for SPI.
    /// - `Err(BUSY)`: the UART is already transmitting and has not made a
    ///   transmission callback yet.
    /// - `Err(FAIL)`: not supported, or some other error.
    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode>;

    /// Abort an outstanding call to `transmit_word` or `transmit_buffer`.
    ///
    /// The return code indicates whether the call has fully terminated or there
    /// will be a callback. Cancelled calls to [`Transmit::transmit_buffer`]
    /// MUST always make a callback, to return the passed buffer back to the
    /// caller.
    ///
    /// If this function returns `Ok(())`, there will be no future callback and
    /// the client may retransmit immediately. If this function returns any
    /// `Err()` there will be a callback. This means that if there is no
    /// outstanding call to [`Transmit::transmit_word`] or
    /// [`Transmit::transmit_buffer`] then a call to this function returns
    /// `Ok(())`.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: The cancel happened immediate and no callback will be
    ///   generated.
    /// - `Err(BUSY)`: There was a transmit operation outstanding and it was
    ///   cancelled successfully. There will be an appropriate callback (based
    ///   on the type of transmission) with a result of `Err(CANCEL)`.
    /// - `Err(FAIL)`: if the outstanding call to either transmit operation
    ///   could not be synchronously cancelled. A callback will be made on the
    ///   client indicating whether the call was successfully cancelled.
    fn transmit_abort(&self) -> Result<(), ErrorCode>;
}

/// Trait for receiving data via a UART bus.
pub trait Receive<'a> {
    /// Set the receive client, which will be called when reads complete.
    fn set_receive_client(&self, client: &'a dyn ReceiveClient);

    /// Receive `rx_len` bytes into `rx_buffer`.
    ///
    /// If this function returns `Ok(())`, there will be a callback to the
    /// [`ReceiveClient::received_buffer`] when the receive is complete.
    ///
    /// Each byte in `rx_buffer` will be a UART transfer word of 8 or fewer
    /// bits. The width is determined by the UART configuration. Clients that
    /// need to transfer 9-bit words should use [`Receive::receive_word`].
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: The receive started successfully and a
    ///   [`ReceiveClient::received_buffer`] callback will be generated when the
    ///   read completes.
    /// - `Err(OFF)`: The underlying hardware is not available, perhaps because
    ///   it has not been initialized or in the case of a shared hardware USART
    ///   controller because it is set up for SPI.
    /// - `Err(BUSY)`: the UART is already receiving and has not made a
    ///   reception `complete` callback yet.
    /// - `Err(SIZE)`: `rx_len` is larger than the passed slice.
    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Receive a single word of data.
    ///
    /// The word length is determined by the UART configuration: it can be 6, 7,
    /// 8, or 9 bits long.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: The receive started successfully and
    ///   [`ReceiveClient::received_word`] will be called.
    /// - `Err(OFF)`: The underlying hardware is not available, perhaps because
    ///   it has not been initialized or in the case of a shared hardware USART
    ///   controller because it is set up for SPI.
    /// - `Err(BUSY)`: the UART is already receiving and has not made a
    ///   reception callback yet.
    /// - `Err(FAIL)`: not supported or some other error.
    fn receive_word(&self) -> Result<(), ErrorCode>;

    /// Abort any ongoing receive transfers and return what has been received.
    ///
    /// If there was an ongoing receive, the received data and the receive
    /// buffer will be provided in the correct callback, either
    /// [`ReceiveClient::received_word`] or [`ReceiveClient::received_buffer`].
    ///
    /// If there is no outstanding receive operation, `Ok(())` is returned and
    /// there will be no callback.
    ///
    ///  ### Return values
    ///
    ///  - `Ok(())`: The abort was successful because there was nothing to
    ///    abort. There will be no callback.
    ///  - `Err(BUSY)`: There is a receive in progress and it canceling it has
    ///    started successfully. A callback will be generated later with a
    ///    result of `Err(CANCEL)`.
    ///  - `Err(FAIL)`: Cancelling an ongoing receive did not occur correctly. A
    ///    future callback will be generated when the receive finishes.
    fn receive_abort(&self) -> Result<(), ErrorCode>;
}

/// Trait implemented by a UART transmitter to receive callbacks when
/// operations complete.
pub trait TransmitClient {
    /// A call to [`Transmit::transmit_word`] completed.
    ///
    /// A call to [`Transmit::transmit_word`] or [`Transmit::transmit_buffer`]
    /// made within this callback SHOULD NOT return `Err(BUSY)`. When this
    /// callback is made the UART should be ready to receive another call.
    ///
    /// `rval` indicates whether the word was successfully transmitted. Possible
    /// `rval` values:
    ///
    /// - `Ok(())`: The word was successfully transmitted.
    /// - `Err(CANCEL)`: The call to [`Transmit::transmit_word`] was cancelled
    ///   and the word was not transmitted.
    /// - `Err(FAIL)`: The transmission failed in some way.
    fn transmitted_word(&self, _rval: Result<(), ErrorCode>) {}

    /// A call to [`Transmit::transmit_buffer`] completed.
    ///
    /// A call to [`Transmit::transmit_word`] or [`Transmit::transmit_buffer`]
    /// made within this callback SHOULD NOT return `Err(BUSY)`. When this
    /// callback is made the UART should be ready to receive another call.
    ///
    /// The `tx_len` argument specifies how many words were transmitted. If the
    /// transmission was successful, `tx_len` in the callback will be the same
    /// as `tx_len` in the initiating call.
    ///
    /// `rval` indicates whether the buffer was successfully transmitted.
    /// Possible `rval` values:
    ///
    /// - `Ok(())`: The full buffer was successfully transmitted.
    /// - `Err(CANCEL)`: The call to [`Transmit::transmit_buffer`] was cancelled
    ///   and the buffer was not fully transmitted. `tx_len` contains how many
    ///   words were transmitted.
    /// - `Err(SIZE)`: The buffer could only be partially transmitted. `tx_len`
    ///   contains how many words were transmitted.
    /// - `Err(FAIL)`: The transmission failed in some way.
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rval: Result<(), ErrorCode>,
    );
}

/// Trait implemented by a UART receiver to receive callbacks when
/// operations complete.
pub trait ReceiveClient {
    /// A call to [`Receive::receive_word`] completed.
    ///
    /// A call to [`Receive::receive_word`] or [`Receive::receive_buffer`] made
    /// within this callback SHOULD NOT return `Err(BUSY)`. When this callback
    /// is made the UART should be ready to receive another call.
    ///
    /// `rval` indicates whether a word was successfully received. Possible
    /// `rval` values:
    ///
    /// - `Ok(())`: The word was successfully received.
    /// - `Err(CANCEL)`: The call to [`Receive::receive_word`] was cancelled and
    ///   the word was not received: `word` should be ignored.
    /// - `Err(FAIL)`: The reception failed in some way and `word` should be
    ///   ignored. `error` may contain further information on the sort of error.
    fn received_word(&self, _word: u32, _rval: Result<(), ErrorCode>, _error: Error) {}

    /// A call to [`Receive::receive_buffer`] completed.
    ///
    /// A call to [`Receive::receive_word`] or [`Receive::receive_buffer`] made
    /// within this callback SHOULD NOT return `Err(BUSY)`. When this callback
    /// is made the UART should be ready to receive another call.
    ///
    /// The `rx_len` argument specifies how many words were received. If the
    /// receive was successful, `rx_len` in the callback will be the same
    /// as `rx_len` in the initiating call.
    ///
    /// `rval` indicates whether a buffer was successfully received. Possible
    /// `rval` values:
    ///
    /// - `Ok(())`: The full buffer was successfully received.
    /// - `Err(CANCEL)`: The call to [`Receive::receive_buffer`] was cancelled
    ///   and the buffer was not fully received. `rx_len` contains how many
    ///   words were received.
    /// - `Err(SIZE)`: The buffer could only be partially received. `rx_len`
    ///   contains how many words were received.
    /// - `Err(FAIL)`: The reception failed in some way: `error` may contain
    ///   further information.
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rval: Result<(), ErrorCode>,
        error: Error,
    );
}

/// Trait with optional UART features that certain hardware may support.
///
/// The operations in this trait are not required for basic UART operation, but
/// provide useful abstractions that capsules may want to be able to leverage.
///
/// The interfaces are included here because some hardware platforms may be able
/// to directly implement them, while others platforms may need to emulate them
/// in software. The ones that can implement them in hardware should be able to
/// leverage that efficiency, and by placing the interfaces here in the HIL they
/// can do that.
///
/// Other interface ideas that have been discussed, but are not included due to
/// the lack of a clear use case, but are noted here in case they might help
/// someone in the future:
/// - `receive_until_terminator()`: This would read in bytes until a specified
///   byte is received (or the buffer is full) and then return to the client.
/// - `receive_len_then_message()`: This would do a one byte read to get a
///   length byte and then read that many more bytes from UART before returning
///   to the client.
pub trait ReceiveAdvanced<'a>: Receive<'a> {
    /// Receive data until `interbyte_timeout` bit periods have passed since the
    /// last byte or buffer is full.
    ///
    /// This does not timeout until at least one byte has been received.
    ///
    /// ### Arguments:
    ///
    /// - `rx_buffer`: Buffer to receive into.
    /// - `rx_len`: Maximum number of bytes to receive.
    /// - `interbyte_timeout`: number of bit periods to wait between bytes
    ///   before returning.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: Receive was started correctly.
    ///   [`ReceiveClient::received_buffer]` will be called.
    /// - `Err(OFF)`: The underlying hardware is not available, perhaps because
    ///   it has not been initialized or in the case of a shared hardware USART
    ///   controller because it is set up for SPI.
    /// - `Err(BUSY)`: the UART is already receiving and has not made a
    ///   reception callback yet.
    /// - `Err(SIZE)`: `rx_len` is larger than the passed slice.
    fn receive_automatic(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        interbyte_timeout: u8,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}
