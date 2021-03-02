//! Tock kernel for the Adafruit CLUE nRF52480 Express.
//!
//! It is based on nRF52840 Express SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use capsules::virtual_aes_ccm::MuxAES128CCM;
use capsules::virtual_alarm::VirtualMuxAlarm;

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::Interrupt;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::symmetric_encryption::AES128;
use kernel::hil::time::Alarm;
use kernel::hil::time::Counter;
use kernel::hil::usb::Client;
use kernel::mpu::MPU;
use kernel::Chip;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// LEDs.
const LED_RED_PIN: Pin = Pin::P1_01;
const LED_WHITE_PIN: Pin = Pin::P0_10;

const LED_KERNEL_PIN: Pin = Pin::P1_01;

// Speaker
const SPEAKER_PIN: Pin = Pin::P1_00;

// Buttons
const BUTTON_LEFT: Pin = Pin::P1_02;
const BUTTON_RIGHT: Pin = Pin::P1_10;

const GPIO_D2: Pin = Pin::P0_03;
const GPIO_D3: Pin = Pin::P0_28;
const GPIO_D4: Pin = Pin::P0_02;
const GPIO_D6: Pin = Pin::P1_09;
const GPIO_D7: Pin = Pin::P0_07;
const GPIO_D8: Pin = Pin::P1_04;
const GPIO_D9: Pin = Pin::P0_27;
const GPIO_D10: Pin = Pin::P0_30;
const GPIO_D12: Pin = Pin::P0_31;

const _UART_TX_PIN: Pin = Pin::P0_05;
const _UART_RX_PIN: Pin = Pin::P0_04;

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_24;
const I2C_SCL_PIN: Pin = Pin::P0_25;

/// Interrupt pin for the APDS9960 sensor.
const APDS9960_PIN: Pin = Pin::P0_09;

/// Personal Area Network ID for the IEEE 802.15.4 radio
const PAN_ID: u16 = 0xABCD;

/// TFT ST7789H2
const ST7789H2_SCK: Pin = Pin::P0_14;
const ST7789H2_MOSI: Pin = Pin::P0_15;
const ST7789H2_MISO: Pin = Pin::P0_26; // ST7789H2 has no MISO Pin, but SPI requires a MISO Pin
const ST7789H2_CS: Pin = Pin::P0_12;
const ST7789H2_DC: Pin = Pin::P0_13;
const ST7789H2_RESET: Pin = Pin::P1_03;

/// TFT backlight
const _ST7789H2_LITE: Pin = Pin::P1_05;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;
static mut CDC_REF_FOR_PANIC: Option<
    &'static capsules::usb::cdc::CdcAcm<
        'static,
        nrf52::usbd::Usbd,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
    >,
