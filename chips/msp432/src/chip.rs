use core::fmt::Write;
use cortexm4;
use kernel::Chip;

use crate::nvic;
use crate::wdt;
use kernel::InterruptService;

pub struct Msp432<'a, I: InterruptService<()> + 'a> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
    watchdog: wdt::Wdt,
    interrupt_service: &'a I,
}

pub struct Msp432DefaultPeripherals<'a> {
    pub adc: crate::adc::Adc<'a>,
    pub uart0: crate::uart::Uart<'a>,
    pub cs: crate::cs::ClockSystem,
    pub dma_channels: crate::dma::DmaChannels<'a>,
    pub adc_ref: crate::ref_module::Ref,
    pub timer_a0: crate::timer::TimerA<'a>,
    pub timer_a1: crate::timer::TimerA<'a>,
    pub timer_a2: crate::timer::TimerA<'a>,
    pub timer_a3: crate::timer::TimerA<'a>,
    pub gpio: crate::gpio::GpioManager<'a>,
}

impl<'a> Msp432DefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            adc: crate::adc::Adc::new(),
            uart0: crate::uart::Uart::new(crate::usci::USCI_A0_BASE, 0, 1, 1, 1),
            cs: crate::cs::ClockSystem::new(),
            dma_channels: crate::dma::DmaChannels::new(),
            adc_ref: crate::ref_module::Ref::new(),
            timer_a0: crate::timer::TimerA::new(crate::timer::TIMER_A0_BASE),
            timer_a1: crate::timer::TimerA::new(crate::timer::TIMER_A1_BASE),
            timer_a2: crate::timer::TimerA::new(crate::timer::TIMER_A2_BASE),
            timer_a3: crate::timer::TimerA::new(crate::timer::TIMER_A3_BASE),
            gpio: crate::gpio::GpioManager::new(),
        }
    }

    pub unsafe fn init(&'a self) {
        // Setup DMA channels
        self.uart0.set_dma(
            &self.dma_channels[self.uart0.tx_dma_chan],
            &self.dma_channels[self.uart0.rx_dma_chan],
        );
        self.dma_channels[self.uart0.tx_dma_chan].set_client(&self.uart0);
        self.dma_channels[self.uart0.rx_dma_chan].set_client(&self.uart0);

        // Setup Reference Module, Timer and DMA for ADC
        self.adc.set_modules(
            &self.adc_ref,
            &self.timer_a3,
            &self.dma_channels[self.adc.dma_chan],
        );
        self.dma_channels[self.adc.dma_chan].set_client(&self.adc);
    }
}

impl<'a> kernel::InterruptService<()> for Msp432DefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::ADC => self.adc.handle_interrupt(),
            nvic::DMA_INT0 => self.dma_channels.handle_interrupt(0),
            nvic::DMA_INT1 => self.dma_channels.handle_interrupt(1),
            nvic::DMA_INT2 => self.dma_channels.handle_interrupt(2),
            nvic::DMA_INT3 => self.dma_channels.handle_interrupt(3),
            nvic::DMA_ERR => self.dma_channels.handle_interrupt(-1),
            nvic::IO_PORT1 => self.gpio.handle_interrupt(0),
            nvic::IO_PORT2 => self.gpio.handle_interrupt(1),
            nvic::IO_PORT3 => self.gpio.handle_interrupt(2),
            nvic::IO_PORT4 => self.gpio.handle_interrupt(3),
            nvic::IO_PORT5 => self.gpio.handle_interrupt(4),
            nvic::IO_PORT6 => self.gpio.handle_interrupt(5),
            nvic::TIMER_A0_0 | nvic::TIMER_A0_1 => self.timer_a0.handle_interrupt(),
            nvic::TIMER_A1_0 | nvic::TIMER_A1_1 => self.timer_a1.handle_interrupt(),
            nvic::TIMER_A2_0 | nvic::TIMER_A2_1 => self.timer_a2.handle_interrupt(),
            nvic::TIMER_A3_0 | nvic::TIMER_A3_1 => self.timer_a3.handle_interrupt(),

            _ => return false,
        }
        true
    }
    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<'a, I: InterruptService<()> + 'a> Msp432<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new_with_calibration(48_000_000),
            watchdog: wdt::Wdt::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService<()> + 'a> Chip for Msp432<'a, I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = wdt::Wdt;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt {}", interrupt);
                    }

                    let n = cortexm4::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &cortexm4::systick::SysTick {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &self.watchdog
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
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
