//! functionality:
//!
//! - Preparing frames (data frame, command frames, beacon frames)
//! - Transmitting and receiving frames
//!
use msg::Frame;
use kernel::ReturnCode;

pub trait Device<'a> {
    /// Sets the transmission client of this MAC device
    fn set_transmit_client(&self, client: &'a TxClient);
    /// Sets the receive client of this MAC device
    fn set_receive_client(&self, client: &'a RxClient);

    /// Returns if the MAC device is currently on.
    fn is_on(&self) -> bool;

    /// Prepares a mutable buffer slice as an frame by writing the appropriate
    /// header bytes into the buffer. This needs to be done before adding the
    /// payload because the length of the header is not fixed.
    ///
    /// - `buf`: The mutable buffer slice to use
    /// - `seq`: The packet sequence for sent packet
    ///
    /// Returns either a Frame that is ready to have payload appended to it, or
    /// the mutable buffer if the frame cannot be prepared for any reason
    fn prepare_data_frame(
        &self,
        buf: &'static mut [u8],
        seq: &'static mut [u8],
    ) -> Result<Frame, &'static mut [u8]>;

    /// Transmits a frame that has been prepared by the above process. If the
    /// transmission process fails, the buffer inside the frame is returned so
    /// that it can be re-used.
    fn transmit(&self, frame: Frame) -> (ReturnCode, Option<&'static mut [u8]>);
}

/// Trait to be implemented by any user of the IEEE 802.15.4 device that
/// transmits frames. Contains a callback through which the static mutable
/// reference to the frame buffer is returned to the client.
pub trait TxClient {
    /// When transmission is complete or fails, return the buffer used for
    /// transmission to the client. `result` indicates whether or not
    /// the transmission was successful.
    ///
    /// - `buf`: The buffer used to contain the transmitted frame is
    /// returned to the client here.
    /// - `result`: This is `ReturnCode::SUCCESS` if the frame was transmitted,
    /// otherwise an error occured in the transmission pipeline.
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode);
}

/// Trait to be implemented by users of the IEEE 802.15.4 device that wish to
/// receive frames. The callback is triggered whenever a valid frame is
/// received, verified and unsecured (via the IEEE 802.15.4 security procedure)
/// successfully.
pub trait RxClient {
    /// When a frame is received, this callback is triggered. The client only
    /// receives an immutable borrow of the buffer. Only completely valid,
    /// unsecured frames that have passed the incoming security procedure are
    /// exposed to the client.
    ///
    /// - `buf`: The entire buffer containing the frame, including extra bytes
    /// in front used for the physical layer.
    /// - `data_len`: Length of the data payload
    fn receive_event<'a>(&self, buf: &'a [u8], data_len: usize);
}
