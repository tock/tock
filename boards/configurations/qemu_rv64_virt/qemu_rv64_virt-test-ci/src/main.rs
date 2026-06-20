// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv64 "virt" machine type

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::{create_capability, debug, static_init};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

type ScreenDriver = capsules_extra::screen::screen::Screen<'static>;
type ScreenAdapter = capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<
    'static,
    qemu_rv64_virt_lib::ScreenHw,
>;
type ScreenSplitUser = components::screen::ScreenSplitUserComponentType<ScreenAdapter>;
type ScreenOnLed = components::screen_on::ScreenOnLedComponentType<ScreenSplitUser, 4, 128, 64>;
type ScreenOnLedSingle =
    capsules_extra::screen::screen_on_led::ScreenOnLedSingle<'static, ScreenOnLed>;

type LedDriver = capsules_core::led::LedDriver<'static, ScreenOnLedSingle, 4>;

type ButtonDriver = capsules_extra::button_keyboard::ButtonKeyboard<'static>;

struct Platform {
    base: qemu_rv64_virt_lib::QemuRv64VirtPlatform,
    screen: Option<&'static ScreenDriver>,
    led: Option<&'static LedDriver>,
    buttons: Option<&'static ButtonDriver>,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::screen::screen::DRIVER_NUM => {
                if let Some(screen_driver) = self.screen {
                    f(Some(screen_driver))
                } else {
                    f(None)
                }
            }
            capsules_core::led::DRIVER_NUM => {
                if let Some(led_driver) = self.led {
                    f(Some(led_driver))
                } else {
                    f(None)
                }
            }
            capsules_core::button::DRIVER_NUM => {
                if let Some(button_driver) = self.buttons {
                    f(Some(button_driver))
                } else {
                    f(None)
                }
            }

            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<qemu_rv64_virt_lib::ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <qemu_rv64_virt_lib::QemuRv64VirtPlatform as KernelResources<
        qemu_rv64_virt_lib::ChipHw,
    >>::SyscallFilter;
    type ProcessFault = <qemu_rv64_virt_lib::QemuRv64VirtPlatform as KernelResources<
        qemu_rv64_virt_lib::ChipHw,
    >>::ProcessFault;
    type Scheduler = <qemu_rv64_virt_lib::QemuRv64VirtPlatform as KernelResources<
        qemu_rv64_virt_lib::ChipHw,
    >>::Scheduler;
    type SchedulerTimer = <qemu_rv64_virt_lib::QemuRv64VirtPlatform as KernelResources<
        qemu_rv64_virt_lib::ChipHw,
    >>::SchedulerTimer;
    type WatchDog = <qemu_rv64_virt_lib::QemuRv64VirtPlatform as KernelResources<
        qemu_rv64_virt_lib::ChipHw,
    >>::WatchDog;
    type ContextSwitchCallback = <qemu_rv64_virt_lib::QemuRv64VirtPlatform as KernelResources<
        qemu_rv64_virt_lib::ChipHw,
    >>::ContextSwitchCallback;

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
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, base_platform, chip) = qemu_rv64_virt_lib::start();

    //--------------------------------------------------------------------------
    // SCREEN
    //--------------------------------------------------------------------------

    let (screen, led) = base_platform
        .virtio_gpu_screen
        .map_or((None, None), |screen| {
            let screen_split = components::screen::ScreenSplitMuxComponent::new(screen).finalize(
                components::screen_split_mux_component_static!(ScreenAdapter),
            );

            let screen_split_userspace =
                components::screen::ScreenSplitUserComponent::new(screen_split, 0, 0, 128, 64)
                    .finalize(components::screen_split_user_component_static!(
                        ScreenAdapter
                    ));

            let screen_split_kernel =
                components::screen::ScreenSplitUserComponent::new(screen_split, 0, 64, 128, 64)
                    .finalize(components::screen_split_user_component_static!(
                        ScreenAdapter
                    ));

            let screen = components::screen::ScreenComponent::new(
                board_kernel,
                capsules_extra::screen::screen::DRIVER_NUM,
                screen_split_userspace,
                None,
            )
            .finalize(components::screen_component_static!(1032));

            let screen_on_leds =
                components::screen_on::ScreenOnLedComponent::new(screen_split_kernel).finalize(
                    components::screen_on_led_component_static!(ScreenSplitUser, 4, 128, 64),
                );

            let led =
                components::led::LedsComponent::new().finalize(components::led_component_static!(
                    ScreenOnLedSingle,
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        0
                    ),
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        1
                    ),
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        2
                    ),
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        3
                    ),
                ));

            (Some(screen), Some(led))
        });

    //--------------------------------------------------------------------------
    // SIMULATED BUTTONS USING KEYBOARD
    //--------------------------------------------------------------------------

    let buttons = base_platform.virtio_input_keyboard.map(|keyboard| {
        let key_mappings = static_init!(
            [u16; 4],
            [
                103, // UP
                14,  // BACKSPACE
                108, // DOWN
                28,  // ENTER
            ]
        );

        components::button_keyboard::KeyboardButtonComponent::new(
            board_kernel,
            capsules_extra::button_keyboard::DRIVER_NUM,
            keyboard,
            key_mappings,
        )
        .finalize(components::keyboard_button_component_static!())
    });

    let platform = Platform {
        base: base_platform,
        screen,
        led,
        buttons,
    };

    // Start the process console:
    let _ = platform.base.pconsole.start();

    // These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
        /// The start of the kernel text (Included only for kernel PMP)
        static _stext: u8;
        /// The end of the kernel text (Included only for kernel PMP)
        static _etext: u8;
        /// The start of the kernel / app / storage flash (Included only for kernel PMP)
        static _sflash: u8;
        /// The end of the kernel / app / storage flash (Included only for kernel PMP)
        static _eflash: u8;
        /// The start of the kernel / app RAM (Included only for kernel PMP)
        static _ssram: u8;
        /// The end of the kernel / app RAM (Included only for kernel PMP)
        static _esram: u8;
    }
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    debug!("Entering main loop.");

    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
