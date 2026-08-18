// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::debug;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability};

// =============================================================================
// SHA Implementation Configuration
// =============================================================================

// --- Option A: Software SHA256 Capsule (Active by default) ---
// type Sha = capsules_extra::sha256_driver::ShaDriver<
//     'static,
//     capsules_extra::sha256::Sha256Software<'static>,
//     32,
// >;

// --- Option B: Userspace Service SHA256 (Disabled by default) ---
type Sha = capsules_extra::sha256_driver::ShaDriver<
    'static,
    capsules_system::userspace_services::services::digest::ServiceInterface<32>,
    32,
>;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

struct Platform {
    base: nrf52840dk_lib::Platform,
    eui64_driver: &'static nrf52840dk_lib::Eui64Driver,
    ieee802154_driver: &'static nrf52840dk_lib::Ieee802154Driver,
    udp_driver: &'static capsules_extra::net::udp::UDPDriver<'static>,
    digest: &'static Sha,
    // Uncomment if Option B is active:
    userspace_services: &'static capsules_system::userspace_services::registry::Registry<2>,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::eui64::DRIVER_NUM => f(Some(self.eui64_driver)),
            capsules_extra::net::udp::DRIVER_NUM => f(Some(self.udp_driver)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_driver)),
            capsules_extra::sha256_driver::DRIVER_NUM => f(Some(self.digest)),

            // Uncomment if Option B is active:
            capsules_system::userspace_services::registry::DRIVER_NUM => {
                f(Some(self.userspace_services))
            }
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

type ChipHw = nrf52840dk_lib::ChipHw;

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::SyscallFilter;
    type ProcessFault = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::ProcessFault;
    type Scheduler = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::Scheduler;
    type SchedulerTimer = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::ContextSwitchCallback;

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
    let (board_kernel, base_platform, chip, default_peripherals, mux_alarm) =
        nrf52840dk_lib::start();

    //--------------------------------------------------------------------------
    // IEEE 802.15.4 and UDP
    //--------------------------------------------------------------------------

    let (eui64_driver, ieee802154_driver, udp_driver) =
        nrf52840dk_lib::ieee802154_udp(board_kernel, default_peripherals, mux_alarm);

    //--------------------------------------------------------------------------
    // Userspace services
    //--------------------------------------------------------------------------

    // === Option A: Software SHA256 (Active by default) ===
    // let sha_software = kernel::static_init!(
    //     capsules_extra::sha256::Sha256Software<'static>,
    //     capsules_extra::sha256::Sha256Software::new()
    // );
    // kernel::deferred_call::DeferredCallClient::register(sha_software);

    // let sha256_driver = kernel::static_init!(
    //     Sha,
    //     Sha::new(
    //         sha_software,
    //         kernel::static_init!([u8; 128], [0; 128]),
    //         kernel::static_init!([u8; 32], [0; 32]),
    //         board_kernel.create_grant(
    //             capsules_extra::sha256_driver::DRIVER_NUM,
    //             &create_capability!(capabilities::MemoryAllocationCapability)
    //         )
    //     )
    // );
    // use kernel::hil::digest::DigestDataHash;
    // sha_software.set_client(sha256_driver);

    // === Option B: Userspace Service SHA256 (Disabled by default) ===
    // Registry capsule for communicating with userspace service applications.
    let userspace_services = kernel::static_init!(
        capsules_system::userspace_services::registry::Registry<2>,
        capsules_system::userspace_services::registry::Registry::new(board_kernel.create_grant(
            capsules_system::userspace_services::registry::DRIVER_NUM,
            &create_capability!(capabilities::MemoryAllocationCapability)
        ))
    );

    // Hashing service interface to translate from HIL call to userspace service application usercall.
    let hashing_service_interface = kernel::static_init!(
        capsules_system::userspace_services::services::digest::ServiceInterface<32>,
        capsules_system::userspace_services::services::digest::ServiceInterface::new(
            userspace_services
        )
    );
    hashing_service_interface.init();

    let sha256_driver = kernel::static_init!(
        Sha,
        Sha::new(
            hashing_service_interface,
            kernel::static_init!([u8; 128], [0; 128]),
            kernel::static_init!([u8; 32], [0; 32]),
            board_kernel.create_grant(
                capsules_extra::sha256_driver::DRIVER_NUM,
                &create_capability!(capabilities::MemoryAllocationCapability)
            )
        )
    );
    use kernel::hil::digest::DigestDataHash;
    hashing_service_interface.set_client(sha256_driver);

    let platform = Platform {
        base: base_platform,
        eui64_driver,
        ieee802154_driver,
        udp_driver,
        digest: sha256_driver,
        // Uncomment if Option B is active:
        userspace_services,
    };

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
    }

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
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
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
