// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the QEMU ARM MPS2 AN385 (Cortex-M3) machine.
//!
//! This is a purely virtual platform: ARM's own CMSDK reference design, as
//! emulated by QEMU, not a real vendor chip. See `chips/qemu_arm_mps2_chip`
//! for the peripheral drivers and `README.md` for what is and is not
//! emulated (notably: GPIO pin state is not observable under this QEMU
//! machine, so this board does not expose a GPIO capsule; LEDs are
//! implemented against the separate, genuinely-emulated FPGAIO block).

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, static_init};

pub mod io;

const NUM_PROCS: usize = 4;

type ChipHw = qemu_arm_mps2_chip::chip::QemuArmMps2Chip<
    'static,
    cortexm3::CortexM3,
    qemu_arm_mps2_chip::Mps2DefaultPeripherals<'static>,
>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;
type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x2000}

struct QemuArmMps2An385 {
    console: &'static capsules_core::console::Console<'static>,
    scheduler: &'static SchedulerInUse,
    systick: cortexm3::systick::SysTick,
    led: &'static capsules_core::led::LedDriver<
        'static,
        qemu_arm_mps2_chip::led::Led<'static>,
        { qemu_arm_mps2_chip::led::NUM_LEDS as usize },
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            qemu_arm_mps2_chip::timer::Timer<'static>,
        >,
    >,
    spi: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            qemu_arm_mps2_chip::spi::Spi<'static>,
        >,
    >,
}

impl SyscallDriverLookup for QemuArmMps2An385 {
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

impl KernelResources<ChipHw> for QemuArmMps2An385 {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = cortexm3::systick::SysTick;
    type WatchDog = ();
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
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    &'static QemuArmMps2An385,
    &'static ChipHw,
) {
    ChipHw::init();

    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    let peripherals = static_init!(
        qemu_arm_mps2_chip::Mps2DefaultPeripherals<'static>,
        qemu_arm_mps2_chip::Mps2DefaultPeripherals::new()
    );

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let chip = static_init!(ChipHw, ChipHw::new(peripherals));

    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let alarm_mux = components::alarm::AlarmMuxComponent::new(&peripherals.timer0).finalize(
        components::alarm_mux_component_static!(qemu_arm_mps2_chip::timer::Timer),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        alarm_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(
        qemu_arm_mps2_chip::timer::Timer
    ));

    kernel::create_typed_capability!(process_console_cap, ProcessConsoleCap:
        kernel::capabilities::ProcessManagementCapability,
        kernel::capabilities::ProcessStartCapability
    );
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        alarm_mux,
        components::process_printer::ProcessPrinterTextComponent::new()
            .finalize(components::process_printer_text_component_static!()),
        None,
        process_console_cap,
    )
    .finalize(components::process_console_component_static!(
        qemu_arm_mps2_chip::timer::Timer,
        ProcessConsoleCap
    ));
    let _ = process_console.start();

    let spi_mux = components::spi::SpiMuxComponent::new(&peripherals.spi_shield0).finalize(
        components::spi_mux_component_static!(qemu_arm_mps2_chip::spi::Spi),
    );

    let spi = components::spi::SpiSyscallComponent::new(
        board_kernel,
        spi_mux,
        qemu_arm_mps2_chip::spi::ChipSelect,
        capsules_core::spi_controller::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::spi_syscall_component_static!(
        qemu_arm_mps2_chip::spi::Spi
    ));

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        qemu_arm_mps2_chip::led::Led<'static>,
        peripherals.fpgaio.led(0),
        peripherals.fpgaio.led(1),
    ));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = static_init!(
        QemuArmMps2An385,
        QemuArmMps2An385 {
            console,
            scheduler,
            systick: cortexm3::systick::SysTick::new(),
            led,
            alarm,
            spi,
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

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let (board_kernel, platform, chip) = start();

    kernel::debug!("QEMU MPS2 AN385 (Cortex-M3) initialization complete.");
    kernel::debug!("Entering main loop.");

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop::<QemuArmMps2An385, ChipHw, { NUM_PROCS as u8 }>(
        platform,
        chip,
        None,
        &main_loop_capability,
    );
}
