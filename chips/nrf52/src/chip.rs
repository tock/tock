use cortexm4::nvic;
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
                    SPI0_TWI0 => spi::SPIM0.handle_interrupt(),
                    SPI1_TWI1 => spi::SPIM1.handle_interrupt(),
                    SPIM2_SPIS2_SPI2 => spi::SPIM2.handle_interrupt(),
                    _ => debug!("NvicIdx not supported by Tock\r\n"),
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
