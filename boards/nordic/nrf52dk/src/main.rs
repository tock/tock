// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52 development kit (DK), a.k.a. the PCA10040.
//!
//! It is based on nRF52838 SoC (Cortex M4 core with a BLE transceiver) with many exported
//! I/O and peripherals.
//!
//! nRF52838 has only one port and uses pins 0-31!
//!
//! Furthermore, there exist another a preview development kit for nRF52840 but it is not supported
//! yet because unfortunately the pin configuration differ from nRF52-DK whereas nRF52840 uses two
//! ports where port 0 has 32 pins and port 1 has 16 pins.
//!
//! Pin Configuration
//! -------------------
//!
//! ### `GPIOs`
//! * P0.27 -> (top left header)
//! * P0.26 -> (top left header)
//! * P0.02 -> (top left header)
//! * P0.25 -> (top left header)
//! * P0.24 -> (top left header)
//! * P0.23 -> (top left header)
//! * P0.22 -> (top left header)
//! * P0.12 -> (top mid header)
//! * P0.11 -> (top mid header)
//! * P0.03 -> (bottom right header)
//! * P0.04 -> (bottom right header)
//! * P0.28 -> (bottom right header)
//! * P0.29 -> (bottom right header)
//! * P0.30 -> (bottom right header)
//! * P0.31 -> (bottom right header)
//!
//! ### `LEDs`
//! * P0.17 -> LED1
//! * P0.18 -> LED2
//! * P0.19 -> LED3
//! * P0.20 -> LED4
//!
//! ### `Buttons`
//! * P0.13 -> Button1
//! * P0.14 -> Button2
//! * P0.15 -> Button3
//! * P0.16 -> Button4
//! * P0.21 -> Reset Button
//!
//! ### `UART`
//! * P0.05 -> RTS
//! * P0.06 -> TXD
//! * P0.07 -> CTS
//! * P0.08 -> RXD
//!
//! ### `NFC`
//! * P0.09 -> NFC1
//! * P0.10 -> NFC2
//!
//! ### `LFXO`
//! * P0.01 -> XL2
//! * P0.00 -> XL1
//!
//! Author
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * July 16, 2017

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use kernel::component::Component;
use kernel::hil::led::LedLow;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{capabilities, create_capability, debug, debug_gpio, debug_verbose, static_init};
use nrf52832::gpio::Pin;
use nrf52832::interrupt_service::Nrf52832DefaultPeripherals;
use nrf52832::rtc::Rtc;
use nrf52_components::{UartChannel, UartPins};

// The nRF52 DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_17;
const LED2_PIN: Pin = Pin::P0_18;
const LED3_PIN: Pin = Pin::P0_19;
const LED4_PIN: Pin = Pin::P0_20;

// The nRF52 DK buttons (see back of board)
const BUTTON1_PIN: Pin = Pin::P0_13;
const BUTTON2_PIN: Pin = Pin::P0_14;
const BUTTON3_PIN: Pin = Pin::P0_15;
const BUTTON4_PIN: Pin = Pin::P0_16;
const BUTTON_RST_PIN: Pin = Pin::P0_21;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

// SPI not used, but keep pins around
const _SPI_MOSI: Pin = Pin::P0_22;
const _SPI_MISO: Pin = Pin::P0_23;
const _SPI_CLK: Pin = Pin::P0_24;

/// UART Writer
pub mod io;

// FIXME: Ideally this should be replaced with Rust's builtin tests by conditional compilation
//
// Also read the instructions in `tests` how to run the tests
#[allow(dead_code)]
mod tests;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] = [None; 4];

// Static reference to chip for panic dumps
static mut CHIP: Option<&'static nrf52832::chip::NRF52<Nrf52832DefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

type TemperatureDriver =
    components::temperature::TemperatureComponentType<nrf52832::temperature::Temp<'static>>;
