use returncode::ReturnCode;

// Defines the Interface between Capsules and Chips
pub trait RadioDriver {
    fn init(&self);
    fn send(&self);
    fn receive(&self);
    fn transmit(&self,
                local_name: &'static mut [u8],
                name_len: usize,
                data: &'static mut [u8],
                data_len: usize)
                -> ReturnCode;
    // ADD MORE LATER
    fn flash_leds(&self);
    fn set_channel(&self, ch: usize);
}

pub trait Client {
    /// Called when a rx or tx is finished
    fn receive_done(&self, rx_data: &'static mut [u8], len: u8) -> ReturnCode;
    fn transmit_done(&self, tx_data: &'static mut [u8], len: u8) -> ReturnCode;
}