> = None;

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
    ieee802154_radio: &'static capsules::ieee802154::RadioDriver<'static>,
    console: &'static capsules::console::Console<'static>,
    proximity: &'static capsules::proximity::ProximitySensor<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led:
        &'static capsules::led::LedDriver<'static, LedHigh<'static, nrf52::gpio::GPIOPin<'static>>>,
    button: &'static capsules::button::Button<'static, nrf52::gpio::GPIOPin<'static>>,
    screen: &'static capsules::screen::Screen<'static>,
    rng: &'static capsules::rng::RngDriver<'static>,
    ipc: kernel::ipc::IPC<NUM_PROCS>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
    buzzer: &'static capsules::buzzer_driver::Buzzer<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    temperature: &'static capsules::temperature::TemperatureSensor<'static>,
    humidity: &'static capsules::humidity::HumiditySensor<'static>,
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
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::screen::DRIVER_NUM => f(Some(self.screen)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_radio)),
            capsules::buzzer_driver::DRIVER_NUM => f(Some(self.buzzer)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules::humidity::DRIVER_NUM => f(Some(self.humidity)),
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
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ppi)
    );

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();

    let base_peripherals = &nrf52840_peripherals.nrf52;

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
        Some(&nrf52840_peripherals.gpio_port[LED_KERNEL_PIN]),
        None,
        None,
    );

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules::gpio::DRIVER_NUM as u32,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            2 => &nrf52840_peripherals.gpio_port[GPIO_D2],
            3 => &nrf52840_peripherals.gpio_port[GPIO_D3],
            4 => &nrf52840_peripherals.gpio_port[GPIO_D4],
            6 => &nrf52840_peripherals.gpio_port[GPIO_D6],
            7 => &nrf52840_peripherals.gpio_port[GPIO_D7],
            8 => &nrf52840_peripherals.gpio_port[GPIO_D8],
            9 => &nrf52840_peripherals.gpio_port[GPIO_D9],
            10 => &nrf52840_peripherals.gpio_port[GPIO_D10],
            12 => &nrf52840_peripherals.gpio_port[GPIO_D12]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        LedHigh<'static, nrf52840::gpio::GPIOPin>,
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED_RED_PIN]),
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED_WHITE_PIN])
    ))
    .finalize(components::led_component_buf!(
        LedHigh<'static, nrf52840::gpio::GPIOPin>
    ));

    //--------------------------------------------------------------------------
    // Buttons
    //--------------------------------------------------------------------------
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules::button::DRIVER_NUM as u32,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840_peripherals.gpio_port[BUTTON_LEFT],
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullUp
            ), // Left
            (
                &nrf52840_peripherals.gpio_port[BUTTON_RIGHT],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ) // Right
        ),
    )
    .finalize(components::button_component_buf!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Deferred Call (Dynamic) Setup
    //--------------------------------------------------------------------------

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 4], Default::default());
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
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules::alarm::DRIVER_NUM as u32,
        mux_alarm,
    )
    .finalize(components::alarm_component_helper!(nrf52::rtc::Rtc));

    //--------------------------------------------------------------------------
    // PWM & BUZZER
    //--------------------------------------------------------------------------

    let mux_pwm = static_init!(
        capsules::virtual_pwm::MuxPwm<'static, nrf52840::pwm::Pwm>,
        capsules::virtual_pwm::MuxPwm::new(&base_peripherals.pwm0)
    );
    let virtual_pwm_buzzer = static_init!(
        capsules::virtual_pwm::PwmPinUser<'static, nrf52840::pwm::Pwm>,
        capsules::virtual_pwm::PwmPinUser::new(
            mux_pwm,
            nrf52840::pinmux::Pinmux::new(SPEAKER_PIN as u32)
        )
    );
    virtual_pwm_buzzer.add_to_mux();

    let virtual_alarm_buzzer = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let buzzer = static_init!(
        capsules::buzzer_driver::Buzzer<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        >,
        capsules::buzzer_driver::Buzzer::new(
            virtual_pwm_buzzer,
            virtual_alarm_buzzer,
            capsules::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
            board_kernel.create_grant(
                capsules::buzzer_driver::DRIVER_NUM as u32,
                &memory_allocation_capability
            )
        )
    );
    virtual_alarm_buzzer.set_alarm_client(buzzer);

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    // Setup the CDC-ACM over USB driver that we will use for UART.
    // We use the Adafruit Vendor ID and Product ID since the device is the same.

    // Create the strings we include in the USB descriptor. We use the hardcoded
    // DEVICEADDR register on the nRF52 to set the serial number.
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str =
        nrf52::ficr::FICR_INSTANCE.address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Adafruit",               // Manufacturer
            "CLUE nRF52840 - TockOS", // Product
            serial_number_string,     // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &nrf52840_peripherals.usbd,
        capsules::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x239a,
        0x8071,
        strings,
        mux_alarm,
        dynamic_deferred_caller,
        None,
    )
    .finalize(components::usb_cdc_acm_component_helper!(
        nrf52::usbd::Usbd,
        nrf52::rtc::Rtc
    ));
    CDC_REF_FOR_PANIC = Some(cdc); //for use by panic handler

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(cdc, 115200, dynamic_deferred_caller)
        .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM as u32,
        uart_mux,
    )
    .finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    //--------------------------------------------------------------------------
    // RANDOM NUMBERS
    //--------------------------------------------------------------------------

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules::rng::DRIVER_NUM as u32,
        &base_peripherals.trng,
    )
    .finalize(());

    //--------------------------------------------------------------------------
    // SENSORS
    //--------------------------------------------------------------------------

    let sensors_i2c_bus = static_init!(
        capsules::virtual_i2c::MuxI2C<'static>,
        capsules::virtual_i2c::MuxI2C::new(&base_peripherals.twim1, None, dynamic_deferred_caller)
    );
    base_peripherals.twim1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );
    base_peripherals.twim1.set_master_client(sensors_i2c_bus);

    let apds9960_i2c = static_init!(
        capsules::virtual_i2c::I2CDevice,
        capsules::virtual_i2c::I2CDevice::new(sensors_i2c_bus, 0x39)
    );

    let apds9960 = static_init!(
        capsules::apds9960::APDS9960<'static>,
        capsules::apds9960::APDS9960::new(
            apds9960_i2c,
            &nrf52840_peripherals.gpio_port[APDS9960_PIN],
            &mut capsules::apds9960::BUFFER
        )
    );
    apds9960_i2c.set_client(apds9960);
    nrf52840_peripherals.gpio_port[APDS9960_PIN].set_client(apds9960);

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let proximity = static_init!(
        capsules::proximity::ProximitySensor<'static>,
        capsules::proximity::ProximitySensor::new(
            apds9960,
            board_kernel.create_grant(capsules::proximity::DRIVER_NUM as u32, &grant_cap)
        )
    );

    kernel::hil::sensors::ProximityDriver::set_client(apds9960, proximity);

    let sht3x = components::sht3x::SHT3xComponent::new(sensors_i2c_bus, mux_alarm).finalize(
        components::sht3x_component_helper!(nrf52::rtc::Rtc<'static>, capsules::sht3x::BASE_ADDR),
    );

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules::temperature::DRIVER_NUM as u32,
        sht3x,
    )
    .finalize(());

    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules::humidity::DRIVER_NUM as u32,
        sht3x,
    )
    .finalize(());

    //--------------------------------------------------------------------------
    // TFT
    //--------------------------------------------------------------------------

    let spi_mux = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_helper!(nrf52840::spi::SPIM));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(ST7789H2_MOSI as u32),
        nrf52840::pinmux::Pinmux::new(ST7789H2_MISO as u32),
        nrf52840::pinmux::Pinmux::new(ST7789H2_SCK as u32),
    );

    let bus = components::bus::SpiMasterBusComponent::new().finalize(
        components::spi_bus_component_helper!(
            // spi type
            nrf52840::spi::SPIM,
            // chip select
            &nrf52840_peripherals.gpio_port[ST7789H2_CS],
            // spi mux
            spi_mux
        ),
    );

    let tft = components::st77xx::ST77XXComponent::new(mux_alarm).finalize(
        components::st77xx_component_helper!(
            // screen
            &capsules::st77xx::ST7789H2,
            // bus type
            capsules::bus::SpiMasterBus<
                'static,
                VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
            >,
            // bus
            &bus,
            // timer type
            nrf52840::rtc::Rtc,
            // pin type
            nrf52::gpio::GPIOPin<'static>,
            // dc
            Some(&nrf52840_peripherals.gpio_port[ST7789H2_DC]),
            // reset
            &nrf52840_peripherals.gpio_port[ST7789H2_RESET]
        ),
    );

    tft.init();

    let screen = components::screen::ScreenComponent::new(
        board_kernel,
        capsules::screen::DRIVER_NUM as u32,
        tft,
        Some(tft),
    )
    .finalize(components::screen_buffer_size!(57600));

    //--------------------------------------------------------------------------
    // WIRELESS
    //--------------------------------------------------------------------------

    let ble_radio = nrf52_components::BLEComponent::new(
        board_kernel,
        capsules::ble_advertising_driver::DRIVER_NUM as u32,
        &base_peripherals.ble_radio,
        mux_alarm,
    )
    .finalize(());

    let aes_mux = static_init!(
        MuxAES128CCM<'static, nrf52840::aes::AesECB>,
        MuxAES128CCM::new(&base_peripherals.ecb, dynamic_deferred_caller)
    );
    base_peripherals.ecb.set_client(aes_mux);
    aes_mux.initialize_callback_handle(
        dynamic_deferred_caller
            .register(aes_mux)
            .expect("no deferred call slot available for ccm mux"),
    );

    let serial_num = nrf52840::ficr::FICR_INSTANCE.address();

    let serial_num_bottom_16 = u16::from_le_bytes([serial_num[0], serial_num[1]]);

    let (ieee802154_radio, _mux_mac) = components::ieee802154::Ieee802154Component::new(
        board_kernel,
        capsules::ieee802154::DRIVER_NUM as u32,
        &base_peripherals.ieee802154_radio,
        aes_mux,
        PAN_ID,
        serial_num_bottom_16,
        dynamic_deferred_caller,
    )
    .finalize(components::ieee802154_component_helper!(
        nrf52840::ieee802154_radio::Radio,
        nrf52840::aes::AesECB<'static>
    ));

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let platform = Platform {
        ble_radio: ble_radio,
        ieee802154_radio: ieee802154_radio,
        console: console,
        proximity: proximity,
        led: led,
        gpio: gpio,
        screen: screen,
        button: button,
        rng: rng,
        buzzer: buzzer,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM as u32,
            &memory_allocation_capability,
        ),
        temperature: temperature,
        humidity: humidity,
    };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
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
