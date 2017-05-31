use returncode::ReturnCode;

// Defines the Interface between Capsules and Chips
pub trait RadioDriver {
    fn start_adv(&self);
    fn continue_adv(&self);
    fn set_adv_data(&self,
                    ad_type: usize,
                    &'static mut [u8],
                    len: usize,
                    offset: usize)
                    -> &'static mut [u8];

    // ADD MORE LATER
    fn flash_leds(&self);
    fn set_channel(&self, ch: usize);

    fn set_adv_txpower(&self, dbm: usize) -> ReturnCode;
}

pub trait Client {
    fn done_adv(&self) -> ReturnCode;
    fn continue_adv(&self);
}
