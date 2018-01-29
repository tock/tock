//! Board file for Hail development platform.
//!
//! - https://github.com/helena-project/tock/tree/master/boards/hail
//! - https://github.com/lab11/hail

#![no_std]
#![no_main]
#![feature(asm, const_fn, lang_items, compiler_builtins_lib)]

extern crate capsules;
extern crate compiler_builtins;
#[allow(unused_imports)]
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate sam4l;
extern crate sensys;

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use kernel::Platform;
use kernel::hil;
use kernel::hil::Controller;
use kernel::hil::spi::SpiMaster;

#[macro_use]
pub mod io;

static mut SPI_READ_BUF: [u8; 64] = [0; 64];
static mut SPI_WRITE_BUF: [u8; 64] = [0; 64];

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 49152] = [0; 49152];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None, None, None, None];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct Hail {
    console: &'static capsules::console::Console<'static, sam4l::usart::USART>,
    sensys: &'static sensys::Sensys<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    gpio: &'static capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
    >,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
    humidity: &'static capsules::humidity::HumiditySensor<'static>,
    spi: &'static capsules::spi::Spi<'static, VirtualSpiMasterDevice<'static, sam4l::spi::Spi>>,
    nrf51822: &'static capsules::nrf51822_serialization::Nrf51822Serialization<
        'static,
        sam4l::usart::USART,
    >,
    adc: &'static capsules::adc::Adc<'static, sam4l::adc::Adc>,
    led: &'static capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
    rng: &'static capsules::rng::SimpleRng<'static, sam4l::trng::Trng<'static>>,
    ipc: kernel::ipc::IPC,
    crc: &'static capsules::crc::Crc<'static, sam4l::crccu::Crccu<'static>>,
    dac: &'static capsules::dac::Dac<'static>,
    aes: &'static capsules::symmetric_encryption::Crypto<'static, sam4l::aes::Aes>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for Hail {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),

            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::spi::DRIVER_NUM => f(Some(self.spi)),
            capsules::nrf51822_serialization::DRIVER_NUM => f(Some(self.nrf51822)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),

            capsules::rng::DRIVER_NUM => f(Some(self.rng)),

            capsules::crc::DRIVER_NUM => f(Some(self.crc)),
            capsules::symmetric_encryption::DRIVER_NUM => f(Some(self.aes)),

            capsules::dac::DRIVER_NUM => f(Some(self.dac)),

            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::{PA, PB};
    use sam4l::gpio::PeripheralFunction::{A, B};

    PA[04].configure(Some(A)); // A0 - ADC0
    PA[05].configure(Some(A)); // A1 - ADC1
    PA[06].configure(Some(A)); // DAC
    PA[07].configure(None); //... WKP - Wakeup
    PA[08].configure(Some(A)); // FTDI_RTS - USART0 RTS
    PA[09].configure(None); //... ACC_INT1 - FXOS8700CQ Interrupt 1
    PA[10].configure(None); //... unused
    PA[11].configure(Some(A)); // FTDI_OUT - USART0 RX FTDI->SAM4L
    PA[12].configure(Some(A)); // FTDI_IN - USART0 TX SAM4L->FTDI
    PA[13].configure(None); //... RED_LED
    PA[14].configure(None); //... BLUE_LED
    PA[15].configure(None); //... GREEN_LED
    PA[16].configure(None); //... BUTTON - User Button
    PA[17].configure(None); //... !NRF_RESET - Reset line for nRF51822
    PA[18].configure(None); //... ACC_INT2 - FXOS8700CQ Interrupt 2
    PA[19].configure(None); //... unused
    PA[20].configure(None); //... !LIGHT_INT - ISL29035 Light Sensor Interrupt
                            // SPI Mode
    PA[21].configure(Some(A)); // D3 - SPI MISO
    PA[22].configure(Some(A)); // D2 - SPI MOSI
    PA[23].configure(Some(A)); // D4 - SPI SCK
    PA[24].configure(Some(A)); // D5 - SPI CS0
                               // // I2C MODE
                               // PA[21].configure(None); // D3
                               // PA[22].configure(None); // D2
                               // PA[23].configure(Some(B)); // D4 - TWIMS0 SDA
                               // PA[24].configure(Some(B)); // D5 - TWIMS0 SCL
                               // UART Mode
    PA[25].configure(Some(B)); // RX - USART2 RXD
    PA[26].configure(Some(B)); // TX - USART2 TXD

    PB[00].configure(Some(A)); // SENSORS_SDA - TWIMS1 SDA
    PB[01].configure(Some(A)); // SENSORS_SCL - TWIMS1 SCL
    PB[02].configure(Some(A)); // A2 - ADC3
    PB[03].configure(Some(A)); // A3 - ADC4
    PB[04].configure(Some(A)); // A4 - ADC5
    PB[05].configure(Some(A)); // A5 - ADC6
    PB[06].configure(Some(A)); // NRF_CTS - USART3 RTS
    PB[07].configure(Some(A)); // NRF_RTS - USART3 CTS
    PB[08].configure(None); //... NRF_INT - Interrupt line nRF->SAM4L
    PB[09].configure(Some(A)); // NRF_OUT - USART3 RXD
    PB[10].configure(Some(A)); // NRF_IN - USART3 TXD
    PB[11].configure(None); //... D6
    PB[12].configure(None); //... D7
    PB[13].configure(None); //... unused
    PB[14].configure(None); //... D0
    PB[15].configure(None); //... D1
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the SAM4L chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    sam4l::pm::PM.setup_system_clock(sam4l::pm::SystemClockSource::PllExternalOscillatorAt48MHz {
        frequency: sam4l::pm::OscillatorFrequency::Frequency16MHz,
        startup_mode: sam4l::pm::OscillatorStartup::FastStart,
    });

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();

    let mut chip = sam4l::chip::Sam4l::new();

    ///////////////////////////////////////////////////////////////////
    // Begin capsule creation and initialization

    let console = static_init!(
        capsules::console::Console<sam4l::usart::USART>,
        capsules::console::Console::new(
            &sam4l::usart::USART0,
            115200,
            &mut capsules::console::WRITE_BUF,
            kernel::Grant::create()
        )
    );
    hil::uart::UART::set_client(&sam4l::usart::USART0, console);

    // Create the Nrf51822Serialization driver for passing BLE commands
    // over UART to the nRF51822 radio.
    let nrf_serialization = static_init!(
        capsules::nrf51822_serialization::Nrf51822Serialization<sam4l::usart::USART>,
        capsules::nrf51822_serialization::Nrf51822Serialization::new(
            &sam4l::usart::USART3,
            &mut capsules::nrf51822_serialization::WRITE_BUF,
            &mut capsules::nrf51822_serialization::READ_BUF
        )
    );
    hil::uart::UART::set_client(&sam4l::usart::USART3, nrf_serialization);

    let ast = &sam4l::ast::AST;

    let mux_alarm = static_init!(
        MuxAlarm<'static, sam4l::ast::Ast>,
        MuxAlarm::new(&sam4l::ast::AST)
    );
    ast.configure(mux_alarm);

    let sensors_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(&sam4l::i2c::I2C1));
    sam4l::i2c::I2C1.set_master_client(sensors_i2c);

    // SI7021 Temperature / Humidity Sensor, address: 0x40
    let si7021_i2c = static_init!(
        capsules::virtual_i2c::I2CDevice,
        capsules::virtual_i2c::I2CDevice::new(sensors_i2c, 0x40)
    );
    let si7021_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let si7021 = static_init!(
        capsules::si7021::SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        capsules::si7021::SI7021::new(
            si7021_i2c,
            si7021_virtual_alarm,
            &mut capsules::si7021::BUFFER
        )
    );
    si7021_i2c.set_client(si7021);
    si7021_virtual_alarm.set_client(si7021);

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(si7021, kernel::Grant::create()),
        96 / 8
    );
    kernel::hil::sensors::TemperatureDriver::set_client(si7021, temp);

    let humidity = static_init!(
        capsules::humidity::HumiditySensor<'static>,
        capsules::humidity::HumiditySensor::new(si7021, kernel::Grant::create()),
        96 / 8
    );
    kernel::hil::sensors::HumidityDriver::set_client(si7021, humidity);

    // Configure the ISL29035, device address 0x44
    let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(sensors_i2c, 0x44));
    let isl29035_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let isl29035 = static_init!(
        capsules::isl29035::Isl29035<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        capsules::isl29035::Isl29035::new(
            isl29035_i2c,
            isl29035_virtual_alarm,
            &mut capsules::isl29035::BUF
        )
    );
    isl29035_i2c.set_client(isl29035);
    isl29035_virtual_alarm.set_client(isl29035);

    let sensys_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let sensys = static_init!(
        sensys::Sensys<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        sensys::Sensys::new(sensys_virtual_alarm, isl29035)
    );
    hil::sensors::AmbientLight::set_client(isl29035, sensys);
    sensys_virtual_alarm.set_client(sensys);

    // Alarm
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, kernel::Grant::create())
    );
    virtual_alarm1.set_client(alarm);

    // FXOS8700CQ accelerometer, device address 0x1e
    let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(sensors_i2c, 0x1e));
    let fxos8700 = static_init!(
        capsules::fxos8700cq::Fxos8700cq<'static>,
        capsules::fxos8700cq::Fxos8700cq::new(
            fxos8700_i2c,
            &sam4l::gpio::PA[9],
            &mut capsules::fxos8700cq::BUF
        )
    );
    fxos8700_i2c.set_client(fxos8700);
    sam4l::gpio::PA[9].set_client(fxos8700);

    let ninedof = static_init!(
        capsules::ninedof::NineDof<'static>,
        capsules::ninedof::NineDof::new(fxos8700, kernel::Grant::create())
    );
    hil::sensors::NineDof::set_client(fxos8700, ninedof);

    // Initialize and enable SPI HAL
    // Set up an SPI MUX, so there can be multiple clients
    let mux_spi = static_init!(
        MuxSpiMaster<'static, sam4l::spi::Spi>,
        MuxSpiMaster::new(&sam4l::spi::SPI)
    );

    sam4l::spi::SPI.set_client(mux_spi);
    sam4l::spi::SPI.init();
    sam4l::spi::SPI.enable();

    // Create a virtualized client for SPI system call interface
    // CS line is CS0
    let syscall_spi_device = static_init!(
        VirtualSpiMasterDevice<'static, sam4l::spi::Spi>,
        VirtualSpiMasterDevice::new(mux_spi, 0)
    );

    // Create the SPI system call capsule, passing the client
    let spi_syscalls = static_init!(
        capsules::spi::Spi<'static, VirtualSpiMasterDevice<'static, sam4l::spi::Spi>>,
        capsules::spi::Spi::new(syscall_spi_device)
    );

    spi_syscalls.config_buffers(&mut SPI_READ_BUF, &mut SPI_WRITE_BUF);
    syscall_spi_device.set_client(spi_syscalls);

    // LEDs
    let led_pins = static_init!(
        [(&'static sam4l::gpio::GPIOPin, capsules::led::ActivationMode); 3],
        [
            (
                &sam4l::gpio::PA[13],
                capsules::led::ActivationMode::ActiveLow
            ), // Red
            (
                &sam4l::gpio::PA[15],
                capsules::led::ActivationMode::ActiveLow
            ), // Green
            (
                &sam4l::gpio::PA[14],
                capsules::led::ActivationMode::ActiveLow
            )
        ]
    ); // Blue
    let led = static_init!(
        capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONs
    let button_pins = static_init!(
        [(&'static sam4l::gpio::GPIOPin, capsules::button::GpioMode); 1],
        [
            (
                &sam4l::gpio::PA[16],
                capsules::button::GpioMode::LowWhenPressed
            )
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // Setup ADC
    let adc_channels = static_init!(
        [&'static sam4l::adc::AdcChannel; 6],
        [
            &sam4l::adc::CHANNEL_AD0, // A0
            &sam4l::adc::CHANNEL_AD1, // A1
            &sam4l::adc::CHANNEL_AD3, // A2
            &sam4l::adc::CHANNEL_AD4, // A3
            &sam4l::adc::CHANNEL_AD5, // A4
            &sam4l::adc::CHANNEL_AD6  // A5
        ]
    );
    let adc = static_init!(
        capsules::adc::Adc<'static, sam4l::adc::Adc>,
        capsules::adc::Adc::new(
            &mut sam4l::adc::ADC0,
            adc_channels,
            &mut capsules::adc::ADC_BUFFER1,
            &mut capsules::adc::ADC_BUFFER2,
            &mut capsules::adc::ADC_BUFFER3
        )
    );
    sam4l::adc::ADC0.set_client(adc);

    // Setup RNG
    let rng = static_init!(
        capsules::rng::SimpleRng<'static, sam4l::trng::Trng>,
        capsules::rng::SimpleRng::new(&sam4l::trng::TRNG, kernel::Grant::create())
    );
    sam4l::trng::TRNG.set_client(rng);

    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 4],
        [
            &sam4l::gpio::PB[14], // D0
            &sam4l::gpio::PB[15], // D1
            &sam4l::gpio::PB[11], // D6
            &sam4l::gpio::PB[12]
        ]
    ); // D7
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // CRC
    let crc = static_init!(
        capsules::crc::Crc<'static, sam4l::crccu::Crccu<'static>>,
        capsules::crc::Crc::new(&mut sam4l::crccu::CRCCU, kernel::Grant::create())
    );
    sam4l::crccu::CRCCU.set_client(crc);

    // DAC
    let dac = static_init!(
        capsules::dac::Dac<'static>,
        capsules::dac::Dac::new(&mut sam4l::dac::DAC)
    );

    // AES
    let aes = static_init!(
        capsules::symmetric_encryption::Crypto<'static, sam4l::aes::Aes>,
        capsules::symmetric_encryption::Crypto::new(
            &mut sam4l::aes::AES,
            kernel::Grant::create(),
            &mut capsules::symmetric_encryption::KEY,
            &mut capsules::symmetric_encryption::BUF,
            &mut capsules::symmetric_encryption::IV
        )
    );
    hil::symmetric_encryption::SymmetricEncryption::set_client(&sam4l::aes::AES, aes);

    let hail = Hail {
        console: console,
        sensys: sensys,
        gpio: gpio,
        alarm: alarm,
        temp: temp,
        humidity: humidity,
        ninedof: ninedof,
        spi: spi_syscalls,
        nrf51822: nrf_serialization,
        adc: adc,
        led: led,
        button: button,
        rng: rng,
        ipc: kernel::ipc::IPC::new(),
        crc: crc,
        dac: dac,
        aes: aes,
    };

    // Need to reset the nRF on boot
    sam4l::gpio::PA[17].enable();
    sam4l::gpio::PA[17].enable_output();
    sam4l::gpio::PA[17].clear();
    sam4l::gpio::PA[17].set();

    hail.console.initialize();
    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(hail.console), kc);

    // Start the SenSys capsule sampling light readings for the
    // console.
    hail.sensys.start();

    hail.nrf51822.initialize();

    // debug!("Initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }
    kernel::process::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    // Begin kernel main loop
    kernel::main(&hail, &mut chip, &mut PROCESSES, &hail.ipc);
}
