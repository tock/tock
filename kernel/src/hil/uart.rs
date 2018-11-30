//! Hardware interface layer (HIL) traits for UART communication.
//!
//!

use returncode::{Error as TockError, Success, ReturnCode};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    One = 1,
    Two = 2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Width {
    Six = 6,
    Seven = 7,
    Eight = 8,
}

#[derive(Copy, Clone, Debug)]
pub struct Parameters {
    pub baud_rate: u32, // baud rate in bit/s
    pub width: Width,
    pub parity: Parity,
    pub stop_bits: StopBits,
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

    /// Read or write was aborted early
    Aborted,

}

pub struct UartError {
    pub error: Error,
    pub buffer: &'static mut [u8],
}

// FIXME: only to show a point (all errors should be handled accordingly)
// None should be Success
impl From<UartError> for TockError {
    fn from(err: UartError) -> TockError {
        err.error.into()
    }
}

impl From<Error> for TockError {
    fn from(err: Error) -> TockError {
        match err {
            _ => TockError::FAIL,
        }
    }
}


pub trait Uart<'a>: Configure + Transmit<'a> + Receive<'a> {}
pub trait UartData<'a>: Transmit<'a> + Receive<'a> {}
pub trait UartAdvanced<'a>: Configure + Transmit<'a> + ReceiveAdvanced<'a> {}
pub trait Client: ReceiveClient + TransmitClient {}

/// Trait for configuring a UART.
pub trait Configure {
    /// Returns SUCCESS, or
    /// - EOFF: The underlying hardware is currently not available, perhaps
    ///         because it has not been initialized or in the case of a shared
    ///         hardware USART controller because it is set up for SPI.
    /// - EINVAL: Impossible parameters (e.g. a `baud_rate` of 0)
    /// - ENOSUPPORT: The underlying UART cannot satisfy this configuration.
    fn configure(&self, params: Parameters) -> ReturnCode;
}

