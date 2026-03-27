// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! Tock kernel for the Raspberry Pi Pico 2.
//!
//! It is based on RP2350SoC SoC (Cortex M33).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use core::ptr::addr_of_mut;

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::led::LedsComponent;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::syscall::SyscallDriver;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{capabilities, create_capability, static_init, Kernel};

use psc3::chip::{Psc3, Psc3DefaultPeripherals};
use psc3::gpio;
use psc3::icache;
use psc3::tcpwm::Tcpwm0;
#[allow(unused)]
use psc3::BASE_VECTORS;

mod io;

// Allocate memory for the stack
kernel::stack_size! {0x3000}

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

type ChipHw = Psc3<'static, Psc3DefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new(PanicResources::new());

type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

/// Supported drivers by the platform
pub struct Psc3Plattform {
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    console: &'static capsules_core::console::Console<'static>,
    scheduler: &'static SchedulerInUse,
    systick: cortexm33::systick::SysTick,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, Tcpwm0<'static>>,
    >,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, gpio::GpioPin<'static>>,
        1,
    >,
    button: &'static capsules_core::button::Button<'static, gpio::GpioPin<'static>>,
    gpio: &'static capsules_core::gpio::GPIO<'static, gpio::GpioPin<'static>>,
}

impl SyscallDriverLookup for Psc3Plattform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

impl KernelResources<Psc3<'static, Psc3DefaultPeripherals<'static>>> for Psc3Plattform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = cortexm33::systick::SysTick;
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
    /// Beginning of the stack region.
    static _sstack: u8;
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    /* Only after peripherals.sys_init() was called peripheral view for debugging works */
    icache::sys_init_enable_cache();
    // TODO Cypress has different register (is it mapped?)
    cortexm33::scb::set_vector_table_offset(core::ptr::addr_of!(BASE_VECTORS).cast::<()>());
    cortexm33::support::dmb();
    cortexm33::nvic::enable_all();

    cortexm33::support::set_msplim(core::ptr::addr_of!(_sstack) as u32);

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Bind global variables to this thread.
    PANIC_RESOURCES.bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>();

    let peripherals = static_init!(Psc3DefaultPeripherals, Psc3DefaultPeripherals::new());

    peripherals.sys_init();
    peripherals.init();

    const GPIO_CONFIG: gpio::PreConfig = gpio::PreConfig {
        out_val: 1,
        drive_mode: gpio::DriveMode::PullUp,
        hsiom: gpio::HsiomFunction::GPIOControlsOut,
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
        non_sec: false,
    };

    peripherals
        .gpio
        .get_pin(gpio::PsocPin::P8_5)
        .preconfigure(&GPIO_CONFIG);

    // Set the UART used for panic
    (*addr_of_mut!(io::WRITER)).set_scb(&peripherals.scb3);

    let chip = static_init!(Psc3<Psc3DefaultPeripherals>, Psc3::new(peripherals));
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    let board_kernel = static_init!(Kernel, Kernel::new(processes.as_slice()));

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.tcpwm)
        .finalize(components::alarm_mux_component_static!(Tcpwm0));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(Tcpwm0));

    let uart_mux = components::console::UartMuxComponent::new(&peripherals.scb3, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // PROCESS CONSOLE
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm33::support::reset),
    )
    .finalize(components::process_console_component_static!(Tcpwm0));
    let _ = process_console.start();

    let led_pin = peripherals.gpio.get_pin(gpio::PsocPin::P8_4);
    led_pin.preconfigure(&GPIO_CONFIG);

    let led = LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, gpio::GpioPin>,
        LedHigh::new(led_pin)
    ));

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            gpio::GpioPin,
            //  Port 0 & 1:
            01 => peripherals.gpio.get_pin(gpio::PsocPin::P0_1), // Header J5.5 (Remove R22, Mount R21)
            10 => peripherals.gpio.get_pin(gpio::PsocPin::P1_0), // Header J24.37 (Remove R18, Mount R17)
            11 => peripherals.gpio.get_pin(gpio::PsocPin::P1_1), // Header J5.7 (Remove R14, Mount R13)

            // Port 2: General Purpose / JTAG
            22 => peripherals.gpio.get_pin(gpio::PsocPin::P2_2), // Header J24.10
            23 => peripherals.gpio.get_pin(gpio::PsocPin::P2_3), // Header J24.9

            // Port 3: Digital I/O Multiplexed
            30 => peripherals.gpio.get_pin(gpio::PsocPin::P3_0), // Header J4.8 / J5.24
            31 => peripherals.gpio.get_pin(gpio::PsocPin::P3_1), // Header J5.28 / J6.4

            // Port 4: PWM/General Purpose
            44 => peripherals.gpio.get_pin(gpio::PsocPin::P4_4), // Header J5.36
            45 => peripherals.gpio.get_pin(gpio::PsocPin::P4_5), // Header J5.37

            // Port 5: CAN/PWM
            51 => peripherals.gpio.get_pin(gpio::PsocPin::P5_1), // Header J24.12

            // Port 7: SPI/PWM
            76 => peripherals.gpio.get_pin(gpio::PsocPin::P7_6), // Header J5.22 / J3.1
            77 => peripherals.gpio.get_pin(gpio::PsocPin::P7_7), // Header J24.4

            // In led capsule and panic_handler
            // // Port 8: LEDs
            // 84 => peripherals.gpio.get_pin(gpio::PsocPin::P8_4), // User LED 2 (and Header J24.6)
            // 85 => peripherals.gpio.get_pin(gpio::PsocPin::P8_5), // User LED 1 (and Header J24.8)

            // Port 9: PWM/Expansion
            91 => peripherals.gpio.get_pin(gpio::PsocPin::P9_1), // Header J5.25
            93 => peripherals.gpio.get_pin(gpio::PsocPin::P9_3), // Header J24.3
        ),
    )
    .finalize(components::gpio_component_static!(gpio::GpioPin));

    //--------------------------------------------------------------------------
    // BUTTON
    //--------------------------------------------------------------------------

    let button_pin = peripherals.gpio.get_pin(gpio::PsocPin::P5_0);
    button_pin.preconfigure(&GPIO_CONFIG);

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            gpio::GpioPin,
            (
                button_pin,
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ),
        ),
    )
    .finalize(components::button_component_static!(gpio::GpioPin));

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let psc3_platform = Psc3Plattform {
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        console,
        alarm,
        scheduler,
        systick: cortexm33::systick::SysTick::new_with_calibration(1_000_000),
        led,
        button,
        gpio,
    };

    kernel::debug!("Initialization complete. Enter main loop");

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
        kernel::debug!("Error loading processes!");
        kernel::debug!("{:?}", err);
    });

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    board_kernel.kernel_loop(
        &psc3_platform,
        chip,
        Some(&psc3_platform.ipc),
        &main_loop_capability,
    );
}
