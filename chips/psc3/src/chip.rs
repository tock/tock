// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Chip trait setup.

use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::cpuss;
use crate::cpuss_ppu;
use crate::gpio;
use crate::hsiom;
use crate::interrupts;
use crate::peri;
use crate::peri_clk;
use crate::pwrmode;
use crate::ramc_ppu;
use crate::scb;
use crate::srss;
use crate::tcpwm;
use cortexm33::{CortexM33, CortexMVariant};

pub struct Psc3<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService> Psc3<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm33::mpu::new(),
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<I: InterruptService> Chip for Psc3<'_, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm33::nvic::next_pending() {
                // ignore PROC1_IRQ_CTI as it is intended for processor 1
                // not able to unset its pending status
                // probably only processor 1 can unset the pending by reading the fifo
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }
                let n = cortexm33::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm33::nvic::has_pending() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm33::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, writer: &mut dyn Write) {
        CortexM33::print_cortexm_state(writer);
    }
}

pub struct Psc3DefaultPeripherals<'a> {
    pub cpuss: cpuss::Cpuss,
    pub gpio: gpio::PsocPins<'a>,
    pub hsiom: hsiom::Hsiom,
    pub scb3: scb::Scb<'a>,
    pub tcpwm: tcpwm::Tcpwm0<'a>,
    peri: peri::Peri,
    peri_clk: peri_clk::PeriPClk,
    pwrmode: pwrmode::PwrMode,
    srss: srss::Srss,
    cpuss_ppu: cpuss_ppu::CpussPpu,
    ramc_ppu: ramc_ppu::RamcPpu,
}

impl<'a> Psc3DefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            cpuss: cpuss::Cpuss::new(),
            hsiom: hsiom::Hsiom::new(),
            peri: peri::Peri::new(),
            scb3: scb::Scb::new(),
            peri_clk: peri_clk::PeriPClk::new(),
            srss: srss::Srss::new(),
            pwrmode: pwrmode::PwrMode::new(),
            tcpwm: tcpwm::Tcpwm0::new(),
            cpuss_ppu: cpuss_ppu::CpussPpu::new(),
            gpio: gpio::PsocPins::new(),
            ramc_ppu: ramc_ppu::RamcPpu::new(),
        }
    }

    pub fn sys_init(&self) {
        self.srss.sys_init_enable_clocks();
        self.peri.sys_init_enable_peri();
    }

    fn init_pwr(&self) {
        self.pwrmode.ppu_init();
        self.cpuss_ppu.init_ppu();
        self.ramc_ppu.init_ppu();
        // TODO
        // (void)Cy_SysPm_SetDeepSleepMode(CY_SYSPM_MODE_DEEPSLEEP);

        // Voltage during debugging was always right and it is unclear how to set the voltage.
        // Cy_SysPm_SystemEnterOd();
    }

    fn init_system(&self) {
        // TODOs:
        // /* Set worst case memory wait states (! ultra low power, 180 MHz), will update at the end */
        // Cy_SysLib_SetWaitStates(false, 180UL);

        /* Unlock WDT to be able to modify LFCLK registers */
        self.srss.wdt_unlock();

        self.init_pwr();

        self.srss.disable_fll();
        self.srss.enable_iho();

        self.srss.init_clock_paths();

        self.srss.init_dpll_lp();

        self.srss.init_clk_hf();
        self.srss.init_clk_path0();

        self.srss.init_fll();
        self.srss.init_clk_hf0();

        // TODO
        // Cy_SysLib_SetWaitStates(CY_CFG_PWR_USING_ULP != 0, CY_CFG_SYSCLK_CLKHF0_FREQ_MHZ);
    }

    pub fn init(&self) {
        self.init_system();

        // TODO: sets warm boot entry
        // result = cybsp_syspm_dsram_init();

        // self.srss.init_clock();
        self.peri_clk.init_clocks();
        self.peri_clk.init_peripherals();

        self.hsiom
            .set_port_sel(6, 2, hsiom::HsiomFunction::ActiveFunctionality6);
        let uart_rx_pin = self.gpio.get_pin(gpio::PsocPin::P6_2);
        uart_rx_pin.configure_drive_mode(gpio::DriveMode::HighZ);
        uart_rx_pin.configure_input(true);
        let uart_tx_pin = self.gpio.get_pin(gpio::PsocPin::P6_3);
        uart_tx_pin.configure_drive_mode(gpio::DriveMode::Strong);
        uart_tx_pin.configure_input(false);

        self.scb3.set_standard_uart_mode();
        self.scb3.enable_scb();
    }
}

impl<'a> InterruptService for Psc3DefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::TCPWM_0_INTERRUPTS_0 => {
                self.tcpwm.handle_interrupt();
                true
            }
            interrupts::SCB_5_INTERRUPT => {
                self.scb3.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}
