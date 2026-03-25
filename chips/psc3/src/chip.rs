// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Chip trait setup.

use core::fmt::Write;
use kernel::hil::gpio::Configure;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::cpuss_ppu;
use crate::flashc;
use crate::gpio;
use crate::hsiom_registers;
use crate::interrupts;
use crate::peri;
use crate::peri_clk;
use crate::pwrmode;
use crate::ramc_ppu;
use crate::scb;
use crate::srss;
use crate::tcpwm;
use cortexm33::{CortexM33, CortexMVariant};

const GPIO_SWDCK_CONFIG: gpio::PreConfig = gpio::PreConfig {
    out_val: 1,
    drive_mode: gpio::DriveMode::PullDown,
    hsiom: hsiom_registers::HsiomFunction::DeepSleepFunctionality5,
    int_edge: false,
    int_mask: 0,
    vtrip: 0,
    fast_slew_rate: true,
    drive_sel: gpio::DriveSelect::Half,
    vreg_en: false,
    ibuf_mode: 0,
    vtrip_sel: 0,
    vref_sel: 0,
    voh_sel: 0,
    non_sec: false,
};

const GPIO_SWDIO_CONFIG: gpio::PreConfig = gpio::PreConfig {
    out_val: 1,
    drive_mode: gpio::DriveMode::PullUp,
    hsiom: gpio::HsiomFunction::DeepSleepFunctionality5,
    int_edge: false,
    int_mask: 0,
    vtrip: 0,
    fast_slew_rate: true,
    drive_sel: gpio::DriveSelect::Half,
    vreg_en: false,
    ibuf_mode: 0,
    vtrip_sel: 0,
    vref_sel: 0,
    voh_sel: 0,
    non_sec: false,
};

pub const GPIO_DEBUG_UART_RX_CONFIG: gpio::PreConfig = gpio::PreConfig {
    out_val: 1,
    drive_mode: gpio::DriveMode::HighZ,
    hsiom: gpio::HsiomFunction::ActiveFunctionality4,
    int_edge: false,
    int_mask: 0,
    vtrip: 0,
    fast_slew_rate: true,
    drive_sel: gpio::DriveSelect::Half,
    vreg_en: false,
    ibuf_mode: 0,
    vtrip_sel: 0,
    vref_sel: 0,
    voh_sel: 0,
    non_sec: false,
};

pub const GPIO_DEBUG_UART_TX_CONFIG: gpio::PreConfig = gpio::PreConfig {
    out_val: 1,
    drive_mode: gpio::DriveMode::Strong,
    hsiom: gpio::HsiomFunction::ActiveFunctionality4,
    int_edge: false,
    int_mask: 0,
    vtrip: 0,
    fast_slew_rate: true,
    drive_sel: gpio::DriveSelect::Half,
    vreg_en: false,
    ibuf_mode: 0,
    vtrip_sel: 0,
    vref_sel: 0,
    voh_sel: 0,
    non_sec: false,
};

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
    pub gpio: gpio::PsocPins<'a>,
    pub scb3: scb::Scb<'a>,
    pub tcpwm: tcpwm::Tcpwm0<'a>,
    peri: peri::Peri,
    peri_clk: peri_clk::PeriPClk,
    pwrmode: pwrmode::PwrMode,
    srss: srss::Srss,
    cpuss_ppu: cpuss_ppu::CpussPpu,
    ramc_ppu: ramc_ppu::RamcPpu,
    flashc: flashc::FlashC,
}

impl<'a> Psc3DefaultPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            peri: peri::Peri::new(),
            scb3: scb::Scb::new(),
            peri_clk: peri_clk::PeriPClk::new(),
            srss: srss::Srss::new(),
            pwrmode: pwrmode::PwrMode::new(),
            tcpwm: tcpwm::Tcpwm0::new(),
            cpuss_ppu: cpuss_ppu::CpussPpu::new(),
            gpio: gpio::PsocPins::new(),
            ramc_ppu: ramc_ppu::RamcPpu::new(),
            flashc: flashc::FlashC::new(),
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

        /* Set Default mode to DEEPSLEEP */
        self.pwrmode
            .ppu_dynamic_enable(pwrmode::PwrPolicy::FullRetention);
        self.cpuss_ppu
            .ppu_dynamic_enable(cpuss_ppu::PwrPolicy::FullRetention);
        self.ramc_ppu
            .ppu_dynamic_enable(ramc_ppu::PwrPolicy::MemoryRetention);

        // Voltage during debugging was always right and it is unclear how to set the voltage.
        // Cy_SysPm_SystemEnterOd();
    }

    fn init_system(&self) {
        self.flashc.set_waitstates(false, 180);

        /* Unlock WDT to be able to modify LFCLK registers */
        self.srss.wdt_unlock();

        self.init_pwr();

        self.srss.disable_fll();
        self.srss.enable_iho();

        self.srss.init_clock_paths();

        self.srss.init_dpll_lp().unwrap();

        self.srss.init_clk_hf();
        self.srss.init_clk_path0();

        self.srss.init_fll().unwrap();
        self.srss.init_clk_hf0();
    }

    fn init_gpio_pins(&self) {
        let swdck_pin = self.gpio.get_pin(gpio::PsocPin::P1_2);
        swdck_pin.preconfigure(&GPIO_SWDCK_CONFIG);
        let swdio_pin = self.gpio.get_pin(gpio::PsocPin::P1_3);
        swdio_pin.preconfigure(&GPIO_SWDIO_CONFIG);
        let uart_rx_pin = self.gpio.get_pin(gpio::PsocPin::P6_2);
        uart_rx_pin.preconfigure(&GPIO_DEBUG_UART_RX_CONFIG);
        uart_rx_pin.make_input();
        let uart_tx_pin = self.gpio.get_pin(gpio::PsocPin::P6_3);
        uart_tx_pin.preconfigure(&GPIO_DEBUG_UART_TX_CONFIG);
    }

    pub fn init(&self) {
        self.init_system();

        self.peri_clk.init_clocks();
        self.peri_clk.init_peripherals();
        self.init_gpio_pins();

        self.scb3.set_standard_uart_mode();
        self.scb3.enable_scb();

        self.tcpwm.init_timer();
    }
}

impl<'a> InterruptService for Psc3DefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::TCPWM_0_INTERRUPTS_0 => {
                self.tcpwm.handle_interrupt();
                true
            }
            interrupts::SCB_3_INTERRUPT => {
                self.scb3.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}
