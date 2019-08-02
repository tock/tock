//! Traits for consoles in Tock.
//!
//! Consoles allow for communication between users and Tock. Common examples
//! include `printf()` in applications, an interactive console to inspect
//! process state, and debugging output from the kernel.

use crate::returncode::ReturnCode;

/// Main interface trait that consoles use to send and receive messages. The
/// buffers provided must not have any console mux header bytes.
pub trait Console<'a> {
    /// Function for a console to be able to send a message. It uses the
    /// standard buffer and length. The buffer should be only the
    /// console-specific data and should not contain any header information.
    ///
    /// The last parameter is an optional application ID that should only be
    /// used by the app console because the app console is actually forwarding a
    /// message on behalf of an application. All other consoles should set this
    /// parameter to `None`.
    ///
    /// The transmitter should not call this multiple times until the
    /// `transmitted_message()` callback has occurred.
    fn transmit_message(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        app_id: Option<u8>,
    ) -> (ReturnCode, Option<&'static mut [u8]>);

    /// Setup a receive buffer for this particular console. Since there will be
    /// many consoles, this buffer will be held by the mux until a received
    /// message comes in for the particular console.
    fn receive_message(
        &self,
        rx_buffer: &'static mut [u8],
    ) -> (ReturnCode, Option<&'static mut [u8]>);

    /// Cancel a receive operation. The message buffer will be returned through
    /// the received_message callback.
    fn receive_message_abort(&self);

    /// Provide a reference to the console client that will be called when
    /// messages come in or when transmissions have finished.
    fn set_client(&self, client: &'a ConsoleClient);
}

/// Callback interface for consoles. This is how consoles are signaled of new
/// messages and when transmissions are finished.
pub trait ConsoleClient {
    /// Called when a message has been sent for the particular client. This will
    /// return the static buffer back to the console.
    fn transmitted_message(&self, message: &'static mut [u8], tx_len: usize, rcode: ReturnCode);

    /// Called when a incoming message has been received for the particular
    /// client.
    fn received_message(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
    );
}
