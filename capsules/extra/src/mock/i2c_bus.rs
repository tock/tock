// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mock I2C bus: a "reverse" I2C virtualizer for connecting mock devices.
//!
//! The normal I2C virtualizer
//! ([`MuxI2C`](capsules_core::virtualizers::virtual_i2c::MuxI2C)) fans a
//! single controller out to many capsules, each bound to one address. This
//! module does the mirror image: it presents a single
//! [`i2c::I2CMaster`] interface "upward" (so the normal virtualizer and the
//! real device drivers can sit on top of it unchanged) and fans "downward" to
//! a list of mock chips, each registered at an I2C address.
//!
//! When a transaction arrives on the [`i2c::I2CMaster`] interface, the bus
//! looks up the attached device whose address matches and forwards the
//! transaction to that device's [`i2c::I2CDevice`] interface. The device
//! completes asynchronously (a mock typically uses a deferred call to imitate
//! the controller interrupt) and calls back on the bus's [`i2c::I2CClient`]
//! implementation, which the bus forwards up to its own
//! [`i2c::I2CHwMasterClient`].
//!
//! ```text
//!            capsule (e.g. SHT4x driver)
//!                     |  i2c::I2CDevice
//!         MuxI2C + virtual_i2c::I2CDevice        (normal, "forward" virtualizer)
//!                     |  i2c::I2CMaster
//!               MockI2CBus  <-- this module      ("reverse" virtualizer)
//!                    / \     i2c::I2CDevice
//!         MockSht4x    (other mock chips ...)
//! ```
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let mock_sht4x = static_init!(MockSht4x, MockSht4x::new());
//! mock_sht4x.register();
//!
//! let i2c_bus = static_init!(MockI2CBus, MockI2CBus::new());
//!
//! let sht4x_on_bus = static_init!(
//!     I2CBusDevice,
//!     I2CBusDevice::new(mock_sht4x, capsules_extra::mock::sensors::sht4x::BASE_ADDR)
//! );
//! i2c_bus.add_device(sht4x_on_bus);
//! mock_sht4x.set_client(i2c_bus);
//!
//! // Stack the normal I2C virtualizer on top of the mock bus:
//! let mux_i2c = components::i2c::I2CMuxComponent::new(i2c_bus, None)
//!     .finalize(components::i2c_mux_component_static!(MockI2CBus<'static>));
//! ```

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::i2c::{self, I2CHwMasterClient};
use kernel::utilities::cells::OptionalCell;

/// Device-side half of a mock I2C chip that can be plugged into a
/// [`MockI2CBus`].
///
/// It is just [`i2c::I2CDevice`] plus a way for the bus to register itself as
/// the chip's completion-callback client, so the mock's asynchronous
/// `command_complete` flows back through the bus to the controller-side
/// client. The mock chip's `'a` is the lifetime of its own storage.
pub trait MockI2CDevice<'a>: i2c::I2CDevice {
    fn set_i2c_client(&'a self, client: &'a dyn i2c::I2CClient);
}

/// One mock device attached to a [`MockI2CBus`] at a fixed I2C address.
///
/// This is the list node the bus iterates over; the board allocates one per
/// attached mock chip and hands it to [`MockI2CBus::add_device`].
pub struct I2CBusDevice<'a> {
    /// The mock chip, addressed through its device-side interface.
    device: &'a dyn i2c::I2CDevice,
    /// The 7-bit I2C address this device answers to.
    address: u8,
    /// Linked-list plumbing.
    next: ListLink<'a, I2CBusDevice<'a>>,
}

impl<'a> I2CBusDevice<'a> {
    pub fn new(device: &'a dyn i2c::I2CDevice, address: u8) -> Self {
        Self {
            device,
            address,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, I2CBusDevice<'a>> for I2CBusDevice<'a> {
    fn next(&'a self) -> &'a ListLink<'a, I2CBusDevice<'a>> {
        &self.next
    }
}

/// A fake I2C bus that dispatches controller-side transactions to a list of
/// attached mock devices by address.
pub struct MockI2CBus<'a> {
    /// Upward client: whatever sits on the controller side (typically a
    /// [`MuxI2C`](capsules_core::virtualizers::virtual_i2c::MuxI2C)).
    master_client: OptionalCell<&'a dyn I2CHwMasterClient>,
    /// The attached mock devices.
    devices: List<'a, I2CBusDevice<'a>>,
    /// The device currently servicing a transaction, if any. The bus is
    /// strictly one-transaction-at-a-time, like real hardware.
    inflight: OptionalCell<&'a I2CBusDevice<'a>>,
}

impl Default for MockI2CBus<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MockI2CBus<'a> {
    pub const fn new() -> Self {
        Self {
            master_client: OptionalCell::empty(),
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    /// Attach a mock device to the bus. `device` must live as long as the bus.
    pub fn add_device(&self, device: &'a I2CBusDevice<'a>) {
        self.devices.push_head(device);
    }

    /// Find the attached device answering to `address`.
    fn device_for(&self, address: u8) -> Option<&'a I2CBusDevice<'a>> {
        self.devices.iter().find(|dev| dev.address == address)
    }
}

impl<'a> i2c::I2CMaster<'a> for MockI2CBus<'a> {
    fn set_master_client(&self, master_client: &'a dyn I2CHwMasterClient) {
        self.master_client.set(master_client);
    }

    fn enable(&self) {
        // Turn on every attached device; a real controller powering up brings
        // the whole bus with it, and the mocks refuse transactions until
        // enabled.
        for dev in self.devices.iter() {
            dev.device.enable();
        }
    }

    fn disable(&self) {
        for dev in self.devices.iter() {
            dev.device.disable();
        }
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.inflight.is_some() {
            return Err((i2c::Error::Busy, data));
        }
        match self.device_for(addr) {
            Some(dev) => match dev.device.write(data, len) {
                Ok(()) => {
                    self.inflight.set(dev);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            None => Err((i2c::Error::AddressNak, data)),
        }
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.inflight.is_some() {
            return Err((i2c::Error::Busy, buffer));
        }
        match self.device_for(addr) {
            Some(dev) => match dev.device.read(buffer, len) {
                Ok(()) => {
                    self.inflight.set(dev);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            None => Err((i2c::Error::AddressNak, buffer)),
        }
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.inflight.is_some() {
            return Err((i2c::Error::Busy, data));
        }
        match self.device_for(addr) {
            Some(dev) => match dev.device.write_read(data, write_len, read_len) {
                Ok(()) => {
                    self.inflight.set(dev);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            None => Err((i2c::Error::AddressNak, data)),
        }
    }
}

impl i2c::I2CClient for MockI2CBus<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        // A mock device finished its transaction; the bus is free again.
        self.inflight.take();
        self.master_client
            .map(move |client| client.command_complete(buffer, status));
    }
}
