// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use kernel::component::Component;
use kernel::hil::led::LedLow;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::process::ProcessLoadingAsync;
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{capabilities, create_capability, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52_components::{UartChannel, UartPins};

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_13;
const LED2_PIN: Pin = Pin::P0_14;
const LED3_PIN: Pin = Pin::P0_15;
const LED4_PIN: Pin = Pin::P0_16;

// The nRF52840DK buttons (see back of board)
const BUTTON1_PIN: Pin = Pin::P0_11;
const BUTTON2_PIN: Pin = Pin::P0_12;
const BUTTON3_PIN: Pin = Pin::P0_24;
const BUTTON4_PIN: Pin = Pin::P0_25;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

/// Debug Writer
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;
// Static reference to process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type AlarmDriver = components::alarm::AlarmDriverComponentType<nrf52840::rtc::Rtc<'static>>;

type NonVolatilePages = components::dynamic_binary_storage::NVPages<nrf52840::nvmc::Nvmc>;
type DynamicBinaryStorage<'a> = kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
    'static,
    'static,
    nrf52840::chip::NRF52<'a, Nrf52840DefaultPeripherals<'a>>,
    kernel::process::ProcessStandardDebugFull,
    NonVolatilePages,
>;

/// Supported drivers by the platform
pub struct Platform {
    console: &'static capsules_core::console::Console<'static>,
    button: &'static capsules_core::button::Button<'static, nrf52840::gpio::GPIOPin<'static>>,
    adc: &'static capsules_core::adc::AdcDedicated<'static, nrf52840::adc::Adc<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        kernel::hil::led::LedLow<'static, nrf52840::gpio::GPIOPin<'static>>,
        4,
    >,
    alarm: &'static AlarmDriver,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
    processes: &'static [Option<&'static dyn kernel::process::Process>],
    dynamic_app_loader: &'static capsules_extra::app_loader::AppLoader<
        DynamicBinaryStorage<'static>,
        DynamicBinaryStorage<'static>,
    >,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::app_loader::DRIVER_NUM => f(Some(self.dynamic_app_loader)),
            _ => f(None),
        }
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn create_peripherals() -> &'static mut Nrf52840DefaultPeripherals<'static> {
    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    nrf52840_peripherals
}

impl KernelResources<nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm4::systick::SysTick;
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

impl kernel::process::ProcessLoadingAsyncClient for Platform {
    fn process_loaded(&self, _result: Result<(), kernel::process::ProcessLoadError>) {}

    fn process_loading_finished(&self) {
        kernel::debug!("Processes Loaded at Main:");

        for (i, proc) in self.processes.iter().enumerate() {
            proc.map(|p| {
                kernel::debug!("[{}] {}", i, p.get_process_name());
                kernel::debug!("    ShortId: {}", p.short_app_id());
            });
        }
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    //--------------------------------------------------------------------------
    // INITIAL SETUP
    //--------------------------------------------------------------------------

    // Apply errata fixes and enable interrupts.
    nrf52840::init();

    // Set up peripheral drivers. Called in separate function to reduce stack
    // usage.
    let nrf52840_peripherals = create_peripherals();

    // Set up circular peripheral dependencies.
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    let processes = &*addr_of!(PROCESSES);

    // Choose the channel for serial output. This board can be configured to use
    // either the Segger RTT channel or via UART with traditional TX/RX GPIO
    // pins.
    let uart_channel = UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD));

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes));

    // Create (and save for panic debugging) a chip object to setup low-level
    // resources (e.g. MPU, systick).
    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    // Do nRF configuration and setup. This is shared code with other nRF-based
    // platforms.
    nrf52_components::startup::NrfStartupComponent::new(
        false,
        BUTTON_RST_PIN,
        nrf52840::uicr::Regulator0Output::DEFAULT,
        &base_peripherals.nvmc,
    )
    .finalize(());

    //--------------------------------------------------------------------------
    // CAPABILITIES
    //--------------------------------------------------------------------------

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, nrf52840::gpio::GPIOPin>,
        LedLow::new(&nrf52840_peripherals.gpio_port[LED1_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED3_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED4_PIN]),
    ));

    //--------------------------------------------------------------------------
    // TIMER
    //--------------------------------------------------------------------------

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(nrf52840::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(nrf52840::rtc::Rtc));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    let uart_channel = nrf52_components::UartChannelComponent::new(
        uart_channel,
        mux_alarm,
        &base_peripherals.uarte0,
    )
    .finalize(nrf52_components::uart_channel_component_static!(
        nrf52840::rtc::Rtc
    ));

    // Virtualize the UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(uart_channel, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the serial console for userspace.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    // Tool for displaying information about processes.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Create the process console, an interactive terminal for managing
    // processes.
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        nrf52840::rtc::Rtc<'static>
    ));

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    //--------------------------------------------------------------------------
    // BUTTONS
    //--------------------------------------------------------------------------

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840_peripherals.gpio_port[BUTTON1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &nrf52840_peripherals.gpio_port[BUTTON2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &nrf52840_peripherals.gpio_port[BUTTON3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &nrf52840_peripherals.gpio_port[BUTTON4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_static!(
        nrf52840::gpio::GPIOPin
    ));

    //--------------------------------------------------------------------------
    // ADC
    //--------------------------------------------------------------------------

    let adc_channels = static_init!(
        [nrf52840::adc::AdcChannelSetup; 6],
        [
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput1),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput2),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput4),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput5),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput6),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput7),
        ]
    );
    let adc = components::adc::AdcDedicatedComponent::new(
        &base_peripherals.adc,
        adc_channels,
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
    )
    .finalize(components::adc_dedicated_component_static!(
        nrf52840::adc::Adc
    ));

    //--------------------------------------------------------------------------
    // NRF CLOCK SETUP
    //--------------------------------------------------------------------------

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    //--------------------------------------------------------------------------
    // Credential Checking
    //--------------------------------------------------------------------------

    // Create the credential checker.
    let checking_policy = components::appid::checker_null::AppCheckerNullComponent::new()
        .finalize(components::app_checker_null_component_static!());

    // Create the AppID assigner.
    let assigner = components::appid::assigner_tbf::AppIdAssignerTbfHeaderComponent::new()
        .finalize(components::appid_assigner_tbf_header_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    let storage_permissions_policy =
        components::storage_permissions::null::StoragePermissionsNullComponent::new().finalize(
            components::storage_permissions_null_component_static!(
                nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
                kernel::process::ProcessStandardDebugFull,
            ),
        );

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
        &mut *addr_of_mut!(PROCESSES),
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // Dynamic App Loading
    //--------------------------------------------------------------------------

    // Create the dynamic binary flasher.
    let dynamic_binary_storage =
        components::dynamic_binary_storage::SequentialBinaryStorageComponent::new(
            &base_peripherals.nvmc,
            loader,
        )
        .finalize(components::sequential_binary_storage_component_static!(
            nrf52840::nvmc::Nvmc,
            nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
            kernel::process::ProcessStandardDebugFull,
        ));

    // Create the dynamic app loader capsule.
    let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
        board_kernel,
        capsules_extra::app_loader::DRIVER_NUM,
        dynamic_binary_storage,
        dynamic_binary_storage,
    )
    .finalize(components::app_loader_component_static!(
        DynamicBinaryStorage<'static>,
        DynamicBinaryStorage<'static>,
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = static_init!(
        Platform,
        Platform {
            console,
            button,
            adc,
            led,
            alarm,
            scheduler,
            systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
            processes,
            dynamic_app_loader,
        }
    );
    loader.set_client(platform);

    let _ = pconsole.start();

    board_kernel.kernel_loop(
        platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
