use cortexm4::nvic;
use i2c;
use kernel;
use nrf5x;
use nrf5x::peripheral_interrupts::*;
use radio;
use spi;
use uart;

pub struct NRF52(());

impl NRF52 {
    pub unsafe fn new() -> NRF52 {
        NRF52(())
    }
}

impl kernel::Chip for NRF52 {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &self.0
    }

    fn systick(&self) -> &Self::SysTick {
        &self.0
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                match interrupt {
                    ECB => nrf5x::aes::AESECB.handle_interrupt(),
                    GPIOTE => nrf5x::gpio::PORT.handle_interrupt(),
                    RADIO => radio::RADIO.handle_interrupt(),
                    RNG => nrf5x::trng::TRNG.handle_interrupt(),
                    RTC1 => nrf5x::rtc::RTC.handle_interrupt(),
                    TEMP => nrf5x::temperature::TEMP.handle_interrupt(),
                    TIMER0 => nrf5x::timer::TIMER0.handle_interrupt(),
                    TIMER1 => nrf5x::timer::ALARM1.handle_interrupt(),
                    TIMER2 => nrf5x::timer::TIMER2.handle_interrupt(),
                    UART0 => uart::UART0.handle_interrupt(),
                    SPI0_TWI0 => {
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
                    SPI1_TWI1 => {
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
                    SPIM2_SPIS2_SPI2 => spi::SPIM2.handle_interrupt(),
                    _ => debug!("NvicIdx not supported by Tock"),
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }
}
