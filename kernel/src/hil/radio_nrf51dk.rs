use returncode::ReturnCode;

// Defines the Interface between Capsules and Chips
pub trait RadioDriver {
    fn init(&self);
    fn start_adv(&self);
    fn continue_adv(&self);
    fn set_adv_data(&self, ad_type: usize, &'static mut [u8], len: usize, offset: usize) -> &'static mut [u8];

    fn set_adv_name(&self, &'static mut [u8], len: usize) -> ReturnCode;
    fn receive(&self);
    // ADD MORE LATER
    fn flash_leds(&self);
    fn set_channel(&self, ch: usize);

    fn set_adv_txpower(&self, dbm: usize) -> ReturnCode;
}

pub trait Client {
    /// Called when a rx or tx is finished
    fn receive_done(&self,
                    rx_data: &'static mut [u8],
                    dmy: &'static mut [u8],
                    len: u8)
                    -> ReturnCode;
    fn done_adv(&self) -> ReturnCode;
    fn continue_adv(&self);
}
