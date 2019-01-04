//! The contract satisfied by an implementation of an IEEE 802.15.4 MAC device.
//! Any IEEE 802.15.4 MAC device should expose the following high-level
//! functionality:
//!
//! - Configuration of addresses and transmit power
//! - Preparing frames (data frame, command frames, beacon frames)
//! - Transmitting and receiving frames
//!
//! Outlining this in a trait allows other implementations of MAC devices that
//! divide the responsibilities of software and hardware differently. For
//! example, a radio chip might be able to completely inline the frame security
//! procedure in hardware, as opposed to requiring a software implementation.

use crate::ieee802154::framer::Frame;
use crate::net::ieee802154::{Header, KeyId, MacAddress, PanID, SecurityLevel};
use kernel::ReturnCode;

pub trait MacDevice<'a> {
    /// Sets the transmission client of this MAC device
    fn set_transmit_client(&self, client: &'a TxClient);
    /// Sets the receive client of this MAC device
    fn set_receive_client(&self, client: &'a RxClient);

    /// The short 16-bit address of the MAC device
    fn get_address(&self) -> u16;
    /// The long 64-bit address (EUI-64) of the MAC device
    fn get_address_long(&self) -> [u8; 8];
    /// The 16-bit PAN ID of the MAC device
    fn get_pan(&self) -> u16;

    /// Set the short 16-bit address of the MAC device
    fn set_address(&self, addr: u16);
    /// Set the long 64-bit address (EUI-64) of the MAC device
    fn set_address_long(&self, addr: [u8; 8]);
    /// Set the 16-bit PAN ID of the MAC device
    fn set_pan(&self, id: u16);

    /// This method must be called after one or more calls to `set_*`. If
    /// `set_*` is called without calling `config_commit`, there is no guarantee
    /// that the underlying hardware configuration (addresses, pan ID) is in
    /// line with this MAC device implementation.
    fn config_commit(&self);

    /// Returns if the MAC device is currently on.
    fn is_on(&self) -> bool;

    /// Prepares a mutable buffer slice as an 802.15.4 frame by writing the appropriate
    /// header bytes into the buffer. This needs to be done before adding the
    /// payload because the length of the header is not fixed.
    ///
    /// - `buf`: The mutable buffer slice to use
    /// - `dst_pan`: The destination PAN ID
    /// - `dst_addr`: The destination MAC address
    /// - `src_pan`: The source PAN ID
    /// - `src_addr`: The source MAC address
    /// - `security_needed`: Whether or not this frame should be secured. This
    /// needs to be specified beforehand so that the auxiliary security header
    /// can be pre-inserted.
    ///
    /// Returns either a Frame that is ready to have payload appended to it, or
    /// the mutable buffer if the frame cannot be prepared for any reason
    fn prepare_data_frame(
        &self,
        buf: &'static mut [u8],
        dst_pan: PanID,
        dst_addr: MacAddress,
        src_pan: PanID,
        src_addr: MacAddress,
        security_needed: Option<(SecurityLevel, KeyId)>,
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
    /// - `spi_buf`: The buffer used to contain the transmitted frame is
    /// returned to the client here.
    /// - `acked`: Whether the transmission was acknowledged.
    /// - `result`: This is `ReturnCode::SUCCESS` if the frame was transmitted,
    /// otherwise an error occured in the transmission pipeline.
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode);
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
    /// - `header`: A fully-parsed representation of the MAC header, with the
    /// caveat that the auxiliary security header is still included if the frame
    /// was previously secured.
    /// - `data_offset`: Offset of the data payload relative to
    /// `buf`, so that the payload of the frame is contained in
    /// `buf[data_offset..data_offset + data_len]`.
    /// - `data_len`: Length of the data payload
    fn receive<'a>(&self, buf: &'a [u8], header: Header<'a>, data_offset: usize, data_len: usize);
}
