//! Implementation of the reset and clock control (RCC) peripheral.

use core::cell::Cell;
use core::mem;
use flash;
use kernel::common::VolatileCell;

#[derive(Copy,Clone,Debug)]
pub enum Clock {
    AHB(AHBClock),
    APB1(APB1Clock),
    APB2(APB2Clock),
}

#[derive(Copy,Clone,Debug)]
pub enum AHBClock {
    DMA1,
    DMA2,
    SRAM,
    FLITF = 4,
    CRC = 6,
    FSMC = 8,
    SDIO = 10,
}

#[derive(Copy,Clone,Debug)]
pub enum APB1Clock {
    TIM2,
    TIM3,
    TIM4,
    TIM5,
    TIM6,
    TIM7,
    TIM12,
    TIM13,
    TIM14,
    WWDG = 11,
    SPI2 = 13,
    SPI3,
    USART2 = 17,
    USART3,
    UART4,
    UART5,
    I2C1,
    I2C2,
    USB,
    CAN,
    BKP,
    PWR,
    DAC,
}

#[derive(Copy,Clone,Debug)]
pub enum APB2Clock {
    AFIO,
    IOPA = 2,
    IOPB,
    IOPC,
    IOPD,
    IOPE,
    IOPF,
    IOPG,
    ADC1,
    ADC2,
    TIM1,
    SPI1,
    TIM8,
    USART1,
    ADC3,
    TIM9 = 19,
    TIM10,
    TIM11,
}

#[repr(C, packed)]
struct Registers {
    cr: VolatileCell<u32>,
    cfgr: VolatileCell<u32>,
    cir: VolatileCell<u32>,
    apb2rstr: VolatileCell<u32>,
    apb1rstr: VolatileCell<u32>,
    ahbenr: VolatileCell<u32>,
    apb2enr: VolatileCell<u32>,
    apb1enr: VolatileCell<u32>,
    bdcr: VolatileCell<u32>,
    csr: VolatileCell<u32>,
}

#[derive(Copy,Clone,Debug)]
pub enum OscillatorFrequency {
    Frequency8MHz,
}

#[derive(Copy,Clone,Debug)]
pub enum SystemClockSource {
    InternalOscillator,
    PllInternalOscillatorAt64MHz,
    PllExternalOscillatorAt72MHz { frequency: OscillatorFrequency },
}

const BASE_ADDRESS: usize = 0x40021000;

pub struct ResetClockControl {
    registers: *mut Registers,
    system_frequency: Cell<u32>,
    system_clock_source: Cell<SystemClockSource>,
}

pub static mut RCC: ResetClockControl = ResetClockControl {
    registers: BASE_ADDRESS as *mut Registers,
    system_frequency: Cell::new(8000000),
    system_clock_source: Cell::new(SystemClockSource::InternalOscillator),
};

impl ResetClockControl {
    /// Sets up the system clock. This should be called as one of the first
    /// lines in the `reset_handler` within the platform's `main.rs`.
    pub unsafe fn setup_system_clock(&self, clock_source: SystemClockSource) {
        self.system_clock_source.set(clock_source);

        match clock_source {
            SystemClockSource::InternalOscillator => {
                // no configurations necessary
                self.system_frequency.set(8000000);
            }

            SystemClockSource::PllInternalOscillatorAt64MHz => {
                configure_internal_oscillator_pll();
                self.system_frequency.set(64000000);
            }

            SystemClockSource::PllExternalOscillatorAt72MHz { frequency } => {
                configure_external_oscillator_pll(frequency);
                self.system_frequency.set(72000000);
            }
        }
    }
}

unsafe fn configure_pll(multiplier: u32, hse: bool) {
    let regs: &mut Registers = mem::transmute(RCC.registers);

    let mut cfgr = regs.cfgr.get() & !(0xf << 18);
    cfgr |= (multiplier - 2) << 18;
    if hse {
        cfgr |= 1 << 16;
    } else {
        cfgr &= !(1 << 16);
    }
    regs.cfgr.set(cfgr);

    regs.cr.set(regs.cr.get() | (1 << 24)); // PLLON
    while regs.cr.get() & (1 << 25) == 0 {} // wait for PLLRDY

    regs.cfgr.set((regs.cfgr.get() & !0b11) | 0b10); // PLL as system clock
    while regs.cfgr.get() & (0b11 << 2) != (0b10 << 2) {} // wait for switch to PLL
}

