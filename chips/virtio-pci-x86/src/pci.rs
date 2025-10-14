// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// Suppress clippy warning generated within a tock-registers macro
#![allow(clippy::modulo_one)]

use core::ptr;

use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};

use pci_x86::cap::{Cap, VendorCap};
use pci_x86::{Command, Device};

use virtio::devices::{VirtIODeviceDriver, VirtIODeviceType};
use virtio::queues::Virtqueue;
use virtio::transports::{VirtIOInitializationError, VirtIOTransport};

/// PCI vendor ID for all Virtio devices
pub const VENDOR_ID: u16 = 0x1AF4;

/// Base value for PCI device IDs for Virtio devices
///
/// Actual device ID of a Virtio device (as reported via PCI configuration space) will be this base
/// value plus the device type ID as defined in section 5 of the Virtio spec.
///
/// For example, the PCI device ID of a Virtio network device (device type 1) would be
/// `DEVICE_ID_BASE + 1 = 0x1041`.
pub const DEVICE_ID_BASE: u16 = 0x1040;

/// Enumeration of Virtio configuration structure types
enum CfgType {
    Common,
    Notify,
    Isr,
    Device,
    Pci,
    SharedMemory,
    Vendor,
    Reserved,
}

/// A vendor-specific PCI capability which provides the location of a
/// Virtio configuration structure.
struct VirtioCap<'a>(VendorCap<'a>);

impl VirtioCap<'_> {
    /// Returns the type of configuration structure described by this
    /// capability.
    fn cfg_type(&self) -> CfgType {
        match self.0.read8(3) {
            1 => CfgType::Common,
            2 => CfgType::Notify,
            3 => CfgType::Isr,
            4 => CfgType::Device,
            5 => CfgType::Pci,
            8 => CfgType::SharedMemory,
            9 => CfgType::Vendor,
            _ => CfgType::Reserved,
        }
    }

    /// Returns index of the BAR containing the configuration structure.
    fn bar(&self) -> u8 {
        self.0.read8(4)
    }

    /// Returns the offset within the BAR region where the configuration
    /// structure starts.
    fn offset(&self) -> u32 {
        self.0.read32(8)
    }
}

register_bitfields![
    u8,

    /// Bits indicating the initialization status of a virtio device
    ///
    /// See virtio spec, section 2.1
    DeviceStatus [
        /// Guest OS has recognized this as a valid virtio device
        ACKNOWLEDGE OFFSET(0) NUMBITS(1) [],

        /// Guest OS has a driver available for this device
        DRIVER OFFSET(1) NUMBITS(1) [],

        /// Guest driver is loaded and ready to use
        DRIVER_OK OFFSET(2) NUMBITS(1) [],

        /// Guest OS has completed feature negotiation
        FEATURES_OK OFFSET(3) NUMBITS(1) [],

        /// Device has experienced an unrecoverable error
        DEVICE_NEEDS_RESET OFFSET(6) NUMBITS(1) [],

        /// Used by guest to indicate an unrecoverable error
        FAILED OFFSET(7) NUMBITS(1) [],
    ],

    /// Virtio ISR status register
    ///
    /// See virtio spec, section 4.1.4.5
    IsrStatus [
        /// Indicates a pending virtqueue interrupt
        QUEUE OFFSET(0) NUMBITS(1) [],

        /// Indicates a device configuration change
        DEVICE_CFG OFFSET(1) NUMBITS(1) [],
    ]
];

