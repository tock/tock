// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 dongle.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::led::LedLow;
use kernel::hil::symmetric_encryption::AES128;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{capabilities, create_capability, debug, debug_gpio, debug_verbose, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52_components::{UartChannel, UartPins};

// The nRF52840 Dongle LEDs
const LED1_PIN: Pin = Pin::P0_06;
const LED2_R_PIN: Pin = Pin::P0_08;
const LED2_G_PIN: Pin = Pin::P1_09;
const LED2_B_PIN: Pin = Pin::P0_12;

// The nRF52840 Dongle button
const BUTTON_PIN: Pin = Pin::P1_06;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Option<Pin> = Some(Pin::P0_13);
const UART_TXD: Pin = Pin::P0_15;
const UART_CTS: Option<Pin> = Some(Pin::P0_17);
const UART_RXD: Pin = Pin::P0_20;

// SPI pins not currently in use, but left here for convenience
const _SPI_MOSI: Pin = Pin::P1_01;
const _SPI_MISO: Pin = Pin::P1_02;
const _SPI_CLK: Pin = Pin::P1_04;

// Constants related to the configuration of the 15.4 network stack; DEFAULT_EXT_SRC_MAC
// should be replaced by an extended src address generated from device serial number
const SRC_MAC: u16 = 0xf00f;
const PAN_ID: u16 = 0xABCD;
const DEFAULT_EXT_SRC_MAC: [u8; 8] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];

/// UART Writer
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Static reference to chip for panic dumps
static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;
// Static reference to process printer for panic dumps
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

type TemperatureDriver =
    components::temperature::TemperatureComponentType<nrf52840::temperature::Temp<'static>>;
type RngDriver = components::rng::RngComponentType<nrf52840::trng::Trng<'static>>;

type Ieee802154Driver = components::ieee802154::Ieee802154ComponentType<
    nrf52840::ieee802154_radio::Radio<'static>,
    nrf52840::aes::AesECB<'static>,
