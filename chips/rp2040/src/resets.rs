// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{FieldValue, ReadWrite, register_bitfields, register_structs};

register_structs! {
    ResetsRegisters {
        /// Reset control. If a bit is set it means the peripheral is in reset. 0 means the
        (0x000 => reset: ReadWrite<u32, RESET::Register>),
        /// Watchdog select. If a bit is set then the watchdog will reset this peripheral wh
        (0x004 => wdsel: ReadWrite<u32, WDSEL::Register>),
        /// Reset done. If a bit is set then a reset done signal has been returned by the pe
        (0x008 => reset_done: ReadWrite<u32, RESET_DONE::Register>),
        (0x00C => @END),
    }
}
register_bitfields![u32,
    RESET [

        usbctrl OFFSET(24) NUMBITS(1) [],

        uart1 OFFSET(23) NUMBITS(1) [],

        uart0 OFFSET(22) NUMBITS(1) [],

        timer OFFSET(21) NUMBITS(1) [],

        tbman OFFSET(20) NUMBITS(1) [],

        sysinfo OFFSET(19) NUMBITS(1) [],

        syscfg OFFSET(18) NUMBITS(1) [],

        spi1 OFFSET(17) NUMBITS(1) [],

        spi0 OFFSET(16) NUMBITS(1) [],

        rtc OFFSET(15) NUMBITS(1) [],

        pwm OFFSET(14) NUMBITS(1) [],

        pll_usb OFFSET(13) NUMBITS(1) [],

        pll_sys OFFSET(12) NUMBITS(1) [],

        pio1 OFFSET(11) NUMBITS(1) [],

        pio0 OFFSET(10) NUMBITS(1) [],

        pads_qspi OFFSET(9) NUMBITS(1) [],

        pads_bank0 OFFSET(8) NUMBITS(1) [],

        jtag OFFSET(7) NUMBITS(1) [],

        io_qspi OFFSET(6) NUMBITS(1) [],

        io_bank0 OFFSET(5) NUMBITS(1) [],

        i2c1 OFFSET(4) NUMBITS(1) [],

        i2c0 OFFSET(3) NUMBITS(1) [],

        dma OFFSET(2) NUMBITS(1) [],

        busctrl OFFSET(1) NUMBITS(1) [],

        adc OFFSET(0) NUMBITS(1) []
    ],
    WDSEL [

        usbctrl OFFSET(24) NUMBITS(1) [],

        uart1 OFFSET(23) NUMBITS(1) [],

        uart0 OFFSET(22) NUMBITS(1) [],

        timer OFFSET(21) NUMBITS(1) [],

        tbman OFFSET(20) NUMBITS(1) [],

        sysinfo OFFSET(19) NUMBITS(1) [],

        syscfg OFFSET(18) NUMBITS(1) [],

        spi1 OFFSET(17) NUMBITS(1) [],

        spi0 OFFSET(16) NUMBITS(1) [],

        rtc OFFSET(15) NUMBITS(1) [],

        pwm OFFSET(14) NUMBITS(1) [],

        pll_usb OFFSET(13) NUMBITS(1) [],

        pll_sys OFFSET(12) NUMBITS(1) [],

        pio1 OFFSET(11) NUMBITS(1) [],

        pio0 OFFSET(10) NUMBITS(1) [],

        pads_qspi OFFSET(9) NUMBITS(1) [],

        pads_bank0 OFFSET(8) NUMBITS(1) [],

        jtag OFFSET(7) NUMBITS(1) [],

        io_qspi OFFSET(6) NUMBITS(1) [],

        io_bank0 OFFSET(5) NUMBITS(1) [],

        i2c1 OFFSET(4) NUMBITS(1) [],

        i2c0 OFFSET(3) NUMBITS(1) [],

        dma OFFSET(2) NUMBITS(1) [],

        busctrl OFFSET(1) NUMBITS(1) [],

        adc OFFSET(0) NUMBITS(1) []
    ],
    RESET_DONE [

        usbctrl OFFSET(24) NUMBITS(1) [],

        uart1 OFFSET(23) NUMBITS(1) [],

        uart0 OFFSET(22) NUMBITS(1) [],

        timer OFFSET(21) NUMBITS(1) [],

        tbman OFFSET(20) NUMBITS(1) [],

        sysinfo OFFSET(19) NUMBITS(1) [],

        syscfg OFFSET(18) NUMBITS(1) [],

        spi1 OFFSET(17) NUMBITS(1) [],

        spi0 OFFSET(16) NUMBITS(1) [],

        rtc OFFSET(15) NUMBITS(1) [],

        pwm OFFSET(14) NUMBITS(1) [],

        pll_usb OFFSET(13) NUMBITS(1) [],

        pll_sys OFFSET(12) NUMBITS(1) [],

        pio1 OFFSET(11) NUMBITS(1) [],

        pio0 OFFSET(10) NUMBITS(1) [],

        pads_qspi OFFSET(9) NUMBITS(1) [],

        pads_bank0 OFFSET(8) NUMBITS(1) [],

        jtag OFFSET(7) NUMBITS(1) [],

        io_qspi OFFSET(6) NUMBITS(1) [],

        io_bank0 OFFSET(5) NUMBITS(1) [],

        i2c1 OFFSET(4) NUMBITS(1) [],

        i2c0 OFFSET(3) NUMBITS(1) [],

        dma OFFSET(2) NUMBITS(1) [],

        busctrl OFFSET(1) NUMBITS(1) [],

        adc OFFSET(0) NUMBITS(1) []
    ]
];
const RESETS_BASE: StaticRef<ResetsRegisters> =
    unsafe { StaticRef::new(0x4000C000 as *const ResetsRegisters) };

