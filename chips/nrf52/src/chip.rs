use crate::deferred_call_tasks::DeferredCallTask;
use core::fmt::Write;
use cortexm4::{self, nvic};
use kernel::common::deferred_call;
use kernel::InterruptService;

pub struct NRF52<'a, I: InterruptService<DeferredCallTask> + 'a> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> NRF52<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            // The NRF52's systick is uncalibrated, but is clocked from the
            // 64Mhz CPU clock.
            scheduler_timer: cortexm4::systick::SysTick::new_with_calibration(64000000),
            interrupt_service,
        }
    }
}

/// This macro defines a struct that, when initialized,
/// instantiates all peripheral drivers shared across all supported nrf52 chips
/// in Tock. If a board
/// wishes to use only a subset of these peripherals, this
/// macro cannot be used, and this struct should be
/// constructed manually in main.rs. The input to the macro is the name of the struct
/// that will hold the peripherals, which can be chosen by the board.
#[macro_export]
macro_rules! create_default_nrf52_peripherals {
    ($N:ident) => {
        struct $N<'a> {
            acomp: acomp::Comparator<'a>,
            ecb: aes::AesECB<'a>,
            gpio_port: &'static gpio::Port<'static>,
            pwr_clk: power::Power<'a>,
            ieee802154_radio: ieee802154_radio::Radio<'a>,
            ble_radio: ble_radio::Radio<'a>,
            trng: trng::Trng<'a>,
            rtc: rtc::Rtc<'a>,
            temp: temperature::Temp<'a>,
            timer0: timer::TimerAlarm<'a>,
            timer1: timer::TimerAlarm<'a>,
            timer2: timer::Timer,
            uarte0: uart::Uarte<'a>,
            spim0: spi::SPIM,
            twim0: i2c::TWIM,
            spim1: spi::SPIM,
            twim1: i2c::TWIM,
            spim2: spi::SPIM,
            adc: adc::Adc,
            nvmc: nvmc::Nvmc,
        }
        impl<'a> $N<'a> {
            fn new(gpio_port: &'static gpio::Port<'static>, ppi: &'a ppi::Ppi) -> Self {
                Self {
                    acomp: acomp::Comparator::new(),
                    ecb: aes::AesECB::new(),
                    gpio_port,
                    pwr_clk: power::Power::new(),
                    ieee802154_radio: ieee802154_radio::Radio::new(ppi),
                    ble_radio: ble_radio::Radio::new(),
                    trng: trng::Trng::new(),
                    rtc: rtc::Rtc::new(),
                    temp: temperature::Temp::new(),
                    timer0: timer::TimerAlarm::new(0),
                    timer1: timer::TimerAlarm::new(1),
                    timer2: timer::Timer::new(2),
                    uarte0: uart::Uarte::new(),
                    spim0: spi::SPIM::new(0),
                    twim0: i2c::TWIM::new_twim0(),
                    spim1: spi::SPIM::new(1),
                    twim1: i2c::TWIM::new_twim1(),
                    spim2: spi::SPIM::new(2),
                    adc: adc::Adc::new(),
                    nvmc: nvmc::Nvmc::new(),
                }
            }
            // Necessary for setting up circular dependencies
            fn init(&'a self) {
                self.ieee802154_radio.set_timer_ref(&self.timer0);
            }
        }
        impl<'a> kernel::InterruptService<DeferredCallTask> for $N<'a> {
            unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
                match interrupt {
                    base_interrupts::COMP => self.acomp.handle_interrupt(),
                    base_interrupts::ECB => self.ecb.handle_interrupt(),
                    base_interrupts::GPIOTE => self.gpio_port.handle_interrupt(),
                    base_interrupts::POWER_CLOCK => self.pwr_clk.handle_interrupt(),
                    base_interrupts::RADIO => {
                        match (
                            self.ieee802154_radio.is_enabled(),
                            self.ble_radio.is_enabled(),
                        ) {
                            (false, false) => (),
                            (true, false) => self.ieee802154_radio.handle_interrupt(),
                            (false, true) => self.ble_radio.handle_interrupt(),
                            (true, true) => debug!(
                                "nRF 802.15.4 and BLE radios cannot be simultaneously enabled!"
                            ),
                        }
                    }
                    base_interrupts::RNG => self.trng.handle_interrupt(),
                    base_interrupts::RTC1 => self.rtc.handle_interrupt(),
                    base_interrupts::TEMP => self.temp.handle_interrupt(),
                    base_interrupts::TIMER0 => self.timer0.handle_interrupt(),
                    base_interrupts::TIMER1 => self.timer1.handle_interrupt(),
                    base_interrupts::TIMER2 => self.timer2.handle_interrupt(),
                    base_interrupts::UART0 => self.uarte0.handle_interrupt(),
                    base_interrupts::SPI0_TWI0 => {
                        // SPI0 and TWI0 share interrupts.
                        // Dispatch the correct handler.
                        match (self.spim0.is_enabled(), self.twim0.is_enabled()) {
                            (false, false) => (),
                            (true, false) => self.spim0.handle_interrupt(),
                            (false, true) => self.twim0.handle_interrupt(),
                            (true, true) => debug_assert!(
                                false,
                                "SPIM0 and TWIM0 cannot be \
                         enabled at the same time."
                            ),
                        }
                    }
                    base_interrupts::SPI1_TWI1 => {
                        // SPI1 and TWI1 share interrupts.
                        // Dispatch the correct handler.
                        match (self.spim1.is_enabled(), self.twim1.is_enabled()) {
                            (false, false) => (),
                            (true, false) => self.spim1.handle_interrupt(),
                            (false, true) => self.twim1.handle_interrupt(),
                            (true, true) => debug_assert!(
                                false,
                                "SPIM1 and TWIM1 cannot be \
                         enabled at the same time."
                            ),
                        }
                    }
                    base_interrupts::SPIM2_SPIS2_SPI2 => self.spim2.handle_interrupt(),
                    base_interrupts::ADC => self.adc.handle_interrupt(),
                    base_interrupts::GPIOTE => self.gpio_port.handle_interrupt(),
                    _ => return false,
                }
                true
            }
            unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
                match task {
                    DeferredCallTask::Nvmc => self.nvmc.handle_interrupt(),
                    _ => return false,
                }
                true
            }
        }
    };
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> kernel::Chip for NRF52<'a, I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match self.interrupt_service.service_deferred_call(task) {
                        true => {}
                        false => panic!("unhandled deferred call task"),
                    }
                } else if let Some(interrupt) = nvic::next_pending() {
                    match self.interrupt_service.service_interrupt(interrupt) {
                        true => {}
                        false => panic!("unhandled interrupt"),
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

    unsafe fn print_state(&self, write: &mut dyn Write) {
        cortexm4::print_cortexm4_state(write);
    }
}
