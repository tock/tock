//! Tock kernel for the Arduino Nano 33 BLE.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::ActivationMode::ActiveLow;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Interrupt;
use kernel::hil::gpio::Output;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::time::Counter;
use kernel::hil::usb::Client;
use kernel::mpu::MPU;
use kernel::Chip;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;

// Three-color LED.
const LED_RED_PIN: Pin = Pin::P0_24;
const LED_GREEN_PIN: Pin = Pin::P0_16;
const LED_BLUE_PIN: Pin = Pin::P0_06;

const LED_KERNEL_PIN: Pin = Pin::P0_13;

const _BUTTON_RST_PIN: Pin = Pin::P0_18;

const GPIO_D2: Pin = Pin::P1_11;
const GPIO_D3: Pin = Pin::P1_12;
const GPIO_D4: Pin = Pin::P1_15;
const GPIO_D5: Pin = Pin::P1_13;
const GPIO_D6: Pin = Pin::P1_14;
const GPIO_D7: Pin = Pin::P0_23;
const GPIO_D8: Pin = Pin::P0_21;
const GPIO_D9: Pin = Pin::P0_27;
const GPIO_D10: Pin = Pin::P1_02;

const _UART_TX_PIN: Pin = Pin::P1_03;
const _UART_RX_PIN: Pin = Pin::P1_10;

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_14;
const I2C_SCL_PIN: Pin = Pin::P0_15;

/// GPIO tied to the VCC of the I2C pullup resistors.
const I2C_PULLUP_PIN: Pin = Pin::P1_00;

/// Interrupt pin for the APDS9960 sensor.
const APDS9960_PIN: Pin = Pin::P0_19;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] = [None; 8];

nrf52840::create_default_nrf52840_peripherals!(Nrf52840Peripherals);
static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840Peripherals>> = None;
static mut CDC_REF_FOR_PANIC: Option<
    &'static capsules::usb::cdc::CdcAcm<'static, nrf52::usbd::Usbd>,
> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    // ble_radio: &'static capsules::ble_advertising_driver::BLE<
    //     'static,
    //     nrf52::ble_radio::Radio,
    //     VirtualMuxAlarm<'static, Rtc<'static>>,
    // >,
    // ieee802154_radio: &'static capsules::ieee802154::RadioDriver<'static>,
    console: &'static capsules::console::Console<'static>,
    proximity: &'static capsules::proximity::ProximitySensor<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led: &'static capsules::led::LED<'static, nrf52::gpio::GPIOPin<'static>>,
    rng: &'static capsules::rng::RngDriver<'static>,
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
            capsules::proximity::DRIVER_NUM => f(Some(self.proximity)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            // capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            // capsules::ieee802154::DRIVER_NUM => f(Some(radio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52840::init();
    let ppi = static_init!(nrf52840::ppi::Ppi, nrf52840::ppi::Ppi::new());
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(Nrf52840Peripherals, Nrf52840Peripherals::new(ppi));

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52_base;

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
    // `debug_gpio!(0, toggle)` macro. We configure these early so that the
    // macro is available during most of the setup code and kernel execution.
    kernel::debug::assign_gpios(
        Some(&base_peripherals.gpio_port[LED_KERNEL_PIN]),
        None,
        None,
    );

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            2 => &base_peripherals.gpio_port[GPIO_D2],
            3 => &base_peripherals.gpio_port[GPIO_D3],
            4 => &base_peripherals.gpio_port[GPIO_D4],
            5 => &base_peripherals.gpio_port[GPIO_D5],
            6 => &base_peripherals.gpio_port[GPIO_D6],
            7 => &base_peripherals.gpio_port[GPIO_D7],
            8 => &base_peripherals.gpio_port[GPIO_D8],
            9 => &base_peripherals.gpio_port[GPIO_D9],
            10 => &base_peripherals.gpio_port[GPIO_D10]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        nrf52840::gpio::GPIOPin,
        (&base_peripherals.gpio_port[LED_RED_PIN], ActiveLow),
        (&base_peripherals.gpio_port[LED_GREEN_PIN], ActiveLow),
        (&base_peripherals.gpio_port[LED_BLUE_PIN], ActiveLow)
    ))
    .finalize(components::led_component_buf!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Deferred Call (Dynamic) Setup
    //--------------------------------------------------------------------------

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
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

    // Setup the CDC-ACM over USB driver that we will use for UART.
    // We use the Arduino Vendor ID and Product ID since the device is the same.

    // Create the strings we include in the USB descriptor. We use the hardcoded
    // DEVICEADDR register on the nRF52 to set the serial number.
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str =
        nrf52::ficr::FICR_INSTANCE.address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Arduino",              // Manufacturer
            "Nano 33 BLE - TockOS", // Product
            serial_number_string,   // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &nrf52840_peripherals.usbd,
        capsules::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x2341,
        0x005a,
        strings,
    )
    .finalize(components::usb_cdc_acm_component_helper!(nrf52::usbd::Usbd));
    CDC_REF_FOR_PANIC = Some(cdc); //for use by panic handler

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(cdc, 115200, dynamic_deferred_caller)
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

    let sensors_i2c_bus = static_init!(
        capsules::virtual_i2c::MuxI2C<'static>,
        capsules::virtual_i2c::MuxI2C::new(&base_peripherals.twim0, None, dynamic_deferred_caller)
    );
    base_peripherals.twim0.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );
    base_peripherals.twim0.set_master_client(sensors_i2c_bus);

    &nrf52840::gpio::PORT[I2C_PULLUP_PIN].make_output();
    &nrf52840::gpio::PORT[I2C_PULLUP_PIN].set();

    let apds9960_i2c = static_init!(
        capsules::virtual_i2c::I2CDevice,
        capsules::virtual_i2c::I2CDevice::new(sensors_i2c_bus, 0x39 << 1)
    );

    let apds9960 = static_init!(
        capsules::apds9960::APDS9960<'static>,
        capsules::apds9960::APDS9960::new(
            apds9960_i2c,
            &nrf52840::gpio::PORT[APDS9960_PIN],
            &mut capsules::apds9960::BUFFER
        )
    );
    apds9960_i2c.set_client(apds9960);
    nrf52840::gpio::PORT[APDS9960_PIN].set_client(apds9960);

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let proximity = static_init!(
        capsules::proximity::ProximitySensor<'static>,
        capsules::proximity::ProximitySensor::new(apds9960, board_kernel.create_grant(&grant_cap))
    );

    kernel::hil::sensors::ProximityDriver::set_client(apds9960, proximity);

    //--------------------------------------------------------------------------
    // WIRELESS
    //--------------------------------------------------------------------------

    // let ble_radio =
    //     BLEComponent::new(board_kernel, &base_peripherals.ble_radio, mux_alarm).finalize(());

    // let (ieee802154_radio, _) = Ieee802154Component::new(
    //     board_kernel,
    //     &base_peripherals.ieee802154_radio,
    //     PAN_ID,
    //     SRC_MAC,
    // )
    // .finalize(());

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52_components::NrfClockComponent::new().finalize(());

    let platform = Platform {
        // ble_radio: ble_radio,
        // ieee802154_radio: ieee802154_radio,
        console: console,
        proximity: proximity,
        led: led,
        gpio: gpio,
        rng: rng,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840Peripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    // Need to disable the MPU because the bootloader seems to set it up.
    chip.mpu().clear_mpu();

    // Configure the USB stack to enable a serial port over CDC-ACM.
    cdc.enable();
    cdc.attach();

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
