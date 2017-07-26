//! Interface to USB controller hardware

use common::VolatileCell;
use core::default::Default;

/// USB controller interface
pub trait UsbController {
    type EndpointState: Default;

    fn enable_device(&self, full_speed: bool);

    fn attach(&self);

    fn endpoint_configure(&self, &'static Self::EndpointState, index: u32);

    fn endpoint_set_buffer(&self, e: u32, buf: &[VolatileCell<u8>]);

    fn endpoint_ctrl_out_enable(&self, e: u32);

    fn set_address(&self, addr: u16);

    fn enable_address(&self);
}

/// USB controller client interface
pub trait Client {
    fn enable(&self);
    fn attach(&self);
    fn bus_reset(&self);

    fn ctrl_setup(&self) -> CtrlSetupResult;
    fn ctrl_in(&self) -> CtrlInResult;
    fn ctrl_out(&self, packet_bytes: u32) -> CtrlOutResult;
    fn ctrl_status(&self);
    fn ctrl_status_complete(&self);
}

#[derive(Debug)]
pub enum CtrlSetupResult {
    /// The Setup request was handled successfully
    Ok,

    // The Setup request cannot be handled; abort this transfer with STALL
    ErrBadLength,
    ErrNoParse,
    ErrNonstandardRequest,
    ErrUnrecognizedDescriptorType,
    ErrUnrecognizedRequestType,
    ErrNoDeviceQualifier,
    ErrInvalidDeviceIndex,
    ErrInvalidConfigurationIndex,
    ErrInvalidStringIndex,
}

pub enum CtrlInResult {
    /// A packet of the given size was written into the endpoint buffer
    Packet(usize, bool),

    /// The client is not yet able to provide data to the host, but may
    /// be able to in the future.  This result causes the controller
    /// to send a NAK token to the host.
    Delay,

    /// The client does not support the request.  This result causes the
    /// controller to send a STALL token to the host.
    Error,
}

pub enum CtrlOutResult {
    /// Data received (send ACK)
    Ok,

    /// Not ready yet (send NAK)
    Delay,

    /// In halt state (send STALL)
    Halted,
}
