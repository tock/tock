//! Interface to USB controller hardware

use crate::common::cells::VolatileCell;

/// USB controller interface
pub trait UsbController {
    // Should be called before `enable_as_device()`
    fn endpoint_set_buffer(&self, endpoint: usize, buf: &[VolatileCell<u8>]);

    // Must be called before `attach()`
    fn enable_as_device(&self, speed: DeviceSpeed);

    fn attach(&self);

    fn detach(&self);

    fn set_address(&self, addr: u16);

    fn enable_address(&self);

    fn endpoint_ctrl_out_enable(&self, endpoint: usize);

    fn endpoint_bulk_in_enable(&self, endpoint: usize);

    fn endpoint_bulk_out_enable(&self, endpoint: usize);

    fn endpoint_bulk_resume(&self, endpoint: usize);
}

pub enum DeviceSpeed {
    Full,
    Low,
}

/// USB controller client interface
pub trait Client {
    fn enable(&self);
    fn attach(&self);
    fn bus_reset(&self);

    fn ctrl_setup(&self, endpoint: usize) -> CtrlSetupResult;
    fn ctrl_in(&self, endpoint: usize) -> CtrlInResult;
    fn ctrl_out(&self, endpoint: usize, packet_bytes: u32) -> CtrlOutResult;
    fn ctrl_status(&self, endpoint: usize);
    fn ctrl_status_complete(&self, endpoint: usize);

    fn bulk_in(&self, endpoint: usize) -> BulkInResult;
    fn bulk_out(&self, endpoint: usize, packet_bytes: u32) -> BulkOutResult;
}

#[derive(Debug)]
pub enum CtrlSetupResult {
    /// The Setup request was handled successfully
    Ok,
    OkSetAddress,

    // The Setup request cannot be handled; abort this transfer with STALL
    ErrBadLength,
    ErrNoParse,
    ErrNonstandardRequest,
    ErrUnrecognizedDescriptorType,
    ErrUnrecognizedRequestType,
    ErrNoDeviceQualifier,
    ErrInvalidDeviceIndex,
    ErrInvalidConfigurationIndex,
    ErrInvalidInterfaceIndex,
    ErrInvalidStringIndex,

    ErrGeneric,
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

pub enum BulkInResult {
    /// A packet of the given size was written into the endpoint buffer
    Packet(usize),

    /// The client is not yet able to provide data to the host, but may
    /// be able to in the future.  This result causes the controller
    /// to send a NAK token to the host.
    Delay,

    /// The client does not support the request.  This result causes the
    /// controller to send a STALL token to the host.
    Error,
}

pub enum BulkOutResult {
    /// The OUT packet was consumed
    Ok,

    /// The client is not yet able to consume data from the host, but may
    /// be able to in the future.  This result causes the controller
    /// to send a NAK token to the host.
    Delay,

    /// The client does not support the request.  This result causes the
    /// controller to send a STALL token to the host.
    Error,
}
