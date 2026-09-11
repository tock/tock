// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the QEMU ARM MPS2 AN386 (Cortex-M4) machine, configured to
//! test isolated nonvolatile storage.
//!
//! The AN386 has no real flash controller, so nonvolatile storage is backed
//! by RAM ([`ram_nonvolatile_storage`]) and does not survive a reset. Apps
//! are assigned storage permissions individually, so each app can only read
//! and write its own storage region.
//!
//! This configuration also exposes a second `Console`, on UART1, at driver
//! number `0x01000001` -- see the "SECOND CONSOLE (UART1)" section in
//! `main()` and the family Makefile for how it's bridged to a host serial
//! port under QEMU.
//!
//! It also copies the dynamic process loading stack from
//! `tutorials/nrf52840dk-dynamic-apps-and-policies`, letting userspace load
//! new apps at runtime through `capsules_extra::app_loader`. Its backing
//! store is a second [`ram_nonvolatile_storage`] instance over the same
//! `_sapps`..`_eapps` region `app_flash` is read from -- see the "DYNAMIC
//! PROCESS LOADING" section in `main()`.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::debug::PanicResources;
use kernel::deferred_call::DeferredCallClient;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::static_init;
use kernel::utilities::single_thread_value::SingleThreadValue;

pub mod io;
mod peripherals;
mod ram_dynamic_binary_storage;
mod ram_isolated_nonvolatile_storage;
mod ram_nonvolatile_storage;

kernel::stack_size! {0x2000}

type ChipHw = qemu_arm_mps2_unsafe::chip::QemuArmMps2Chip<
    'static,
    qemu_arm_mps2_an386::CortexM4,
    peripherals::Peripherals<'static>,
>;

// How much nonvolatile storage space to allocate per-app.
const APP_STORAGE_REGION_SIZE: usize = 2048;

// Total bytes of RAM standing in for nonvolatile storage. Lost on reset --
// the AN386 QEMU machine has no flash controller.
const RAM_NONVOLATILE_STORAGE_SIZE: usize = 32768;
static mut RAM_NONVOLATILE_STORAGE: [u8; RAM_NONVOLATILE_STORAGE_SIZE] =
    [0; RAM_NONVOLATILE_STORAGE_SIZE];

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::StopWithDebugFaultPolicy =
    capsules_system::process_policies::StopWithDebugFaultPolicy {};

// A second console, on UART1, at an out-of-tree driver number: `mps2_base`
// only wires UART0 up to the debug/process console, so apps that want a
// dedicated second serial channel need their own `Console` here.
const CONSOLE1_DRIVER_NUM: usize = 0x01000001;

type NonvolatileStorageDriver =
    ram_isolated_nonvolatile_storage::RamIsolatedNonvolatileStorageComponentType<
        APP_STORAGE_REGION_SIZE,
    >;

type DynamicBinaryStorageDriver = kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
    'static,
    'static,
    ChipHw,
    kernel::process::ProcessStandardDebugFull,
    ram_nonvolatile_storage::RamNonvolatileStorage<'static>,
>;
type AppLoaderDriver = capsules_extra::app_loader::AppLoader<
    DynamicBinaryStorageDriver,
    DynamicBinaryStorageDriver,
    DynamicBinaryStorageDriver,
>;

/// Board-owned panic-time resources, populated by `mps2_base` during boot
/// and read back by the `#[panic_handler]` in `io.rs`.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, mps2_base::ProcessPrinterInUse>> =
    SingleThreadValue::new();