unsafe fn configure_internal_oscillator_pll() {
    let regs: &mut Registers = mem::transmute(RCC.registers);

    regs.cfgr.set((0b000 << 11) | (0b100 << 8) | (0b0000 << 4) | 0b00); // APB2 | APB1 | AHB | HSI
    while regs.cfgr.get() & (0b11 << 2) != (0b00 << 2) {} // wait for switch to HSI

    flash::FLASH.set_latency(flash::Latency::TwoWaitStates);

    configure_pll(16, false);
}

unsafe fn configure_external_oscillator_pll(frequency: OscillatorFrequency) {
    let regs: &mut Registers = mem::transmute(RCC.registers);

    regs.cfgr.set((0b000 << 11) | (0b100 << 8) | (0b0000 << 4) | 0b00); // APB2 | APB1 | AHB | HSI
    while regs.cfgr.get() & (0b11 << 2) != (0b00 << 2) {} // wait for switch to HSI

    flash::FLASH.set_latency(flash::Latency::TwoWaitStates);

    regs.cr.set(regs.cr.get() | (1 << 16)); // HSEON
    while regs.cr.get() & (1 << 17) == 0 {} // wait for HSERDY

    configure_pll(9, true);

    regs.cr.set(regs.cr.get() & !(1 << 0)); // disable HSI
}

fn get_ahb_prescaler() -> u32 {
    let regs: &mut Registers = unsafe { mem::transmute(RCC.registers) };
    let bits = (regs.cfgr.get() >> 4) & 0xf;
    if bits & (1 << 3) != 0 {
        1 << (1 + (bits & 0b111))
    } else {
        1
    }
}

fn get_apb1_prescaler() -> u32 {
    let regs: &mut Registers = unsafe { mem::transmute(RCC.registers) };
    let bits = (regs.cfgr.get() >> 8) & 0b111;
    if bits & (1 << 2) != 0 {
        1 << (1 + (bits & 0b11))
    } else {
        1
    }
}

fn get_apb2_prescaler() -> u32 {
    let regs: &mut Registers = unsafe { mem::transmute(RCC.registers) };
    let bits = (regs.cfgr.get() >> 11) & 0b111;
    if bits & (1 << 2) != 0 {
        1 << (1 + (bits & 0b11))
    } else {
        1
    }
}

fn get_prescaler(clock: Clock) -> u32 {
    match clock {
        Clock::AHB(_) => get_ahb_prescaler(),
        Clock::APB1(a) => {
            match a {
                APB1Clock::TIM2 | APB1Clock::TIM3 | APB1Clock::TIM4 | APB1Clock::TIM5 |
                APB1Clock::TIM6 | APB1Clock::TIM7 if get_apb1_prescaler() > 1 => {
                    get_ahb_prescaler() * get_apb1_prescaler() / 2
                }
                _ => get_ahb_prescaler() * get_apb1_prescaler(),
            }
        }
        Clock::APB2(a) => {
            match a {
                APB2Clock::TIM1 | APB2Clock::TIM8 if get_apb2_prescaler() > 1 => {
                    get_ahb_prescaler() * get_apb2_prescaler() / 2
                }
                _ => get_ahb_prescaler() * get_apb2_prescaler(),
            }
        }
    }
}

pub fn get_system_frequency() -> u32 {
    unsafe { RCC.system_frequency.get() }
}

pub fn get_frequency(clock: Clock) -> u32 {
    get_system_frequency() / get_prescaler(clock)
}

pub unsafe fn enable_clock(clock: Clock) {
    let regs: &mut Registers = unsafe { mem::transmute(RCC.registers) };
    match clock {
        Clock::AHB(c) => regs.ahbenr.set(regs.ahbenr.get() | 1 << (c as u32)),
        Clock::APB1(c) => regs.apb1enr.set(regs.apb1enr.get() | 1 << (c as u32)),
        Clock::APB2(c) => regs.apb2enr.set(regs.apb2enr.get() | 1 << (c as u32)),
    }
}
