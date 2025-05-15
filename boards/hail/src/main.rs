// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for Hail development platform.
//!
//! - <https://github.com/tock/tock/tree/master/boards/hail>
//! - <https://github.com/lab11/hail>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::led::LedLow;
use kernel::hil::Controller;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};
use sam4l::chip::Sam4lDefaultPeripherals;

/// Support routines for debugging I/O.
///
/// Note: Use of this module will trample any other USART0 configuration.
pub mod io;
#[allow(dead_code)]
mod test_take_map_cell;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 20;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static sam4l::chip::Sam4l<Sam4lDefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

type SI7021Sensor = components::si7021::SI7021ComponentType<
    capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, sam4l::i2c::I2CHw<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<SI7021Sensor>;
type HumidityDriver = components::humidity::HumidityComponentType<SI7021Sensor>;
type RngDriver = components::rng::RngComponentType<sam4l::trng::Trng<'static>>;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct Hail {
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, sam4l::gpio::GPIOPin<'static>>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            sam4l::ast::Ast<'static>,
        >,
    >,
    ambient_light: &'static capsules_extra::ambient_light::AmbientLight<'static>,
    temp: &'static TemperatureDriver,
    ninedof: &'static capsules_extra::ninedof::NineDof<'static>,
    humidity: &'static HumidityDriver,
    spi: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            sam4l::spi::SpiHw<'static>,
        >,
    >,
    nrf51822: &'static capsules_extra::nrf51822_serialization::Nrf51822Serialization<'static>,
    adc: &'static capsules_core::adc::AdcDedicated<'static, sam4l::adc::Adc<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, sam4l::gpio::GPIOPin<'static>>,
        3,
    >,
    button: &'static capsules_core::button::Button<'static, sam4l::gpio::GPIOPin<'static>>,
    rng: &'static RngDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    crc: &'static capsules_extra::crc::CrcDriver<'static, sam4l::crccu::Crccu<'static>>,
    dac: &'static capsules_extra::dac::Dac<'static>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for Hail {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),

            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::spi_controller::DRIVER_NUM => f(Some(self.spi)),
            capsules_extra::nrf51822_serialization::DRIVER_NUM => f(Some(self.nrf51822)),
            capsules_extra::ambient_light::DRIVER_NUM => f(Some(self.ambient_light)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules_extra::ninedof::DRIVER_NUM => f(Some(self.ninedof)),

            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),

            capsules_extra::crc::DRIVER_NUM => f(Some(self.crc)),

            capsules_extra::dac::DRIVER_NUM => f(Some(self.dac)),

            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<sam4l::chip::Sam4l<Sam4lDefaultPeripherals>> for Hail {
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

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions(peripherals: &Sam4lDefaultPeripherals) {
    use sam4l::gpio::PeripheralFunction::{A, B};

    peripherals.pa[04].configure(Some(A)); // A0 - ADC0
    peripherals.pa[05].configure(Some(A)); // A1 - ADC1
                                           // DAC/WKP mode
    peripherals.pa[06].configure(Some(A)); // DAC
    peripherals.pa[07].configure(None); //... WKP - Wakeup
                                        // // Analog Comparator Mode
                                        // peripherals.pa[06].configure(Some(E)); // ACAN0 - ACIFC
                                        // peripherals.pa[07].configure(Some(E)); // ACAP0 - ACIFC
    peripherals.pa[08].configure(Some(A)); // FTDI_RTS - USART0 RTS
    peripherals.pa[09].configure(None); //... ACC_INT1 - FXOS8700CQ Interrupt 1
    peripherals.pa[10].configure(None); //... unused
    peripherals.pa[11].configure(Some(A)); // FTDI_OUT - USART0 RX FTDI->SAM4L
    peripherals.pa[12].configure(Some(A)); // FTDI_IN - USART0 TX SAM4L->FTDI
    peripherals.pa[13].configure(None); //... RED_LED
    peripherals.pa[14].configure(None); //... BLUE_LED
    peripherals.pa[15].configure(None); //... GREEN_LED
    peripherals.pa[16].configure(None); //... BUTTON - User Button
    peripherals.pa[17].configure(None); //... !NRF_RESET - Reset line for nRF51822
    peripherals.pa[18].configure(None); //... ACC_INT2 - FXOS8700CQ Interrupt 2
    peripherals.pa[19].configure(None); //... unused
    peripherals.pa[20].configure(None); //... !LIGHT_INT - ISL29035 Light Sensor Interrupt
                                        // SPI Mode
    peripherals.pa[21].configure(Some(A)); // D3 - SPI MISO
    peripherals.pa[22].configure(Some(A)); // D2 - SPI MOSI
    peripherals.pa[23].configure(Some(A)); // D4 - SPI SCK
    peripherals.pa[24].configure(Some(A)); // D5 - SPI CS0
                                           // // I2C Mode
                                           // peripherals.pa[21].configure(None); // D3
                                           // peripherals.pa[22].configure(None); // D2
                                           // peripherals.pa[23].configure(Some(B)); // D4 - TWIMS0 SDA
                                           // peripherals.pa[24].configure(Some(B)); // D5 - TWIMS0 SCL
                                           // UART Mode
    peripherals.pa[25].configure(Some(B)); // RX - USART2 RXD
    peripherals.pa[26].configure(Some(B)); // TX - USART2 TXD

    peripherals.pb[00].configure(Some(A)); // SENSORS_SDA - TWIMS1 SDA
    peripherals.pb[01].configure(Some(A)); // SENSORS_SCL - TWIMS1 SCL
                                           // ADC Mode
    peripherals.pb[02].configure(Some(A)); // A2 - ADC3
    peripherals.pb[03].configure(Some(A)); // A3 - ADC4
                                           // // Analog Comparator Mode
                                           // peripherals.pb[02].configure(Some(E)); // ACBN0 - ACIFC
                                           // peripherals.pb[03].configure(Some(E)); // ACBP0 - ACIFC
    peripherals.pb[04].configure(Some(A)); // A4 - ADC5
    peripherals.pb[05].configure(Some(A)); // A5 - ADC6
    peripherals.pb[06].configure(Some(A)); // NRF_CTS - USART3 RTS
    peripherals.pb[07].configure(Some(A)); // NRF_RTS - USART3 CTS
    peripherals.pb[08].configure(None); //... NRF_INT - Interrupt line nRF->SAM4L
    peripherals.pb[09].configure(Some(A)); // NRF_OUT - USART3 RXD
    peripherals.pb[10].configure(Some(A)); // NRF_IN - USART3 TXD
    peripherals.pb[11].configure(None); //... D6
    peripherals.pb[12].configure(None); //... D7
    peripherals.pb[13].configure(None); //... unused
    peripherals.pb[14].configure(None); //... D0
    peripherals.pb[15].configure(None); //... D1
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    Hail,
    &'static sam4l::chip::Sam4l<Sam4lDefaultPeripherals>,
) {
    sam4l::init();

    let pm = static_init!(sam4l::pm::PowerManager, sam4l::pm::PowerManager::new());
    let peripherals = static_init!(Sam4lDefaultPeripherals, Sam4lDefaultPeripherals::new(pm));

    pm.setup_system_clock(
        sam4l::pm::SystemClockSource::PllExternalOscillatorAt48MHz {
            frequency: sam4l::pm::OscillatorFrequency::Frequency16MHz,
            startup_mode: sam4l::pm::OscillatorStartup::SlowStart,
        },
        &peripherals.flash_controller,
    );

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions(peripherals);
    peripherals.setup_circular_deps();
    let chip = static_init!(
        sam4l::chip::Sam4l<Sam4lDefaultPeripherals>,
        sam4l::chip::Sam4l::new(pm, peripherals)
    );
    CHIP = Some(chip);

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&peripherals.pa[13]),
        Some(&peripherals.pa[15]),
        Some(&peripherals.pa[14]),
    );

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Initialize USART0 for Uart
    peripherals.usart0.set_mode(sam4l::usart::UsartMode::Uart);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.usart0, 115200)
        .finalize(components::uart_mux_component_static!());
    uart_mux.initialize();

    hil::uart::Transmit::set_transmit_client(&peripherals.usart0, uart_mux);
    hil::uart::Receive::set_receive_client(&peripherals.usart0, uart_mux);

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.ast)
        .finalize(components::alarm_mux_component_static!(sam4l::ast::Ast));
    peripherals.ast.configure(mux_alarm);

    // Setup the console and the process inspection console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        sam4l::ast::Ast<'static>
    ));
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // Initialize USART3 for UART for the nRF serialization link.
    peripherals.usart3.set_mode(sam4l::usart::UsartMode::Uart);
    // Create the Nrf51822Serialization driver for passing BLE commands
    // over UART to the nRF51822 radio.
    let nrf_serialization = components::nrf51822::Nrf51822Component::new(
        board_kernel,
        capsules_extra::nrf51822_serialization::DRIVER_NUM,
        &peripherals.usart3,
        &peripherals.pa[17],
    )
    .finalize(components::nrf51822_component_static!());

    let sensors_i2c = components::i2c::I2CMuxComponent::new(&peripherals.i2c1, None)
        .finalize(components::i2c_mux_component_static!(sam4l::i2c::I2CHw));

    // SI7021 Temperature / Humidity Sensor, address: 0x40
    let si7021 = components::si7021::SI7021Component::new(sensors_i2c, mux_alarm, 0x40).finalize(
        components::si7021_component_static!(sam4l::ast::Ast, sam4l::i2c::I2CHw),
    );
    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        si7021,
    )
    .finalize(components::temperature_component_static!(SI7021Sensor));
    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules_extra::humidity::DRIVER_NUM,
        si7021,
    )
    .finalize(components::humidity_component_static!(SI7021Sensor));

    // Configure the ISL29035, device address 0x44
    let isl29035 = components::isl29035::Isl29035Component::new(sensors_i2c, mux_alarm).finalize(
        components::isl29035_component_static!(sam4l::ast::Ast, sam4l::i2c::I2CHw),
    );
    let ambient_light = components::isl29035::AmbientLightComponent::new(
        board_kernel,
        capsules_extra::ambient_light::DRIVER_NUM,
        isl29035,
    )
    .finalize(components::ambient_light_component_static!());

    // Alarm
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(sam4l::ast::Ast));

    // FXOS8700CQ accelerometer, device address 0x1e
    let fxos8700 =
        components::fxos8700::Fxos8700Component::new(sensors_i2c, 0x1e, &peripherals.pa[9])
            .finalize(components::fxos8700_component_static!(sam4l::i2c::I2CHw));

    let ninedof = components::ninedof::NineDofComponent::new(
        board_kernel,
        capsules_extra::ninedof::DRIVER_NUM,
    )
    .finalize(components::ninedof_component_static!(fxos8700));

    // SPI
    // Set up a SPI MUX, so there can be multiple clients.
    let mux_spi = components::spi::SpiMuxComponent::new(&peripherals.spi)
        .finalize(components::spi_mux_component_static!(sam4l::spi::SpiHw));
    // Create the SPI system call capsule.
    let spi_syscalls = components::spi::SpiSyscallComponent::new(
        board_kernel,
        mux_spi,
        sam4l::spi::Peripheral::Peripheral0,
        capsules_core::spi_controller::DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(sam4l::spi::SpiHw));

    // LEDs
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, sam4l::gpio::GPIOPin>,
        LedLow::new(&peripherals.pa[13]), // Red
        LedLow::new(&peripherals.pa[15]), // Green
        LedLow::new(&peripherals.pa[14]), // Blue
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            sam4l::gpio::GPIOPin,
            (
                &peripherals.pa[16],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_static!(sam4l::gpio::GPIOPin));

    // Setup ADC
    let adc_channels = static_init!(
        [sam4l::adc::AdcChannel; 6],
        [
            sam4l::adc::AdcChannel::new(sam4l::adc::Channel::AD0), // A0
            sam4l::adc::AdcChannel::new(sam4l::adc::Channel::AD1), // A1
            sam4l::adc::AdcChannel::new(sam4l::adc::Channel::AD3), // A2
            sam4l::adc::AdcChannel::new(sam4l::adc::Channel::AD4), // A3
            sam4l::adc::AdcChannel::new(sam4l::adc::Channel::AD5), // A4
            sam4l::adc::AdcChannel::new(sam4l::adc::Channel::AD6), // A5
        ]
    );
    let adc = components::adc::AdcDedicatedComponent::new(
        &peripherals.adc,
        adc_channels,
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
    )
    .finalize(components::adc_dedicated_component_static!(sam4l::adc::Adc));

    // Setup RNG
    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &peripherals.trng,
    )
    .finalize(components::rng_component_static!(sam4l::trng::Trng));

    // set GPIO driver controlling remaining GPIO pins
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            sam4l::gpio::GPIOPin,
            0 => &peripherals.pb[14], // D0
            1 => &peripherals.pb[15], // D1
            2 => &peripherals.pb[11], // D6
            3 => &peripherals.pb[12]  // D7
        ),
    )
    .finalize(components::gpio_component_static!(sam4l::gpio::GPIOPin));

    // CRC
    let crc = components::crc::CrcComponent::new(
        board_kernel,
        capsules_extra::crc::DRIVER_NUM,
        &peripherals.crccu,
    )
    .finalize(components::crc_component_static!(sam4l::crccu::Crccu));

    // DAC
    let dac = components::dac::DacComponent::new(&peripherals.dac)
        .finalize(components::dac_component_static!());

    // // DEBUG Restart All Apps
    // //
    // // Uncomment to enable a button press to restart all apps.
    // //
    // // Create a dummy object that provides the `ProcessManagementCapability` to
    // // the `debug_process_restart` capsule.
    // struct ProcessMgmtCap;
    // unsafe impl capabilities::ProcessManagementCapability for ProcessMgmtCap {}
    // let debug_process_restart = static_init!(
    //     capsules_core::debug_process_restart::DebugProcessRestart<
    //         ProcessMgmtCap,
    //     >,
    //     capsules_core::debug_process_restart::DebugProcessRestart::new(
    //         board_kernel,
    //         &peripherals.pa[16],
    //         ProcessMgmtCap
    //     )
    // );
    // peripherals.pa[16].set_client(debug_process_restart);

    // Configure application fault policy
    let fault_policy = static_init!(
        capsules_system::process_policies::ThresholdRestartThenPanicFaultPolicy,
        capsules_system::process_policies::ThresholdRestartThenPanicFaultPolicy::new(4)
    );

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let hail = Hail {
        console,
        gpio,
        alarm,
        ambient_light,
        temp,
        humidity,
        ninedof,
        spi: spi_syscalls,
        nrf51822: nrf_serialization,
        adc,
        led,
        button,
        rng,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        crc,
        dac,
        scheduler,
        systick: cortexm4::systick::SysTick::new(),
    };

    // Setup the UART bus for nRF51 serialization..
    hail.nrf51822.initialize();

    let _ = process_console.start();

    // Uncomment to measure overheads for TakeCell and MapCell:
    // test_take_map_cell::test_take_map_cell();

    debug!("Initialization complete. Entering main loop.");

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
        fault_policy,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, hail, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
