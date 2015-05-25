#[derive(Copy, Clone)]
pub enum Parity {
    Even = 0,
    Odd = 1,
    ForceZero = 2,
    ForceOne = 3,
    None = 4,
    Multidrop = 6
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    // Parity and stop bits should both be enums.
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity
}

pub trait UART {
    fn init(&mut self, params: UARTParams);
    fn send_byte(&mut self, byte: u8);
    fn read_byte(&self) -> u8;
    fn enable_rx(&mut self);
    fn disable_rx(&mut self);
    fn enable_tx(&mut self);
    fn disable_tx(&mut self);
}

pub trait Reader {
    fn read_done(&mut self, byte: u8);
}