pub enum Peripheral {
    Adc,
    BusController,
    Dma,
    I2c0,
    I2c1,
    IOBank0,
    IOQSpi,
    Jtag,
    PadsBank0,
    PadsQSpi,
    Pio0,
    Pio1,
    PllSys,
    PllUsb,
    Pwm,
    Rtc,
    Spi0,
    Spi1,
    Syscfg,
    SysInfo,
    TBMan,
    Timer,
    Uart0,
    Uart1,
    UsbCtrl,
}

impl Peripheral {
    fn get_reset_field_set(&self) -> FieldValue<u32, RESET::Register> {
        match self {
            Self::Adc => RESET::adc::SET,
            Self::BusController => RESET::busctrl::SET,
            Self::Dma => RESET::dma::SET,
            Self::I2c0 => RESET::i2c0::SET,
            Self::I2c1 => RESET::i2c1::SET,
            Self::IOBank0 => RESET::io_bank0::SET,
            Self::IOQSpi => RESET::io_qspi::SET,
            Self::Jtag => RESET::jtag::SET,
            Self::PadsBank0 => RESET::pads_bank0::SET,
            Self::PadsQSpi => RESET::pads_qspi::SET,
            Self::Pio0 => RESET::pio0::SET,
            Self::Pio1 => RESET::pio1::SET,
            Self::PllSys => RESET::pll_sys::SET,
            Self::PllUsb => RESET::pll_usb::SET,
            Self::Pwm => RESET::pwm::SET,
            Self::Rtc => RESET::rtc::SET,
            Self::Spi0 => RESET::spi0::SET,
            Self::Spi1 => RESET::spi1::SET,
            Self::Syscfg => RESET::syscfg::SET,
            Self::SysInfo => RESET::sysinfo::SET,
            Self::TBMan => RESET::tbman::SET,
            Self::Timer => RESET::timer::SET,
            Self::Uart0 => RESET::uart0::SET,
            Self::Uart1 => RESET::uart1::SET,
            Self::UsbCtrl => RESET::usbctrl::SET,
        }
    }

    fn get_reset_field_clear(&self) -> FieldValue<u32, RESET::Register> {
        match self {
            Self::Adc => RESET::adc::CLEAR,
            Self::BusController => RESET::busctrl::CLEAR,
            Self::Dma => RESET::dma::CLEAR,
            Self::I2c0 => RESET::i2c0::CLEAR,
            Self::I2c1 => RESET::i2c1::CLEAR,
            Self::IOBank0 => RESET::io_bank0::CLEAR,
            Self::IOQSpi => RESET::io_qspi::CLEAR,
            Self::Jtag => RESET::jtag::CLEAR,
            Self::PadsBank0 => RESET::pads_bank0::CLEAR,
            Self::PadsQSpi => RESET::pads_qspi::CLEAR,
            Self::Pio0 => RESET::pio0::CLEAR,
            Self::Pio1 => RESET::pio1::CLEAR,
            Self::PllSys => RESET::pll_sys::CLEAR,
            Self::PllUsb => RESET::pll_usb::CLEAR,
            Self::Pwm => RESET::pwm::CLEAR,
            Self::Rtc => RESET::rtc::CLEAR,
            Self::Spi0 => RESET::spi0::CLEAR,
            Self::Spi1 => RESET::spi1::CLEAR,
            Self::Syscfg => RESET::syscfg::CLEAR,
            Self::SysInfo => RESET::sysinfo::CLEAR,
            Self::TBMan => RESET::tbman::CLEAR,
            Self::Timer => RESET::timer::CLEAR,
            Self::Uart0 => RESET::uart0::CLEAR,
            Self::Uart1 => RESET::uart1::CLEAR,
            Self::UsbCtrl => RESET::usbctrl::CLEAR,
        }
    }

