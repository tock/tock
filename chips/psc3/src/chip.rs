// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! Chip trait setup and default peripheral initialization.

use core::fmt::Write;
use kernel::hil::gpio::Configure;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::chip_init;
use crate::gpio;
use crate::hsiom_registers;
use crate::icache;
use crate::interrupts;
use crate::peri_clk;
use crate::scb;
use crate::tcpwm;
use cortexm33::{CortexM33, CortexMVariant};

// Configuration generated in MTB for SWD and Debug UART pins.
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
    non_sec: true,
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
    non_sec: true,
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
    // TODO why does this pin need to be non-secure for interrupts to work?
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
    non_sec: true,
};
pub const GPIO_SEC_DEBUG_UART_RX_CONFIG: gpio::PreConfig = gpio::PreConfig {
    out_val: 1,
    drive_mode: gpio::DriveMode::HighZ,
    hsiom: gpio::HsiomFunction::DeepSleepFunctionality2,
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
    // TODO why does this pin need to be non-secure for interrupts to work?
    non_sec: false,
};
pub const GPIO_SEC_DEBUG_UART_TX_CONFIG: gpio::PreConfig = gpio::PreConfig {
    out_val: 1,
    drive_mode: gpio::DriveMode::Strong,
    hsiom: gpio::HsiomFunction::DeepSleepFunctionality2,
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
    non_sec: true,
};

pub fn init_gpio_pins() {
    let gpio = gpio::PsocPins::new(true);

    let swdck_pin = gpio.get_pin(gpio::PsocPin::P1_2);
    swdck_pin.preconfigure(&GPIO_SWDCK_CONFIG);
    let swdio_pin = gpio.get_pin(gpio::PsocPin::P1_3);
    swdio_pin.preconfigure(&GPIO_SWDIO_CONFIG);
    let uart_rx_pin = gpio.get_pin(gpio::PsocPin::P6_2);
    uart_rx_pin.preconfigure(&GPIO_DEBUG_UART_RX_CONFIG);
    uart_rx_pin.make_input();
    let uart_tx_pin = gpio.get_pin(gpio::PsocPin::P6_3);
    uart_tx_pin.preconfigure(&GPIO_DEBUG_UART_TX_CONFIG);

    let sec_uart_rx_pin = gpio.get_pin(gpio::PsocPin::P9_2);
    sec_uart_rx_pin.preconfigure(&GPIO_SEC_DEBUG_UART_RX_CONFIG);
    sec_uart_rx_pin.make_input();
    let secu_uart_tx_pin = gpio.get_pin(gpio::PsocPin::P9_3);
    secu_uart_tx_pin.preconfigure(&GPIO_SEC_DEBUG_UART_TX_CONFIG);
}

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
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }
                let n = cortexm33::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn init() {
        icache::sys_init_enable_cache();
        cortexm33::nvic::disable_all();
        cortexm33::nvic::clear_all_pending();
        cortexm33::nvic::enable_all();
    }

    fn has_pending_interrupts(&self) -> bool {
        cortexm33::nvic::has_pending()
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
}

impl Psc3DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            scb3: scb::Scb::new_scb3(),
            tcpwm: tcpwm::Tcpwm0::new(),
            gpio: gpio::PsocPins::new(false),
        }
    }

    /// Initialize GPIO pins for SWD and Debug UART.
    // fn init_gpio_pins(&self) {
    //     return;
    //     let swdck_pin = self.gpio.get_pin(gpio::PsocPin::P1_2);
    //     swdck_pin.preconfigure(&GPIO_SWDCK_CONFIG);
    //     let swdio_pin = self.gpio.get_pin(gpio::PsocPin::P1_3);
    //     swdio_pin.preconfigure(&GPIO_SWDIO_CONFIG);
    //     let uart_rx_pin = self.gpio.get_pin(gpio::PsocPin::P6_2);
    //     uart_rx_pin.preconfigure(&GPIO_DEBUG_UART_RX_CONFIG);
    //     uart_rx_pin.make_input();
    //     let uart_tx_pin = self.gpio.get_pin(gpio::PsocPin::P6_3);
    //     uart_tx_pin.preconfigure(&GPIO_DEBUG_UART_TX_CONFIG);
    // }

    /// Initialize all peripherals.
    pub fn init(&self) {
        chip_init::init_system();

        // Route clk to scb and tcpwm
        peri_clk::enable_scb3();
        peri_clk::enable_tcpwm0();

        // TODO split non-secure
        // self.init_gpio_pins();

        self.scb3.set_standard_uart_mode();
        self.scb3.enable_scb();

        self.tcpwm.init_timer();
    }
}

impl InterruptService for Psc3DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        // handle all GPIO interrupts
        if interrupt <= interrupts::IOSS_INTERRUPT_SEC_GPIO {
            self.gpio.handle_interrupt();
            return true;
        }
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
