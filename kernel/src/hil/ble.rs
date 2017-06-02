use returncode::ReturnCode;

// Defines the Interface between Capsules and Chips
pub trait BleAdvertisementDriver {

    // Starts periodic advertisements on channel 37
    fn start_adv(&self);

    // Continues to send advertisements on channel 38 and 39.
    // This functions introduces a necessary delay between the transmissions.
    fn continue_adv(&self);

    // Sets the advertisement data for the specified AD TYPE.
    // The offset calculates where to put the data within the payload, this
    // enables multiple AD TYPES in the same payload.
    fn set_adv_data(&self,
                    ad_type: usize,
                    &'static mut [u8],
                    len: usize,
                    offset: usize)
                    -> &'static mut [u8];

    // Clear the payload.
    // The advertisement must be stopped before doing this.
    fn clear_adv_data(&self);

    // FIXME: Remove, should not be available for the capsule.
    fn set_channel(&self, ch: usize);

    // Sets the transmission power.
    fn set_adv_txpower(&self, dbm: usize) -> ReturnCode;
}

pub trait Client {
    // Signals that the radio has transmitted on all three channels.
    fn done_adv(&self) -> ReturnCode;

    // Tells the capsule that not all advertisements have been sent.
    fn continue_adv(&self);
}
