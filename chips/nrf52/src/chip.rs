use crate::adc;
use crate::deferred_call_tasks::DeferredCallTask;
use crate::i2c;
use crate::nvmc;
use crate::radio;
use crate::spi;
use crate::uart;
use cortexm4::{self, nvic};
use kernel::common::deferred_call;
use kernel::common::deferred_call_mux::call_global_mux;
use kernel::debug;
use nrf5x::peripheral_interrupts;

pub struct NRF52 {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl NRF52 {
    pub unsafe fn new() -> NRF52 {
        NRF52 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            // The NRF52's systick is uncalibrated, but is clocked from the
            // 64Mhz CPU clock.
            systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
        }
    }
}

impl kernel::Chip for NRF52 {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        DeferredCallTask::Nvmc => nvmc::NVMC.handle_interrupt(),
                        DeferredCallTask::DeferredCallMux => {
                            call_global_mux();
                        }
                    }
                } else if let Some(interrupt) = nvic::next_pending() {
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
                        _ => debug!("NvicIdx not supported by Tock"),
                    }
                    let n = nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
