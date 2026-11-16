// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! High-level chip setup and interrupt dispatch for the MPS3-AN536.
//!
//! This module implements [`kernel::platform::chip::Chip`] for the
//! Cortex-R52-based AN536 subsystem.

use crate::{uart::ApbUart, dualtimer::DualTimer};
use cortexr52::{gentimer::GenTimer, mpu::MPU, support, syscall::SysCall};
use cortexr52::gic::{self, Irq};
use kernel::platform::chip::Chip;
use crate::config;

/// The MPS3-AN536 chip.
pub struct XckuChip {
    mpu: MPU,
    syscall: SysCall,
    pub gic: gic::Gic,
    pub uart2: ApbUart<'static>,
    pub gentimer: GenTimer<'static>,
    pub dualtimer1: DualTimer<'static>, //unused for now
    //pub dualtimer2: dualtimer::DualTimer<'static>,
}

impl XckuChip {
    /// Create a new `XckuChip`.
    pub unsafe fn new() -> Self {
        Self {
            syscall: SysCall {},
            mpu : MPU::new(),
            gic: gic::Gic::new(),
            uart2: ApbUart::new(config::BASE_UART2),
            gentimer: GenTimer::new(),
            dualtimer1: DualTimer::new(config::BASE_DUALTIMER1),
            //dualtimer2: DualTimer::new(config::BASE_DUALTIMER2),
        }
    }
}

//
// INTERRUPTS MAP
//


// Note: Internal PPI is Cortex 52 specific and defined in gic.rs; Also,
// "External" PPI is different from "Extended". Extended PPI are not used with An536

// Idx  SPI                             Exter PPI (per core)
// -----------------------------------------------------------------
//  0   WDG                             UART 0/1 Receive Interrupt
//  1   DualTimer 1                     UART 0/1 Transmit Interrupt
//  2   DualTimer 2                     UART 0/1 Combined Interrupt
//  3   DualTimer Combined              UART 0/1 Overflow
//  4   RTC
//  5   UART 2 Receive Interrupt
//  6   UART 2 Transmit Interrupt
//  7   UART 3 Receive Interrupt
//  8   UART 3 Transmit Interrupt
//  9   UART 4 Receive Interrupt
// 10   UART 4 Transmit Interrupt
// 11   UART 5 Receive Interrupt
// 12   UART 5 Transmit Interrupt
// 13   UART 2 Combined Interrupt
// 14   UART 3 Combined Interrupt
// 15   UART 4 Combined Interrupt
// 16   UART 5 Combined Interrupt
// 17   UART Overflow (2, 3, 4 & 5)
// 18   Ethernet
// 19   USB
// 20   FPGA Audio I2S
// 21   Touch Screen
// 22   SPI ADC
// 23   SPI (Shield 0)
// 24   SPI (Shield 1)
// 25   HDCLCD Interrupt
// 26   GPIO 0 Combined Interrupt

//pub const IRQ_WDG = spi!(0, todo);
pub const IRQ_DUALTIMER1: Irq = Irq::spi(1, Irq::TRIG_EDGE, 10);
pub const IRQ_DUALTIMER2: Irq = Irq::spi(2, Irq::TRIG_EDGE, 10);
pub const IRQ_DUALTIMER_CMB: Irq = Irq::spi(3, Irq::TRIG_EDGE, 10);
//pub const IRQ_RTC: Irq = Irq::spi(4, todo, todo);
pub const IRQ_UART2_RX: Irq = Irq::spi(5, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART2_TX: Irq = Irq::spi(6, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART3_RX: Irq = Irq::spi(7, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART3_TX: Irq = Irq::spi(8, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART4_RX: Irq = Irq::spi(9, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART4_TX: Irq = Irq::spi(10, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART5_RX: Irq = Irq::spi(11, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART5_TX: Irq = Irq::spi(12, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART2_CMB: Irq = Irq::spi(13, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART3_CMB: Irq = Irq::spi(14, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART4_CMB: Irq = Irq::spi(15, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART5_CMB: Irq = Irq::spi(16, Irq::TRIG_LEVEL, 12);
pub const IRQ_UART_OVF: Irq = Irq::spi(17, Irq::TRIG_LEVEL, 12);
//pub const IRQ_ETHERNET  : Irq = Irq::spi(18, todo, todo);
//pub const IRQ_USB: Irq = Irq::spi(19, todo, todo);
//pub const IRQ_FPGA_AUDIO_I2S: Irq = Irq::spi(20, todo, todo);
//pub const IRQ_TOUCH_SCREEN: Irq = Irq::spi(21, todo, todo);
//pub const IRQ_SPI_ADC: Irq = Irq::spi(22, todo, todo);
//pub const IRQ_SPI_SHIELD0: Irq = Irq::spi(23, todo, todo);
//pub const IRQ_SPI_SHIELD1: Irq = Irq::spi(24, todo, todo);
//pub const IRQ_HDCLCD: Irq = Irq::spi(25, todo, todo);
//pub const IRQ_GPIO0_COMB: Irq = Irq::spi(26, todo, todo);
pub const IRQ_UART01_RX: Irq = Irq::ppi(0, Irq::TRIG_LEVEL, 11);
pub const IRQ_UART01_TX: Irq = Irq::ppi(1, Irq::TRIG_LEVEL, 11);
pub const IRQ_UART01_CMB: Irq = Irq::ppi(2, Irq::TRIG_LEVEL, 11);
pub const IRQ_UART01_OVF: Irq = Irq::ppi(3, Irq::TRIG_LEVEL, 11);

impl Chip for XckuChip {
    type MPU = MPU;
    type UserspaceKernelBoundary = SysCall;
    type ThreadIdProvider = cortexr52::thread_id::CortexR52ThreadIdProvider;

    fn init() {
        // Just enable FIQs once setup is done
        support::irq_enable();

        //TODO: set SCTLR.I (I cache) .C (D cache) but done in MPU
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.syscall
    }

    fn service_pending_interrupts(&self) {
        while let Some(intid) = self.gic.next_pending() {
            match intid {
                id if id == cortexr52::gic::IRQ_GENTIMER_PHY.intid() => {
                    self.gentimer.handle_interrupt();
                }
                id if id == IRQ_UART2_TX.intid() => {
                    self.uart2.handle_tx_irq();
                }
                id if id == IRQ_UART2_RX.intid() => {
                    self.uart2.handle_rx_irq();
                }
                //id if id == cortexr52::irq_id!(config::IRQ_DUALTIMER1) => {
                //    self.dualtimer1.handle_interrupt();
                //}
                _ => panic!("unexpected interrupt: {:?}", intid),
            }

            self.gic.complete(intid);
        }

        // FIQs were disabled in handler and must be re-enabled here
        support::irq_enable();
    }

    fn has_pending_interrupts(&self) -> bool {
        self.gic.has_pending()
    }

    fn sleep(&self) {
        let cap = kernel::create_capability!(kernel::capabilities::CpuStateControlCapability);
        support::wfi(&cap);
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, writer: &mut dyn core::fmt::Write) {
        // TODO
        unsafe { support::print_state(writer); }
        //if let Some(_) = this {
        //    let _ = writer.write_fmt(format_args!("{}", t.pmp.pmp));
        //}
    }
}
