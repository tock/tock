//! Board file for Hail development platform.
//!
//! - <https://github.com/tock/tock/tree/master/boards/hail>
//! - <https://github.com/lab11/hail>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::Controller;
use kernel::Platform;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};

/// Support routines for debugging I/O.
///
/// Note: Use of this module will trample any other USART0 configuration.
pub mod io;
#[allow(dead_code)]
mod test_take_map_cell;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 20;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 49152] = [0; 49152];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct Hail {
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
    >,
    ambient_light: &'static capsules::ambient_light::AmbientLight<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
    humidity: &'static capsules::humidity::HumiditySensor<'static>,
    spi: &'static capsules::spi::Spi<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>,
    nrf51822: &'static capsules::nrf51822_serialization::Nrf51822Serialization<'static>,
    adc: &'static capsules::adc::Adc<'static, sam4l::adc::Adc>,
    led: &'static capsules::led::LED<'static>,
    button: &'static capsules::button::Button<'static>,
    rng: &'static capsules::rng::RngDriver<'static>,
    ipc: kernel::ipc::IPC,
    crc: &'static capsules::crc::Crc<'static, sam4l::crccu::Crccu<'static>>,
    dac: &'static capsules::dac::Dac<'static>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for Hail {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),

            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::spi::DRIVER_NUM => f(Some(self.spi)),
            capsules::nrf51822_serialization::DRIVER_NUM => f(Some(self.nrf51822)),
            capsules::ambient_light::DRIVER_NUM => f(Some(self.ambient_light)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),

            capsules::rng::DRIVER_NUM => f(Some(self.rng)),

            capsules::crc::DRIVER_NUM => f(Some(self.crc)),

            capsules::dac::DRIVER_NUM => f(Some(self.dac)),

            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::PeripheralFunction::{A, B};
    use sam4l::gpio::{PA, PB};

    PA[04].configure(Some(A)); // A0 - ADC0
    PA[05].configure(Some(A)); // A1 - ADC1
                               // DAC/WKP mode
    PA[06].configure(Some(A)); // DAC
    PA[07].configure(None); //... WKP - Wakeup
                            // // Analog Comparator Mode
                            // PA[06].configure(Some(E)); // ACAN0 - ACIFC
                            // PA[07].configure(Some(E)); // ACAP0 - ACIFC
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
                               // // I2C Mode
                               // PA[21].configure(None); // D3
                               // PA[22].configure(None); // D2
                               // PA[23].configure(Some(B)); // D4 - TWIMS0 SDA
                               // PA[24].configure(Some(B)); // D5 - TWIMS0 SCL
                               // UART Mode
    PA[25].configure(Some(B)); // RX - USART2 RXD
    PA[26].configure(Some(B)); // TX - USART2 TXD

