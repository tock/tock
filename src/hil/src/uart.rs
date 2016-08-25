#[derive(Copy, Clone)]
pub enum Parity {
    Even = 0,
    Odd = 1,
    ForceZero = 2,
    ForceOne = 3,
    None = 4,
    Multidrop = 6,
}

#[derive(Copy, Clone)]
pub enum Mode {
    Normal = 0,
    FlowControl = 2,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    // Parity and stop bits should both be enums.
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub mode: Mode,
}

pub trait UART {
    fn init(&mut self, params: UARTParams);
    fn send_byte(&self, byte: u8);
    fn send_bytes(&self, bytes: &'static mut [u8], len: usize);
    fn read_byte(&self) -> u8;
    fn rx_ready(&self) -> bool;
    fn tx_ready(&self) -> bool;
    fn enable_rx(&self);
    fn disable_rx(&mut self);
    fn enable_tx(&self);
    fn disable_tx(&mut self);
}

pub trait Client {
    fn read_done(&self, byte: u8);
    fn write_done(&self, buffer: &'static mut [u8]);
}
