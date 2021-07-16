//! Chip trait setup.

use core::fmt::Write;
use kernel::deferred_call;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::adc;
use crate::clocks::Clocks;
use crate::gpio::{RPPins, SIO};
use crate::interrupts;
use crate::resets::Resets;
use crate::spi;
use crate::sysinfo;
use crate::timer::RPTimer;
use crate::uart::Uart;
use crate::watchdog::Watchdog;
use crate::xosc::Xosc;
use cortexm0p::interrupt_mask;

#[repr(u8)]
pub enum Processor {
    Processor0 = 0,
    Processor1 = 1,
}

pub struct Rp2040<'a, I: InterruptService<()> + 'a> {
    mpu: cortexm0p::mpu::MPU,
    userspace_kernel_boundary: cortexm0p::syscall::SysCall,
    interrupt_service: &'a I,
    sio: &'a SIO,
    processor0_interrupt_mask: (u128, u128),
    processor1_interrupt_mask: (u128, u128),
}

impl<'a, I: InterruptService<()>> Rp2040<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I, sio: &'a SIO) -> Self {
        Self {
            mpu: cortexm0p::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm0p::syscall::SysCall::new(),
            interrupt_service,
            sio: sio,
            processor0_interrupt_mask: interrupt_mask!(interrupts::SIO_IRQ_PROC1),
            processor1_interrupt_mask: interrupt_mask!(interrupts::SIO_IRQ_PROC0),
        }
    }
}

impl<'a, I: InterruptService<()>> Chip for Rp2040<'a, I> {
    type MPU = cortexm0p::mpu::MPU;
    type UserspaceKernelBoundary = cortexm0p::syscall::SysCall;

    fn service_pending_interrupts(&self) {
        unsafe {
            let mask = match self.sio.get_processor() {
                Processor::Processor0 => self.processor0_interrupt_mask,
                Processor::Processor1 => self.processor1_interrupt_mask,
            };
            loop {
                if let Some(interrupt) = cortexm0p::nvic::next_pending_with_mask(mask) {
                    // ignore SIO_IRQ_PROC1 as it is intended for processor 1
                    // not able to unset its pending status
                    // probably only processor 1 can unset the pending by reading the fifo
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt {}", interrupt);
                    }
                    let n = cortexm0p::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        // ignore SIO_IRQ_PROC1 as it is intended for processor 1
        // not able to unset its pending status
        // probably only processor 1 can unset the pending by reading the fifo
        let mask = match self.sio.get_processor() {
            Processor::Processor0 => self.processor0_interrupt_mask,
            Processor::Processor1 => self.processor1_interrupt_mask,
        };
        unsafe { cortexm0p::nvic::has_pending_with_mask(mask) || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm0p::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm0p::support::atomic(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        cortexm0p::print_cortexm0_state(writer);
    }
}

pub struct Rp2040DefaultPeripherals<'a> {
    pub resets: Resets,
    pub sio: SIO,
    pub clocks: Clocks,
    pub xosc: Xosc,
    pub timer: RPTimer<'a>,
    pub watchdog: Watchdog,
    pub pins: RPPins<'a>,
    pub uart0: Uart<'a>,
    pub adc: adc::Adc,
    pub spi0: spi::Spi<'a>,
    pub sysinfo: sysinfo::SysInfo,
}

impl<'a> Rp2040DefaultPeripherals<'a> {
    pub const fn new() -> Self {
        Self {
            resets: Resets::new(),
            sio: SIO::new(),
            clocks: Clocks::new(),
            xosc: Xosc::new(),
            timer: RPTimer::new(),
            watchdog: Watchdog::new(),
            pins: RPPins::new(),
            uart0: Uart::new_uart0(),
            adc: adc::Adc::new(),
            spi0: spi::Spi::new_spi0(),
            sysinfo: sysinfo::SysInfo::new(),
        }
    }

    pub fn set_clocks(&'a self) {
        self.spi0.set_clocks(&self.clocks);
    }
}

impl InterruptService<()> for Rp2040DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::TIMER_IRQ_0 => {
                self.timer.handle_interrupt();
                true
            }
            interrupts::SIO_IRQ_PROC0 => {
                self.sio.handle_proc_interrupt(Processor::Processor0);
                true
            }
            interrupts::SIO_IRQ_PROC1 => {
                self.sio.handle_proc_interrupt(Processor::Processor1);
                true
            }
            interrupts::SPI0_IRQ => {
                self.spi0.handle_interrupt();
                true
            }
            interrupts::UART0_IRQ => {
                self.uart0.handle_interrupt();
                true
            }
            interrupts::ADC_IRQ_FIFO => {
                self.adc.handle_interrupt();
                true
            }
            interrupts::IO_IRQ_BANK0 => {
                self.pins.handle_interrupt();
                true
            }
            _ => false,
        }
    }

    unsafe fn service_deferred_call(&self, _task: ()) -> bool {
        false
    }
}
