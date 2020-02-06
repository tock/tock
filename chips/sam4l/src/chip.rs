//! Interrupt mapping and DMA channel setup.

use crate::acifc;
use crate::adc;
use crate::aes;
use crate::ast;
use crate::crccu;
use crate::dac;
use crate::deferred_call_tasks::Task;
use crate::dma;
use crate::eic;
use crate::flashcalw;
use crate::gpio;
use crate::i2c;
use crate::nvic;
use crate::pm;
use crate::spi;
use crate::trng;
use crate::usart;
use crate::usbc;

use core::fmt::Write;
use cortexm4;
use cortexm4::syscall::SCB_REGISTERS;
use kernel::common::deferred_call;
use kernel::Chip;

pub struct Sam4l {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl Sam4l {
    pub unsafe fn new() -> Sam4l {
        usart::USART0.set_dma(&dma::DMA_CHANNELS[0], &dma::DMA_CHANNELS[1]);
        dma::DMA_CHANNELS[0].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[1].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);

        usart::USART1.set_dma(&dma::DMA_CHANNELS[2], &dma::DMA_CHANNELS[3]);
        dma::DMA_CHANNELS[2].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[3].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);

        usart::USART2.set_dma(&dma::DMA_CHANNELS[4], &dma::DMA_CHANNELS[5]);
        dma::DMA_CHANNELS[4].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[5].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);

        usart::USART3.set_dma(&dma::DMA_CHANNELS[6], &dma::DMA_CHANNELS[7]);
        dma::DMA_CHANNELS[6].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[7].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);

        spi::SPI.set_dma(&dma::DMA_CHANNELS[8], &dma::DMA_CHANNELS[9]);
        dma::DMA_CHANNELS[8].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[9].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);

        i2c::I2C0.set_dma(&dma::DMA_CHANNELS[10]);
        dma::DMA_CHANNELS[10].initialize(&mut i2c::I2C0, dma::DMAWidth::Width8Bit);

        i2c::I2C1.set_dma(&dma::DMA_CHANNELS[11]);
        dma::DMA_CHANNELS[11].initialize(&mut i2c::I2C1, dma::DMAWidth::Width8Bit);

        i2c::I2C2.set_dma(&dma::DMA_CHANNELS[12]);
        dma::DMA_CHANNELS[12].initialize(&mut i2c::I2C2, dma::DMAWidth::Width8Bit);

        adc::ADC0.set_dma(&dma::DMA_CHANNELS[13]);
        dma::DMA_CHANNELS[13].initialize(&mut adc::ADC0, dma::DMAWidth::Width16Bit);

        Sam4l {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Sam4l {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        Task::Flashcalw => flashcalw::FLASH_CONTROLLER.handle_interrupt(),
                    }
                } else if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::ASTALARM => ast::AST.handle_interrupt(),

                        nvic::USART0 => usart::USART0.handle_interrupt(),
                        nvic::USART1 => usart::USART1.handle_interrupt(),
                        nvic::USART2 => usart::USART2.handle_interrupt(),
                        nvic::USART3 => usart::USART3.handle_interrupt(),

                        nvic::PDCA0 => dma::DMA_CHANNELS[0].handle_interrupt(),
                        nvic::PDCA1 => dma::DMA_CHANNELS[1].handle_interrupt(),
                        nvic::PDCA2 => dma::DMA_CHANNELS[2].handle_interrupt(),
                        nvic::PDCA3 => dma::DMA_CHANNELS[3].handle_interrupt(),
                        nvic::PDCA4 => dma::DMA_CHANNELS[4].handle_interrupt(),
                        nvic::PDCA5 => dma::DMA_CHANNELS[5].handle_interrupt(),
                        nvic::PDCA6 => dma::DMA_CHANNELS[6].handle_interrupt(),
                        nvic::PDCA7 => dma::DMA_CHANNELS[7].handle_interrupt(),
                        nvic::PDCA8 => dma::DMA_CHANNELS[8].handle_interrupt(),
                        nvic::PDCA9 => dma::DMA_CHANNELS[9].handle_interrupt(),
                        nvic::PDCA10 => dma::DMA_CHANNELS[10].handle_interrupt(),
                        nvic::PDCA11 => dma::DMA_CHANNELS[11].handle_interrupt(),
                        nvic::PDCA12 => dma::DMA_CHANNELS[12].handle_interrupt(),
                        nvic::PDCA13 => dma::DMA_CHANNELS[13].handle_interrupt(),
                        nvic::PDCA14 => dma::DMA_CHANNELS[14].handle_interrupt(),
                        nvic::PDCA15 => dma::DMA_CHANNELS[15].handle_interrupt(),

                        nvic::CRCCU => crccu::CRCCU.handle_interrupt(),
                        nvic::USBC => usbc::USBC.handle_interrupt(),

                        nvic::GPIO0 => gpio::PA.handle_interrupt(),
                        nvic::GPIO1 => gpio::PA.handle_interrupt(),
                        nvic::GPIO2 => gpio::PA.handle_interrupt(),
                        nvic::GPIO3 => gpio::PA.handle_interrupt(),
                        nvic::GPIO4 => gpio::PB.handle_interrupt(),
                        nvic::GPIO5 => gpio::PB.handle_interrupt(),
                        nvic::GPIO6 => gpio::PB.handle_interrupt(),
                        nvic::GPIO7 => gpio::PB.handle_interrupt(),
                        nvic::GPIO8 => gpio::PC.handle_interrupt(),
                        nvic::GPIO9 => gpio::PC.handle_interrupt(),
                        nvic::GPIO10 => gpio::PC.handle_interrupt(),
                        nvic::GPIO11 => gpio::PC.handle_interrupt(),

                        nvic::SPI => spi::SPI.handle_interrupt(),

                        nvic::TWIM0 => i2c::I2C0.handle_interrupt(),
                        nvic::TWIM1 => i2c::I2C1.handle_interrupt(),
                        nvic::TWIM2 => i2c::I2C2.handle_interrupt(),
                        nvic::TWIM3 => i2c::I2C3.handle_interrupt(),

                        nvic::TWIS0 => i2c::I2C0.handle_slave_interrupt(),
                        nvic::TWIS1 => i2c::I2C1.handle_slave_interrupt(),

                        nvic::HFLASHC => flashcalw::FLASH_CONTROLLER.handle_interrupt(),
                        nvic::ADCIFE => adc::ADC0.handle_interrupt(),
                        nvic::DACC => dac::DAC.handle_interrupt(),
                        nvic::ACIFC => acifc::ACIFC.handle_interrupt(),

                        nvic::TRNG => trng::TRNG.handle_interrupt(),
                        nvic::AESA => aes::AES.handle_interrupt(),

                        nvic::EIC1 => eic::EIC.handle_interrupt(&eic::Line::Ext1),
                        nvic::EIC2 => eic::EIC.handle_interrupt(&eic::Line::Ext2),
                        nvic::EIC3 => eic::EIC.handle_interrupt(&eic::Line::Ext3),
                        nvic::EIC4 => eic::EIC.handle_interrupt(&eic::Line::Ext4),
                        nvic::EIC5 => eic::EIC.handle_interrupt(&eic::Line::Ext5),
                        nvic::EIC6 => eic::EIC.handle_interrupt(&eic::Line::Ext6),
                        nvic::EIC7 => eic::EIC.handle_interrupt(&eic::Line::Ext7),
                        nvic::EIC8 => eic::EIC.handle_interrupt(&eic::Line::Ext8),

                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
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

    fn systick(&self) -> &cortexm4::systick::SysTick {
        &self.systick
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


    unsafe fn write_state(&self, writer: &mut dyn Write) {
        let _ccr = SCB_REGISTERS[0];
        let cfsr = SCB_REGISTERS[1];
        let hfsr = SCB_REGISTERS[2];
        let mmfar = SCB_REGISTERS[3];
        let bfar = SCB_REGISTERS[4];

        let iaccviol = (cfsr & 0x01) == 0x01;
        let daccviol = (cfsr & 0x02) == 0x02;
        let munstkerr = (cfsr & 0x08) == 0x08;
        let mstkerr = (cfsr & 0x10) == 0x10;
        let mlsperr = (cfsr & 0x20) == 0x20;
        let mmfarvalid = (cfsr & 0x80) == 0x80;

        let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
        let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
        let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
        let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
        let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
        let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
        let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

        let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
        let invstate = ((cfsr >> 16) & 0x02) == 0x02;
        let invpc = ((cfsr >> 16) & 0x04) == 0x04;
        let nocp = ((cfsr >> 16) & 0x08) == 0x08;
        let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
        let divbysero = ((cfsr >> 16) & 0x200) == 0x200;

        let vecttbl = (hfsr & 0x02) == 0x02;
        let forced = (hfsr & 0x40000000) == 0x40000000;

        let _ = writer.write_fmt(format_args!("\r\n---| Fault Status |---\r\n"));

        if iaccviol {
            let _ = writer.write_fmt(format_args!(
                "Instruction Access Violation:       {}\r\n",
                iaccviol
            ));
        }
        if daccviol {
            let _ = writer.write_fmt(format_args!(
                "Data Access Violation:              {}\r\n",
                daccviol
            ));
        }
        if munstkerr {
            let _ = writer.write_fmt(format_args!(
                "Memory Management Unstacking Fault: {}\r\n",
                munstkerr
            ));
        }
        if mstkerr {
            let _ = writer.write_fmt(format_args!(
                "Memory Management Stacking Fault:   {}\r\n",
                mstkerr
            ));
        }
        if mlsperr {
            let _ = writer.write_fmt(format_args!(
                "Memory Management Lazy FP Fault:    {}\r\n",
                mlsperr
            ));
        }

        if ibuserr {
            let _ = writer.write_fmt(format_args!(
                "Instruction Bus Error:              {}\r\n",
                ibuserr
            ));
        }
        if preciserr {
            let _ = writer.write_fmt(format_args!(
                "Precise Data Bus Error:             {}\r\n",
                preciserr
            ));
        }
        if impreciserr {
            let _ = writer.write_fmt(format_args!(
                "Imprecise Data Bus Error:           {}\r\n",
                impreciserr
            ));
        }
        if unstkerr {
            let _ = writer.write_fmt(format_args!(
                "Bus Unstacking Fault:               {}\r\n",
                unstkerr
            ));
        }
        if stkerr {
            let _ = writer.write_fmt(format_args!(
                "Bus Stacking Fault:                 {}\r\n",
                stkerr
            ));
        }
        if lsperr {
            let _ = writer.write_fmt(format_args!(
                "Bus Lazy FP Fault:                  {}\r\n",
                lsperr
            ));
        }
        if undefinstr {
            let _ = writer.write_fmt(format_args!(
                "Undefined Instruction Usage Fault:  {}\r\n",
                undefinstr
            ));
        }
        if invstate {
            let _ = writer.write_fmt(format_args!(
                "Invalid State Usage Fault:          {}\r\n",
                invstate
            ));
        }
        if invpc {
            let _ = writer.write_fmt(format_args!(
                "Invalid PC Load Usage Fault:        {}\r\n",
                invpc
            ));
        }
        if nocp {
            let _ = writer.write_fmt(format_args!(
                "No Coprocessor Usage Fault:         {}\r\n",
                nocp
            ));
        }
        if unaligned {
            let _ = writer.write_fmt(format_args!(
                "Unaligned Access Usage Fault:       {}\r\n",
                unaligned
            ));
        }
        if divbysero {
            let _ = writer.write_fmt(format_args!(
                "Divide By Zero:                     {}\r\n",
                divbysero
            ));
        }

        if vecttbl {
            let _ = writer.write_fmt(format_args!(
                "Bus Fault on Vector Table Read:     {}\r\n",
                vecttbl
            ));
        }
        if forced {
            let _ = writer.write_fmt(format_args!(
                "Forced Hard Fault:                  {}\r\n",
                forced
            ));
        }

        if mmfarvalid {
            let _ = writer.write_fmt(format_args!(
                "Faulting Memory Address:            {:#010X}\r\n",
                mmfar
            ));
        }
        if bfarvalid {
            let _ = writer.write_fmt(format_args!(
                "Bus Fault Address:                  {:#010X}\r\n",
                bfar
            ));
        }

        if cfsr == 0 && hfsr == 0 {
            let _ = writer.write_fmt(format_args!("No faults detected.\r\n"));
        } else {
            let _ = writer.write_fmt(format_args!(
                "Fault Status Register (CFSR):       {:#010X}\r\n",
                cfsr
            ));
            let _ = writer.write_fmt(format_args!(
                "Hard Fault Status Register (HFSR):  {:#010X}\r\n",
                hfsr
            ));
        }
    }
}
