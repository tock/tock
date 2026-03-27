use crate::clocks::Stm32u5Clocks;
use crate::rcc::GpioPort;
use kernel::platform::chip::ClockInterface;

pub struct PeripheralClock<'a> {
    pub clock: PeripheralClockType,
    clocks: &'a dyn Stm32u5Clocks,
}

#[derive(Copy, Clone)]
pub enum PeripheralClockType {
    APB1(PCLK1),
    APB2(PCLK2),
    AHB1(HCLK1),
}

#[derive(Copy, Clone)]
pub enum PCLK1 {
    TIM2,
}

#[derive(Copy, Clone)]
pub enum PCLK2 {
    USART1,
}

#[derive(Copy, Clone)]
pub enum HCLK1 {
    GPIOA,
    GPIOB,
    GPIOC,
    GPIOH,
}

impl<'a> PeripheralClock<'a> {
    pub const fn new(clock: PeripheralClockType, clocks: &'a dyn Stm32u5Clocks) -> Self {
        Self { clock, clocks }
    }

    pub fn configure_rng_clock(&self) {
        self.clocks.get_rcc().configure_rng_clock();
    }

    pub fn get_frequency(&self) -> u32 {
        match &self.clock {
            PeripheralClockType::AHB1(_) => self.clocks.get_ahb_frequency() as u32,
            PeripheralClockType::APB1(_) => self.clocks.get_apb1_frequency() as u32,
            PeripheralClockType::APB2(_) => self.clocks.get_apb2_frequency() as u32,
        }
    }
}

impl ClockInterface for PeripheralClock<'_> {
    fn is_enabled(&self) -> bool {
        let rcc = self.clocks.get_rcc();
        match &self.clock {
            PeripheralClockType::AHB1(port) => match port {
                HCLK1::GPIOA => rcc.is_enabled_gpio_port(GpioPort::A),
                HCLK1::GPIOB => rcc.is_enabled_gpio_port(GpioPort::B),
                HCLK1::GPIOC => rcc.is_enabled_gpio_port(GpioPort::C),
                HCLK1::GPIOH => rcc.is_enabled_gpio_port(GpioPort::H),
            },
            PeripheralClockType::APB2(periph) => match periph {
                PCLK2::USART1 => rcc.is_enabled_usart1_clock(),
            },
            PeripheralClockType::APB1(periph) => match periph {
                PCLK1::TIM2 => rcc.is_enabled_tim2_clock(),
            },
        }
    }

    fn disable(&self) {
        let rcc = self.clocks.get_rcc();
        match &self.clock {
            PeripheralClockType::AHB1(port) => match port {
                HCLK1::GPIOA => rcc.disable_gpio_port(GpioPort::A),
                HCLK1::GPIOB => rcc.disable_gpio_port(GpioPort::B),
                HCLK1::GPIOC => rcc.disable_gpio_port(GpioPort::C),
                HCLK1::GPIOH => rcc.disable_gpio_port(GpioPort::H),
            },
            PeripheralClockType::APB2(periph) => match periph {
                PCLK2::USART1 => rcc.disable_usart1_clock(),
            },
            PeripheralClockType::APB1(periph) => match periph {
                PCLK1::TIM2 => rcc.disable_tim2_clock(),
            },
        }
    }

    fn enable(&self) {
        let rcc = self.clocks.get_rcc();
        match &self.clock {
            PeripheralClockType::AHB1(port) => match port {
                HCLK1::GPIOA => rcc.enable_gpio_port(GpioPort::A),
                HCLK1::GPIOB => rcc.enable_gpio_port(GpioPort::B),
                HCLK1::GPIOC => rcc.enable_gpio_port(GpioPort::C),
                HCLK1::GPIOH => rcc.enable_gpio_port(GpioPort::H),
            },
            PeripheralClockType::APB2(periph) => match periph {
                PCLK2::USART1 => rcc.enable_usart1_clock(),
            },
            PeripheralClockType::APB1(periph) => match periph {
                PCLK1::TIM2 => rcc.enable_tim2_clock(),
            },
        }
    }
}
