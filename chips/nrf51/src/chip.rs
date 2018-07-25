use cortexm0;
use cortexm0::nvic;
use i2c;
use kernel;
use nrf5x;
use nrf5x::peripheral_interrupts;
use radio;
use uart;

pub struct NRF51(());

impl NRF51 {
    pub unsafe fn new() -> NRF51 {
        NRF51(())
    }
}

impl kernel::Chip for NRF51 {
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
                    peripheral_interrupts::ECB => nrf5x::aes::AESECB.handle_interrupt(),
                    peripheral_interrupts::GPIOTE => nrf5x::gpio::PORT.handle_interrupt(),
                    peripheral_interrupts::RADIO => radio::RADIO.handle_interrupt(),
                    peripheral_interrupts::RNG => nrf5x::trng::TRNG.handle_interrupt(),
                    peripheral_interrupts::RTC1 => nrf5x::rtc::RTC.handle_interrupt(),
                    peripheral_interrupts::TEMP => nrf5x::temperature::TEMP.handle_interrupt(),
                    peripheral_interrupts::TIMER0 => nrf5x::timer::TIMER0.handle_interrupt(),
                    peripheral_interrupts::TIMER1 => nrf5x::timer::ALARM1.handle_interrupt(),
                    peripheral_interrupts::TIMER2 => nrf5x::timer::TIMER2.handle_interrupt(),
                    peripheral_interrupts::UART0 => uart::UART0.handle_interrupt(),
                    peripheral_interrupts::SPI0_TWI0 => {
                        // SPI0 and TWI0 share interrupts.
                        // Dispatch the correct handler.
                        // match (spi::SPIM0.is_enabled(), i2c::TWIM0.is_enabled()) {
                        match (false, i2c::TWIM0.is_enabled()) {
                            (false, false) => (),
                            (true, false) => panic!("SPI is not yet implemented"),
                            // spi::SPIM0.handle_interrupt(),
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
                        // match (spi::SPIM1.is_enabled(), i2c::TWIM1.is_enabled()) {
                        match (false, i2c::TWIM1.is_enabled()) {
                            (false, false) => (),
                            (true, false) => panic!("SPI is not yet implemented"),
                            // spi::SPIM1.handle_interrupt(),
                            (false, true) => i2c::TWIM1.handle_interrupt(),
                            (true, true) => debug_assert!(
                                false,
                                "SPIM1 and TWIM1 cannot be \
                                 enabled at the same time."
                            ),
                        }
                    }
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

    fn sleep(&self) {
        unsafe {
            cortexm0::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm0::support::atomic(f)
    }
}
