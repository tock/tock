//! Hardware interface layer (HIL) traits for UART communication.
//!
//!

use returncode::ReturnCode;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    One = 0,
    Two = 2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
}

#[derive(Copy, Clone, Debug)]
pub struct UARTParameters {
    pub baud_rate: u32, // baud rate in bit/s
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub hw_flow_control: bool,
}

/// The type of error encountered during UART transaction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
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

    /// Read or write was aborted early
    Aborted,

    /// No error occurred and the command completed successfully
    CommandComplete,
}

pub trait Configure {
    /// Configure UART
    ///
    /// Returns SUCCESS, or
    ///
    /// - EOFF: The underlying hardware is currently not available, perhaps
    ///         because it has not been initialized or in the case of a shared
    ///         hardware USART controller because it is set up for SPI.
    /// - EINVAL: Impossible parameters (e.g. a `baud_rate` of 0)
    /// - ENOSUPPORT: The underlying UART cannot satisfy this configuration.
    fn configure(&self, params: UARTParameters) -> ReturnCode;
}

pub trait Transmit<'a> {
    /// Set the transmit client, which will be called when transmissions
    /// complete;
    fn set_client(&self, client: &'a TransmitClient);

    /// Transmit a buffer of data. On completion, `complete` in
    /// the `TransmitClient` will be called.
    ///
    /// If `transmit` returns SUCCESS, it will issue a `complete` callback
    /// in the future. Other valid return values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already transmitting and has not made a
    ///           transmission `complete` callback yet.
    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) -> ReturnCode;

    /// Abort the ongoing transmission.  If SUCCESS is returned, there
    /// will be no callback (no call to `transmit` was
    /// outstanding). If there was a `transmit` outstanding, which is
    /// cancelled successfully then `EBUSY` will be returned and a
    /// there will be a callback with a `ReturnCode` of `ECANCEL`.  If
    /// there was a transmit outstanding, which is not cancelled
    /// successfully, then `FAIL` will be returned and there will be a
    /// later callback.
    fn abort(&self) -> ReturnCode;
}

pub trait Receive<'a> {
    /// Set the receive client, which will he called when reads complete.
    fn set_client(&self, client: &'a ReceiveClient);

    /// Receive `rx_len` bytes into `rx_buffer`, making a callback to the
    /// `ReceiveClient` when complete.
    /// If `receive` returns SUCCESS, it will issue a `complete` callback
    /// in the future. Other valid return values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already receiving and has not made a
    ///           transmission `complete` callback yet.
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) -> ReturnCode;
    /// Abort any ongoing receive transfers and return what is in the
    /// receive buffer with the `receive_complete` callback. If
    /// SUCCESS is returned, there will be no callback (no call to
    /// `receive` was outstanding). If there was a `receive`
    /// outstanding, which is cancelled successfully then `EBUSY` will
    /// be returned and a there will be a callback with a `ReturnCode`
    /// of `ECANCEL`.  If there was a reception outstanding, which is
    /// not cancelled successfully, then `FAIL` will be returned and
    /// there will be a later callback.
    fn abort(&self) -> ReturnCode;
}

/// Trait that isn't required for basic UART operation, but provides useful
/// abstractions that capsules may want to be able to leverage.
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
/// - `receive_until_terminator`: This would read in bytes until a specified
///   byte is received (or the buffer is full) and then return to the client.
/// - `receive_len_then_message`: This would do a one byte read to get a length
///   byte and then read that many more bytes from UART before returning to the
///   client.
pub trait UARTReceiveAdvanced: UART {
    /// Receive data until `interbyte_timeout` bit periods have passed since the
    /// last byte or buffer is full. Does not timeout until at least one byte
    /// has been received.
    ///
    /// * `interbyte_timeout`: number of bit periods since last data received.
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], interbyte_timeout: u8);
}

/// Implement Client to receive callbacks from UART.
pub trait Client {
    /// UART transmit complete.
    fn transmit_complete(&self, tx_buffer: &'static mut [u8], error: Error);

    /// UART receive complete.
    fn receive_complete(&self, rx_buffer: &'static mut [u8], rx_len: usize, error: Error);
}