>;

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        nrf52840::ble_radio::Radio<'static>,
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    ieee802154_radio: &'static Ieee802154Driver,
    button: &'static capsules_core::button::Button<'static, nrf52840::gpio::GPIOPin<'static>>,
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
        components::process_console::Capability,
    >,
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52840::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, nrf52840::gpio::GPIOPin<'static>>,
        4,
    >,
    rng: &'static RngDriver,
    temp: &'static TemperatureDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    analog_comparator: &'static capsules_extra::analog_comparator::AnalogComparator<
        'static,
        nrf52840::acomp::Comparator<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52840::rtc::Rtc<'static>,
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
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_radio)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules_extra::analog_comparator::DRIVER_NUM => f(Some(self.analog_comparator)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    Platform,
    &'static nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>,
) {
    nrf52840::init();

    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // GPIOs
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            // left side of the USB plug
            0 => &nrf52840_peripherals.gpio_port[Pin::P0_13],
            1 => &nrf52840_peripherals.gpio_port[Pin::P0_15],
            2 => &nrf52840_peripherals.gpio_port[Pin::P0_17],
            3 => &nrf52840_peripherals.gpio_port[Pin::P0_20],
            4 => &nrf52840_peripherals.gpio_port[Pin::P0_22],
            5 => &nrf52840_peripherals.gpio_port[Pin::P0_24],
            6 => &nrf52840_peripherals.gpio_port[Pin::P1_00],
            7 => &nrf52840_peripherals.gpio_port[Pin::P0_09],
            8 => &nrf52840_peripherals.gpio_port[Pin::P0_10],
            // right side of the USB plug
            9 => &nrf52840_peripherals.gpio_port[Pin::P0_31],
            10 => &nrf52840_peripherals.gpio_port[Pin::P0_29],
            11 => &nrf52840_peripherals.gpio_port[Pin::P0_02],
            12 => &nrf52840_peripherals.gpio_port[Pin::P1_15],
            13 => &nrf52840_peripherals.gpio_port[Pin::P1_13],
            14 => &nrf52840_peripherals.gpio_port[Pin::P1_10],
            // Below the PCB
            15 => &nrf52840_peripherals.gpio_port[Pin::P0_26],
            16 => &nrf52840_peripherals.gpio_port[Pin::P0_04],
            17 => &nrf52840_peripherals.gpio_port[Pin::P0_11],
            18 => &nrf52840_peripherals.gpio_port[Pin::P0_14],
            19 => &nrf52840_peripherals.gpio_port[Pin::P1_11],
            20 => &nrf52840_peripherals.gpio_port[Pin::P1_07],
            21 => &nrf52840_peripherals.gpio_port[Pin::P1_01],
            22 => &nrf52840_peripherals.gpio_port[Pin::P1_04],
            23 => &nrf52840_peripherals.gpio_port[Pin::P1_02]
        ),
    )
    .finalize(components::gpio_component_static!(nrf52840::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840_peripherals.gpio_port[BUTTON_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_static!(
        nrf52840::gpio::GPIOPin
    ));

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, nrf52840::gpio::GPIOPin>,
        LedLow::new(&nrf52840_peripherals.gpio_port[LED1_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_R_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_G_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_B_PIN]),
    ));

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    nrf52_components::startup::NrfStartupComponent::new(
        false,
        BUTTON_RST_PIN,
        nrf52840::uicr::Regulator0Output::V3_0,
        &base_peripherals.nvmc,
    )
    .finalize(());

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let gpio_port = &nrf52840_peripherals.gpio_port;

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&gpio_port[LED2_R_PIN]),
        Some(&gpio_port[LED2_G_PIN]),
        Some(&gpio_port[LED2_B_PIN]),
    );

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
    let uart_channel = UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD));
    let channel = nrf52_components::UartChannelComponent::new(
        uart_channel,
        mux_alarm,
        &base_peripherals.uarte0,
    )
    .finalize(nrf52_components::uart_channel_component_static!(
        nrf52840::rtc::Rtc
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
    .finalize(components::process_console_component_static!(
        nrf52840::rtc::Rtc<'static>
    ));

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
        nrf52840::rtc::Rtc,
        nrf52840::ble_radio::Radio
    ));

    let aes_mux = static_init!(
        MuxAES128CCM<'static, nrf52840::aes::AesECB>,
        MuxAES128CCM::new(&base_peripherals.ecb,)
    );
    aes_mux.register();
    base_peripherals.ecb.set_client(aes_mux);

    let (ieee802154_radio, _mux_mac) = components::ieee802154::Ieee802154Component::new(
        board_kernel,
        capsules_extra::ieee802154::DRIVER_NUM,
        &nrf52840_peripherals.ieee802154_radio,
        aes_mux,
        PAN_ID,
        SRC_MAC,
        DEFAULT_EXT_SRC_MAC,
    )
    .finalize(components::ieee802154_component_static!(
        nrf52840::ieee802154_radio::Radio,
        nrf52840::aes::AesECB<'static>
    ));

    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        &base_peripherals.temp,
    )
    .finalize(components::temperature_component_static!(
        nrf52840::temperature::Temp
    ));

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &base_peripherals.trng,
    )
    .finalize(components::rng_component_static!(nrf52840::trng::Trng));

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let analog_comparator = components::analog_comparator::AnalogComparatorComponent::new(
        &base_peripherals.acomp,
        components::analog_comparator_component_helper!(
            nrf52840::acomp::Channel,
            &*addr_of!(nrf52840::acomp::CHANNEL_AC0)
        ),
        board_kernel,
        capsules_extra::analog_comparator::DRIVER_NUM,
    )
    .finalize(components::analog_comparator_component_static!(
        nrf52840::acomp::Comparator
    ));

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        button,
        ble_radio,
        ieee802154_radio,
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
    debug!("{}", &*addr_of!(nrf52840::ficr::FICR_INSTANCE));

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
