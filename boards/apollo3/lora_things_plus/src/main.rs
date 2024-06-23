// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for SparkFun LoRa Thing Plus - expLoRaBLE
//!
//! - <https://www.sparkfun.com/products/17506>
//!
//! A Semtech SX1262 is connected as a SPI slave to IOM3
//! See <https://www.northernmechatronics.com/_files/ugd/3c68cb_764598422c704ed1b32400b047fc7651.pdf>
//! and <https://www.northernmechatronics.com/nm180100> for details
//!
//! See <https://github.com/NorthernMechatronics/nmsdk/blob/master/bsp/nm180100evb/bsp_pins.src>
//! and <https://cdn.sparkfun.com/assets/4/4/f/7/e/expLoRaBLE_Thing_Plus_schematic.pdf>
//! for details on the pin break outs
//!
//! ION0: Qwiic I2C
//! IOM1: Not connected
//! IOM2: Broken out SPI
//! IOM3: Semtech SX1262
//!     Apollo 3 Pin Number | Apollo 3 Name | SX1262 Pin Number | SX1262 Name | SX1262 Description
//!                      H6 |       GPIO 36 |                19 |  NSS        | SPI slave select
//!                      J6 |       GPIO 38 |                17 |  MOSI       | SPI slave input
//!                      J5 |       GPIO 43 |                16 |  MISO       | SPI slave output
//!                      H5 |       GPIO 42 |                18 |  SCK        | SPI clock input
//!                      J8 |       GPIO 39 |                14 |  BUSY       | Radio busy indicator
//!                      J9 |       GPIO 40 |                13 |  DIO1       | Multipurpose digital I/O
//!                      H9 |       GPIO 47 |                6  |  DIO3       | Multipurpose digital I/O
//!                      J7 |       GPIO 44 |                15 |  NRESET     | Radio reset signal, active low
//! IOM4: Not connected
//! IOM5: Pins used by UART0

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use apollo3::chip::Apollo3DefaultPeripherals;
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::bme280::Bme280Component;
use components::ccs811::Ccs811Component;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::spi::SpiMaster;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

#[cfg(test)]
mod tests;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] = [None; 4];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static apollo3::chip::Apollo3<Apollo3DefaultPeripherals>> = None;
// Static reference to process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Test access to the peripherals
#[cfg(test)]
static mut PERIPHERALS: Option<&'static Apollo3DefaultPeripherals> = None;
// Test access to board
#[cfg(test)]
static mut BOARD: Option<&'static kernel::Kernel> = None;
// Test access to platform
#[cfg(test)]
static mut PLATFORM: Option<&'static LoRaThingsPlus> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, apollo3::stimer::STimer<'static>>> = None;
// Test access to sensors
static mut BME280: Option<
    &'static capsules_extra::bme280::Bme280<
        'static,
        capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, apollo3::iom::Iom<'static>>,
    >,
> = None;
static mut CCS811: Option<&'static capsules_extra::ccs811::Ccs811<'static>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

const LORA_SPI_DRIVER_NUM: usize = capsules_core::driver::NUM::LoRaPhySPI as usize;
const LORA_GPIO_DRIVER_NUM: usize = capsules_core::driver::NUM::LoRaPhyGPIO as usize;

type BME280Sensor = components::bme280::Bme280ComponentType<
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, apollo3::iom::Iom<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<BME280Sensor>;
type HumidityDriver = components::humidity::HumidityComponentType<BME280Sensor>;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct LoRaThingsPlus {
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, apollo3::gpio::GpioPin<'static>>,
        1,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, apollo3::gpio::GpioPin<'static>>,
    console: &'static capsules_core::console::Console<'static>,
    i2c_master:
        &'static capsules_core::i2c_master::I2CMasterDriver<'static, apollo3::iom::Iom<'static>>,
    external_spi_controller: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            apollo3::iom::Iom<'static>,
        >,
    >,
    sx1262_spi_controller: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            apollo3::iom::Iom<'static>,
        >,
    >,
    sx1262_gpio: &'static capsules_core::gpio::GPIO<'static, apollo3::gpio::GpioPin<'static>>,
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        apollo3::ble::Ble<'static>,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
    temperature: &'static TemperatureDriver,
    humidity: &'static HumidityDriver,
    air_quality: &'static capsules_extra::air_quality::AirQualitySensor<'static>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for LoRaThingsPlus {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            capsules_core::spi_controller::DRIVER_NUM => f(Some(self.external_spi_controller)),
            LORA_SPI_DRIVER_NUM => f(Some(self.sx1262_spi_controller)),
            LORA_GPIO_DRIVER_NUM => f(Some(self.sx1262_gpio)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules_extra::air_quality::DRIVER_NUM => f(Some(self.air_quality)),
            _ => f(None),
        }
    }
}

