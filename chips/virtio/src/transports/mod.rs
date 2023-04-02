// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! VirtIO transports.
//!
//! This module and its submodules provide abstractions for and implementations
//! of VirtIO transports. For more information, see the documentation of the
//! [`VirtIOTransport`] trait.

use kernel::ErrorCode;

use super::devices::{VirtIODeviceDriver, VirtIODeviceType};
use super::queues::Virtqueue;

pub mod mmio;

#[derive(Debug, Copy, Clone)]
pub enum VirtIOInitializationError {
    /// Device does not identify itself or can be recognized as a VirtIO device.
    NotAVirtIODevice,
    /// An unknown or incompatible VirtIO standard version.
    InvalidVirtIOVersion,
    /// An unknown or incompatible VirtIO transport device version.
    InvalidTransportVersion,
    /// Unknown VirtIO device type (as defined by the [`VirtIODeviceType`] enum).
    UnknownDeviceType(u32),
    /// Driver does not support driving the recognized device type.
    IncompatibleDriverDeviceType(VirtIODeviceType),
    /// Feature negotiation between the device and transport + device driver has
    /// failed.
    ///
    /// The device offered the `offered` features, the driver has either not
    /// accepted this feature set `accepted == None` or has responded with the
    /// `accepted` feature bitset, which the device has subsequently not
    /// acknowledged.
    FeatureNegotiationFailed { offered: u64, accepted: Option<u64> },
    /// The requested [`Virtqueue`] with respective index is not available with
    /// this VirtIO device.
    VirtqueueNotAvailable(usize),
    /// An error was reported by the
    /// [`VirtIODeviceDriver::pre_device_initialization`] function. The device
    /// has been put into the `FAILED` state.
    DriverPreInitializationError(ErrorCode),
    /// An error was reported by the [`VirtIODeviceDriver::device_initialized`]
    /// function. The device status has been previously indicated as `DRIVER_OK`
    /// and the transport has **NOT** put it into the `FAILED` state, although
    /// the driver might have. The initialization has continued as usual,
    /// despite reporting this error, hence it also carries the device type.
    DriverInitializationError(VirtIODeviceType, ErrorCode),
    /// An invariant was violated by the device. This error should not occur
    /// assuming a compliant device, transport and driver implementation.
    DeviceError,
}

/// VirtIO transports.
///
/// VirtIO can be used over multiple different transports, such as over a PCI
/// bus, an MMIO device or channel IO. This trait provides a basic abstraction
/// over such transports.
pub trait VirtIOTransport {
    /// Initialize the VirtIO transport using a device driver instance.
    ///
    /// This function is expected to run the basic initialization routine as
    /// defined for the various VirtIO transports. As part of this routine, it
    /// shall
    ///
    /// - negotiate device features,
    /// - invoke the driver `pre_device_initialization` hook before and
    ///   `device_initialized` hook after announcing the `DRIVER_OK` device
    ///   status flag to the device,
    /// - register the passed [`Virtqueue`]s with the device, calling the
    ///   [`Virtqueue::initialize`] function with the registered queue ID
    ///   _before_ registration,
    /// - as well as perform any other required initialization of the VirtIO
    ///   transport.
    ///
    /// The passed [`Virtqueue`]s are registered with a queue ID matching their
    /// offset in the supplied slice.
    ///
    /// If the initialization fails, it shall report this condition to the
    /// device _(setting `FAILED`) if_ it has started initializing the device
    /// (setting the `ACKNOWLEDGE` device status flag), and return an
    /// appropriate [`VirtIOInitializationError`]. Otherwise, it shall return
    /// the type of device connected to this VirtIO transport.
    fn initialize(
        &self,
        driver: &dyn VirtIODeviceDriver,
        queues: &'static [&'static dyn Virtqueue],
    ) -> Result<VirtIODeviceType, VirtIOInitializationError>;

    /// Notify the device of a changed [`Virtqueue`].
    ///
    /// Whenever a queue has been updated (e.g. move descriptors from the used
    /// to available ring) and these updates shall be made visible to the
    /// driver, the queue can invoke this function, passing its own respective
    /// queue ID.
    fn queue_notify(&self, queue_id: u32);
}
