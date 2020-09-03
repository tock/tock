//! Interface for USB HID (Human Interface Device) class

use crate::returncode::ReturnCode;

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a> {
    /// Called when a packet is received
    fn packet_received(&'a self, buffer: &'static mut [u8; 64], len: usize, endpoint: usize);

    /// Called when a packet has been finished transmitting.
    fn packet_transmitted(
        &'a self,
        result: Result<(), ReturnCode>,
        buffer: &'static mut [u8; 64],
        len: usize,
        endpoint: usize,
    );

    /// Called when checking if we can receive any more data.
    /// Should return true if we are ready to receive.
    fn can_receive(&'a self) -> bool;
}

pub trait UsbHid<'a> {
    /// Sets the buffer where recevied data should be set.
    fn set_recv_buffer(&'a self, recv: &'static mut [u8; 64]);

    /// Sets the buffer to be sent and starts a send transaction.
    /// On success returns the number of bytes sent.
    /// On failure returns an error code and the buffer passed in.
    fn send_buffer(
        &'a self,
        send: &'static mut [u8; 64],
    ) -> Result<usize, (ReturnCode, &'static mut [u8; 64])>;

    /// Indicate that we can now receive data. Nothing will be received until
    /// this is called.
    fn allow_receive(&'a self);
}
