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
pub struct UartParameters {
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

pub trait Uart<'a>: Configure + Transmit<'a> + Receive<'a> {}
pub trait UartData<'a>: Transmit<'a> + Receive<'a> {}
pub trait UartAdvanced<'a>: Configure + Transmit<'a> + ReceiveAdvanced<'a> {}
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
    fn configure(&self, params: UartParameters) -> ReturnCode;
}

pub trait Transmit<'a> {
    /// Set the transmit client, which will be called when transmissions
    /// complete.
    fn set_client(&self, client: &'a TransmitClient);

    /// Transmit a buffer of data. On completion, `transmitted_buffer`
    /// in the `TransmitClient` will be called.  If the `ReturnCode`
    /// of `transmit`'s return tuple is SUCCESS, the `Option` will be
    /// `None` and the struct will issue a `complete` callback in the
    /// future. If the value of the `ReturnCode` is not SUCCESS, then
    /// the `tx_data` argument is returned in the `Option`. Other
    /// valid `ReturnCode` values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    ///
    /// Each byte in `tx_data` is a UART transfer word of 8 or fewer
    /// bits.  The width is determined by the UART
    /// configuration. Clients that need to transfer 9-bit words
    /// should use `transmit_word`.  Calling `transmit_word` while
    /// there is an outstanding `transmit_buffer` or `transmit_word`
    /// operation will return EBUSY.
    fn transmit_buffer(&self, tx_data: &'a mut [u8], tx_len: usize) -> (ReturnCode, Option<&'a mut [u8]>);


    /// Transmit a single word of data. The word length is determined
    /// by the UART configuration: it can be 6, 7, 8, or 9 bits long.
    /// If the `ReturnCode` is SUCCESS, on completion,
    /// `transmitted_word` will be called on the `TransmitClient`.
    /// Other valid `ReturnCode` values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    /// If the `ReturnCode` is not SUCCESS, no callback will be made.
    /// Calling `transmit_word` while there is an outstanding
    /// `transmit_buffer` or `transmit_word` operation will return
    /// EBUSY.
    fn transmit_word(&self, word: u32) -> ReturnCode;

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

    /// Receive `rx_len` bytes into `rx_buffer`, making a callback to
    /// the `ReceiveClient` when complete.  If the `ReturnCode` of
    /// `receive_buffer`'s return tuple is SUCCESS, the `Option` will
    /// be `None` and the struct will issue a `received_buffer`
    /// callback in the future. If the value of the `ReturnCode` is
    /// not SUCCESS, then the `rx_data` argument is returned in the
    /// `Option`. Other valid return values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already receiving and has not made a
    ///           transmission `complete` callback yet.
    /// Each byte in `rx_data` is a UART transfer word of 8 or fewer
    /// bits.  The width is determined by the UART
    /// configuration. Clients that need to transfer 9-bit words
    /// should use `receive_word`.  Calling `receive_word` while
    /// there is an outstanding `receive_buffer` or `receive_word`
    /// operation will return EBUSY.
    fn receive_buffer(&self, rx_buffer: &'a mut [u8], rx_len: usize) -> (ReturnCode, Option<&'a mut [u8]>);

    /// Receive a single word of data. The word length is determined
    /// by the UART configuration: it can be 6, 7, 8, or 9 bits long.
    /// If the `ReturnCode` is SUCCESS, on completion,
    /// `received_word` will be called on the `ReceiveClient`.
    /// Other valid `ReturnCode` values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already receiving and has not made a
    ///           reception callback yet.
    /// Calling `receive_word` while there is an outstanding
    /// `receive_buffer` or `receive_word` operation will return
    /// EBUSY.
    fn receive_word(&self) -> ReturnCode;

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


/// Implement Client to receive callbacks from UART.
pub trait TransmitClient<'a> {
    /// UART transmit complete.
    fn transmitted_buffer(&self, tx_buffer: &'a mut [u8], rval: ReturnCode);
    fn transmitted_word(&self, _rval: ReturnCode) {}
}

pub trait ReceiveClient<'a> {
    /// UART receive complete.
    fn received_buffer(&self, rx_buffer: &'a mut [u8], rx_len: usize, code: ReturnCode, error: Error);
    fn received_word(&self, _word: u32, _rcode: ReturnCode, _err: Error) {}
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
pub trait ReceiveAdvanced<'a>: Receive<'a> {
    /// Receive data until `interbyte_timeout` bit periods have passed since the
    /// last byte or buffer is full. Does not timeout until at least one byte
    /// has been received.
    ///
    /// * `interbyte_timeout`: number of bit periods since last data received.
    fn receive_automatic(&self, rx_buffer: &'a mut [u8], interbyte_timeout: u8);
}
