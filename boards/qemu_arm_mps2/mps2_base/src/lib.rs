// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Shared kernel setup for the QEMU ARM MPS2 boards.
//!
//! The MPS2 images differ only in their CPU core, so this crate is generic
//! over [`CortexMVariant`] and holds everything that genericity allows to be
//! shared: the `Platform` syscall driver lookup, the `ChipHw` type, and the
//! process/capsule setup in [`early_init()`] / [`finish_start()`].
//!
//! What stays in each board's own `main.rs`/`io.rs`:
//! - The `static_init!(ChipHw<C>, ...)` allocation between those two calls: a
//!   `static`'s type cannot name the enclosing generic function's own type
//!   parameter.
//! - `#[panic_handler]`, which has to be a concrete non-generic function, and
//!   the `PANIC_RESOURCES` static it reads.
//! - `kernel::stack_size!` and the board name in the boot banner.
//!
//! Both boards use the same memory layout, so the two linker scripts'
//! `MEMORY` blocks are identical copies; each board crate needs its own.
//! Processes are loaded from flash at 0x00040000-0x0007FFFF into the RAM
//! above the kernel's own static allocations -- `_sappmem` in the built ELF
//! -- up to 0x21020000.

#![no_std]

use cortexm::CortexMVariant;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, static_init};

pub const NUM_PROCS: usize = 4;

pub type ChipHw<C> = qemu_arm_mps2::chip::QemuArmMps2Chip<
    'static,
    C,
    qemu_arm_mps2::Mps2DefaultPeripherals<'static>,
>;
pub type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;
type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

pub struct Platform {
    console: &'static capsules_core::console::Console<'static>,
    scheduler: &'static SchedulerInUse,
    systick: cortexm::systick::SysTick,
    led: &'static capsules_core::led::LedDriver<
        'static,
        qemu_arm_mps2::led::Led<'static>,
        { qemu_arm_mps2::led::NUM_LEDS as usize },
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            qemu_arm_mps2::timer::Timer<'static>,
        >,
    >,
    spi: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            qemu_arm_mps2::spi::Spi<'static>,
        >,
    >,
    watchdog: &'static qemu_arm_mps2::watchdog::Watchdog,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::spi_controller::DRIVER_NUM => f(Some(self.spi)),
            _ => f(None),
        }
    }
}

impl<C: CortexMVariant> KernelResources<ChipHw<C>> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = cortexm::systick::SysTick;
    type WatchDog = qemu_arm_mps2::watchdog::Watchdog;
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.systick
    }
    fn watchdog(&self) -> &Self::WatchDog {
        self.watchdog
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Peripherals and kernel state ready for allocating the chip and calling
/// [`finish_start()`].
pub struct EarlyInit<C: CortexMVariant + 'static> {
    pub peripherals: &'static qemu_arm_mps2::Mps2DefaultPeripherals<'static>,
    processes: &'static kernel::process::ProcessArray<NUM_PROCS>,
    board_kernel: &'static kernel::Kernel,
    panic_resources: &'static SingleThreadValue<PanicResources<ChipHw<C>, ProcessPrinterInUse>>,
}

/// Runs the CPU-variant init and allocates peripherals/kernel state.
///
/// A board calls `early_init`, then its own `static_init!(ChipHw<C>, ...)`,
/// then [`finish_start()`]. `panic_resources` is populated across both
/// halves; it travels in [`EarlyInit`] rather than being passed to each, so
/// the two cannot be handed different statics.
///
/// # Safety
///
/// Must be called exactly once, before any other access to the chip's
/// peripherals or kernel state, from the board's `main()` entry point --
/// this performs one-time hardware init and allocates `'static` state via
/// `static_init!()`, which does not itself guard against being called more
/// than once. `C` must be the actual `CortexMVariant` of the CPU this is
/// running on.
// inline(never) so this frame, and the stack the `static_init!()`s below use,
// is reclaimed when it returns rather than held for the life of the kernel.
#[inline(never)]
pub unsafe fn early_init<C: CortexMVariant>(
    panic_resources: &'static SingleThreadValue<PanicResources<ChipHw<C>, ProcessPrinterInUse>>,
) -> EarlyInit<C> {
    ChipHw::<C>::init();

    kernel::deferred_call::initialize_deferred_call_state::<<ChipHw<C> as Chip>::ThreadIdProvider>(
    );

    let _ = panic_resources
        .bind_to_thread::<<ChipHw<C> as Chip>::ThreadIdProvider>(PanicResources::new());

    let peripherals = static_init!(
        qemu_arm_mps2::Mps2DefaultPeripherals<'static>,
        qemu_arm_mps2::Mps2DefaultPeripherals::new()
    );

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    panic_resources.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    EarlyInit {
        peripherals,
        processes,
        board_kernel,
        panic_resources,
    }
}

