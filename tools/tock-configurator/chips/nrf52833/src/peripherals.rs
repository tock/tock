// Copyright OxidOS Automotive 2024.

use crate::gpio::Gpio;
use crate::{timer, uart, FlashType, TemperatureType, Twi, UartType};
use parse::constants::PERIPHERALS;
use quote::{format_ident, quote};
use std::rc::Rc;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Peripherals {
    // Default peripherals for the Microbit.
    uart: [Rc<crate::uart::Uart>; 1],
    timer: [Rc<crate::timer::Timer>; 1],
    ble: [Rc<crate::ble::Ble>; 1],
    rng: [Rc<crate::rng::Rng>; 1],
    temperature: [Rc<crate::temperature::Temperature>; 1],
    twi: [Rc<crate::twi::Twi>; 1],
    gpio: [Rc<crate::gpio::Gpio>; 1],
    flash: [Rc<crate::flash::Flash>; 1],
}

impl Peripherals {
    pub fn new() -> Self {
        Self {
            uart: [Rc::new(uart::Uart::new(UartType::Uart0))],
            timer: [Rc::new(timer::Timer::new(crate::TimerType::Rtc))],
            ble: [Rc::new(crate::ble::Ble::new(crate::BleType::RadioBle))],
            rng: [Rc::new(crate::rng::Rng::new(crate::RngType::Rng))],
            temperature: [Rc::new(crate::temperature::Temperature::new(
                TemperatureType::Temp,
            ))],
            twi: [Rc::new(crate::Twi::new())],
            flash: [Rc::new(crate::Flash::new(FlashType::Flash0))],
            gpio: [Rc::new(crate::gpio::Gpio::new())],
        }
    }
}

impl Default for Peripherals {
    fn default() -> Self {
        Self::new()
    }
}

impl parse::Component for Peripherals {
    fn init_expr(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote! {
             kernel::static_init!(
                 nrf52833::interrupt_service::Nrf52833DefaultPeripherals,
                 nrf52833::interrupt_service::Nrf52833DefaultPeripherals::new()
             )
        })
    }

    fn before_init(&self) -> Option<parse::proc_macro2::TokenStream> {
        Some(quote! (nrf52833::init();))
    }

    fn after_init(&self) -> Option<parse::proc_macro2::TokenStream> {
        let ident = format_ident!("{}", PERIPHERALS.clone());
        Some(quote! {
            #ident.init();
        })
    }
}

impl parse::DefaultPeripherals for Peripherals {
    type Gpio = Gpio;
    type Uart = crate::Uart;
    type Timer = crate::Timer;
    type I2c = Twi;
    type Spi = parse::NoSupport;
    type Rng = crate::Rng;
    type BleAdvertisement = crate::Ble;
    type Temperature = crate::Temperature;
    type Flash = crate::Flash;

    fn uart(&self) -> Result<&[Rc<Self::Uart>], parse::Error> {
        Ok(&self.uart)
    }

    fn timer(&self) -> Result<&[Rc<Self::Timer>], parse::Error> {
        Ok(&self.timer)
    }

    fn i2c(&self) -> Result<&[Rc<Self::I2c>], parse::Error> {
        Ok(&self.twi)
    }

    fn ble(&self) -> Result<&[Rc<Self::BleAdvertisement>], parse::Error> {
        Ok(&self.ble)
    }

    fn flash(&self) -> Result<&[Rc<Self::Flash>], parse::Error> {
        Ok(&self.flash)
    }

    fn temp(&self) -> Result<&[Rc<Self::Temperature>], parse::Error> {
        Ok(&self.temperature)
    }

    fn rng(&self) -> Result<&[Rc<Self::Rng>], parse::Error> {
        Ok(&self.rng)
    }

    fn gpio(&self) -> Result<&[Rc<Self::Gpio>], parse::Error> {
        Ok(&self.gpio)
    }
}
