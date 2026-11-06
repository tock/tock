// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! USB regulator for the nRF5340 application core.
//!
//! It reports the VBUS and USB supply events that the USBD needs to power up
//! and down, which the POWER peripheral reports on nRF52.

use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};
use nrf5x::usbd::{UsbPower, Usbd};

register_structs! {
    pub UsbRegRegisters {
        (0x000 => _reserved0),
        (0x100 => events_usbdetected: ReadWrite<u32, Event::Register>),
        (0x104 => events_usbremoved: ReadWrite<u32, Event::Register>),
        (0x108 => events_usbpwrrdy: ReadWrite<u32, Event::Register>),
        (0x10C => _reserved1),
        (0x304 => intenset: ReadWrite<u32, Interrupt::Register>),
        (0x308 => _reserved2),
        (0x400 => usbregstatus: ReadOnly<u32, UsbRegStatus::Register>),
        (0x404 => @END),
    }
}

register_bitfields! [u32,
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],
    Interrupt [
        USBDETECTED OFFSET(0) NUMBITS(1),
        USBREMOVED OFFSET(1) NUMBITS(1),
        USBPWRRDY OFFSET(2) NUMBITS(1)
    ],
    UsbRegStatus [
        VBUSDETECT OFFSET(0) NUMBITS(1),
        OUTPUTRDY OFFSET(1) NUMBITS(1)
    ]
];

pub struct UsbReg<'a> {
    registers: StaticRef<UsbRegRegisters>,
    usbd: OptionalCell<&'a Usbd<'a>>,
}

impl<'a> UsbReg<'a> {
    pub const fn new(registers: StaticRef<UsbRegRegisters>) -> Self {
        Self {
            registers,
            usbd: OptionalCell::empty(),
        }
    }

    /// Forward USB plug, unplug and supply-ready events to `usbd`.
    pub fn set_usbd(&self, usbd: &'a Usbd<'a>) {
        self.usbd.set(usbd);
        self.registers.intenset.write(
            Interrupt::USBDETECTED::SET + Interrupt::USBREMOVED::SET + Interrupt::USBPWRRDY::SET,
        );
    }

    pub fn handle_interrupt(&self) {
        if self.registers.events_usbdetected.is_set(Event::READY) {
            self.registers.events_usbdetected.write(Event::READY::CLEAR);
            self.usbd.map(|usbd| usbd.enable());
        }
        if self.registers.events_usbremoved.is_set(Event::READY) {
            self.registers.events_usbremoved.write(Event::READY::CLEAR);
            self.usbd.map(|usbd| usbd.disable());
        }
        if self.registers.events_usbpwrrdy.is_set(Event::READY) {
            self.registers.events_usbpwrrdy.write(Event::READY::CLEAR);
            self.usbd.map(|usbd| usbd.power_ready());
        }
    }
}

impl UsbPower for UsbReg<'_> {
    fn is_vbus_present(&self) -> bool {
        self.registers.usbregstatus.is_set(UsbRegStatus::VBUSDETECT)
    }

    fn is_usb_power_ready(&self) -> bool {
        self.registers.usbregstatus.is_set(UsbRegStatus::OUTPUTRDY)
    }
}
