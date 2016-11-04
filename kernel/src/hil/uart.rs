/// UART hardware interface
#[derive(Copy, Clone)]
pub enum StopBits {
    One = 0,
    Two = 2,
}

#[derive(Copy, Clone)]
pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32, // baud rate in bit/s
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub hw_flow_control: bool,
}

/// The type of error encountered during UART transaction
#[derive(Copy, Clone, PartialEq)]
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

    /// No error occurred and the command completed successfully
    CommandComplete,
}

pub trait UART {
    /// Initialize UART
    ///
    /// # Panics if UARTParams are invalid for the current chip
    fn init(&self, params: UARTParams);

    /// Transmit data
    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize);

    /// Receive data until buffer is full
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize);
}


/// Implement Client to receive callbacks from UART
pub trait Client {
    /// UART transmit complete
    fn transmit_complete(&self, tx_buffer: &'static mut [u8], error: Error);

    /// UART receive complete
    fn receive_complete(&self, rx_buffer: &'static mut [u8], rx_len: usize, error: Error);
}