register_structs! {
    /// Virtio common configuration registers, as defined in section 4.1.4.3
    CommonCfg {
        //
        // About the whole device
        //

        /// Selects feature bits to be offered by the device
        (0x00 => device_feature_select: ReadWrite<u32>),

        /// Reports feature bits offered by the device
        (0x04 => device_feature: ReadOnly<u32>),

        /// Selects feature bits accepted by the driver
        (0x08 => driver_feature_select: ReadWrite<u32>),

        /// Reports feature bits accepted by the driver
        (0x0C => driver_feature: ReadWrite<u32>),

        /// Selects the MSI-X vector used by the device to report
        /// configuration changes.
        (0x10 => config_msix_vector: ReadWrite<u16>),

        /// Maximum number of virtqueues supported by the device (not
        /// including administration queues).
        (0x12 => num_queues: ReadOnly<u16>),

        /// Reports device status, as defined in section 2.1
        (0x14 => device_status: ReadWrite<u8, DeviceStatus::Register>),

        /// Updated by device each time configuration noticeably changes.
        (0x15 => config_generation: ReadOnly<u8>),

        //
        // About a specific virtqueue
        //

        /// Selects which virtqueue the following fields refer to
        (0x16 => queue_select: ReadWrite<u16>),

        /// Specifies the size of the selected virtqueue.
        ///
        /// On reset, specifies the maximum queue size supported by the
        /// device. A value of 0 means the queue is not available.
        (0x18 => queue_size: ReadWrite<u16>),

        /// Selects the MSI-X vector used by the device for virtqueue
        /// notifications.
        (0x1A => queue_msix_vector: ReadWrite<u16>),

        /// May be used by the driver to inhibit executiton of requests
        /// from the selected virtqueue.
        (0x1C => queue_enable: ReadWrite<u16>),

        /// Offset of notification address for the selected virtqueue
        ///
        /// See section 4.1.4.4 for details on computing the absolute
        /// notification address.
        (0x1E => queue_notify_off: ReadOnly<u16>),

        /// Physical address of descriptor area for the selected virtqueue
        (0x20 => queue_desc: ReadWrite<u64>),

        /// Physical address of driver area for the selected virtqueue
        (0x28 => queue_driver: ReadWrite<u64>),

        /// Physical address of device area for the selected virtqueue
        (0x30 => queue_device: ReadWrite<u64>),

        /// Value to be used by the driver when sending an available buffer
        /// notification to the device.
        ///
        /// Only relevant if the NOTIF_CONFIG_DATA feature bit has been
        /// negotiated.
        (0x38 => queue_notif_config_data: ReadOnly<u16>),

        /// Can be used to reset the selected virtqueue
        (0x3A => queue_reset: ReadWrite<u16>),

        //
        // About the administration virtqueue
        //

        /// Index of the first administration virtqueue
        (0x3C => admin_queue_index: ReadOnly<u16>),

        /// Number of administration virtqueues supported by the device
        (0x3E => admin_queue_num: ReadOnly<u16>),

        (0x40 => @END),
    },

    /// Virtio ISR status register, as defined by section 4.1.4.5
    IsrStatusCfg {
        (0x00 => isr_status: ReadOnly<u8, IsrStatus::Register>),

        (0x01 => @END),
    }
}

// Transport feature bits used here
const F_VERSION_1: u64 = 1u64 << 32;

pub struct VirtIOPCIDevice {
    dev: Device,
    dev_type: VirtIODeviceType,
    common_cfg: &'static CommonCfg,
    isr_cfg: &'static IsrStatusCfg,
    notify_base: usize,
    notify_off_multiplier: usize,
    queues: OptionalCell<&'static [&'static dyn Virtqueue]>,
}

impl VirtIOPCIDevice {
    /// Construct from a PCI device by parsing virtio-pci capability list.
    pub fn from_pci_device(dev: Device, dev_type: VirtIODeviceType) -> Option<Self> {
        let mut common_cfg_ptr = ptr::null_mut::<CommonCfg>();
        let mut isr_cfg_ptr = ptr::null_mut::<IsrStatusCfg>();
        let mut notify_base = 0;
        let mut notify_off_multiplier = 0;

        // Iterate over Virtio capabilities
        for cap in dev.capabilities() {
            let Cap::Vendor(cap) = cap else { continue };
            let cap = VirtioCap(cap);

            // Read capability fields and compute address of the
            // configuration structure
            let bar = cap.bar();
            let base = dev.bar_addr(bar)?;
            let offset = cap.offset() as usize;
            let addr = base + offset;

            // Store the pointer, but only the first time we encounter
            // it (as per the Virtio spec, section 4.1.4.1)
            match cap.cfg_type() {
                CfgType::Common => {
                    if common_cfg_ptr.is_null() {
                        common_cfg_ptr = addr as *mut CommonCfg;
                    }
                }

                CfgType::Isr => {
                    if isr_cfg_ptr.is_null() {
                        isr_cfg_ptr = addr as *mut IsrStatusCfg;
                    }
                }

                CfgType::Notify => {
                    notify_base = addr;
                    // virtio_pci_notify_cap.notify_off_multiplier at offset 16
                    notify_off_multiplier = cap.0.read32(16) as usize;
                }

                _ => {}
            }
        }

        // Return None if we were not able to find pointers for all the
        // necessary configuration structures. Otherwise, construct and return
        // the device object.
        //
        // Safety: We assume hardware is providing us with valid pointers.
        let common_cfg = unsafe { common_cfg_ptr.as_ref() }?;
        let isr_cfg = unsafe { isr_cfg_ptr.as_ref() }?;
        if notify_base == 0 || notify_off_multiplier == 0 {
            return None;
        }

        Some(Self {
            dev,
            dev_type,
            common_cfg,
            isr_cfg,
            notify_base,
            notify_off_multiplier,
            queues: OptionalCell::empty(),
        })
    }

