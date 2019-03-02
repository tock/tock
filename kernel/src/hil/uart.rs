//! Hardware interface layer (HIL) traits for UART communication.
//!
//!
use crate::ikc;
use crate::returncode::ReturnCode;

use crate::ikc::DriverState::IDLE;

pub type TxRequest<'a> = ikc::TxRequest<'a, u8>;
pub type RxRequest<'a> = ikc::RxRequest<'a, u8>;
pub type State = ikc::DriverState;

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

/// The type of error encountered during UART Request.
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

pub trait Uart<'a>: Configure + Transmit<'a> + Receive<'a> {}
pub trait UartData<'a>: Transmit<'a> + Receive<'a> {}
pub trait UartPeripheral<'a>:
    Configure + Transmit<'a> + Receive<'a> + InterruptHandler<'a>
{
}

pub trait UartAdvanced<'a>: Configure + Transmit<'a> + ReceiveAdvanced<'a> {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PeripheralState {
    pub tx: State,
    pub rx: State,
}

impl<'a> PeripheralState {
    pub fn new() -> PeripheralState {
        PeripheralState { tx: IDLE, rx: IDLE }
    }
}

/// Trait for configuring a UART.
pub trait InterruptHandler<'a> {
    fn handle_interrupt(&self, state: PeripheralState) -> (Option<&mut TxRequest<'a>>, Option<&mut RxRequest<'a>>);
}

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
    /// Transmit a buffer of data. On completion, `transmitted_buffer`
    /// in the `TransmitClient` will be called.  If the `ReturnCode`
    /// of `transmit`'s return tuple is SUCCESS, the `Option` will be
    /// `None` and the struct will issue a `transmitted_buffer`
    /// callback in the future. If the value of the `ReturnCode` is
    /// not SUCCESS, then the `tx_buffer` argument is returned in the
    /// `Option`. Other valid `ReturnCode` values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    ///  - ESIZE : `tx_len` is larger than the passed slice.
    ///  - FAIL: some other error.
    ///
    /// Each byte in `tx_buffer` is a UART transfer word of 8 or fewer
    /// bits.  The word width is determined by the UART configuration,
    /// truncating any more significant bits. E.g., 0x18f transmitted in
    /// 8N1 will be sent as 0x8f and in 7N1 will be sent as 0x0f. Clients
    /// that need to transfer 9-bit words should use `transmit_word`.
    ///
    /// Calling `transmit_buffer` while there is an outstanding
    /// `transmit_buffer` or `transmit_word` operation will return EBUSY.
    fn transmit_buffer(
        &self,
        req: &'a mut TxRequest<'a>,
    ) -> ReturnCode;

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
    ///  - FAIL: not supported, or some other error.
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
    fn transmit_abort(&self) -> Option<&'a mut TxRequest<'a>>;
}

pub trait Receive<'a> {
    /// Receive `rx_len` bytes into `rx_buffer`, making a callback to
    /// the `ReceiveClient` when complete.  If the `ReturnCode` of
    /// `receive_buffer`'s return tuple is SUCCESS, the `Option` will
    /// be `None` and the struct will issue a `received_buffer`
    /// callback in the future. If the value of the `ReturnCode` is
    /// not SUCCESS, then the `rx_buffer` argument is returned in the
    /// `Option`. Other valid return values are:
    ///  - EOFF: The underlying hardware is not available, perhaps because
    ///          because it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - EBUSY: the UART is already receiving and has not made a
    ///           reception `complete` callback yet.
    ///  - ESIZE : `rx_len` is larger than the passed slice.
    /// Each byte in `rx_buffer` is a UART transfer word of 8 or fewer
    /// bits.  The width is determined by the UART
    /// configuration. Clients that need to transfer 9-bit words
    /// should use `receive_word`.  Calling `receive_buffer` while
    /// there is an outstanding `receive_buffer` or `receive_word`
    /// operation will return EBUSY.
    fn receive_buffer(
        &self,
        req: &'a mut RxRequest<'a>,
    ) -> ReturnCode;

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
    ///  - FAIL: not supported or some other error.
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
    fn receive_abort(&self) -> Option<&'a mut RxRequest<'a>>;
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
    fn receive_automatic(
        &self,
        rx_buffer: &'a mut [u8],
        rx_len: usize,
        interbyte_timeout: u8,
    ) -> (ReturnCode, Option<&'a mut [u8]>);
}

pub trait Client<'a> {
    fn has_tx_request(&self) -> bool;
    fn get_tx_request(&self) -> Option<&mut TxRequest<'a>>;
    // uart_num allows client to identify which uart this tx_request_complete call is originating from 
    // for the case where it is client of multiple UARTS
    fn tx_request_complete(&self, uart_num: usize, returned_request: &'a mut TxRequest<'a>);

    fn has_rx_request(&self) -> bool;

    fn get_rx_request(&self) -> Option<&mut RxRequest<'a>>;
    // uart_num allows client to identify which uart this rx_request_complete call is originating from 
    // for the case where it is client of multiple UARTS
    fn rx_request_complete(&self, _uart_num: usize, _returned_request: &'a mut RxRequest<'a>);
}