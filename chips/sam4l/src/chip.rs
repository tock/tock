//! Interrupt mapping and DMA channel setup.

use crate::deferred_call_tasks::Task;
use crate::pm;

use core::fmt::Write;
use cortexm4;
use kernel::common::deferred_call;
use kernel::{Chip, InterruptService};

pub struct Sam4l<I: InterruptService<Task> + 'static> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
    pub pm: &'static crate::pm::PowerManager,
    interrupt_service: &'static I,
}

impl<I: InterruptService<Task> + 'static> Sam4l<I> {
    pub unsafe fn new(pm: &'static crate::pm::PowerManager, interrupt_service: &'static I) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new(),
            pm,
            interrupt_service,
        }
    }
}

/// This struct, when initialized, instantiates all peripheral drivers for the apollo3.
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
pub struct Sam4lDefaultPeripherals {
    pub acifc: crate::acifc::Acifc<'static>,
    pub adc: crate::adc::Adc,
    pub aes: crate::aes::Aes<'static>,
    pub ast: crate::ast::Ast<'static>,
    pub crccu: crate::crccu::Crccu<'static>,
    pub dac: crate::dac::Dac,
    pub dma_channels: [crate::dma::DMAChannel; 16],
    pub eic: crate::eic::Eic<'static>,
    pub flash_controller: crate::flashcalw::FLASHCALW,
    pub gloc: crate::gloc::Gloc,
    pub pa: crate::gpio::Port<'static>,
    pub pb: crate::gpio::Port<'static>,
    pub pc: crate::gpio::Port<'static>,
    pub i2c0: crate::i2c::I2CHw,
    pub i2c1: crate::i2c::I2CHw,
    pub i2c2: crate::i2c::I2CHw,
    pub i2c3: crate::i2c::I2CHw,
    pub spi: crate::spi::SpiHw,
    pub trng: crate::trng::Trng<'static>,
    pub usart0: crate::usart::USART<'static>,
    pub usart1: crate::usart::USART<'static>,
    pub usart2: crate::usart::USART<'static>,
    pub usart3: crate::usart::USART<'static>,
    pub usbc: crate::usbc::Usbc<'static>,
}

impl Sam4lDefaultPeripherals {
    pub fn new(pm: &'static crate::pm::PowerManager) -> Self {
        use crate::dma::{DMAChannel, DMAChannelNum};
        Self {
            acifc: crate::acifc::Acifc::new(),
            adc: crate::adc::Adc::new(crate::dma::DMAPeripheral::ADCIFE_RX, pm),
            aes: crate::aes::Aes::new(),
            ast: crate::ast::Ast::new(),
            crccu: crate::crccu::Crccu::new(),
            dac: crate::dac::Dac::new(),
            dma_channels: [
                DMAChannel::new(DMAChannelNum::DMAChannel00),
                DMAChannel::new(DMAChannelNum::DMAChannel01),
                DMAChannel::new(DMAChannelNum::DMAChannel02),
                DMAChannel::new(DMAChannelNum::DMAChannel03),
                DMAChannel::new(DMAChannelNum::DMAChannel04),
                DMAChannel::new(DMAChannelNum::DMAChannel05),
                DMAChannel::new(DMAChannelNum::DMAChannel06),
                DMAChannel::new(DMAChannelNum::DMAChannel07),
                DMAChannel::new(DMAChannelNum::DMAChannel08),
                DMAChannel::new(DMAChannelNum::DMAChannel09),
                DMAChannel::new(DMAChannelNum::DMAChannel10),
                DMAChannel::new(DMAChannelNum::DMAChannel11),
                DMAChannel::new(DMAChannelNum::DMAChannel12),
                DMAChannel::new(DMAChannelNum::DMAChannel13),
                DMAChannel::new(DMAChannelNum::DMAChannel14),
                DMAChannel::new(DMAChannelNum::DMAChannel15),
            ],
            eic: crate::eic::Eic::new(),
            flash_controller: crate::flashcalw::FLASHCALW::new(
                crate::pm::HSBClock::FLASHCALW,
                crate::pm::HSBClock::FLASHCALWP,
                crate::pm::PBBClock::FLASHCALW,
            ),
            gloc: crate::gloc::Gloc::new(),
            pa: crate::gpio::Port::new_port_a(),
            pb: crate::gpio::Port::new_port_b(),
            pc: crate::gpio::Port::new_port_c(),
            i2c0: crate::i2c::I2CHw::new_i2c0(pm),
            i2c1: crate::i2c::I2CHw::new_i2c1(pm),
            i2c2: crate::i2c::I2CHw::new_i2c2(pm),
            i2c3: crate::i2c::I2CHw::new_i2c3(pm),
            spi: crate::spi::SpiHw::new(pm),
            trng: crate::trng::Trng::new(),
            usart0: crate::usart::USART::new_usart0(pm),
            usart1: crate::usart::USART::new_usart1(pm),
            usart2: crate::usart::USART::new_usart2(pm),
            usart3: crate::usart::USART::new_usart3(pm),
            usbc: crate::usbc::Usbc::new(pm),
        }
    }

