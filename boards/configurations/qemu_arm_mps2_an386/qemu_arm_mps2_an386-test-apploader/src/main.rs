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
//! This configuration also exposes a second `Console`, on UART1, at a second
//! instance of the console driver number -- see the "SECOND CONSOLE (UART1)"
//! section in `main()` and the family Makefile for how it's bridged to a
//! host serial port under QEMU.
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
use kernel::platform::chip::InterruptService;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::static_init;
use kernel::utilities::single_thread_value::SingleThreadValue;
use qemu_arm_mps2::{interrupts, uart};

pub mod io;
mod ram_dynamic_binary_storage;
mod ram_isolated_nonvolatile_storage;
mod ram_nonvolatile_storage;

kernel::stack_size! {0x2000}

// How much nonvolatile storage space to allocate per-app.
const APP_STORAGE_REGION_SIZE: usize = 2048;

// Total bytes of RAM standing in for nonvolatile storage. Lost on reset --
// the AN386 QEMU machine has no flash controller.
const RAM_NONVOLATILE_STORAGE_SIZE: usize = 32768;

//--------------------------------------------------------------------------
// TYPES
//--------------------------------------------------------------------------

type ChipHw = qemu_arm_mps2_unsafe::chip::QemuArmMps2Chip<
    'static,
    qemu_arm_mps2_an386::CortexM4,
    Peripherals<'static>,
>;

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
    DynamicBinaryStorageDriver,
>;

//--------------------------------------------------------------------------
// KERNEL CONFIG
//--------------------------------------------------------------------------

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::StopWithDebugFaultPolicy =
    capsules_system::process_policies::StopWithDebugFaultPolicy {};

/// Board-owned panic-time resources, populated by `mps2_base` during boot
/// and read back by the `#[panic_handler]` in `io.rs`.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, mps2_base::ProcessPrinterInUse>> =
    SingleThreadValue::new();

// A second console, on UART1: `mps2_base` only wires UART0 up to the
// debug/process console, so apps that want a dedicated second serial
// channel need their own `Console` here, at a second instance of its driver
// number.
const CONSOLE1_DRIVER_NUM: usize =
    capsules_core::driver::dup_driver_num(capsules_core::console::DRIVER_NUM, 1);

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

/// This configuration's `InterruptService`, layering UART1 on top of the
/// peripherals `mps2_base` already builds.
pub struct Peripherals<'a> {
    base: &'a qemu_arm_mps2::Mps2DefaultPeripherals<'a>,
    pub uart1: &'a uart::Uart<'a>,
}

impl<'a> Peripherals<'a> {
    pub fn new(
        base: &'a qemu_arm_mps2::Mps2DefaultPeripherals<'a>,
        uart1: &'a uart::Uart<'a>,
    ) -> Self {
        Self { base, uart1 }
    }
}

impl InterruptService for Peripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART1_RX | interrupts::UART1_TX => {
                self.uart1.handle_interrupt();
                true
            }
            _ => self.base.service_interrupt(interrupt),
        }
    }
}

impl Peripherals<'_> {
    /// Registers deferred calls owned by these peripherals. The board must
    /// call this once, after placing the peripherals in `'static` storage
    /// (e.g. via `static_init!()`).
    pub fn init(&'static self) {
        self.base.init();
        self.uart1.register();
    }
}

/// This is in a separate, `#[inline(never)]` function so that its stack frame
/// is reclaimed when it returns.
#[inline(never)]
unsafe fn start() -> (&'static kernel::Kernel, Platform, &'static ChipHw) {
    //--------------------------------------------------------------------------
    // SHARED BOARD SETUP
    //--------------------------------------------------------------------------

    let (board_kernel, base_platform, base_peripherals) = unsafe {
        mps2_base::start_without_loading_processes::<
            qemu_arm_mps2_an386::CortexM4,
            Peripherals<'static>,
        >(&PANIC_RESOURCES)
    };

    //--------------------------------------------------------------------------
    // UART1 FOR CONSOLE2
    //--------------------------------------------------------------------------

    let uart1 = static_init!(
        qemu_arm_mps2::uart::Uart<'static>,
        qemu_arm_mps2::uart::Uart::new(qemu_arm_mps2_unsafe::addresses::UART1_BASE)
    );
    let uart1_ring_buffer_bytes = static_init!([u8; 2048], [0; 2048]);
    let uart1_ring_buffer = static_init!(
        kernel::collections::ring_buffer::RingBuffer<'static, u8>,
        kernel::collections::ring_buffer::RingBuffer::new(
            kernel::utilities::slice_uninit::mut_slice_as_maybeuninit(uart1_ring_buffer_bytes)
        )
    );
    uart1.set_ring_buffer(uart1_ring_buffer);

    //--------------------------------------------------------------------------
    // CHIP
    //--------------------------------------------------------------------------

    // The chip instance names the Cortex-M variant and our `Peripherals`
    // type concretely, which `static_init!()` cannot do inside a generic
    // function.
    let custom_peripherals = static_init!(
        Peripherals<'static>,
        Peripherals::new(base_peripherals, uart1)
    );
    custom_peripherals.init();
    let chip = static_init!(ChipHw, unsafe { ChipHw::new(custom_peripherals) });
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    #[allow(clippy::large_stack_arrays)]
    let ram_storage = static_init!(
        [u8; RAM_NONVOLATILE_STORAGE_SIZE],
        [0; RAM_NONVOLATILE_STORAGE_SIZE]
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

    let console1 = components::console::ConsoleNoMuxComponent::new(
        board_kernel,
        CONSOLE1_DRIVER_NUM,
        uart1,
        115200,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_no_mux_component_static!(2048, 2048));

    //--------------------------------------------------------------------------
    // APP IDENTIFIERS
    //--------------------------------------------------------------------------

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
    unsafe extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let app_flash = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        )
    };
    let app_memory = unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        )
    };

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
    let dynamic_app_storage =
        static_init!(ram_nonvolatile_storage::RamNonvolatileStorage, unsafe {
            ram_nonvolatile_storage::RamNonvolatileStorage::new_at(
                core::ptr::slice_from_raw_parts_mut(
                    core::ptr::addr_of!(_sapps).cast_mut(),
                    core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
                ),
                ram_nonvolatile_storage::AddressMode::Absolute,
            )
        });
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
        dynamic_binary_storage,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::app_loader_component_static!(
        DynamicBinaryStorageDriver,
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

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[unsafe(no_mangle)]
pub unsafe fn main() {
    let (board_kernel, platform, chip) = unsafe { start() };

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
