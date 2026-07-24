// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board file for NXP S32G3 SAIL.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::create_capability;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
struct Platform {
    base: nxp_s32g3_sail_lib::NxpS32g3SailPlatform,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        self.base.with_driver(driver_num, f)
    }
}

impl KernelResources<nxp_s32g3_sail_lib::ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <nxp_s32g3_sail_lib::NxpS32g3SailPlatform as KernelResources<
        nxp_s32g3_sail_lib::ChipHw,
    >>::SyscallFilter;
    type ProcessFault = <nxp_s32g3_sail_lib::NxpS32g3SailPlatform as KernelResources<
        nxp_s32g3_sail_lib::ChipHw,
    >>::ProcessFault;
    type Scheduler = <nxp_s32g3_sail_lib::NxpS32g3SailPlatform as KernelResources<
        nxp_s32g3_sail_lib::ChipHw,
    >>::Scheduler;
    type SchedulerTimer = <nxp_s32g3_sail_lib::NxpS32g3SailPlatform as KernelResources<
        nxp_s32g3_sail_lib::ChipHw,
    >>::SchedulerTimer;
    type WatchDog = <nxp_s32g3_sail_lib::NxpS32g3SailPlatform as KernelResources<
        nxp_s32g3_sail_lib::ChipHw,
    >>::WatchDog;
    type ContextSwitchCallback = <nxp_s32g3_sail_lib::NxpS32g3SailPlatform as KernelResources<
        nxp_s32g3_sail_lib::ChipHw,
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

#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, base_platform, chip) = nxp_s32g3_sail_lib::start();

    let platform = Platform {
        base: base_platform,
    };
    let _ = platform.base.pconsole.start();
    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
