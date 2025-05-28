// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

#![no_std]
#![no_main]
#![deny(missing_docs)]

//! Tock kernel for the CY8CPROTO-062-4343W.

mod io;

/// Kernel stack memory
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::led::LedsComponent;
use core::ptr::{addr_of, addr_of_mut};
use kernel::component::Component;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{capabilities, create_capability, static_init, Kernel};

#[allow(unused)]
use psoc62xa::{
    chip::{PsoC62xaDefaultPeripherals, Psoc62xa},
    gpio::GpioPin,
    tcpwm::Tcpwm0,
    BASE_VECTORS,
};

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static Psoc62xa<PsoC62xaDefaultPeripherals>> = None;

static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Supported drivers by the platform
pub struct Cy8cproto0624343w {
    console: &'static capsules_core::console::Console<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, psoc62xa::tcpwm::Tcpwm0<'static>>,
    >,
    led: &'static capsules_core::led::LedDriver<'static, LedHigh<'static, GpioPin<'static>>, 1>,
    button: &'static capsules_core::button::Button<'static, GpioPin<'static>>,
    gpio: &'static capsules_core::gpio::GPIO<'static, psoc62xa::gpio::GpioPin<'static>>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm0p::systick::SysTick,
}

impl SyscallDriverLookup for Cy8cproto0624343w {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

impl KernelResources<Psoc62xa<'static, PsoC62xaDefaultPeripherals<'static>>> for Cy8cproto0624343w {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm0p::systick::SysTick;
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

fn init_clocks(peripherals: &PsoC62xaDefaultPeripherals) {
    peripherals.srss.init_clock();
    peripherals.cpuss.init_clock();
    peripherals.peri.init_uart_clock();
    peripherals.peri.init_alarm_clock();
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    // Set the offset of the vector table
    cortexm0p::scb::set_vector_table_offset(0x10000000 as *const ());

    let peripherals = static_init!(
        PsoC62xaDefaultPeripherals,
        PsoC62xaDefaultPeripherals::new()
    );

    // Initialize clocks
    init_clocks(peripherals);

    // Enable interrupts
    peripherals.cpuss.enable_int_for_scb5();
    peripherals.cpuss.enable_int_for_tcpwm00();
    peripherals.cpuss.enable_int_for_gpio0();
    cortexm0p::nvic::enable_all();

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    peripherals.scb.set_standard_uart_mode();
    peripherals.scb.enable_scb();
    peripherals.hsiom.enable_uart();
    let uart_tx_pin = peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P5_1);
    uart_tx_pin.configure_drive_mode(psoc62xa::gpio::DriveMode::Strong);
    uart_tx_pin.configure_input(false);
    let uart_rx_pin = peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P5_0);
    uart_rx_pin.configure_drive_mode(psoc62xa::gpio::DriveMode::HighZ);
    uart_rx_pin.configure_input(true);
    let chip = static_init!(
        Psoc62xa<PsoC62xaDefaultPeripherals>,
        Psoc62xa::new(peripherals)
    );

    let board_kernel = static_init!(Kernel, Kernel::new(&*addr_of!(PROCESSES)));

    // Create a shared UART channel for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.scb, 115200)
        .finalize(components::uart_mux_component_static!());

    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------
    peripherals.tcpwm.init_timer();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.tcpwm)
        .finalize(components::alarm_mux_component_static!(Tcpwm0));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(Tcpwm0));

    //--------------------------------------------------------------------------
    // PROCESS CONSOLE
    //--------------------------------------------------------------------------
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm0p::support::reset),
    )
    .finalize(components::process_console_component_static!(Tcpwm0));
    let _ = process_console.start();

    let led_pin = peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P13_7);

    let led = LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, GpioPin>,
        LedHigh::new(led_pin)
    ));

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            psoc62xa::gpio::GpioPin,
            5 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P0_5),
            44 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P5_4),
            45 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P5_5),
            46 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P5_6),
            47 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P5_7),
            50 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P6_2),
            51 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P6_3),
            52 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P6_4),
            53 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P6_5),
            64 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P8_0),
            72 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_0),
            73 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_1),
            74 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_2),
            76 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_4),
            77 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_5),
            78 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_6),
            79 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P9_7),
            96 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P12_0),
            97 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P12_1),
            99 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P12_3),
            108 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P13_4),
            110 => peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P13_6),
        ),
    )
    .finalize(components::gpio_component_static!(psoc62xa::gpio::GpioPin));

    //--------------------------------------------------------------------------
    // BUTTON
    //--------------------------------------------------------------------------

    let button_pin = peripherals.gpio.get_pin(psoc62xa::gpio::PsocPin::P0_4);

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            GpioPin,
            (
                button_pin,
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ),
        ),
    )
    .finalize(components::button_component_static!(GpioPin));

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let cy8cproto0624343w = Cy8cproto0624343w {
        console,
        alarm,
        scheduler,
        led,
        button,
        gpio,
        // Currently, the CPU runs at 8MHz, that being the frequency of the IMO.
        systick: cortexm0p::systick::SysTick::new_with_calibration(8_000_000),
    };

    CHIP = Some(chip);
    (*addr_of_mut!(io::WRITER)).set_scb(&peripherals.scb);

    kernel::debug!("Initialization complete. Entering main loop.");

    //--------------------------------------------------------------------------
    // PROCESSES AND MAIN LOOP
    //--------------------------------------------------------------------------

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
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        kernel::debug!("Error loading processes!");
        kernel::debug!("{:?}", err);
    });

    board_kernel.kernel_loop(
        &cy8cproto0624343w,
        chip,
        None::<kernel::ipc::IPC<{ NUM_PROCS as u8 }>>.as_ref(),
        &main_loop_capability,
    );
}