type RngDriver = components::rng::RngComponentType<nrf52832::trng::Trng<'static>>;

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        nrf52832::ble_radio::Radio<'static>,
        VirtualMuxAlarm<'static, Rtc<'static>>,
    >,
    button: &'static capsules_core::button::Button<'static, nrf52832::gpio::GPIOPin<'static>>,
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        VirtualMuxAlarm<'static, Rtc<'static>>,
        components::process_console::Capability,
    >,
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52832::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, nrf52832::gpio::GPIOPin<'static>>,
        4,
    >,
    rng: &'static RngDriver,
    temp: &'static TemperatureDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    analog_comparator: &'static capsules_extra::analog_comparator::AnalogComparator<
        'static,
        nrf52832::acomp::Comparator<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52832::rtc::Rtc<'static>,
        >,
    >,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules_extra::analog_comparator::DRIVER_NUM => f(Some(self.analog_comparator)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<nrf52832::chip::NRF52<'static, Nrf52832DefaultPeripherals<'static>>>
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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    Platform,
    &'static nrf52832::chip::NRF52<'static, Nrf52832DefaultPeripherals<'static>>,
) {
    nrf52832::init();

    let nrf52832_peripherals = static_init!(
        Nrf52832DefaultPeripherals,
        Nrf52832DefaultPeripherals::new()
    );

    // set up circular peripheral dependencies
    nrf52832_peripherals.init();
    let base_peripherals = &nrf52832_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52832::gpio::GPIOPin,
            // Bottom right header on DK board
            0 => &nrf52832_peripherals.gpio_port[Pin::P0_03],
            1 => &nrf52832_peripherals.gpio_port[Pin::P0_04],
            2 => &nrf52832_peripherals.gpio_port[Pin::P0_28],
            3 => &nrf52832_peripherals.gpio_port[Pin::P0_29],
            4 => &nrf52832_peripherals.gpio_port[Pin::P0_30],
            5 => &nrf52832_peripherals.gpio_port[Pin::P0_31],
            // Top mid header on DK board
            6 => &nrf52832_peripherals.gpio_port[Pin::P0_12],
            7 => &nrf52832_peripherals.gpio_port[Pin::P0_11],
            // Top left header on DK board
            8 => &nrf52832_peripherals.gpio_port[Pin::P0_27],
            9 => &nrf52832_peripherals.gpio_port[Pin::P0_26],
            10 => &nrf52832_peripherals.gpio_port[Pin::P0_02],
            11 => &nrf52832_peripherals.gpio_port[Pin::P0_25]
        ),
    )
    .finalize(components::gpio_component_static!(nrf52832::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52832::gpio::GPIOPin,
            (
                &nrf52832_peripherals.gpio_port[BUTTON1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //13
            (
                &nrf52832_peripherals.gpio_port[BUTTON2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //14
            (
                &nrf52832_peripherals.gpio_port[BUTTON3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //15
            (
                &nrf52832_peripherals.gpio_port[BUTTON4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ) //16
        ),
    )
    .finalize(components::button_component_static!(
        nrf52832::gpio::GPIOPin
    ));

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, nrf52832::gpio::GPIOPin>,
        LedLow::new(&nrf52832_peripherals.gpio_port[LED1_PIN]),
        LedLow::new(&nrf52832_peripherals.gpio_port[LED2_PIN]),
        LedLow::new(&nrf52832_peripherals.gpio_port[LED3_PIN]),
        LedLow::new(&nrf52832_peripherals.gpio_port[LED4_PIN]),
    ));

    let chip = static_init!(
        nrf52832::chip::NRF52<Nrf52832DefaultPeripherals>,
        nrf52832::chip::NRF52::new(nrf52832_peripherals)
    );
    CHIP = Some(chip);

    nrf52_components::startup::NrfStartupComponent::new(
        false,
        BUTTON_RST_PIN,
        nrf52832::uicr::Regulator0Output::DEFAULT,
        &base_peripherals.nvmc,
    )
    .finalize(());

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let gpio_port = &nrf52832_peripherals.gpio_port;
    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&gpio_port[LED1_PIN]),
        Some(&gpio_port[LED2_PIN]),
        Some(&gpio_port[LED3_PIN]),
    );

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(nrf52832::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(nrf52832::rtc::Rtc));
    let uart_channel = UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD));
    let channel = nrf52_components::UartChannelComponent::new(
        uart_channel,
        mux_alarm,
        &base_peripherals.uarte0,
    )
    .finalize(nrf52_components::uart_channel_component_static!(
        nrf52832::rtc::Rtc
    ));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(channel, 115200)
        .finalize(components::uart_mux_component_static!());

    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(Rtc<'static>));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        capsules_extra::ble_advertising_driver::DRIVER_NUM,
        &base_peripherals.ble_radio,
        mux_alarm,
    )
    .finalize(components::ble_component_static!(
        nrf52832::rtc::Rtc,
        nrf52832::ble_radio::Radio
    ));

    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        &base_peripherals.temp,
    )
    .finalize(components::temperature_component_static!(
        nrf52832::temperature::Temp
    ));

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &base_peripherals.trng,
    )
    .finalize(components::rng_component_static!(nrf52832::trng::Trng));

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let analog_comparator = components::analog_comparator::AnalogComparatorComponent::new(
        &base_peripherals.acomp,
        components::analog_comparator_component_helper!(
            nrf52832::acomp::Channel,
            &*addr_of!(nrf52832::acomp::CHANNEL_AC0)
        ),
        board_kernel,
        capsules_extra::analog_comparator::DRIVER_NUM,
    )
    .finalize(components::analog_comparator_component_static!(
        nrf52832::acomp::Comparator
    ));

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        button,
        ble_radio,
        pconsole,
        console,
        led,
        gpio,
        rng,
        temp,
        alarm,
        analog_comparator,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    let _ = platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &*addr_of!(nrf52832::ficr::FICR_INSTANCE));

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
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
