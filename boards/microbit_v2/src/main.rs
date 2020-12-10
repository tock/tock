//! Tock kernel for the micro:bit v2.
//!
//! It is based on nRF52833 SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::time::Counter;
use kernel::mpu::MPU;
use kernel::Chip;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52833::gpio::Pin;
use nrf52833::interrupt_service::Nrf52832DefaultPeripherals;

// Buttons
const BUTTON_A: Pin = Pin::P0_14;
const BUTTON_B: Pin = Pin::P0_23;
const TOUCH_LOGO: Pin = Pin::P1_04;

const GPIO_D0: Pin = Pin::P0_02;
const GPIO_D1: Pin = Pin::P0_03;
const GPIO_D2: Pin = Pin::P0_04;
const GPIO_D8: Pin = Pin::P0_10;
const GPIO_D9: Pin = Pin::P0_09;
const GPIO_D16: Pin = Pin::P1_02;

const UART_TX_PIN: Pin = Pin::P0_06;
const UART_RX_PIN: Pin = Pin::P1_08;

/// LED matrix
const LED_MATRIX_COLS: [Pin; 5] = [Pin::P0_28, Pin::P0_11, Pin::P0_31, Pin::P1_05, Pin::P0_30];
const LED_MATRIX_ROWS: [Pin; 5] = [Pin::P0_21, Pin::P0_22, Pin::P0_15, Pin::P0_24, Pin::P0_19];

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_16;
const I2C_SCL_PIN: Pin = Pin::P0_08;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52833::chip::NRF52<Nrf52832DefaultPeripherals>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52::ble_radio::Radio<'static>,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led: &'static capsules::led_matrix::LedMatrixDriver<
        'static,
        nrf52::gpio::GPIOPin<'static>,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
    button: &'static capsules::button::Button<'static, nrf52::gpio::GPIOPin<'static>>,
    rng: &'static capsules::rng::RngDriver<'static>,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
    lsm303agr: &'static capsules::lsm303agr::Lsm303agrI2C<'static>,
    temperature: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::led_matrix::DRIVER_NUM => f(Some(self.led)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules::lsm303agr::DRIVER_NUM => f(Some(self.lsm303agr)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52833::init();
    let ppi = static_init!(nrf52833::ppi::Ppi, nrf52833::ppi::Ppi::new());
    // Initialize chip peripheral drivers
    let nrf52832_peripherals = static_init!(
        Nrf52832DefaultPeripherals,
        Nrf52832DefaultPeripherals::new(ppi)
    );

    // set up circular peripheral dependencies
    nrf52832_peripherals.init();

    let base_peripherals = &nrf52832_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    //--------------------------------------------------------------------------
    // CAPABILITIES
    //--------------------------------------------------------------------------

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    //--------------------------------------------------------------------------
    // DEBUG GPIO
    //--------------------------------------------------------------------------

    // Configure kernel debug GPIOs as early as possible. These are used by the
    // `debug_gpio!(0, toggle)` macro. We uconfigure these early so that the
    // macro is available during most of the setup code and kernel exection.
    // kernel::debug::assign_gpios(
    //     Some(&base_peripherals.gpio_port[LED_KERNEL_PIN]),
    //     None,
    //     None,
    // );

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52833::gpio::GPIOPin,
            0 => &base_peripherals.gpio_port[GPIO_D0],
            1 => &base_peripherals.gpio_port[GPIO_D1],
            2 => &base_peripherals.gpio_port[GPIO_D2],
            8 => &base_peripherals.gpio_port[GPIO_D8],
            9 => &base_peripherals.gpio_port[GPIO_D9],
            16 => &base_peripherals.gpio_port[GPIO_D16],
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52833::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Buttons
    //--------------------------------------------------------------------------
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            nrf52833::gpio::GPIOPin,
            (
                &base_peripherals.gpio_port[BUTTON_A],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ), // A
            (
                &base_peripherals.gpio_port[BUTTON_B],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ), // B
            (
                &base_peripherals.gpio_port[TOUCH_LOGO],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), // Touch Logo
        ),
    )
    .finalize(components::button_component_buf!(nrf52833::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Deferred Call (Dynamic) Setup
    //--------------------------------------------------------------------------

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 3], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &base_peripherals.rtc;
    rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52::rtc::Rtc));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    base_peripherals.uarte0.initialize(
        nrf52::pinmux::Pinmux::new(UART_TX_PIN as u32),
        nrf52::pinmux::Pinmux::new(UART_RX_PIN as u32),
        None,
        None,
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &base_peripherals.uarte0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    //--------------------------------------------------------------------------
    // RANDOM NUMBERS
    //--------------------------------------------------------------------------

    let rng = components::rng::RngComponent::new(board_kernel, &base_peripherals.trng).finalize(());

    //--------------------------------------------------------------------------
    // SENSORS
    //--------------------------------------------------------------------------

    base_peripherals.twim0.configure(
        nrf52833::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52833::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );

    // base_peripherals.twim0.enable ();

    let sensors_i2c_bus = static_init!(
        capsules::virtual_i2c::MuxI2C<'static>,
        capsules::virtual_i2c::MuxI2C::new(&base_peripherals.twim0, None, dynamic_deferred_caller)
    );

    base_peripherals.twim0.set_master_client(sensors_i2c_bus);

    // LSM303AGR

    let lsm303agr = components::lsm303agr::Lsm303agrI2CComponent::new().finalize(
        components::lsm303agr_i2c_component_helper!(
            sensors_i2c_bus,
            capsules::lsm303xx::ACCELEROMETER_BASE_ADDRESS << 1,
            capsules::lsm303xx::MAGNETOMETER_BASE_ADDRESS << 1
        ),
    );

    lsm303agr.configure(
        capsules::lsm303xx::Lsm303AccelDataRate::DataRate25Hz,
        false,
        capsules::lsm303xx::Lsm303Scale::Scale2G,
        false,
        true,
        capsules::lsm303xx::Lsm303MagnetoDataRate::DataRate3_0Hz,
        capsules::lsm303xx::Lsm303Range::Range1_9G,
    );

    let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
        .finalize(components::ninedof_component_helper!(lsm303agr));

    let temperature =
        components::temperature::TemperatureComponent::new(board_kernel, lsm303agr).finalize(());

    //--------------------------------------------------------------------------
    // WIRELESS
    //--------------------------------------------------------------------------

    let ble_radio =
        nrf52_components::BLEComponent::new(board_kernel, &base_peripherals.ble_radio, mux_alarm)
            .finalize(());

    //--------------------------------------------------------------------------
    // LED Matrix
    //--------------------------------------------------------------------------

    let led = components::led_matrix_component_helper!(
        nrf52833::gpio::GPIOPin,
        nrf52::rtc::Rtc<'static>,
        mux_alarm,
        @fps => 60,
        @cols => kernel::hil::gpio::ActivationMode::ActiveLow,
            &base_peripherals.gpio_port[LED_MATRIX_COLS[0]],
            &base_peripherals.gpio_port[LED_MATRIX_COLS[1]],
            &base_peripherals.gpio_port[LED_MATRIX_COLS[2]],
            &base_peripherals.gpio_port[LED_MATRIX_COLS[3]],
            &base_peripherals.gpio_port[LED_MATRIX_COLS[4]],
        @rows => kernel::hil::gpio::ActivationMode::ActiveHigh,
            &base_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
            &base_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
            &base_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
            &base_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
            &base_peripherals.gpio_port[LED_MATRIX_ROWS[4]]

    )
    .finalize(components::led_matrix_component_buf!(
        nrf52833::gpio::GPIOPin,
        nrf52::rtc::Rtc<'static>
    ));

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // it seems that microbit v2 has no external clock
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}

    let platform = Platform {
        ble_radio: ble_radio,
        console: console,
        gpio: gpio,
        button: button,
        led: led,
        rng: rng,
        temperature: temperature,
        lsm303agr: lsm303agr,
        ninedof: ninedof,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(
        nrf52833::chip::NRF52<Nrf52832DefaultPeripherals>,
        nrf52833::chip::NRF52::new(nrf52832_peripherals)
    );
    CHIP = Some(chip);

    // Need to disable the MPU because the bootloader seems to set it up.
    chip.mpu().clear_mpu();

    debug!("Initialization complete. Entering main loop.");

    //--------------------------------------------------------------------------
    // PROCESSES AND MAIN LOOP
    //--------------------------------------------------------------------------

    /// These symbols are defined in the linker script.
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

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.ipc),
        scheduler,
        &main_loop_capability,
    );
}
