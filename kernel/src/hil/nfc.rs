use crate::returncode::ReturnCode;

pub trait NfcTag<'a> {
    fn set_client(&self, client: &'a dyn Client<'a>);

    /// Enable NFC sense field mode, and subscribe
    /// to the relevant interrupt.
    fn enable(&self);

    /// Enables tag emulation.
    fn activate(&self);
    /// Disables tag emulation.
    fn deactivate(&self);

    /// Set the buffer to be transmitted and the amount of data
    /// trigger the task for transmission.
    fn transmit_buffer(&self, tx_buffer: &'static mut [u8], tx_amount: usize);

    /// Pass a buffer for receiving.
    fn receive_buffer(&self, rx_buffer: &'static mut [u8]);

    /// Configuration of the Tag according to its Type.
    fn configure(&self, tag_type: u8);

    /// Set the maximum frame delay in number of 13.56 MHz clocks.
    fn set_framedelaymax(&self, max_delay: u32);

    /// Enable the interrupt for event SELECTED
    fn unmask_select(&self);
}

pub trait Client<'a> {
    /// Notify the capsule of selection taking place.
    fn tag_selected(&'a self);

    /// Notify the app of the presence of
    /// a field to activate the tag.
    fn field_detected(&'a self);

    /// Notify the app of the absence of
    /// a field to deactivate the tag.
    fn field_lost(&'a self);

    /// Notify the app that a frame was received
    /// and can be read from the shared buffer.
    fn frame_received(&'a self, buffer: &'static mut [u8], rx_len: usize, result: ReturnCode);

    /// Notify the app that a frame was transmitted.
    fn frame_transmitted(&'a self, buffer: &'static mut [u8], result: ReturnCode);
}
