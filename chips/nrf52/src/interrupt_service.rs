use crate::adc;
use crate::ble_radio;
use crate::i2c;
use crate::ieee802154_radio;
use crate::power;
use crate::spi;
use crate::uart;
use kernel::debug;
use nrf5x::peripheral_interrupts;

pub trait InterruptService {
    /// Service an interrupt, if supported by this chip. If this interrupt number is not supported,
    /// return false.
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool;
}

pub struct Nrf52InterruptService {
    gpio_port: &'static nrf5x::gpio::Port,
}

impl Nrf52InterruptService {
    pub unsafe fn new(gpio_port: &'static nrf5x::gpio::Port) -> Nrf52InterruptService {
        Nrf52InterruptService { gpio_port }
    }
}

impl InterruptService for Nrf52InterruptService {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            peripheral_interrupts::ECB => nrf5x::aes::AESECB.handle_interrupt(),
            peripheral_interrupts::GPIOTE => self.gpio_port.handle_interrupt(),
            peripheral_interrupts::POWER_CLOCK => power::POWER.handle_interrupt(),
            peripheral_interrupts::RADIO => {
                match (
                    ieee802154_radio::RADIO.is_enabled(),
                    ble_radio::RADIO.is_enabled(),
                ) {
                    (false, false) => (),
                    (true, false) => ieee802154_radio::RADIO.handle_interrupt(),
                    (false, true) => ble_radio::RADIO.handle_interrupt(),
                    (true, true) => {
                        debug!("nRF 802.15.4 and BLE radios cannot be simultaneously enabled!")
                    }
                }
            }
            peripheral_interrupts::RNG => nrf5x::trng::TRNG.handle_interrupt(),
            peripheral_interrupts::RTC1 => nrf5x::rtc::RTC.handle_interrupt(),
            peripheral_interrupts::TEMP => nrf5x::temperature::TEMP.handle_interrupt(),
            peripheral_interrupts::TIMER0 => nrf5x::timer::TIMER0.handle_interrupt(),
            peripheral_interrupts::TIMER1 => nrf5x::timer::ALARM1.handle_interrupt(),
            peripheral_interrupts::TIMER2 => nrf5x::timer::TIMER2.handle_interrupt(),
            peripheral_interrupts::UART0 => uart::UARTE0.handle_interrupt(),
            peripheral_interrupts::SPI0_TWI0 => {
                // SPI0 and TWI0 share interrupts.
                // Dispatch the correct handler.
                match (spi::SPIM0.is_enabled(), i2c::TWIM0.is_enabled()) {
                    (false, false) => (),
                    (true, false) => spi::SPIM0.handle_interrupt(),
                    (false, true) => i2c::TWIM0.handle_interrupt(),
                    (true, true) => debug_assert!(
                        false,
                        "SPIM0 and TWIM0 cannot be \
                         enabled at the same time."
                    ),
                }
            }
            peripheral_interrupts::SPI1_TWI1 => {
                // SPI1 and TWI1 share interrupts.
                // Dispatch the correct handler.
                match (spi::SPIM1.is_enabled(), i2c::TWIM1.is_enabled()) {
                    (false, false) => (),
                    (true, false) => spi::SPIM1.handle_interrupt(),
                    (false, true) => i2c::TWIM1.handle_interrupt(),
                    (true, true) => debug_assert!(
                        false,
                        "SPIM1 and TWIM1 cannot be \
                         enabled at the same time."
                    ),
                }
            }
            peripheral_interrupts::SPIM2_SPIS2_SPI2 => spi::SPIM2.handle_interrupt(),
            peripheral_interrupts::ADC => adc::ADC.handle_interrupt(),
            _ => return false,
        }
        true
    }
}