    // Sam4l was the only chip that partially initialized some drivers in new, I
    // have moved that initialization to this helper function.
    // TODO: Delete explanation
    pub fn setup_dma(&'static self) {
        use crate::dma;
        self.usart0
            .set_dma(&self.dma_channels[0], &self.dma_channels[1]);
        self.dma_channels[0].initialize(&self.usart0, dma::DMAWidth::Width8Bit);
        self.dma_channels[1].initialize(&self.usart0, dma::DMAWidth::Width8Bit);

        self.usart1
            .set_dma(&self.dma_channels[2], &self.dma_channels[3]);
        self.dma_channels[2].initialize(&self.usart1, dma::DMAWidth::Width8Bit);
        self.dma_channels[3].initialize(&self.usart1, dma::DMAWidth::Width8Bit);

        self.usart2
            .set_dma(&self.dma_channels[4], &self.dma_channels[5]);
        self.dma_channels[4].initialize(&self.usart2, dma::DMAWidth::Width8Bit);
        self.dma_channels[5].initialize(&self.usart2, dma::DMAWidth::Width8Bit);

        self.usart3
            .set_dma(&self.dma_channels[6], &self.dma_channels[7]);
        self.dma_channels[6].initialize(&self.usart3, dma::DMAWidth::Width8Bit);
        self.dma_channels[7].initialize(&self.usart3, dma::DMAWidth::Width8Bit);

        self.spi
            .set_dma(&self.dma_channels[8], &self.dma_channels[9]);
        self.dma_channels[8].initialize(&self.spi, dma::DMAWidth::Width8Bit);
        self.dma_channels[9].initialize(&self.spi, dma::DMAWidth::Width8Bit);

        self.i2c0.set_dma(&self.dma_channels[10]);
        self.dma_channels[10].initialize(&self.i2c0, dma::DMAWidth::Width8Bit);

        self.i2c1.set_dma(&self.dma_channels[11]);
        self.dma_channels[11].initialize(&self.i2c1, dma::DMAWidth::Width8Bit);

        self.i2c2.set_dma(&self.dma_channels[12]);
        self.dma_channels[12].initialize(&self.i2c2, dma::DMAWidth::Width8Bit);

        self.adc.set_dma(&self.dma_channels[13]);
        self.dma_channels[13].initialize(&self.adc, dma::DMAWidth::Width16Bit);
    }
}
impl kernel::InterruptService<Task> for Sam4lDefaultPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        use crate::nvic;
        match interrupt {
            nvic::ASTALARM => self.ast.handle_interrupt(),

            nvic::USART0 => self.usart0.handle_interrupt(),
            nvic::USART1 => self.usart1.handle_interrupt(),
            nvic::USART2 => self.usart2.handle_interrupt(),
            nvic::USART3 => self.usart3.handle_interrupt(),

            nvic::PDCA0 => self.dma_channels[0].handle_interrupt(),
            nvic::PDCA1 => self.dma_channels[1].handle_interrupt(),
            nvic::PDCA2 => self.dma_channels[2].handle_interrupt(),
            nvic::PDCA3 => self.dma_channels[3].handle_interrupt(),
            nvic::PDCA4 => self.dma_channels[4].handle_interrupt(),
            nvic::PDCA5 => self.dma_channels[5].handle_interrupt(),
            nvic::PDCA6 => self.dma_channels[6].handle_interrupt(),
            nvic::PDCA7 => self.dma_channels[7].handle_interrupt(),
            nvic::PDCA8 => self.dma_channels[8].handle_interrupt(),
            nvic::PDCA9 => self.dma_channels[9].handle_interrupt(),
            nvic::PDCA10 => self.dma_channels[10].handle_interrupt(),
            nvic::PDCA11 => self.dma_channels[11].handle_interrupt(),
            nvic::PDCA12 => self.dma_channels[12].handle_interrupt(),
            nvic::PDCA13 => self.dma_channels[13].handle_interrupt(),
            nvic::PDCA14 => self.dma_channels[14].handle_interrupt(),
            nvic::PDCA15 => self.dma_channels[15].handle_interrupt(),

            nvic::CRCCU => self.crccu.handle_interrupt(),
            nvic::USBC => self.usbc.handle_interrupt(),

            nvic::GPIO0 => self.pa.handle_interrupt(),
            nvic::GPIO1 => self.pa.handle_interrupt(),
            nvic::GPIO2 => self.pa.handle_interrupt(),
            nvic::GPIO3 => self.pa.handle_interrupt(),
            nvic::GPIO4 => self.pb.handle_interrupt(),
            nvic::GPIO5 => self.pb.handle_interrupt(),
            nvic::GPIO6 => self.pb.handle_interrupt(),
            nvic::GPIO7 => self.pb.handle_interrupt(),
            nvic::GPIO8 => self.pc.handle_interrupt(),
            nvic::GPIO9 => self.pc.handle_interrupt(),
            nvic::GPIO10 => self.pc.handle_interrupt(),
            nvic::GPIO11 => self.pc.handle_interrupt(),

            nvic::SPI => self.spi.handle_interrupt(),

            nvic::TWIM0 => self.i2c0.handle_interrupt(),
            nvic::TWIM1 => self.i2c1.handle_interrupt(),
            nvic::TWIM2 => self.i2c2.handle_interrupt(),
            nvic::TWIM3 => self.i2c3.handle_interrupt(),
            nvic::TWIS0 => self.i2c0.handle_slave_interrupt(),
            nvic::TWIS1 => self.i2c1.handle_slave_interrupt(),

            nvic::HFLASHC => self.flash_controller.handle_interrupt(),
            nvic::ADCIFE => self.adc.handle_interrupt(),
            nvic::DACC => self.dac.handle_interrupt(),
            nvic::ACIFC => self.acifc.handle_interrupt(),

            nvic::TRNG => self.trng.handle_interrupt(),
            nvic::AESA => self.aes.handle_interrupt(),

            nvic::EIC1 => self.eic.handle_interrupt(&crate::eic::Line::Ext1),
            nvic::EIC2 => self.eic.handle_interrupt(&crate::eic::Line::Ext2),
            nvic::EIC3 => self.eic.handle_interrupt(&crate::eic::Line::Ext3),
            nvic::EIC4 => self.eic.handle_interrupt(&crate::eic::Line::Ext4),
            nvic::EIC5 => self.eic.handle_interrupt(&crate::eic::Line::Ext5),
            nvic::EIC6 => self.eic.handle_interrupt(&crate::eic::Line::Ext6),
            nvic::EIC7 => self.eic.handle_interrupt(&crate::eic::Line::Ext7),
            nvic::EIC8 => self.eic.handle_interrupt(&crate::eic::Line::Ext8),
            _ => return false,
        }
        true
    }
    unsafe fn service_deferred_call(&self, task: Task) -> bool {
        match task {
            crate::deferred_call_tasks::Task::Flashcalw => self.flash_controller.handle_interrupt(),
        }
        true
    }
}

impl<I: InterruptService<Task> + 'static> Chip for Sam4l<I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match self.interrupt_service.service_deferred_call(task) {
                        true => {}
                        false => panic!("unhandled deferred call task"),
                    }
                } else if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match self.interrupt_service.service_interrupt(interrupt) {
                        true => {}
                        false => panic!("unhandled interrupt"),
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
        unsafe { cortexm4::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        if pm::deep_sleep_ready() {
            unsafe {
                cortexm4::scb::set_sleepdeep();
            }
        } else {
            unsafe {
                cortexm4::scb::unset_sleepdeep();
            }
        }

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

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        cortexm4::print_cortexm4_state(writer);
    }
}
