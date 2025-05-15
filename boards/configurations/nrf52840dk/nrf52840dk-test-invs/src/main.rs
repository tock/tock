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
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{capabilities, create_capability, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52_components::{UartChannel, UartPins};

mod invs_permissions;

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_13;
const LED2_PIN: Pin = Pin::P0_14;
const LED3_PIN: Pin = Pin::P0_15;
const LED4_PIN: Pin = Pin::P0_16;

const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

const SPI_MOSI: Pin = Pin::P0_20;
const SPI_MISO: Pin = Pin::P0_21;
const SPI_CLK: Pin = Pin::P0_19;

const SPI_MX25R6435F_CHIP_SELECT: Pin = Pin::P0_17;
const SPI_MX25R6435F_WRITE_PROTECT_PIN: Pin = Pin::P0_22;
const SPI_MX25R6435F_HOLD_PIN: Pin = Pin::P0_23;

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

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

const APP_STORAGE_REGION_SIZE: usize = 4096;

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type AlarmDriver = components::alarm::AlarmDriverComponentType<nrf52840::rtc::Rtc<'static>>;

type Mx25r6435f = components::mx25r6435f::Mx25r6435fComponentType<
    nrf52840::spi::SPIM<'static>,
    nrf52840::gpio::GPIOPin<'static>,
    nrf52840::rtc::Rtc<'static>,
>;
type InvsDriver = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponentType<
    APP_STORAGE_REGION_SIZE,
>;

/// Supported drivers by the platform
pub struct Platform {
    console: &'static capsules_core::console::Console<'static>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        kernel::hil::led::LedLow<'static, nrf52840::gpio::GPIOPin<'static>>,
        4,
    >,
    alarm: &'static AlarmDriver,
    invs: &'static InvsDriver,
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
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM => f(Some(self.invs)),
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

    // Choose the channel for serial output. This board can be configured to use
    // either the Segger RTT channel or via UART with traditional TX/RX GPIO
    // pins.
    let uart_channel = UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD));

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

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

    //--------------------------------------------------------------------------
    // ONBOARD EXTERNAL FLASH
    //--------------------------------------------------------------------------

    let mux_spi = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_static!(nrf52840::spi::SPIM));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(SPI_MOSI as u32),
        nrf52840::pinmux::Pinmux::new(SPI_MISO as u32),
        nrf52840::pinmux::Pinmux::new(SPI_CLK as u32),
    );

    let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
        Some(&nrf52840_peripherals.gpio_port[SPI_MX25R6435F_WRITE_PROTECT_PIN]),
        Some(&nrf52840_peripherals.gpio_port[SPI_MX25R6435F_HOLD_PIN]),
        &nrf52840_peripherals.gpio_port[SPI_MX25R6435F_CHIP_SELECT],
        mux_alarm,
        mux_spi,
    )
    .finalize(components::mx25r6435f_component_static!(
        nrf52840::spi::SPIM,
        nrf52840::gpio::GPIOPin,
        nrf52840::rtc::Rtc
    ));

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    let invs = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM,
        mx25r6435f,
        0x40000,  // start address
        0x100000, // length
    )
    .finalize(components::isolated_nonvolatile_storage_component_static!(
        Mx25r6435f,
        APP_STORAGE_REGION_SIZE
    ));

    //--------------------------------------------------------------------------
    // NRF CLOCK SETUP
    //--------------------------------------------------------------------------

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    //--------------------------------------------------------------------------
    // Credential Checking
    //--------------------------------------------------------------------------

    // Create the software-based SHA engine.
    let sha = components::sha::ShaSoftware256Component::new()
        .finalize(components::sha_software_256_component_static!());

    // Create the credential checker.
    let checking_policy = components::appid::checker_sha::AppCheckerSha256Component::new(sha)
        .finalize(components::app_checker_sha256_component_static!());

    // Create the AppID assigner.
    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    // We use a custom storage permissions assigner that is based on the TBF
    // header if present, and otherwise defaults to allowing apps to access
    // their own state.

    #[derive(Clone)]
    pub struct AppStoreCapability;
    unsafe impl capabilities::ApplicationStorageCapability for AppStoreCapability {}

    let storage_permissions_policy = static_init!(
        invs_permissions::InvsStoragePermissions<
            nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
            kernel::process::ProcessStandardDebugFull,
            AppStoreCapability,
        >,
        invs_permissions::InvsStoragePermissions::new(AppStoreCapability)
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
    let _loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
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
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        console,
        led,
        alarm,
        invs,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
