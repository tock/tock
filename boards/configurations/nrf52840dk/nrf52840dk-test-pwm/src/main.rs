// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! This configuration exercises the PWM stack using all four PWM hardware
//! elements.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::static_init;
use kernel::utilities::StaticRef;
use nrf52840::pwm::{PwmRegisters, PwmRegistersManager};

mod pwm_pin;

/// PWM1 Hardware Registers
///
/// # Safety
///
/// This is the correct location of a PWM instance:
/// <https://docs.nordicsemi.com/r/bundle/ps_nrf52840/page/pwm.html#d1052e1269>.
const PWM1_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x40021000 as *const PwmRegisters) };

/// PWM2 Hardware Registers
///
/// # Safety
///
/// This is the correct location of a PWM instance:
/// <https://docs.nordicsemi.com/r/bundle/ps_nrf52840/page/pwm.html#d1052e1269>.
const PWM2_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x40022000 as *const PwmRegisters) };

/// PWM3 Hardware Registers
///
/// # Safety
///
/// This is the correct location of a PWM instance:
/// <https://docs.nordicsemi.com/r/bundle/ps_nrf52840/page/pwm.html#d1052e1269>.
const PWM3_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x4002D000 as *const PwmRegisters) };

type ChipHw = nrf52840dk_test_base_lib::ChipHw;

type PwmHw = nrf52840::pwm::Pwm;
type PwmDriver = components::pwm::PwmDriverComponentType<4>;

/// Supported drivers by the platform
pub struct Platform {
    base: nrf52840dk_test_base_lib::Platform,
    pwm: &'static PwmDriver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            // The LED pins are being driven as PWM outputs, so intercept the
            // LED driver and always return an error.
            capsules_core::led::DRIVER_NUM => f(None),
            capsules_extra::pwm::DRIVER_NUM => f(Some(self.pwm)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::SyscallFilter;
    type ProcessFault =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::ProcessFault;
    type Scheduler = <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::Scheduler;
    type SchedulerTimer =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        self.base.syscall_filter()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        self.base.process_fault()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.base.scheduler()
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.base.scheduler_timer()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        self.base.watchdog()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        self.base.context_switch_callback()
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let (board_kernel, base_platform, chip, nrf52840_peripherals, _mux_uart, _mux_alarm) =
        nrf52840dk_test_base_lib::start();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    //--------------------------------------------------------------------------
    // PWM
    //--------------------------------------------------------------------------

    // PWM0 is already set up as part of the shared chip peripherals.
    let pwm0 = &base_peripherals.pwm0;

    // Instantiate PWM1-PWM3 directly, each with its own duty-cycle buffer.
    let pwm1_dutycycle_buf = static_init!([u16; 4], [0; 4]);
    let pwm1_registers = PwmRegistersManager::new(PWM1_BASE, pwm1_dutycycle_buf);
    let pwm1 = static_init!(PwmHw, PwmHw::new(pwm1_registers));

    let pwm2_dutycycle_buf = static_init!([u16; 4], [0; 4]);
    let pwm2_registers = PwmRegistersManager::new(PWM2_BASE, pwm2_dutycycle_buf);
    let pwm2 = static_init!(PwmHw, PwmHw::new(pwm2_registers));

    let pwm3_dutycycle_buf = static_init!([u16; 4], [0; 4]);
    let pwm3_registers = PwmRegistersManager::new(PWM3_BASE, pwm3_dutycycle_buf);
    let pwm3 = static_init!(PwmHw, PwmHw::new(pwm3_registers));

    // Each LED pin gets its own, dedicated PWM hardware peripheral rather
    // than sharing one peripheral through a virtualizer.
    let pwm_pin1 = static_init!(
        pwm_pin::PwmPinStatic<'static, PwmHw>,
        pwm_pin::PwmPinStatic::new(
            pwm0,
            nrf52840::pinmux::Pinmux::new(nrf52840dk_test_base_lib::LED1_PIN)
        )
    );
    let pwm_pin2 = static_init!(
        pwm_pin::PwmPinStatic<'static, PwmHw>,
        pwm_pin::PwmPinStatic::new(
            pwm1,
            nrf52840::pinmux::Pinmux::new(nrf52840dk_test_base_lib::LED2_PIN)
        )
    );
    let pwm_pin3 = static_init!(
        pwm_pin::PwmPinStatic<'static, PwmHw>,
        pwm_pin::PwmPinStatic::new(
            pwm2,
            nrf52840::pinmux::Pinmux::new(nrf52840dk_test_base_lib::LED3_PIN)
        )
    );
    let pwm_pin4 = static_init!(
        pwm_pin::PwmPinStatic<'static, PwmHw>,
        pwm_pin::PwmPinStatic::new(
            pwm3,
            nrf52840::pinmux::Pinmux::new(nrf52840dk_test_base_lib::LED4_PIN)
        )
    );

    let pwm = components::pwm::PwmDriverComponent::new(
        board_kernel,
        capsules_extra::pwm::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::pwm_driver_component_helper!(
        pwm_pin1, pwm_pin2, pwm_pin3, pwm_pin4,
    ));

    //--------------------------------------------------------------------------
    // PLATFORM
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        pwm,
    };

    //--------------------------------------------------------------------------
    // PROCESS LOADING
    //--------------------------------------------------------------------------

    // These symbols are defined in the standard Tock linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let app_flash = core::slice::from_raw_parts(
        core::ptr::addr_of!(_sapps),
        core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
    );
    let app_memory = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    kernel::process::load_processes(
        board_kernel,
        chip,
        app_flash,
        app_memory,
        &nrf52840dk_test_base_lib::FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        kernel::debug!("Error loading processes!");
        kernel::debug!("{:?}", err);
    });

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let main_loop_capability = create_capability!(kernel::capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
