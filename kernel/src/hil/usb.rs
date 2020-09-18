//! Interface to USB controller hardware

use crate::common::cells::VolatileCell;

/// USB controller interface
pub trait UsbController<'a> {
    // Should be called before `enable_as_device()`
    fn endpoint_set_ctrl_buffer(&self, buf: &'a [VolatileCell<u8>]);
    fn endpoint_set_in_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]);
    fn endpoint_set_out_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]);

    // Must be called before `attach()`
    fn enable_as_device(&self, speed: DeviceSpeed);

    fn attach(&self);

    fn detach(&self);

    fn set_address(&self, addr: u16);

    fn enable_address(&self);

    fn endpoint_in_enable(&self, transfer_type: TransferType, endpoint: usize);

    fn endpoint_out_enable(&self, transfer_type: TransferType, endpoint: usize);

    fn endpoint_in_out_enable(&self, transfer_type: TransferType, endpoint: usize);

    fn endpoint_resume_in(&self, endpoint: usize);

    fn endpoint_resume_out(&self, endpoint: usize);

    fn set_config(&self, configuration_value: u8);

    fn endpoint_in_reset(&self, endpoint: usize);

    fn endpoint_out_reset(&self, endpoint: usize);
}

#[derive(Clone, Copy, Debug)]
pub enum TransferType {
    Control = 0,
    Isochronous,
    Bulk,
    Interrupt,
}

#[derive(Clone, Copy, Debug)]
pub enum DeviceSpeed {
    Full,
    Low,
}

/// USB controller client interface
pub trait Client<'a> {
    fn enable(&'a self);
    fn attach(&'a self);
    fn bus_reset(&'a self);

    fn ctrl_setup(&'a self, endpoint: usize) -> CtrlSetupResult;
    fn ctrl_in(&'a self, endpoint: usize) -> CtrlInResult;
    fn ctrl_out(&'a self, endpoint: usize, packet_bytes: u32) -> CtrlOutResult;
    fn ctrl_status(&'a self, endpoint: usize);
    fn ctrl_status_complete(&'a self, endpoint: usize);

    fn packet_in(&'a self, transfer_type: TransferType, endpoint: usize) -> InResult;
    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> OutResult;

    fn packet_transmitted(&'a self, endpoint: usize);
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

/// Result for IN packets sent on bulk or interrupt endpoints.
#[derive(Debug)]
pub enum InResult {
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

/// Result for OUT packets sent on bulk or interrupt endpoints.
#[derive(Debug)]
pub enum OutResult {
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