    /// Helper method to negotiate device features during initialization.
    fn negotiate_features(
        &self,
        driver: &dyn VirtIODeviceDriver,
    ) -> Result<(), VirtIOInitializationError> {
        // Read the first 64 feature bits (spec doesn't define anything beyond that)
        let mut feats_offered = 0u64;
        self.common_cfg.device_feature_select.set(0);
        feats_offered |= self.common_cfg.device_feature.get() as u64;
        self.common_cfg.device_feature_select.set(1);
        feats_offered |= (self.common_cfg.device_feature.get() as u64) << 32;

        let mut feats_requested = 0u64;

        // Only transport feature we currently support or require is F_VERSION_1
        if (feats_offered & F_VERSION_1) != 0 {
            feats_requested |= F_VERSION_1;
        } else {
            return Err(VirtIOInitializationError::InvalidVirtIOVersion);
        }

        // Negotiate device-specific features
        let drv_feats_offered = feats_offered & 0xFFF;
        let drv_feats_requested = driver.negotiate_features(drv_feats_offered).ok_or(
            VirtIOInitializationError::FeatureNegotiationFailed {
                offered: feats_offered,
                accepted: None,
            },
        )?;
        feats_requested |= drv_feats_requested & 0xFFF;

        // Write requested features back to device
        self.common_cfg.driver_feature_select.set(0);
        self.common_cfg
            .driver_feature
            .set((feats_requested & 0xFFFF_FFFF) as u32);
        self.common_cfg.driver_feature_select.set(1);
        self.common_cfg
            .driver_feature
            .set(((feats_requested >> 32) & 0xFFFF_FFFF) as u32);

        // Lock in requested features and verify that the device accepted them
        self.common_cfg
            .device_status
            .modify(DeviceStatus::FEATURES_OK::SET);
        let feats_accepted = self
            .common_cfg
            .device_status
            .is_set(DeviceStatus::FEATURES_OK);
        if !feats_accepted {
            return Err(VirtIOInitializationError::FeatureNegotiationFailed {
                offered: feats_offered,
                accepted: Some(feats_requested),
            });
        }

        Ok(())
    }

    /// Helper method to prepare virtqueues during initialization.
    fn init_queues(
        &self,
        queues: &'static [&'static dyn Virtqueue],
    ) -> Result<(), VirtIOInitializationError> {
        for (index, queue) in queues.iter().enumerate() {
            self.common_cfg.queue_select.set(index as u16);

            // Must be disabled before configuration
            let enabled = self.common_cfg.queue_enable.get();
            if enabled != 0 {
                return Err(VirtIOInitializationError::DeviceError);
            }

            // Read max queue size; 0 means unavailable
            let max_size = self.common_cfg.queue_size.get() as usize;
            if max_size == 0 {
                return Err(VirtIOInitializationError::VirtqueueNotAvailable(index));
            }

            let size = queue.negotiate_queue_size(max_size);
            queue.initialize(index as u32, size);

            // Program selected size
            self.common_cfg.queue_size.set(size as u16);

            // Set queue addresses
            let addrs = queue.physical_addresses();
            self.common_cfg.queue_desc.set(addrs.descriptor_area);
            self.common_cfg.queue_driver.set(addrs.driver_area);
            self.common_cfg.queue_device.set(addrs.device_area);

            // Enable queue
            self.common_cfg.queue_enable.set(1);
        }

        self.queues.set(queues);

        Ok(())
    }

    /// Handle interrupt by checking ISR status and dispatching to queues.
    pub fn handle_interrupt(&self) {
        // Reading clears
        let isr = self.isr_cfg.isr_status.extract();

        if isr.is_set(IsrStatus::QUEUE) {
            self.queues.map(|queues| {
                for q in queues.iter() {
                    q.used_interrupt();
                }
            });
        }

        if isr.is_set(IsrStatus::DEVICE_CFG) {
            // Config change is currently unhandled
        }
    }
}

