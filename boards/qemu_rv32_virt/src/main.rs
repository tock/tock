// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::{create_capability, debug};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

type ScreenDriver = capsules_extra::screen::screen::Screen<'static>;

struct Platform {
    base: qemu_rv32_virt_lib::QemuRv32VirtPlatform,
    screen: Option<&'static ScreenDriver>,
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

            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<qemu_rv32_virt_lib::ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::SyscallFilter;
    type ProcessFault = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::ProcessFault;
    type Scheduler = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::Scheduler;
    type SchedulerTimer = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::SchedulerTimer;
    type WatchDog = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::WatchDog;
    type ContextSwitchCallback = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
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

    let (board_kernel, base_platform, chip) = qemu_rv32_virt_lib::start(None);

    let screen = base_platform.virtio_gpu_screen.map(|screen| {
        components::screen::ScreenComponent::new(
            board_kernel,
            capsules_extra::screen::screen::DRIVER_NUM,
            screen,
            None,
            create_capability!(capabilities::MemoryAllocationCapability),
        )
        .finalize(components::screen_component_static!(1032))
    });

    let platform = Platform {
        base: base_platform,
        screen,
    };

    // Start the process console:
    let _ = platform.base.process_console_start();

    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);

    let app_flash_region = rv32i::support::linker_region_slice_ptr!("_sapps", "_eapps")
        .expect("App flash region is invalid");
    if app_flash_region.len() > isize::MAX as usize {
        // This check is only required because we're creating a Rust slice
        // below, we can remove it once `load_processes` uses raw slice
        // pointers.
        panic!("[_sapps; _eapps) is longer than isize::MAX, can't back a Rust slice!");
    }
    let app_ram_region = rv32i::support::linker_region_slice_ptr!("_sappmem", "_eappmem")
        .expect("App RAM region is invalid");
    if app_ram_region.len() > isize::MAX as usize {
        // This check is only required because we're creating a Rust slice
        // below, we can remove it once `load_processes` uses raw slice
        // pointers.
        panic!("[_sappmem; _eappmem) is longer than isize::MAX, can't back a Rust slice!");
    }

    kernel::process::load_processes(
        board_kernel,
        chip,
        // This may well be unsound on many platforms: apps can initiate writes to
        // their own flash while a shared slice reference derived from this slice is
        // stored in `ProcessStandard`.
        //
        // SAFETY: TODO. This is unsound, in the general case, for at least some
        // of our boards.
        unsafe { &*app_flash_region },
        // We should similarly avoid creating a temporary exclusive slice
        // reference over the application's RAM; while `ProcessStandard` only
        // stores raw slice pointers (`*mut [u8]`) it is hard to rule out an
        // instant where an app or the kernel can modify its RAM while a
        // concurrent, other exclusive slice reference exists.
        //
        // SAFETY: TODO. This is unsound, in the general case, for at least some
        // of our boards.
        unsafe { &mut *app_ram_region },
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