    fn get_reset_done_field_set(&self) -> FieldValue<u32, RESET_DONE::Register> {
        match self {
            Self::Adc => RESET_DONE::adc::SET,
            Self::BusController => RESET_DONE::busctrl::SET,
            Self::Dma => RESET_DONE::dma::SET,
            Self::I2c0 => RESET_DONE::i2c0::SET,
            Self::I2c1 => RESET_DONE::i2c1::SET,
            Self::IOBank0 => RESET_DONE::io_bank0::SET,
            Self::IOQSpi => RESET_DONE::io_qspi::SET,
            Self::Jtag => RESET_DONE::jtag::SET,
            Self::PadsBank0 => RESET_DONE::pads_bank0::SET,
            Self::PadsQSpi => RESET_DONE::pads_qspi::SET,
            Self::Pio0 => RESET_DONE::pio0::SET,
            Self::Pio1 => RESET_DONE::pio1::SET,
            Self::PllSys => RESET_DONE::pll_sys::SET,
            Self::PllUsb => RESET_DONE::pll_usb::SET,
            Self::Pwm => RESET_DONE::pwm::SET,
            Self::Rtc => RESET_DONE::rtc::SET,
            Self::Spi0 => RESET_DONE::spi0::SET,
            Self::Spi1 => RESET_DONE::spi1::SET,
            Self::Syscfg => RESET_DONE::syscfg::SET,
            Self::SysInfo => RESET_DONE::sysinfo::SET,
            Self::TBMan => RESET_DONE::tbman::SET,
            Self::Timer => RESET_DONE::timer::SET,
            Self::Uart0 => RESET_DONE::uart0::SET,
            Self::Uart1 => RESET_DONE::uart1::SET,
            Self::UsbCtrl => RESET_DONE::usbctrl::SET,
        }
    }
}

pub struct Resets {
    registers: StaticRef<ResetsRegisters>,
}

impl Resets {
    pub const fn new() -> Self {
        Self {
            registers: RESETS_BASE,
        }
    }

    pub fn reset(&self, peripherals: &'static [Peripheral]) {
        if peripherals.len() > 0 {
            let mut value: FieldValue<u32, RESET::Register> = peripherals[0].get_reset_field_set();
            for peripheral in peripherals {
                value += peripheral.get_reset_field_set();
            }
            self.registers.reset.modify(value);
        }
    }

    pub fn unreset(&self, peripherals: &'static [Peripheral], wait_for: bool) {
        if peripherals.len() > 0 {
            let mut value: FieldValue<u32, RESET::Register> =
                peripherals[0].get_reset_field_clear();
            for peripheral in peripherals {
                value += peripheral.get_reset_field_clear();
            }
            self.registers.reset.modify(value);

            if wait_for {
                let mut value_done: FieldValue<u32, RESET_DONE::Register> =
                    peripherals[0].get_reset_done_field_set();
                for peripheral in peripherals {
                    value_done += peripheral.get_reset_done_field_set();
                }
                while !self.registers.reset_done.matches_all(value_done) {}
            }
        }
    }

    pub fn reset_all_except(&self, peripherals: &'static [Peripheral]) {
        let mut value = 0xFFFFFF;
        for peripheral in peripherals {
            value ^= peripheral.get_reset_field_set().value;
        }
        self.registers.reset.set(value);
    }

    pub fn unreset_all_except(&self, peripherals: &'static [Peripheral], wait_for: bool) {
        let mut value = 0;
        for peripheral in peripherals {
            value |= peripheral.get_reset_field_set().value;
        }

        self.registers.reset.set(value);

        if wait_for {
            value = !value & 0xFFFFF;
            while (self.registers.reset_done.get() & value) != value {}
        }
    }

    pub fn watchdog_reset_all_except(&self, peripherals: &'static [Peripheral]) {
        let mut value = 0xFFFFFF;
        for peripheral in peripherals {
            value ^= peripheral.get_reset_field_set().value;
        }
        self.registers.wdsel.set(value);
    }
}