/// Supported drivers by the platform.
struct Platform {
    base: &'static mps2_base::Platform,
    nonvolatile_storage: &'static NonvolatileStorageDriver,
    console1: &'static capsules_core::console::Console<'static>,
    dynamic_app_loader: &'static AppLoaderDriver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(self.nonvolatile_storage))
            }
            CONSOLE1_DRIVER_NUM => f(Some(self.console1)),
            capsules_extra::app_loader::DRIVER_NUM => f(Some(self.dynamic_app_loader)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <mps2_base::Platform as KernelResources<ChipHw>>::SyscallFilter;
    type ProcessFault = <mps2_base::Platform as KernelResources<ChipHw>>::ProcessFault;
    type Scheduler = <mps2_base::Platform as KernelResources<ChipHw>>::Scheduler;
    type SchedulerTimer = <mps2_base::Platform as KernelResources<ChipHw>>::SchedulerTimer;
    type WatchDog = <mps2_base::Platform as KernelResources<ChipHw>>::WatchDog;
    type ContextSwitchCallback =
        <mps2_base::Platform as KernelResources<ChipHw>>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        <mps2_base::Platform as KernelResources<ChipHw>>::syscall_filter(self.base)
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        <mps2_base::Platform as KernelResources<ChipHw>>::process_fault(self.base)
    }
    fn scheduler(&self) -> &Self::Scheduler {
        <mps2_base::Platform as KernelResources<ChipHw>>::scheduler(self.base)
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        <mps2_base::Platform as KernelResources<ChipHw>>::scheduler_timer(self.base)
    }
    fn watchdog(&self) -> &Self::WatchDog {
        <mps2_base::Platform as KernelResources<ChipHw>>::watchdog(self.base)
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        <mps2_base::Platform as KernelResources<ChipHw>>::context_switch_callback(self.base)
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    // UART1 doesn't depend on anything `start_without_loading_processes()`
    // builds, so it's allocated up front: `alloc_chip` below captures it to
    // build this board's `Peripherals`, and it's used again afterward (in
    // the "SECOND CONSOLE" section) to build the console on top of it.
    //
    // SAFETY: We promise to be the only caller that constructs a `Uart` at
    // `UART1_BASE`.
    let uart1 = static_init!(
        qemu_arm_mps2::uart::Uart<'static>,
        qemu_arm_mps2::uart::Uart::new(qemu_arm_mps2_unsafe::addresses::UART1_BASE)
    );

    // SAFETY: `main` is only ever invoked once, by the reset handler, before
    // anything else touches the chip's peripherals or kernel state -- see
    // `mps2_base::start_without_loading_processes()`'s safety doc. `CortexM4`
    // is this board's actual CPU core.
    let (board_kernel, base_platform, chip, _base_peripherals) = unsafe {
        mps2_base::start_without_loading_processes::<
            qemu_arm_mps2_an386::CortexM4,
            peripherals::Peripherals<'static>,
            _,
        >(&PANIC_RESOURCES, |base_peripherals| {
            // The chip instance names the Cortex-M variant and our
            // `Peripherals` type concretely, which `static_init!()` cannot
            // do inside a generic function.
            let custom_peripherals = static_init!(
                peripherals::Peripherals<'static>,
                peripherals::Peripherals::new(base_peripherals, uart1)
            );
            static_init!(ChipHw, ChipHw::new(custom_peripherals))
        })
    };

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    // SAFETY: `main` runs once, so this is the only outstanding reference to
    // `RAM_NONVOLATILE_STORAGE`.
    let ram_storage: &'static mut [u8] = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(RAM_NONVOLATILE_STORAGE) as *mut u8,
        RAM_NONVOLATILE_STORAGE_SIZE,
    );

    let nonvolatile_storage =
        ram_isolated_nonvolatile_storage::RamIsolatedNonvolatileStorageComponent::new(
            board_kernel,
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM,
            ram_storage,
            create_capability!(capabilities::MemoryAllocationCapability),
        )
        .finalize(
            ram_isolated_nonvolatile_storage::ram_isolated_nonvolatile_storage_component_static!(
                APP_STORAGE_REGION_SIZE
            ),
        );

    //--------------------------------------------------------------------------
    // SECOND CONSOLE (UART1)
    //--------------------------------------------------------------------------
    //
    // QEMU's `mps2-an386` machine emulates five CMSDK UARTs; `mps2_base`
    // only wires UART0 up (to the debug/process console). This gives
    // userspace a second, independent serial channel over UART1 (`uart1`,
    // allocated above so both this and `peripherals::Peripherals` could use
    // it).

    let uart1_mux = components::console::UartMuxComponent::new(uart1, 115200)
        .finalize(components::uart_mux_component_static!());

    let console1 = components::console::ConsoleComponent::new(
        board_kernel,
        CONSOLE1_DRIVER_NUM,
        uart1_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    //--------------------------------------------------------------------------
    // APP IDENTIFIERS
    //--------------------------------------------------------------------------
    //
    // This configuration exists to test nonvolatile storage, not app
    // credentials, so processes are approved unconditionally and identified
    // by name. `IndividualStoragePermissions` only needs each process to
    // have a `Fixed` `ShortId` to grant it access to its own storage region.

    let checking_policy = components::appid::checker_null::AppCheckerNullComponent::new()
        .finalize(components::app_checker_null_component_static!());

    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());

    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    #[derive(Clone)]
    struct AppStorageCapability;
    unsafe impl capabilities::ApplicationStorageCapability for AppStorageCapability {}

    let storage_permissions_policy =
        components::storage_permissions::individual::StoragePermissionsIndividualComponent::new(
            AppStorageCapability,
        )
        .finalize(
            components::storage_permissions_individual_component_static!(
                ChipHw,
                kernel::process::ProcessStandardDebugFull,
                AppStorageCapability,
            ),
        );

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

    // Create and start the asynchronous process loader.
    let loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
        create_capability!(capabilities::ProcessManagementCapability),
    )
    .finalize(components::process_loader_sequential_component_static!(
        ChipHw,
        kernel::process::ProcessStandardDebugFull,
        mps2_base::NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // DYNAMIC PROCESS LOADING
    //--------------------------------------------------------------------------
    //
    // Apps loaded this way land in the same `_sapps`..`_eapps` region
    // `app_flash` above points at -- the "range of memory where apps are
    // stored in flash" on this board (there being no real flash, that
    // range is itself just RAM `tockloader`/QEMU's `-device loader`
    // pre-populate before boot). `loader` already holds a `&'static [u8]`
    // over that range for its own scanning, so this second
    // `RamNonvolatileStorage` is built with `new_at()` rather than `new()`:
    // it writes through raw pointers instead of a `&mut [u8]`, so it never
    // creates a live reference that would alias `loader`'s -- see
    // `ram_nonvolatile_storage`'s module docs.

    // SAFETY: `[_sapps, _eapps)` is valid for the life of the kernel (it's
    // the linker-defined app flash region), and the only other access to it
    // is `loader`'s read-only `&'static [u8]` scan above -- see this
    // section's comment and `RamNonvolatileStorage::new_at()`'s safety doc.
    let dynamic_app_storage = static_init!(
        ram_nonvolatile_storage::RamNonvolatileStorage,
        ram_nonvolatile_storage::RamNonvolatileStorage::new_at(
            core::ptr::addr_of!(_sapps) as usize,
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        )
    );
    dynamic_app_storage.register();

    let dynamic_binary_storage = ram_dynamic_binary_storage::RamDynamicBinaryStorageComponent::new(
        board_kernel,
        dynamic_app_storage,
        loader,
    )
    .finalize(
        ram_dynamic_binary_storage::ram_dynamic_binary_storage_component_static!(
            ChipHw,
            kernel::process::ProcessStandardDebugFull,
        ),
    );

    let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
        board_kernel,
        capsules_extra::app_loader::DRIVER_NUM,
        dynamic_binary_storage,
        dynamic_binary_storage,
        dynamic_binary_storage,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::app_loader_component_static!(
        DynamicBinaryStorageDriver,
        DynamicBinaryStorageDriver,
        DynamicBinaryStorageDriver,
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        nonvolatile_storage,
        console1,
        dynamic_app_loader,
    };

    kernel::debug!("QEMU MPS2 AN386 (Cortex-M4) isolated nonvolatile storage test.");
    kernel::debug!("Entering main loop.");

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
