// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![deny(missing_docs)]

use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_core::virtualizers::virtual_uart::MuxUart;
use kernel::component::Component;
use kernel::hil::led::LedLow;
use kernel::hil::time::Counter;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::static_init;
use kernel::{capabilities, create_capability};
use nrf52_components::{UartChannel, UartPins};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

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

/// Debug Writer
pub mod io;

/// How should the kernel respond when a process faults.
pub const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// Number of concurrent processes this platform supports.
pub const NUM_PROCS: usize = 8;

/// Type of the nrf52840 chip.
pub type ChipHw = nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>;

kernel::stack_size! {0x2000}

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type ConsoleDriver = components::console::ConsoleComponentType;

type Led = kernel::hil::led::LedLow<'static, nrf52840::gpio::GPIOPin<'static>>;
type LedDriver = components::led::LedsComponentType<Led, 4>;

type AlarmDriver = components::alarm::AlarmDriverComponentType<nrf52840::rtc::Rtc<'static>>;

type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

/// Supported drivers by the platform
pub struct Platform {
    console: &'static ConsoleDriver,
    led: &'static LedDriver,
    alarm: &'static AlarmDriver,
    scheduler: &'static SchedulerInUse,
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
    let aes_ecb_buf = static_init!([u8; 48], [0; 48]);
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf, aes_ecb_buf)
    );

    nrf52840_peripherals
}

impl KernelResources<nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
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

/// Do baseline setup for a testing configuration board.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    Platform,
    &'static ChipHw,
    &'static Nrf52840DefaultPeripherals<'static>,
    &'static MuxUart<'static>,
    &'static MuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
) {
    //--------------------------------------------------------------------------
    // INITIAL SETUP
    //--------------------------------------------------------------------------

    // Apply errata fixes and enable interrupts.
    ChipHw::init();

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

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

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // Create (and save for panic debugging) a chip object to setup low-level
    // resources (e.g. MPU, systick).
    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );

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
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        Led,
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
        create_capability!(capabilities::MemoryAllocationCapability),
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
    let mux_uart = components::console::UartMuxComponent::new(uart_channel, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the serial console for userspace.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        mux_uart,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    //--------------------------------------------------------------------------
    // NRF CLOCK SETUP
    //--------------------------------------------------------------------------

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        console,
        led,
        alarm,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    (
        board_kernel,
        platform,
        chip,
        nrf52840_peripherals,
        mux_uart,
        mux_alarm,
    )
}
