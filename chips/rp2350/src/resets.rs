// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, FieldValue, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {

    ResetsRegisters {

        (0x000 => reset: ReadWrite<u32, RESET::Register>),

        (0x004 => wdsel: ReadWrite<u32, WDSEL::Register>),

        (0x008 => reset_done: ReadWrite<u32, RESET_DONE::Register>),
        (0x00C => @END),
    }
}
register_bitfields![u32,
RESET [

    USBCTRL OFFSET(28) NUMBITS(1) [],

    UART1 OFFSET(27) NUMBITS(1) [],

    UART0 OFFSET(26) NUMBITS(1) [],

    TRNG OFFSET(25) NUMBITS(1) [],

    TIMER1 OFFSET(24) NUMBITS(1) [],

    TIMER0 OFFSET(23) NUMBITS(1) [],

    TBMAN OFFSET(22) NUMBITS(1) [],

    SYSINFO OFFSET(21) NUMBITS(1) [],

    SYSCFG OFFSET(20) NUMBITS(1) [],

    SPI1 OFFSET(19) NUMBITS(1) [],

    SPI0 OFFSET(18) NUMBITS(1) [],

    SHA256 OFFSET(17) NUMBITS(1) [],

    PWM OFFSET(16) NUMBITS(1) [],

    PLL_USB OFFSET(15) NUMBITS(1) [],

    PLL_SYS OFFSET(14) NUMBITS(1) [],

    PIO2 OFFSET(13) NUMBITS(1) [],

    PIO1 OFFSET(12) NUMBITS(1) [],

    PIO0 OFFSET(11) NUMBITS(1) [],

    PADS_QSPI OFFSET(10) NUMBITS(1) [],

    PADS_BANK0 OFFSET(9) NUMBITS(1) [],

    JTAG OFFSET(8) NUMBITS(1) [],

    IO_QSPI OFFSET(7) NUMBITS(1) [],

    IO_BANK0 OFFSET(6) NUMBITS(1) [],

    I2C1 OFFSET(5) NUMBITS(1) [],

    I2C0 OFFSET(4) NUMBITS(1) [],

    HSTX OFFSET(3) NUMBITS(1) [],

    DMA OFFSET(2) NUMBITS(1) [],

    BUSCTRL OFFSET(1) NUMBITS(1) [],

    ADC OFFSET(0) NUMBITS(1) []
],
WDSEL [

    USBCTRL OFFSET(28) NUMBITS(1) [],

    UART1 OFFSET(27) NUMBITS(1) [],

    UART0 OFFSET(26) NUMBITS(1) [],

    TRNG OFFSET(25) NUMBITS(1) [],

    TIMER1 OFFSET(24) NUMBITS(1) [],

    TIMER0 OFFSET(23) NUMBITS(1) [],

    TBMAN OFFSET(22) NUMBITS(1) [],

    SYSINFO OFFSET(21) NUMBITS(1) [],

    SYSCFG OFFSET(20) NUMBITS(1) [],

    SPI1 OFFSET(19) NUMBITS(1) [],

    SPI0 OFFSET(18) NUMBITS(1) [],

    SHA256 OFFSET(17) NUMBITS(1) [],

    PWM OFFSET(16) NUMBITS(1) [],

    PLL_USB OFFSET(15) NUMBITS(1) [],

    PLL_SYS OFFSET(14) NUMBITS(1) [],

    PIO2 OFFSET(13) NUMBITS(1) [],

    PIO1 OFFSET(12) NUMBITS(1) [],

    PIO0 OFFSET(11) NUMBITS(1) [],

    PADS_QSPI OFFSET(10) NUMBITS(1) [],

    PADS_BANK0 OFFSET(9) NUMBITS(1) [],

    JTAG OFFSET(8) NUMBITS(1) [],

    IO_QSPI OFFSET(7) NUMBITS(1) [],

    IO_BANK0 OFFSET(6) NUMBITS(1) [],

    I2C1 OFFSET(5) NUMBITS(1) [],

    I2C0 OFFSET(4) NUMBITS(1) [],

    HSTX OFFSET(3) NUMBITS(1) [],

    DMA OFFSET(2) NUMBITS(1) [],

    BUSCTRL OFFSET(1) NUMBITS(1) [],

    ADC OFFSET(0) NUMBITS(1) []
],
RESET_DONE [

    USBCTRL OFFSET(28) NUMBITS(1) [],

    UART1 OFFSET(27) NUMBITS(1) [],

    UART0 OFFSET(26) NUMBITS(1) [],

    TRNG OFFSET(25) NUMBITS(1) [],

    TIMER1 OFFSET(24) NUMBITS(1) [],

    TIMER0 OFFSET(23) NUMBITS(1) [],

    TBMAN OFFSET(22) NUMBITS(1) [],

    SYSINFO OFFSET(21) NUMBITS(1) [],

    SYSCFG OFFSET(20) NUMBITS(1) [],

    SPI1 OFFSET(19) NUMBITS(1) [],

    SPI0 OFFSET(18) NUMBITS(1) [],

    SHA256 OFFSET(17) NUMBITS(1) [],

    PWM OFFSET(16) NUMBITS(1) [],

    PLL_USB OFFSET(15) NUMBITS(1) [],

    PLL_SYS OFFSET(14) NUMBITS(1) [],

    PIO2 OFFSET(13) NUMBITS(1) [],

    PIO1 OFFSET(12) NUMBITS(1) [],

    PIO0 OFFSET(11) NUMBITS(1) [],

    PADS_QSPI OFFSET(10) NUMBITS(1) [],

    PADS_BANK0 OFFSET(9) NUMBITS(1) [],

    JTAG OFFSET(8) NUMBITS(1) [],

    IO_QSPI OFFSET(7) NUMBITS(1) [],

    IO_BANK0 OFFSET(6) NUMBITS(1) [],

    I2C1 OFFSET(5) NUMBITS(1) [],

    I2C0 OFFSET(4) NUMBITS(1) [],

    HSTX OFFSET(3) NUMBITS(1) [],

    DMA OFFSET(2) NUMBITS(1) [],

    BUSCTRL OFFSET(1) NUMBITS(1) [],

    ADC OFFSET(0) NUMBITS(1) []
]
];
const RESETS_BASE: StaticRef<ResetsRegisters> =
    unsafe { StaticRef::new(0x40020000 as *const ResetsRegisters) };