impl VirtIOTransport for VirtIOPCIDevice {
    fn initialize(
        &self,
        driver: &dyn VirtIODeviceDriver,
        queues: &'static [&'static dyn Virtqueue],
    ) -> Result<VirtIODeviceType, VirtIOInitializationError> {
        // Ensure the given driver matches the device type reported by the PCI device
        if self.dev_type != driver.device_type() {
            return Err(VirtIOInitializationError::IncompatibleDriverDeviceType(
                self.dev_type,
            ));
        }

        // Ensure bus properties are configured on the PCI device
        let mut cmd = self.dev.command();
        cmd.modify(Command::MEM_SPACE::SET);
        cmd.modify(Command::BUS_MASTER::SET);
        self.dev.set_command(cmd);

        //
        // The following code implements the Virtio initialization sequence as described in section
        // 3.1.1 of the spec, taking into account the specifics of the PCI transport as described in
        // section 4.1.5.1.
        //

        // 1. Reset device
        self.common_cfg.device_status.set(0);

        // 2. Set ACKNOWLEDGE bit, indicating we have recognized the device
        self.common_cfg
            .device_status
            .modify(DeviceStatus::ACKNOWLEDGE::SET);

        // 3. Set DRIVER bit, indicating we know how to drive the device
        self.common_cfg
            .device_status
            .modify(DeviceStatus::DRIVER::SET);

        let res = (|| {
            // 4-6. Feature negotiation
            self.negotiate_features(driver)?;

            // 7. Queue setup
            self.init_queues(queues)?;

            // Pre init hook
            driver
                .pre_device_initialization()
                .map_err(VirtIOInitializationError::DriverPreInitializationError)?;

            // 8. DRIVER_OK
            self.common_cfg
                .device_status
                .modify(DeviceStatus::DRIVER_OK::SET);

            // Live
            driver.device_initialized().map_err(|err| {
                VirtIOInitializationError::DriverInitializationError(self.dev_type, err)
            })?;

            Ok(self.dev_type)
        })();

        if res.is_err() {
            self.common_cfg
                .device_status
                .modify(DeviceStatus::FAILED::SET);
        }

        res
    }

    fn queue_notify(&self, queue_id: u32) {
        // When using PCI transport, available notifications are delivered by writing a value into
        // the memory-mapped "notification area" of the device.
        //
        // Section 4.1.4.4 describes how to compute the address to write for a specific virtqueue.
        //
        // Section 4.1.5.2 describes the value that should be written. Note that the current PCI
        // transport implementation in this module does not negotiate either the F_NOTIFICATION_DATA
        // or F_NOTIF_CONFIG_DATA feature bits, so the value written is simply the 16-bit queue
        // index.

        self.common_cfg.queue_select.set(queue_id as u16);

        // If queue index isn't valid, return early before attempting to write notification
        if self.common_cfg.queue_size.get() == 0 {
            return;
        }

        // Compute the byte address to write
        let off = self.common_cfg.queue_notify_off.get() as usize;
        let notify_addr = self.notify_base + (off * self.notify_off_multiplier);

        // Write notification value to the computed address
        //
        // Safety: Address is computed according to virtio spec. We have verified the specified
        // queue index is valid, and we assume all other values reported by the hardware are valid.
        let notify_ptr = notify_addr as *mut u16;
        unsafe {
            notify_ptr.write_volatile(queue_id as u16);
        }
    }
}
