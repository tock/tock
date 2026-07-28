// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Particle Boron.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::FloatingState;
use kernel::hil::i2c::{I2CMaster, I2CSlave};
use kernel::hil::led::LedLow;
use kernel::hil::time::Counter;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::single_thread_value::SingleThreadValue;
#[allow(unused_imports)]
use kernel::{capabilities, create_capability, debug, debug_gpio, debug_verbose, static_init};
#[allow(unused_imports)]
use nrf52_components::{self, UartChannel, UartPins};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// The Particle Boron LEDs
const LED_USR_PIN: Pin = Pin::P1_12;
const LED2_R_PIN: Pin = Pin::P0_13;
const LED2_G_PIN: Pin = Pin::P0_14;
const LED2_B_PIN: Pin = Pin::P0_15;

// The Particle Boron buttons
const BUTTON_PIN: Pin = Pin::P0_11;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

// UART Pins (CTS/RTS Unused)
const _UART_RTS: Option<Pin> = Some(Pin::P0_30);
const _UART_CTS: Option<Pin> = Some(Pin::P0_31);
const UART_TXD: Pin = Pin::P0_06;
const UART_RXD: Pin = Pin::P0_08;

// SPI pins not currently in use, but left here for convenience
const _SPI_MOSI: Pin = Pin::P1_13;
const _SPI_MISO: Pin = Pin::P1_14;
const _SPI_CLK: Pin = Pin::P1_15;

// I2C Pins
const I2C_SDA_PIN: Pin = Pin::P0_26;
const I2C_SCL_PIN: Pin = Pin::P0_27;

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

type ChipHw = nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x1000}

type TemperatureDriver =
    components::temperature::TemperatureComponentType<nrf52840::temperature::Temp<'static>>;
type RngDriver = components::rng::RngComponentType<nrf52840::trng::Trng<'static>>;

type Ieee802154Driver = components::ieee802154::Ieee802154ComponentType<
    nrf52840::ieee802154_radio::Radio<'static>,
    nrf52840::aes::AesECB<'static>,
>;

type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type BleHw = nrf52840::ble_radio::Radio<'static>;
type AlarmHw = nrf52840::rtc::Rtc<'static>;
type GpioHw = nrf52840::gpio::GPIOPin<'static>;
type LedHw = kernel::hil::led::LedLow<'static, nrf52840::gpio::GPIOPin<'static>>;
type I2cHw = nrf52840::i2c::TWI<'static>;