pub enum Peripheral {
    Adc,
    BusController,
    Dma,
    Hstx,
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
    Spi0,
    Spi1,
    Syscfg,
    SysInfo,
    TBMan,
    Timer0,
    Timer1,
    Uart0,
    Uart1,
    UsbCtrl,
}

impl Peripheral {
    fn get_reset_field_set(&self) -> FieldValue<u32, RESET::Register> {
        match self {
            Peripheral::Adc => RESET::ADC::SET,
            Peripheral::BusController => RESET::BUSCTRL::SET,
            Peripheral::Dma => RESET::DMA::SET,
            Peripheral::Hstx => RESET::HSTX::SET,
            Peripheral::I2c0 => RESET::I2C0::SET,
            Peripheral::I2c1 => RESET::I2C1::SET,
            Peripheral::IOBank0 => RESET::IO_BANK0::SET,
            Peripheral::IOQSpi => RESET::IO_QSPI::SET,
            Peripheral::Jtag => RESET::JTAG::SET,
            Peripheral::PadsBank0 => RESET::PADS_BANK0::SET,
            Peripheral::PadsQSpi => RESET::PADS_QSPI::SET,
            Peripheral::Pio0 => RESET::PIO0::SET,
            Peripheral::Pio1 => RESET::PIO1::SET,
            Peripheral::PllSys => RESET::PLL_SYS::SET,
            Peripheral::PllUsb => RESET::PLL_USB::SET,
            Peripheral::Pwm => RESET::PWM::SET,
            Peripheral::Spi0 => RESET::SPI0::SET,
            Peripheral::Spi1 => RESET::SPI1::SET,
            Peripheral::Syscfg => RESET::SYSCFG::SET,
            Peripheral::SysInfo => RESET::SYSINFO::SET,
            Peripheral::TBMan => RESET::TBMAN::SET,
            Peripheral::Timer0 => RESET::TIMER0::SET,
            Peripheral::Timer1 => RESET::TIMER1::SET,
            Peripheral::Uart0 => RESET::UART0::SET,
            Peripheral::Uart1 => RESET::UART1::SET,
            Peripheral::UsbCtrl => RESET::USBCTRL::SET,
        }
    }

