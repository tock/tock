use crate::returncode::ReturnCode;

/// Controls an NFC tag main functionalities
pub trait NfcTag<'a> {
    /// Set the client instance that will handle callbacks
    fn set_client(&self, client: &'a dyn Client<'a>);

    /// Enable NFC sense field mode, and subscribe to the relevant interrupts.
    /// Also set up the default configurations of how frame delay should be
    /// dealt with (e.g. the maximum delay before a timeout).
    /// This function should never fail.
    fn enable(&self);

    /// Enable tag emulation by triggering the necessary task.
    /// This function should never fail.
    fn activate(&self);
    /// Notify the client by calling `field_lost()`. Then disable tag emulation
    /// by triggering the necessary task. And go back to state of sense field.
    /// This function should never fail.
    fn deactivate(&self);

    /// Pass the buffer to be transmitted and the amount of data and take
    /// ownership of it. Subscribe to the relevant interrupt and trigger
    /// the task for transmission.
    ///     
    /// On success returns the length of data to be sent.
    /// On failure returns an error code and the buffer passed in.
    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_amount: usize,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])>;

    /// Pass a buffer for receiving data and take ownership of it.
    ///     
    /// On success returns nothing.
    /// On failure returns an error code and the buffer passed in.
    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;

    /// Configuration of the Tag according to its Type.
    ///
    /// Return `SUCCESS` if the tag type is supported and
    /// `ENOSUPPORT` otherwise.
    fn configure(&self, tag_type: u8) -> ReturnCode;

    /// Set the maximum frame delay in number of 13.56 MHz clocks.
    fn set_framedelaymax(&self, max_delay: u32);

    /// Enable the interrupt for event SELECTED
    fn unmask_select(&self);
}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a> {
    /// Called when a selection event takes place.
    /// This will call `set_framedelaymax()` to update
    /// the default value in use.
    fn tag_selected(&'a self);

    /// Called when a field is detected.
    /// This will notify the app of the presence of a field to activate the tag.
    fn field_detected(&'a self);

    /// Called when a field is lost.
    /// This will notify the app of the absence of a field to deactivate the tag.
    fn field_lost(&'a self);

    /// Called when a frame is received.
    /// This will return the buffer passed into `receive_buffer()`.
    /// If the buffer length is smaller then the data length the buffer will only contain part
    /// of the frame the `result` will contain an `ENOMEM` error. If the received frame contained
    /// errors the `result` will contain a `FAIL` error.
    fn frame_received(&'a self, buffer: &'static mut [u8], rx_len: usize, result: ReturnCode);

    /// Called when a frame has finished transmitting.
    /// This will return the buffer passed into `transmit_buffer()`.
    /// If not all of the data could be sent because of a timeout the `result` will contain
    /// a `FAIL` error.
    fn frame_transmitted(&'a self, buffer: &'static mut [u8], result: ReturnCode);
}
