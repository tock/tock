//! Implementation of the MK20 System Integration Module

use core::mem;
use regs::sim::*;
use common::regs::FieldValue;

pub type Clock1 = FieldValue<u32, SystemClockGatingControl1>;
pub type Clock2 = FieldValue<u32, SystemClockGatingControl2>;
pub type Clock3 = FieldValue<u32, SystemClockGatingControl3>;
pub type Clock4 = FieldValue<u32, SystemClockGatingControl4>;
pub type Clock5 = FieldValue<u32, SystemClockGatingControl5>;
pub type Clock6 = FieldValue<u32, SystemClockGatingControl6>;
pub type Clock7 = FieldValue<u32, SystemClockGatingControl7>;

pub trait Clock {
    fn enable(self);
}

impl Clock for Clock1 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc1.modify(self);
    }
}

impl Clock for Clock2 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc2.modify(self);
    }
}

impl Clock for Clock3 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc3.modify(self);
    }
}

impl Clock for Clock4 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc4.modify(self);
    }
}

impl Clock for Clock5 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc5.modify(self);
    }
}

impl Clock for Clock6 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc6.modify(self);
    }
}

impl Clock for Clock7 {
    fn enable(self) {
        let regs: &mut Registers = unsafe { mem::transmute(SIM) };
        regs.scgc7.modify(self);
    }
}

pub mod clocks {
    use sim::{Clock1, Clock2, Clock3, Clock4, Clock5, Clock6, Clock7};
    use regs::sim::*;

    pub const UART4: Clock1 = SCGC1::UART4::SET;
    pub const I2C2: Clock1 = SCGC1::I2C2::SET;
    pub const I2C3: Clock1 = SCGC1::I2C3::SET;

    pub const DAC1: Clock2 = SCGC2::DAC1::SET;
    pub const DAC0: Clock2 = SCGC2::DAC0::SET;
    pub const TPM2: Clock2 = SCGC2::TPM2::SET;
    pub const TPM1: Clock2 = SCGC2::TPM1::SET;
    pub const LPUART0: Clock2 = SCGC2::LPUART0::SET;
    pub const ENET: Clock2 = SCGC2::ENET::SET;

    pub const ADC1: Clock3 = SCGC3::ADC1::SET;
    pub const FTM3: Clock3 = SCGC3::FTM3::SET;
    pub const FTM2: Clock3 = SCGC3::FTM2::SET;
    pub const SDHC: Clock3 = SCGC3::SDHC::SET;
    pub const SPI2: Clock3 = SCGC3::SPI2::SET;
    pub const FLEXCAN1: Clock3 = SCGC3::FLEXCAN1::SET;
    pub const USBHSDCD: Clock3 = SCGC3::USBHSDCD::SET;
    pub const USBHSPHY: Clock3 = SCGC3::USBHSPHY::SET;
    pub const USBHS: Clock3 = SCGC3::USBHS::SET;
    pub const RNGA: Clock3 = SCGC3::RNGA::SET;

    pub const VREF: Clock4 = SCGC4::VREF::SET;
    pub const CMP: Clock4 = SCGC4::CMP::SET;
    pub const USBOTG: Clock4 = SCGC4::USBOTG::SET;
    pub const UART3: Clock4 = SCGC4::UART3::SET;
    pub const UART2: Clock4 = SCGC4::UART2::SET;
    pub const UART1: Clock4 = SCGC4::UART1::SET;
    pub const UART0: Clock4 = SCGC4::UART0::SET;
    pub const I2C1: Clock4 = SCGC4::I2C1::SET;
    pub const I2C0: Clock4 = SCGC4::I2C0::SET;
    pub const CMT: Clock4 = SCGC4::CMT::SET;
    pub const EWM: Clock4 = SCGC4::EWM::SET;

    pub const PORTE: Clock5 = SCGC5::PORT::E;
    pub const PORTD: Clock5 = SCGC5::PORT::D;
    pub const PORTC: Clock5 = SCGC5::PORT::C;
    pub const PORTB: Clock5 = SCGC5::PORT::B;
    pub const PORTA: Clock5 = SCGC5::PORT::A;
    pub const PORTABCDE: Clock5 = SCGC5::PORT::All;
    pub const TSI: Clock5 = SCGC5::TSI::SET;
    pub const LPTMR: Clock5 = SCGC5::LPTMR::SET;

    // DAC0,
    pub const RTC: Clock6 = SCGC6::RTC::SET;
    pub const ADC0: Clock6 = SCGC6::ADC0::SET;
    // FTM2,
    pub const FTM1: Clock6 = SCGC6::FTM1::SET;
    pub const FTM0: Clock6 = SCGC6::FTM0::SET;
    pub const PIT: Clock6 = SCGC6::PIT::SET;
    pub const PDB: Clock6 = SCGC6::PDB::SET;
    pub const USBDCD: Clock6 = SCGC6::USBDCD::SET;
    pub const CRC: Clock6 = SCGC6::CRC::SET;
    pub const I2S: Clock6 = SCGC6::I2S::SET;
    pub const SPI1: Clock6 = SCGC6::SPI1::SET;
    pub const SPI0: Clock6 = SCGC6::SPI0::SET;

    // RNGA,
    pub const FLEXCAN0: Clock6 = SCGC6::FLEXCAN0::SET;
    pub const DMAMUX: Clock6 = SCGC6::DMAMUX::SET;
    pub const FTF: Clock6 = SCGC6::FTF::SET;

    pub const SDRAMC: Clock7 = SCGC7::SDRAMC::SET;
    pub const MPU: Clock7 = SCGC7::MPU::SET;
    pub const DMA: Clock7 = SCGC7::DMA::SET;
    pub const FLEXBUS: Clock7 = SCGC7::FLEXBUS::SET;
}

pub fn set_dividers(core: u32, bus: u32, flash: u32) {
    let regs: &mut Registers = unsafe { mem::transmute(SIM) };

    regs.clkdiv1.modify(CLKDIV1::Core.val(core - 1) + 
                        CLKDIV1::Bus.val(bus - 1) + 
                        CLKDIV1::FlexBus.val(bus - 1) +
                        CLKDIV1::Flash.val(flash - 1));
}
