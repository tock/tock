// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::ErrorCode;

pub mod virtio_gpu;
pub mod virtio_input;
pub mod virtio_net;
pub mod virtio_rng;

/// VirtIO Device Types.
///
/// VirtIO is a flexible bus which can be used to expose various kinds of
/// virtual devices, such as network drivers, serial consoles, block devices or
/// random number generators. A VirtIO bus endpoint announces which type of
/// device it represents (and hence also which rules and semantics the VirtIO
/// driver should follow).
///
/// This enum maps the VirtIO device IDs to human-readable variants of an enum,
/// which can be used throughout the code base. Users should not rely on this
/// enum not being extended. Whenever an official device ID is missing, it can
/// be added to this enumeration.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum VirtIODeviceType {
    NetworkCard = 1,
    BlockDevice = 2,
    Console = 3,
    EntropySource = 4,
    TraditionalMemoryBallooning = 5,
    IoMemory = 6,
    RPMSG = 7,
    SCSIHost = 8,
    Transport9P = 9,
    Mac80211Wlan = 10,
    RPROCSerial = 11,
    VirtIOCAIF = 12,
    MemoryBalloon = 13,
    GPUDevice = 16,
    TimerClockDevice = 17,
    InputDevice = 18,
    SocketDevice = 19,
    CryptoDevice = 20,
    SignalDistributionModule = 21,
    PstoreDevice = 22,
    IOMMUDevice = 23,
    MemoryDevice = 24,
}

impl VirtIODeviceType {
    /// Try to create a [`VirtIODeviceType`] enum variant from a supplied
    /// numeric device ID.
    pub fn from_device_id(id: u32) -> Option<VirtIODeviceType> {
        use VirtIODeviceType as DT;

        match id {
            1 => Some(DT::NetworkCard),
            2 => Some(DT::BlockDevice),
            3 => Some(DT::Console),
            4 => Some(DT::EntropySource),
            5 => Some(DT::TraditionalMemoryBallooning),
            6 => Some(DT::IoMemory),
            7 => Some(DT::RPMSG),
            8 => Some(DT::SCSIHost),
            9 => Some(DT::Transport9P),
            10 => Some(DT::Mac80211Wlan),
            11 => Some(DT::RPROCSerial),
            12 => Some(DT::VirtIOCAIF),
            13 => Some(DT::MemoryBalloon),
            16 => Some(DT::GPUDevice),
            17 => Some(DT::TimerClockDevice),
            18 => Some(DT::InputDevice),
            19 => Some(DT::SocketDevice),
            20 => Some(DT::CryptoDevice),
            21 => Some(DT::SignalDistributionModule),
            22 => Some(DT::PstoreDevice),
            23 => Some(DT::IOMMUDevice),
            24 => Some(DT::MemoryDevice),
            _ => None,
        }
    }

    /// Convert a [`VirtIODeviceType`] variant to its corresponding device ID.
    pub fn to_device_id(device_type: VirtIODeviceType) -> u32 {
        device_type as u32
    }
}

/// VirtIO Device Driver.
///
/// This trait is to be implemented by drivers for exposed VirtIO devices, using
/// the transports provided in [`crate::transports`] and queues in
/// [`crate::queues`] to communicate with VirtIO devices.
pub trait VirtIODeviceDriver {
    /// VirtIO feature negotiation.
    ///
    /// This function is passed all driver-specific feature bits which the
    /// device exposes. Based on this function, the driver can select which
    /// features to enable through the return value of this function. This
    /// function is executed through the VirtIO transport, potentially before
    /// the device is initialized. As such, implementations of this function
    /// should be pure and only depend on the `offered_features` input
    /// parameter.
    fn negotiate_features(&self, offered_features: u64) -> Option<u64>;

    /// VirtIO device type which the driver supports.
    ///
    /// This function must return the VirtIO device type that the driver is able
    /// to drive. Implementations of this function must be pure and return a
    /// constant value.
    fn device_type(&self) -> VirtIODeviceType;

    /// Hook called before the transport indicates `DRIVER_OK` to the device.
    ///
    /// Because this trait must be object safe, a device cannot convey arbitrary
    /// errors through this interface. When this function returns an error, the
    /// transport will indicate `FAILED` to the device and return the error to
    /// the caller of
    /// [`VirtIOTransport::initialize`](super::transports::VirtIOTransport::initialize).
    /// The driver can store a more elaborate error internally and expose it
    /// through a custom interface.
    ///
    /// A default implementation of this function is provided which does nothing
    /// and returns `Ok(())`.
    fn pre_device_initialization(&self) -> Result<(), ErrorCode> {
        Ok(())
    }

    /// Hook called after the transport indicated `DRIVER_OK` to the device.
    ///
    /// Because this trait must be object safe, a device cannot convey arbitrary
    /// errors through this interface. When this function returns an error, the
    /// transport will **NOT** indicate `FAILED` to the device, but return the
    /// error to the caller of
    /// [`VirtIOTransport::initialize`](super::transports::VirtIOTransport::initialize).
    /// The driver can store a more elaborate error internally and expose it
    /// through a custom interface. The driver remains responsible for any
    /// deinitialization of the device as a result of this error.
    ///
    /// A default implementation of this function is provided which does nothing
    /// and returns `Ok(())`.
    fn device_initialized(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}