impl KernelResources<apollo3::chip::Apollo3<Apollo3DefaultPeripherals>> for LoRaThingsPlus {
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

// Ensure that `setup()` is never inlined
// This helps reduce the stack frame, see https://github.com/tock/tock/issues/3518
#[inline(never)]
unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static LoRaThingsPlus,
    &'static apollo3::chip::Apollo3<Apollo3DefaultPeripherals>,
    &'static Apollo3DefaultPeripherals,
) {
    let peripherals = static_init!(Apollo3DefaultPeripherals, Apollo3DefaultPeripherals::new());

    // No need to statically allocate mcu/pwr/clk_ctrl because they are only used in main!
    let mcu_ctrl = apollo3::mcuctrl::McuCtrl::new();
    let pwr_ctrl = apollo3::pwrctrl::PwrCtrl::new();
    let clkgen = apollo3::clkgen::ClkGen::new();

    clkgen.set_clock_frequency(apollo3::clkgen::ClockFrequency::Freq48MHz);

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // Power up components
    pwr_ctrl.enable_uart0();
    pwr_ctrl.enable_iom0();
    pwr_ctrl.enable_iom2();
    pwr_ctrl.enable_iom3();

    // Enable PinCfg
    peripherals
        .gpio_port
        .enable_uart(&peripherals.gpio_port[48], &peripherals.gpio_port[49]);
    // Enable SDA and SCL for I2C (exposed via Qwiic)
    peripherals
        .gpio_port
        .enable_i2c(&peripherals.gpio_port[6], &peripherals.gpio_port[5]);
    // Enable Main SPI
    peripherals.gpio_port.enable_spi(
        &peripherals.gpio_port[27],
        &peripherals.gpio_port[28],
        &peripherals.gpio_port[25],
    );
    // Enable SPI for SX1262
    peripherals.gpio_port.enable_spi(
        &peripherals.gpio_port[42],
        &peripherals.gpio_port[38],
        &peripherals.gpio_port[43],
    );
    // Enable the radio pins
    peripherals.gpio_port.enable_sx1262_radio_pins();

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(Some(&peripherals.gpio_port[26]), None, None);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux)
        .finalize(components::debug_writer_component_static!());

    // LEDs
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, apollo3::gpio::GpioPin>,
        LedHigh::new(&peripherals.gpio_port[19]),
    ));

    // GPIOs
    // Details are at: https://github.com/NorthernMechatronics/nmsdk/blob/master/bsp/nm180100evb/bsp_pins.src
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            apollo3::gpio::GpioPin,
            0 => &peripherals.gpio_port[13],  // A0
            1 => &peripherals.gpio_port[12],  // A1
            2 => &peripherals.gpio_port[32],  // A2
            3 => &peripherals.gpio_port[35],  // A3
            4 => &peripherals.gpio_port[34],  // A4
        ),
    )
    .finalize(components::gpio_component_static!(apollo3::gpio::GpioPin));

    // Create a shared virtualisation mux layer on top of a single hardware
    // alarm.
    let _ = peripherals.stimer.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.stimer).finalize(
        components::alarm_mux_component_static!(apollo3::stimer::STimer),
    );
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(apollo3::stimer::STimer));
    ALARM = Some(mux_alarm);

    // Create a process printer for panic.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Init the I2C device attached via Qwiic
    let i2c_master_buffer = static_init!(
        [u8; capsules_core::i2c_master::BUFFER_LENGTH],
        [0; capsules_core::i2c_master::BUFFER_LENGTH]
    );
    let i2c_master = static_init!(
        capsules_core::i2c_master::I2CMasterDriver<'static, apollo3::iom::Iom<'static>>,
        capsules_core::i2c_master::I2CMasterDriver::new(
            &peripherals.iom0,
            i2c_master_buffer,
            board_kernel.create_grant(
                capsules_core::i2c_master::DRIVER_NUM,
                &memory_allocation_cap
            )
        )
    );

    peripherals.iom0.set_master_client(i2c_master);
    peripherals.iom0.enable();

    let mux_i2c = components::i2c::I2CMuxComponent::new(&peripherals.iom0, None).finalize(
        components::i2c_mux_component_static!(apollo3::iom::Iom<'static>),
    );

    let bme280 = Bme280Component::new(mux_i2c, 0x77).finalize(
        components::bme280_component_static!(apollo3::iom::Iom<'static>),
    );
    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        bme280,
    )
    .finalize(components::temperature_component_static!(BME280Sensor));
    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules_extra::humidity::DRIVER_NUM,
        bme280,
    )
    .finalize(components::humidity_component_static!(BME280Sensor));
    BME280 = Some(bme280);

    let ccs811 = Ccs811Component::new(mux_i2c, 0x5B).finalize(
        components::ccs811_component_static!(apollo3::iom::Iom<'static>),
    );
    let air_quality = components::air_quality::AirQualityComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        ccs811,
    )
    .finalize(components::air_quality_component_static!());
    CCS811 = Some(ccs811);

    // Init the broken out SPI controller
    let external_mux_spi = components::spi::SpiMuxComponent::new(&peripherals.iom2).finalize(
        components::spi_mux_component_static!(apollo3::iom::Iom<'static>),
    );

    let external_spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        external_mux_spi,
        &peripherals.gpio_port[11], // A5
        capsules_core::spi_controller::DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        apollo3::iom::Iom<'static>
    ));

    // Init the internal SX1262 SPI controller
    let sx1262_mux_spi = components::spi::SpiMuxComponent::new(&peripherals.iom3).finalize(
        components::spi_mux_component_static!(apollo3::iom::Iom<'static>),
    );

    let sx1262_spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        sx1262_mux_spi,
        &peripherals.gpio_port[36], // H6 - SX1262 Slave Select
        LORA_SPI_DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        apollo3::iom::Iom<'static>
    ));
    peripherals
        .iom3
        .specify_chip_select(&peripherals.gpio_port[36])
        .unwrap();

    let sx1262_gpio = components::gpio::GpioComponent::new(
        board_kernel,
        LORA_GPIO_DRIVER_NUM,
        components::gpio_component_helper!(
            apollo3::gpio::GpioPin,
            0 => &peripherals.gpio_port[36], // H6 - SX1262 Slave Select
            1 => &peripherals.gpio_port[39], // J8 - SX1262 Radio Busy Indicator
            2 => &peripherals.gpio_port[40], // J9 - SX1262 Multipurpose digital I/O (DIO1)
            3 => &peripherals.gpio_port[47], // H9 - SX1262 Multipurpose digital I/O (DIO3)
            4 => &peripherals.gpio_port[44], // J7 - SX1262 Reset
        ),
    )
    .finalize(components::gpio_component_static!(apollo3::gpio::GpioPin));

    // Setup BLE
    mcu_ctrl.enable_ble();
    clkgen.enable_ble();
    pwr_ctrl.enable_ble();
    peripherals.ble.setup_clocks();
    mcu_ctrl.reset_ble();
    peripherals.ble.power_up();
    peripherals.ble.ble_initialise();

    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        capsules_extra::ble_advertising_driver::DRIVER_NUM,
        &peripherals.ble,
        mux_alarm,
    )
    .finalize(components::ble_component_static!(
        apollo3::stimer::STimer,
        apollo3::ble::Ble,
    ));

    mcu_ctrl.print_chip_revision();

    debug!("Initialization complete. Entering main loop");

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

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let systick = cortexm4::systick::SysTick::new_with_calibration(48_000_000);

    let artemis_nano = static_init!(
        LoRaThingsPlus,
        LoRaThingsPlus {
            alarm,
            led,
            gpio,
            console,
            i2c_master,
            external_spi_controller,
            sx1262_spi_controller,
            sx1262_gpio,
            ble_radio,
            temperature,
            humidity,
            air_quality,
            scheduler,
            systick,
        }
    );

    let chip = static_init!(
        apollo3::chip::Apollo3<Apollo3DefaultPeripherals>,
        apollo3::chip::Apollo3::new(peripherals)
    );
    CHIP = Some(chip);

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
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, artemis_nano, chip, peripherals)
}

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    apollo3::init();

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        let (board_kernel, sf_lora_thing_plus_board, chip, _peripherals) = setup();

        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(
            sf_lora_thing_plus_board,
            chip,
            None::<&kernel::ipc::IPC<{ NUM_PROCS as u8 }>>,
            &main_loop_cap,
        );
    }
}

#[cfg(test)]
use kernel::platform::watchdog::WatchDog;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unsafe {
        let (board_kernel, sf_lora_thing_plus_board, _chip, peripherals) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&sf_lora_thing_plus_board);
        PERIPHERALS = Some(peripherals);
        MAIN_CAP = Some(&create_capability!(capabilities::MainLoopCapability));

        PLATFORM.map(|p| {
            p.watchdog().setup();
        });

        for test in tests {
            test();
        }
    }

    loop {}
}