    PB[00].configure(Some(A)); // SENSORS_SDA - TWIMS1 SDA
    PB[01].configure(Some(A)); // SENSORS_SCL - TWIMS1 SCL
                               // ADC Mode
    PB[02].configure(Some(A)); // A2 - ADC3
    PB[03].configure(Some(A)); // A3 - ADC4
                               // // Analog Comparator Mode
                               // PB[02].configure(Some(E)); // ACBN0 - ACIFC
                               // PB[03].configure(Some(E)); // ACBP0 - ACIFC
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
        startup_mode: sam4l::pm::OscillatorStartup::SlowStart,
    });

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&sam4l::gpio::PA[13]),
        Some(&sam4l::gpio::PA[15]),
        Some(&sam4l::gpio::PA[14]),
    );

    let chip = static_init!(sam4l::chip::Sam4l, sam4l::chip::Sam4l::new());

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Initialize USART0 for Uart
    sam4l::usart::USART0.set_mode(sam4l::usart::UsartMode::Uart);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &sam4l::usart::USART0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());
    uart_mux.initialize();

    hil::uart::Transmit::set_transmit_client(&sam4l::usart::USART0, uart_mux);
    hil::uart::Receive::set_receive_client(&sam4l::usart::USART0, uart_mux);

    // Setup the console and the process inspection console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    let process_console =
        components::process_console::ProcessConsoleComponent::new(board_kernel, uart_mux)
            .finalize(());
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // Initialize USART3 for UART for the nRF serialization link.
    sam4l::usart::USART3.set_mode(sam4l::usart::UsartMode::Uart);
    // Create the Nrf51822Serialization driver for passing BLE commands
    // over UART to the nRF51822 radio.
    let nrf_serialization =
        components::nrf51822::Nrf51822Component::new(&sam4l::usart::USART3, &sam4l::gpio::PA[17])
            .finalize(());

    let ast = &sam4l::ast::AST;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(ast)
        .finalize(components::alarm_mux_component_helper!(sam4l::ast::Ast));
    ast.configure(mux_alarm);

    let sensors_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(&sam4l::i2c::I2C1));
    sam4l::i2c::I2C1.set_master_client(sensors_i2c);

    // SI7021 Temperature / Humidity Sensor, address: 0x40
    let si7021 = components::si7021::SI7021Component::new(sensors_i2c, mux_alarm, 0x40)
        .finalize(components::si7021_component_helper!(sam4l::ast::Ast));
    let temp = components::si7021::TemperatureComponent::new(board_kernel, si7021).finalize(());
    let humidity = components::si7021::HumidityComponent::new(board_kernel, si7021).finalize(());

    // Configure the ISL29035, device address 0x44
    let ambient_light =
        components::isl29035::AmbientLightComponent::new(board_kernel, sensors_i2c, mux_alarm)
            .finalize(components::isl29035_component_helper!(sam4l::ast::Ast));

    // Alarm
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(sam4l::ast::Ast));

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
        capsules::ninedof::NineDof::new(
            fxos8700,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::sensors::NineDof::set_client(fxos8700, ninedof);

    // SPI
    // Set up a SPI MUX, so there can be multiple clients.
    let mux_spi = components::spi::SpiMuxComponent::new(&sam4l::spi::SPI)
        .finalize(components::spi_mux_component_helper!(sam4l::spi::SpiHw));
    // Create the SPI system call capsule.
    let spi_syscalls = components::spi::SpiSyscallComponent::new(mux_spi, 0)
        .finalize(components::spi_syscall_component_helper!(sam4l::spi::SpiHw));

    // LEDs
    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
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
        ) // Blue
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!((
            &sam4l::gpio::PA[16],
            capsules::button::GpioMode::LowWhenPressed
        )),
    );

    // Setup ADC
    let adc_channels = static_init!(
        [&'static sam4l::adc::AdcChannel; 6],
        [
            &sam4l::adc::CHANNEL_AD0, // A0
            &sam4l::adc::CHANNEL_AD1, // A1
            &sam4l::adc::CHANNEL_AD3, // A2
            &sam4l::adc::CHANNEL_AD4, // A3
            &sam4l::adc::CHANNEL_AD5, // A4
            &sam4l::adc::CHANNEL_AD6, // A5
        ]
    );
    let adc = static_init!(
        capsules::adc::Adc<'static, sam4l::adc::Adc>,
        capsules::adc::Adc::new(
            &sam4l::adc::ADC0,
            adc_channels,
            &mut capsules::adc::ADC_BUFFER1,
            &mut capsules::adc::ADC_BUFFER2,
            &mut capsules::adc::ADC_BUFFER3
        )
    );
    sam4l::adc::ADC0.set_client(adc);

    // Setup RNG
    let rng = components::rng::RngComponent::new(board_kernel, &sam4l::trng::TRNG).finalize(());

    // set GPIO driver controlling remaining GPIO pins
    let gpio = components::gpio::GpioComponent::new(board_kernel).finalize(
        components::gpio_component_helper!(
            &sam4l::gpio::PC[14], // DO
            &sam4l::gpio::PC[15], // D1
            &sam4l::gpio::PC[11], // D6
            &sam4l::gpio::PC[12]  // D7
        ),
    );

    // CRC
    let crc = components::crc::CrcComponent::new(board_kernel, &sam4l::crccu::CRCCU)
        .finalize(components::crc_component_helper!(sam4l::crccu::Crccu));

    // DAC
    let dac = static_init!(
        capsules::dac::Dac<'static>,
        capsules::dac::Dac::new(&sam4l::dac::DAC)
    );

    // // DEBUG Restart All Apps
    // //
    // // Uncomment to enable a button press to restart all apps.
    // //
    // // Create a dummy object that provides the `ProcessManagementCapability` to
    // // the `debug_process_restart` capsule.
    // struct ProcessMgmtCap;
    // unsafe impl capabilities::ProcessManagementCapability for ProcessMgmtCap {}
    // let debug_process_restart = static_init!(
    //     capsules::debug_process_restart::DebugProcessRestart<
    //         'static,
    //         sam4l::gpio::GPIOPin,
    //         ProcessMgmtCap,
    //     >,
    //     capsules::debug_process_restart::DebugProcessRestart::new(
    //         board_kernel,
    //         &sam4l::gpio::PA[16],
    //         ProcessMgmtCap
    //     )
    // );
    // sam4l::gpio::PA[16].set_client(debug_process_restart);

    let hail = Hail {
        console: console,
        gpio: gpio,
        alarm: alarm,
        ambient_light: ambient_light,
        temp: temp,
        humidity: humidity,
        ninedof: ninedof,
        spi: spi_syscalls,
        nrf51822: nrf_serialization,
        adc: adc,
        led: led,
        button: button,
        rng: rng,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        crc: crc,
        dac: dac,
    };

    // Reset the nRF and setup the UART bus.
    hail.nrf51822.reset();
    hail.nrf51822.initialize();

    process_console.start();

    // Uncomment to measure overheads for TakeCell and MapCell:
    // test_take_map_cell::test_take_map_cell();

    debug!("Initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );
    board_kernel.kernel_loop(&hail, chip, Some(&hail.ipc), &main_loop_capability);
}