    fn get_reset_field_clear(&self) -> FieldValue<u32, RESET::Register> {
        match self {
            Peripheral::Adc => RESET::ADC::CLEAR,
            Peripheral::BusController => RESET::BUSCTRL::CLEAR,
            Peripheral::Dma => RESET::DMA::CLEAR,
            Peripheral::Hstx => RESET::HSTX::CLEAR,
            Peripheral::I2c0 => RESET::I2C0::CLEAR,
            Peripheral::I2c1 => RESET::I2C1::CLEAR,
            Peripheral::IOBank0 => RESET::IO_BANK0::CLEAR,
            Peripheral::IOQSpi => RESET::IO_QSPI::CLEAR,
            Peripheral::Jtag => RESET::JTAG::CLEAR,
            Peripheral::PadsBank0 => RESET::PADS_BANK0::CLEAR,
            Peripheral::PadsQSpi => RESET::PADS_QSPI::CLEAR,
            Peripheral::Pio0 => RESET::PIO0::CLEAR,
            Peripheral::Pio1 => RESET::PIO1::CLEAR,
            Peripheral::PllSys => RESET::PLL_SYS::CLEAR,
            Peripheral::PllUsb => RESET::PLL_USB::CLEAR,
            Peripheral::Pwm => RESET::PWM::CLEAR,
            Peripheral::Spi0 => RESET::SPI0::CLEAR,
            Peripheral::Spi1 => RESET::SPI1::CLEAR,
            Peripheral::Syscfg => RESET::SYSCFG::CLEAR,
            Peripheral::SysInfo => RESET::SYSINFO::CLEAR,
            Peripheral::TBMan => RESET::TBMAN::CLEAR,
            Peripheral::Timer0 => RESET::TIMER0::CLEAR,
            Peripheral::Timer1 => RESET::TIMER1::CLEAR,
            Peripheral::Uart0 => RESET::UART0::CLEAR,
            Peripheral::Uart1 => RESET::UART1::CLEAR,
            Peripheral::UsbCtrl => RESET::USBCTRL::CLEAR,
        }
    }

    fn get_reset_done_field_set(&self) -> FieldValue<u32, RESET_DONE::Register> {
        match self {
            Peripheral::Adc => RESET_DONE::ADC::SET,
            Peripheral::BusController => RESET_DONE::BUSCTRL::SET,
            Peripheral::Dma => RESET_DONE::DMA::SET,
            Peripheral::Hstx => RESET_DONE::HSTX::SET,
            Peripheral::I2c0 => RESET_DONE::I2C0::SET,
            Peripheral::I2c1 => RESET_DONE::I2C1::SET,
            Peripheral::IOBank0 => RESET_DONE::IO_BANK0::SET,
            Peripheral::IOQSpi => RESET_DONE::IO_QSPI::SET,
            Peripheral::Jtag => RESET_DONE::JTAG::SET,
            Peripheral::PadsBank0 => RESET_DONE::PADS_BANK0::SET,
            Peripheral::PadsQSpi => RESET_DONE::PADS_QSPI::SET,
            Peripheral::Pio0 => RESET_DONE::PIO0::SET,
            Peripheral::Pio1 => RESET_DONE::PIO1::SET,
            Peripheral::PllSys => RESET_DONE::PLL_SYS::SET,
            Peripheral::PllUsb => RESET_DONE::PLL_USB::SET,
            Peripheral::Pwm => RESET_DONE::PWM::SET,
            Peripheral::Spi0 => RESET_DONE::SPI0::SET,
            Peripheral::Spi1 => RESET_DONE::SPI1::SET,
            Peripheral::Syscfg => RESET_DONE::SYSCFG::SET,
            Peripheral::SysInfo => RESET_DONE::SYSINFO::SET,
            Peripheral::TBMan => RESET_DONE::TBMAN::SET,
            Peripheral::Timer0 => RESET_DONE::TIMER0::SET,
            Peripheral::Timer1 => RESET_DONE::TIMER1::SET,
            Peripheral::Uart0 => RESET_DONE::UART0::SET,
            Peripheral::Uart1 => RESET_DONE::UART1::SET,
            Peripheral::UsbCtrl => RESET_DONE::USBCTRL::SET,
        }
    }
}

pub struct Resets {
    registers: StaticRef<ResetsRegisters>,
}

impl Resets {
    pub const fn new() -> Resets {
        Resets {
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
        let mut value = 0x1FFFFFFF;
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
            value = !value & 0x1FFFFFFF;
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