type BleDriver = components::ble::BLEComponentType<BleHw, AlarmHw>;
type AlarmDriver = components::alarm::AlarmDriverComponentType<AlarmHw>;
type GpioDriver = components::gpio::GpioComponentType<GpioHw>;
type LedDriver = components::led::LedsComponentType<LedHw, 4>;
type ButtonDriver = components::button::ButtonComponentType<GpioHw>;
type ConsoleDriver = components::console::ConsoleComponentType;
type AdcDriver = components::adc::AdcVirtualComponentType;
type I2CMasterSlaveDriver = components::i2c::I2CMasterSlaveDriverComponentType<I2cHw>;

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static BleDriver,
    ieee802154_radio: &'static Ieee802154Driver,
    button: &'static ButtonDriver,
    console: &'static ConsoleDriver,
    gpio: &'static GpioDriver,
    led: &'static LedDriver,
    adc: &'static AdcDriver,
    rng: &'static RngDriver,
    temp: &'static TemperatureDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    i2c_master_slave: &'static I2CMasterSlaveDriver,
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
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_radio)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temp)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::i2c_master_slave_driver::DRIVER_NUM => f(Some(self.i2c_master_slave)),
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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start_particle_boron() -> (
    &'static kernel::Kernel,
    Platform,
    &'static nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>,
) {
    ChipHw::init();

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Bind global variables to this thread.
    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    let nrf52840_peripherals = create_peripherals();

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    //--------------------------------------------------------------------------
    // CAPABILITIES
    //--------------------------------------------------------------------------

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    //--------------------------------------------------------------------------
    // DEBUG GPIO
    //--------------------------------------------------------------------------

    let gpio_port = &nrf52840_peripherals.gpio_port;
    // Configure kernel debug GPIOs as early as possible. These are used by the
    // `debug_gpio!(0, toggle)` macro. We configure these early so that the
    // macro is available during most of the setup code and kernel execution.
    let debug_gpios = static_init!(
        [&'static dyn kernel::hil::gpio::Pin; 1],
        [&gpio_port[LED2_R_PIN]]
    );
    kernel::debug::initialize_debug_gpio::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();
    kernel::debug::assign_gpios(debug_gpios);

    let uart_channel = UartChannel::Pins(UartPins::new(None, UART_TXD, None, UART_RXD));

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            GpioHw,
            // Left Side pins on mesh feather
            // A0 - ADC
            // 0 => &nrf52840_peripherals.gpio_port[Pin::P0_03],
            // A1 - ADC
            // 1 => &nrf52840_peripherals.gpio_port[Pin::P0_04],
            // A2 - ADC
            // 2 => &nrf52840_peripherals.gpio_port[Pin::P0_28],
            // A3 - ADC
            // 3 => &nrf52840_peripherals.gpio_port[Pin::P0_29],
            // A4 - ADC
            // 4 => &nrf52840_peripherals.gpio_port[Pin::P0_30],
            // A5 - ADC
            // 5 => &nrf52840_peripherals.gpio_port[Pin::P0_31],
            //D13
            6 => &nrf52840_peripherals.gpio_port[Pin::P1_15],
            //D12
            7 => &nrf52840_peripherals.gpio_port[Pin::P1_13],
            //D11
            8 => &nrf52840_peripherals.gpio_port[Pin::P1_14],
            //D10
            9 => &nrf52840_peripherals.gpio_port[Pin::P0_08],
            //D9
            10 => &nrf52840_peripherals.gpio_port[Pin::P0_06],
            // Right Side pins on mesh feather
            //D8
            11 => &nrf52840_peripherals.gpio_port[Pin::P1_03],
            //D7: Bound to LED_USR_PIN (Active Low)
            12 => &nrf52840_peripherals.gpio_port[Pin::P1_12],
            //D6
            13 => &nrf52840_peripherals.gpio_port[Pin::P1_11],
            //D5
            14 => &nrf52840_peripherals.gpio_port[Pin::P1_10],
            //D4
            15 => &nrf52840_peripherals.gpio_port[Pin::P1_08],
            //D3
            16 => &nrf52840_peripherals.gpio_port[Pin::P1_02],
            //D2
            17 => &nrf52840_peripherals.gpio_port[Pin::P0_01],
            //D1
            18 => &nrf52840_peripherals.gpio_port[Pin::P0_27],
            //D0
            19 => &nrf52840_peripherals.gpio_port[Pin::P0_26],
        ),
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::gpio_component_static!(GpioHw));

    //--------------------------------------------------------------------------
    // Buttons
    //--------------------------------------------------------------------------

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            GpioHw,
            (
                &nrf52840_peripherals.gpio_port[BUTTON_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::button_component_static!(GpioHw));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, GpioHw>,
        LedLow::new(&nrf52840_peripherals.gpio_port[LED_USR_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_R_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_G_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_B_PIN]),
    ));

    nrf52_components::startup::NrfStartupComponent::new(
        false,
        BUTTON_RST_PIN,
        nrf52840::uicr::Regulator0Output::V3_0,
        &base_peripherals.nvmc,
    )
    .finalize(());

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(AlarmHw));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(AlarmHw));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    let uart_channel = nrf52_components::UartChannelComponent::new(
        uart_channel,
        mux_alarm,
        &base_peripherals.uarte0,
    )
    .finalize(nrf52_components::uart_channel_component_static!(AlarmHw));

    // Process Printer for displaying process information.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(uart_channel, 115200)
        .finalize(components::uart_mux_component_static!(132));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!(132, 132));
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    //--------------------------------------------------------------------------
    // WIRELESS
    //--------------------------------------------------------------------------

    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        capsules_extra::ble_advertising_driver::DRIVER_NUM,
        &base_peripherals.ble_radio,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::ble_component_static!(AlarmHw, BleHw));

    let aes_mux = components::aes::AesMuxComponent::new(&base_peripherals.ecb)
        .finalize(components::aes_mux_component_static!(nrf52840::aes::AesECB));

    let (ieee802154_radio, _mux_mac) = components::ieee802154::Ieee802154Component::new(
        board_kernel,
        capsules_extra::ieee802154::DRIVER_NUM,
        &nrf52840_peripherals.ieee802154_radio,
        aes_mux,
        PAN_ID,
        SRC_MAC,
        DEFAULT_EXT_SRC_MAC,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::ieee802154_component_static!(
        nrf52840::ieee802154_radio::Radio,
        nrf52840::aes::AesECB<'static>
    ));

    //--------------------------------------------------------------------------
    // Sensor
    //--------------------------------------------------------------------------

    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        &base_peripherals.temp,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::temperature_component_static!(
        nrf52840::temperature::Temp
    ));

    //--------------------------------------------------------------------------
    // RANDOM NUMBERS
    //--------------------------------------------------------------------------

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &base_peripherals.trng,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::rng_component_static!(nrf52840::trng::Trng));

    //--------------------------------------------------------------------------
    // ADC
    //--------------------------------------------------------------------------

    base_peripherals.adc.calibrate();

    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc)
        .finalize(components::adc_mux_component_static!(nrf52840::adc::Adc));

    let adc_syscall = components::adc::AdcVirtualComponent::new(
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::adc_syscall_component_helper!(
        // BRD_A0
        components::adc::AdcComponent::new(
            adc_mux,
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput1)
        )
        .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
        // BRD_A1
        components::adc::AdcComponent::new(
            adc_mux,
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput2)
        )
        .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
        // BRD_A2
        components::adc::AdcComponent::new(
            adc_mux,
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput4)
        )
        .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
        // BRD_A3
        components::adc::AdcComponent::new(
            adc_mux,
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput5)
        )
        .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
        // BRD_A4
        components::adc::AdcComponent::new(
            adc_mux,
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput6)
        )
        .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
        // BRD_A5
        components::adc::AdcComponent::new(
            adc_mux,
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput7)
        )
        .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
    ));

    //--------------------------------------------------------------------------
    // I2C Master/Slave
    //--------------------------------------------------------------------------

    let i2c_master_buffer = static_init!([u8; 128], [0; 128]);
    let i2c_slave_buffer1 = static_init!([u8; 128], [0; 128]);
    let i2c_slave_buffer2 = static_init!([u8; 128], [0; 128]);

    let i2c_master_slave = static_init!(
        I2CMasterSlaveDriver,
        I2CMasterSlaveDriver::new(
            &base_peripherals.twi1,
            i2c_master_buffer,
            i2c_slave_buffer1,
            i2c_slave_buffer2,
            board_kernel.create_grant(
                capsules_core::i2c_master_slave_driver::DRIVER_NUM,
                &memory_allocation_capability
            ),
        )
    );
    base_peripherals.twi1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN),
    );
    base_peripherals.twi1.set_master_client(i2c_master_slave);
    base_peripherals.twi1.set_slave_client(i2c_master_slave);
    // Note: strongly suggested to use external pull-ups for higher speeds
    //       to maintain signal integrity.
    base_peripherals.twi1.set_speed(nrf52840::i2c::Speed::K400);

    // I2C pin cfg for target
    nrf52840_peripherals.gpio_port[I2C_SDA_PIN].set_i2c_pin_cfg();
    nrf52840_peripherals.gpio_port[I2C_SCL_PIN].set_i2c_pin_cfg();
    // Enable internal pull-ups
    nrf52840_peripherals.gpio_port[I2C_SDA_PIN].set_floating_state(FloatingState::PullUp);
    nrf52840_peripherals.gpio_port[I2C_SCL_PIN].set_floating_state(FloatingState::PullUp);

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        button,
        ble_radio,
        ieee802154_radio,
        console,
        led,
        gpio,
        adc: adc_syscall,
        rng,
        temp,
        alarm,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        i2c_master_slave,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    debug!("Particle Boron: Initialization complete. Entering main loop\r");

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
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start_particle_boron();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