/// Finishes board setup and starts loading processes.
///
/// `chip` must have been allocated by the caller via
/// `static_init!(ChipHw<C>, ChipHw::<C>::new(early_init.peripherals))`
/// after [`early_init()`] — see its docs for why.
///
/// # Safety
///
/// Must be called exactly once, immediately after the [`early_init()`] call
/// that produced `early` and the `static_init!()` that produced `chip` (both
/// from the same boot, same `C`) -- this allocates more `'static` state and
/// starts loading processes from the linker-defined app regions, neither of
/// which is safe to repeat.
// inline(never) so this frame, and the stack the `static_init!()`s below use,
// is reclaimed when it returns rather than held for the life of the kernel.
#[inline(never)]
pub unsafe fn finish_start<C: CortexMVariant>(
    early: EarlyInit<C>,
    chip: &'static ChipHw<C>,
) -> (
    &'static kernel::Kernel,
    &'static Platform,
    &'static ChipHw<C>,
) {
    let EarlyInit {
        peripherals,
        processes,
        board_kernel,
        panic_resources,
    } = early;

    panic_resources.get().map(|resources| {
        resources.chip.put(chip);
    });

    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    components::debug_writer::DebugWriterComponent::new::<<ChipHw<C> as Chip>::ThreadIdProvider>(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let alarm_mux = components::alarm::AlarmMuxComponent::new(&peripherals.timer0).finalize(
        components::alarm_mux_component_static!(qemu_arm_mps2::timer::Timer),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        alarm_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(
        qemu_arm_mps2::timer::Timer
    ));

    kernel::create_typed_capability!(process_console_cap, ProcessConsoleCap:
        kernel::capabilities::ProcessManagementCapability,
        kernel::capabilities::ProcessStartCapability
    );
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());

    panic_resources.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        alarm_mux,
        process_printer,
        None,
        process_console_cap,
    )
    .finalize(components::process_console_component_static!(
        qemu_arm_mps2::timer::Timer,
        ProcessConsoleCap
    ));
    let _ = process_console.start();

    let spi_mux = components::spi::SpiMuxComponent::new(&peripherals.spi_shield0).finalize(
        components::spi_mux_component_static!(qemu_arm_mps2::spi::Spi),
    );

    let spi = components::spi::SpiSyscallComponent::new(
        board_kernel,
        spi_mux,
        qemu_arm_mps2::spi::ChipSelect,
        capsules_core::spi_controller::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::spi_syscall_component_static!(
        qemu_arm_mps2::spi::Spi
    ));

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        qemu_arm_mps2::led::Led<'static>,
        peripherals.fpgaio.led::<0>(),
        peripherals.fpgaio.led::<1>(),
    ));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = static_init!(
        Platform,
        Platform {
            console,
            scheduler,
            systick: cortexm::systick::SysTick::new(),
            led,
            alarm,
            spi,
            watchdog: &peripherals.watchdog,
        }
    );

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
        &capsules_system::process_policies::PanicFaultPolicy {},
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        kernel::debug!("Error loading processes!");
        kernel::debug!("{:?}", err);
    });

    (board_kernel, platform, chip)
}