pub trait Transmit<'a> {
    /// Set the transmit client, which will be called when transmissions
    /// complete.
    fn set_transmit_client(&self, client: &'a TransmitClient);

    /// Transmit a buffer of data. On completion, `transmitted_buffer`
    /// in the `TransmitClient` will be called.  If the `ReturnCode`
    /// of `transmit`'s return tuple is SUCCESS, the `Option` will be
    /// `None` and the struct will issue a `transmitted_buffer`
    /// callback in the future. If the value of the `ReturnCode` is
    /// not SUCCESS, then the `tx_data` argument is returned in the
    /// `Option`. Other valid `ReturnCode` values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    ///  - ESIZE : `tx_len` is larger than the passed slice.
    ///  - FAIL: some other error.
    ///
    /// Each byte in `tx_data` is a UART transfer word of 8 or fewer
    /// bits.  The word width is determined by the UART configuration,
    /// truncating any more significant bits. E.g., 0x18f transmitted in
    /// 8N1 will be sent as 0x8f and in 7N1 will be sent as 0x0f. Clients
    /// that need to transfer 9-bit words should use `transmit_word`.
    ///
    /// Calling `transmit_word` while there is an outstanding
    /// `transmit_buffer` or `transmit_word` operation will return EBUSY.
    fn transmit_buffer(&self, tx_data: &'static mut [u8], tx_len: usize) -> Result<Success, UartError>;


    /// Transmit a single word of data asynchronously. The word length is
    /// determined by the UART configuration: it can be 6, 7, 8, or 9 bits long.
    /// If the `ReturnCode` is SUCCESS, on completion,
    /// `transmitted_word` will be called on the `TransmitClient`.
    /// Other valid `ReturnCode` values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    ///  - FAIL: some other error.
    /// If the `ReturnCode` is not SUCCESS, no callback will be made.
    /// Calling `transmit_word` while there is an outstanding
    /// `transmit_buffer` or `transmit_word` operation will return
    /// EBUSY.
    fn transmit_word(&self, word: u32) -> ReturnCode;

    /// Abort an outstanding call to `transmit_word` or `transmit_buffer`.
    /// The return code indicates whether the call has fully terminated or
    /// there will be a callback. Cancelled calls to `transmit_buffer` MUST
    /// always make a callback, to return the passed buffer back to the caller.
    ///
    /// If abort_transmit returns SUCCESS, there will be no future
    /// callback and the client may retransmit immediately. If
    /// abort_transmit returns any other `ReturnCode` there will be a
    /// callback. This means that if there is no outstanding call to
    /// `transmit_word` or `transmit_buffer` then a call to
    /// `abort_transmit` returns SUCCESS. If there was a `transmit`
    /// outstanding and is cancelled successfully then `EBUSY` will
    /// be returned and a there will be a callback with a `ReturnCode`
    /// of `ECANCEL`.  If there was a reception outstanding, which is
    /// not cancelled successfully, then `FAIL` will be returned and
    /// there will be a later callback.
    ///
    /// Returns SUCCESS or
    ///  - FAIL if the outstanding call to either transmit operation could
    ///    not be synchronously cancelled. A callback will be made on the
    ///    client indicating whether the call was successfully cancelled.
    fn transmit_abort(&self) -> ReturnCode;
}

pub trait Receive<'a> {
    /// Set the receive client, which will he called when reads complete.
    fn set_receive_client(&self, client: &'a ReceiveClient);

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
    ///           reception `complete` callback yet.
    ///  - ESIZE : `rx_len` is larger than the passed slice.
    /// Each byte in `rx_data` is a UART transfer word of 8 or fewer
    /// bits.  The width is determined by the UART
    /// configuration. Clients that need to transfer 9-bit words
    /// should use `receive_word`.  Calling `receive_word` while
    /// there is an outstanding `receive_buffer` or `receive_word`
    /// operation will return EBUSY.
    fn receive_buffer(&self, rx_buffer: &'static mut [u8], rx_len: usize) -> Result<Success, UartError>;

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
    fn receive_abort(&self) -> ReturnCode;
}


/// Trait implemented by a UART transmitter to receive callbacks when
/// operations complete.
pub trait TransmitClient {

    /// A call to `Transmit::transmit_word` completed. The `ReturnCode`
    /// indicates whether the word was successfully transmitted. A call
    /// to `transmit_word` or `transmit_buffer` made within this callback
    /// SHOULD NOT return EBUSY: when this callback is made the UART should
    /// be ready to receive another call.
    ///
    /// `rval` is SUCCESS if the word was successfully transmitted, or
    ///   - ECANCEL if the call to `transmit_word` was cancelled and
    ///     the word was not transmitted.
    ///   - FAIL if the transmission failed in some way.
    fn transmitted_word(&self, _rval: ReturnCode) {}

    /// A call to `Transmit::transmit_buffer` completed. The `ReturnCode`
    /// indicates whether the buffer was successfully transmitted. A call
    /// to `transmit_word` or `transmit_buffer` made within this callback
    /// SHOULD NOT return EBUSY: when this callback is made the UART should
    /// be ready to receive another call.
    ///
    /// The `tx_len` argument specifies how many words were transmitted.
    /// An `rval` of SUCCESS indicates that every requested word was
    /// transmitted: `tx_len` in the callback should be the same as
    /// `tx_len` in the initiating call.
    ///
    /// `rval` is SUCCESS if the full buffer was successfully transmitted, or
    ///   - ECANCEL if the call to `transmit_buffer` was cancelled and
    ///     the buffer was not fully transmitted. `tx_len` contains
    ///     how many words were transmitted.
    ///   - ESIZE if the buffer could only be partially transmitted. `tx_len`
    ///     contains how many words were transmitted.
    ///   - FAIL if the transmission failed in some way.
    fn transmitted_buffer(&self, tx_buffer: &'static mut [u8], tx_len: usize, rval: ReturnCode);
}

pub trait ReceiveClient {

    /// A call to `Receive::receive_word` completed. The `ReturnCode`
    /// indicates whether the word was successfully received. A call
    /// to `receive_word` or `receive_buffer` made within this callback
    /// SHOULD NOT return EBUSY: when this callback is made the UART should
    /// be ready to receive another call.
    ///
    /// `rval` SUCCESS if the word was successfully received, or
    ///   - ECANCEL if the call to `receive_word` was cancelled and
    ///     the word was not received: `word` should be ignored.
    ///   - FAIL if the reception failed in some way and `word`
    ///     should be ignored. `error` may contain further information
    ///     on the sort of error.
    fn received_word(&self, _word: u32, _rval: ReturnCode, _error: Error) {}

    /// A call to `Receive::receive_buffer` completed. The `ReturnCode`
    /// indicates whether the buffer was successfully received. A call
    /// to `receive_word` or `receive_buffer` made within this callback
    /// SHOULD NOT return EBUSY: when this callback is made the UART should
    /// be ready to receive another call.
    ///
    /// The `rx_len` argument specifies how many words were transmitted.
    /// An `rval` of SUCCESS indicates that every requested word was
    /// received: `rx_len` in the callback should be the same as
    /// `rx_len` in the initiating call.
    ///
    /// `rval` is SUCCESS if the full buffer was successfully received, or
    ///   - ECANCEL if the call to `received_buffer` was cancelled and
    ///     the buffer was not fully received. `rx_len` contains
    ///     how many words were received.
    ///   - ESIZE if the buffer could only be partially received. `rx_len`
    ///     contains how many words were transmitted.
    ///   - FAIL if reception failed in some way: `error` may contain further
    ///     information.
    fn received_buffer(&self, rx_buffer: &'static mut [u8], rx_len: usize, error: UartError);

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
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], rx_len: usize, interbyte_timeout: u8) -> Result<Success, UartError>;
}
