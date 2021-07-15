use crate::deferred_call_tasks::DeferredCallTask;
use core::fmt::Write;
use cortexm4::{self, nvic};
use kernel::deferred_call;
use kernel::hil::time::Alarm;
use kernel::platform::chip::InterruptService;

pub struct NRF52<'a, I: InterruptService<DeferredCallTask> + 'a> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> NRF52<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

/// This struct, when initialized, instantiates all peripheral drivers for the apollo3.
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
pub struct Nrf52DefaultPeripherals<'a> {
    pub acomp: crate::acomp::Comparator<'a>,
    pub ecb: crate::aes::AesECB<'a>,
    pub pwr_clk: crate::power::Power<'a>,
    pub ieee802154_radio: crate::ieee802154_radio::Radio<'a>,
    pub ble_radio: crate::ble_radio::Radio<'a>,
    pub trng: crate::trng::Trng<'a>,
    pub rtc: crate::rtc::Rtc<'a>,
    pub temp: crate::temperature::Temp<'a>,
    pub timer0: crate::timer::TimerAlarm<'a>,
    pub timer1: crate::timer::TimerAlarm<'a>,
    pub timer2: crate::timer::Timer,
    pub uarte0: crate::uart::Uarte<'a>,
    pub spim0: crate::spi::SPIM,
    pub twi0: crate::i2c::TWI,
    pub spim1: crate::spi::SPIM,
    pub twi1: crate::i2c::TWI,
    pub spim2: crate::spi::SPIM,
    pub adc: crate::adc::Adc,
    pub nvmc: crate::nvmc::Nvmc,
    pub clock: crate::clock::Clock,
    pub pwm0: crate::pwm::Pwm,
}

impl<'a> Nrf52DefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            acomp: crate::acomp::Comparator::new(),
            ecb: crate::aes::AesECB::new(),
            pwr_clk: crate::power::Power::new(),
            ieee802154_radio: crate::ieee802154_radio::Radio::new(),
            ble_radio: crate::ble_radio::Radio::new(),
            trng: crate::trng::Trng::new(),
            rtc: crate::rtc::Rtc::new(),
            temp: crate::temperature::Temp::new(),
            timer0: crate::timer::TimerAlarm::new(0),
            timer1: crate::timer::TimerAlarm::new(1),
            timer2: crate::timer::Timer::new(2),
            uarte0: crate::uart::Uarte::new(),
            spim0: crate::spi::SPIM::new(0),
            twi0: crate::i2c::TWI::new_twi0(),
            spim1: crate::spi::SPIM::new(1),
            twi1: crate::i2c::TWI::new_twi1(),
            spim2: crate::spi::SPIM::new(2),
            adc: crate::adc::Adc::new(),
            nvmc: crate::nvmc::Nvmc::new(),
            clock: crate::clock::Clock::new(),
            pwm0: crate::pwm::Pwm::new(),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'a self) {
        self.ieee802154_radio.set_timer_ref(&self.timer0);
        self.timer0.set_alarm_client(&self.ieee802154_radio);
    }
}
impl<'a> kernel::platform::chip::InterruptService<DeferredCallTask>
    for Nrf52DefaultPeripherals<'a>
{
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            crate::peripheral_interrupts::COMP => self.acomp.handle_interrupt(),
            crate::peripheral_interrupts::ECB => self.ecb.handle_interrupt(),
            crate::peripheral_interrupts::POWER_CLOCK => self.pwr_clk.handle_interrupt(),
            crate::peripheral_interrupts::RADIO => {
                match (
                    self.ieee802154_radio.is_enabled(),
                    self.ble_radio.is_enabled(),
                ) {
                    (false, false) => (),
                    (true, false) => self.ieee802154_radio.handle_interrupt(),
                    (false, true) => self.ble_radio.handle_interrupt(),
                    (true, true) => kernel::debug!(
                        "nRF 802.15.4 and BLE radios cannot be simultaneously enabled!"
                    ),
                }
            }
            crate::peripheral_interrupts::RNG => self.trng.handle_interrupt(),
            crate::peripheral_interrupts::RTC1 => self.rtc.handle_interrupt(),
            crate::peripheral_interrupts::TEMP => self.temp.handle_interrupt(),
            crate::peripheral_interrupts::TIMER0 => self.timer0.handle_interrupt(),
            crate::peripheral_interrupts::TIMER1 => self.timer1.handle_interrupt(),
            crate::peripheral_interrupts::TIMER2 => self.timer2.handle_interrupt(),
            crate::peripheral_interrupts::UART0 => self.uarte0.handle_interrupt(),
            crate::peripheral_interrupts::SPI0_TWI0 => {
                // SPI0 and TWI0 share interrupts.
                // Dispatch the correct handler.
                match (self.spim0.is_enabled(), self.twi0.is_enabled()) {
                    (false, false) => (),
                    (true, false) => self.spim0.handle_interrupt(),
                    (false, true) => self.twi0.handle_interrupt(),
                    (true, true) => debug_assert!(
                        false,
                        "SPIM0 and TWIM0 cannot be \
                         enabled at the same time."
                    ),
                }
            }
            crate::peripheral_interrupts::SPI1_TWI1 => {
                // SPI1 and TWI1 share interrupts.
                // Dispatch the correct handler.
                match (self.spim1.is_enabled(), self.twi1.is_enabled()) {
                    (false, false) => (),
                    (true, false) => self.spim1.handle_interrupt(),
                    (false, true) => self.twi1.handle_interrupt(),
                    (true, true) => debug_assert!(
                        false,
                        "SPIM1 and TWIM1 cannot be \
                         enabled at the same time."
                    ),
                }
            }
            crate::peripheral_interrupts::SPIM2_SPIS2_SPI2 => self.spim2.handle_interrupt(),
            crate::peripheral_interrupts::ADC => self.adc.handle_interrupt(),
            _ => return false,
        }
        true
    }
    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        match task {
            DeferredCallTask::Nvmc => self.nvmc.handle_interrupt(),
        }
        true
    }
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> kernel::platform::chip::Chip for NRF52<'a, I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    if !self.interrupt_service.service_deferred_call(task) {
                        panic!("unhandled deferred call task");
                    }
                } else if let Some(interrupt) = nvic::next_pending() {
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt {}", interrupt);
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
