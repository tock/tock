// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! High-level chip setup and interrupt dispatch for the MPS3-AN536.
//!
//! This module implements [`kernel::platform::chip::Chip`] for the
//! Cortex-R52-based AN536 subsystem.

use crate::{an536_inst, uart};
use cortexr52::gic;
use kernel::platform::chip::Chip;

/// The MPS3-AN536 chip.
pub struct XckuChip {
    mpu: cortexr52::mpu::MPU,
    syscall: cortexr52::syscall::SysCall,
    pub gic: gic::Gic,
    pub uart2: uart::ApbUart<'static>,
    pub gentimer: an536_inst::GenTimer<'static>,
    pub dualtimer1: an536_inst::DualTimer<'static>, //unused for now
                                                    //pub dualtimer2: dualtimer::DualTimer<'static>,
}

impl XckuChip {
    /// Create a new `XckuChip`.
    pub unsafe fn new() -> Self {
        Self {
            syscall: cortexr52::syscall::SysCall {},
            mpu: cortexr52::mpu::MPU {},
            gic: gic::Gic::new(an536_inst::BASE_GICD, an536_inst::BASE_GICR),
            uart2: uart::ApbUart::new(an536_inst::BASE_UART2, an536_inst::FREQ_ACLK),
            gentimer: cortexr52::gentimer::GenTimer::new(),
            dualtimer1: an536_inst::DualTimer::new(an536_inst::BASE_DUALTIMER1),
            //dualtimer2: dualtimer::DualTimer::new(an536_inst::BASE_DUALTIMER2),
        }
    }
}

impl Chip for XckuChip {
    type MPU = cortexr52::mpu::MPU;
    type UserspaceKernelBoundary = cortexr52::syscall::SysCall;
    type ThreadIdProvider = cortexr52::support::CortexR52ThreadIdProvider;

    fn init() {
        // Just enable FIQs once setup is done
        unsafe {
            cortexr52::support::irq_enable();
        }
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
                id if id == gic::IRQ_GENTIMER_PHY.id => {
                    self.gentimer.handle_interrupt();
                }
                id if id == an536_inst::IRQ_UART2_TX.id => {
                    self.uart2.handle_tx_irq();
                }
                id if id == an536_inst::IRQ_UART2_RX.id => {
                    // todo!
                    self.uart2.handle_rx_irq();
                }
                //id if id == an536_inst::IRQ_DUALTIMER1.id => {
                //    self.dualtimer1.handle_interrupt();
                //}
                _ => panic!("unexpected interrupt: {:?}", intid),
            }

            self.gic.complete(intid);
        }

        // FIQs were disabled in handler and must be re-enabled here
        unsafe {
            cortexr52::support::irq_enable();
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        self.gic.has_pending()
    }

    fn sleep(&self) {
        cortexr52::support::wfi();
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexr52::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(this: Option<&Self>, writer: &mut dyn core::fmt::Write) {
        // TODO
        cortexr52::support::print_state(writer);
        if let Some(_) = this {
            //let _ = writer.write_fmt(format_args!("{}", t.pmp.pmp));
        }
    }
}
