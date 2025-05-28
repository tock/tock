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
//! IOM0: Qwiic I2C
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
#![no_main]
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
use kernel::hil::flash::HasClient;
use kernel::hil::hasher::Hasher;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::spi::SpiMaster;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};

#[cfg(feature = "atecc508a")]
use {
    capsules_core::virtualizers::virtual_i2c::MuxI2C,
    components::atecc508a::Atecc508aComponent,
    kernel::hil::entropy::Entropy32,
    kernel::hil::gpio::{Configure, Output},
    kernel::hil::rng::Rng,
};

#[cfg(any(feature = "chirp_i2c_moisture", feature = "dfrobot_i2c_rainfall"))]
use capsules_core::virtualizers::virtual_i2c::MuxI2C;

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
#[cfg(feature = "atecc508a")]
static mut ATECC508A: Option<&'static capsules_extra::atecc508a::Atecc508a<'static>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

const LORA_SPI_DRIVER_NUM: usize = capsules_core::driver::NUM::LoRaPhySPI as usize;
const LORA_GPIO_DRIVER_NUM: usize = capsules_core::driver::NUM::LoRaPhyGPIO as usize;

type ChirpI2cMoistureType = components::chirp_i2c_moisture::ChirpI2cMoistureComponentType<
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, apollo3::iom::Iom<'static>>,
>;
type DFRobotRainFallType = components::dfrobot_rainfall_sensor::DFRobotRainFallSensorComponentType<
    capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
        'static,
        apollo3::stimer::STimer<'static>,
    >,
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, apollo3::iom::Iom<'static>>,
>;
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
    temperature: &'static TemperatureDriver,
    humidity: &'static HumidityDriver,
    air_quality: &'static capsules_extra::air_quality::AirQualitySensor<'static>,
    moisture: Option<&'static components::moisture::MoistureComponentType<ChirpI2cMoistureType>>,
    rainfall: Option<&'static components::rainfall::RainFallComponentType<DFRobotRainFallType>>,
    rng: Option<
        &'static capsules_core::rng::RngDriver<
            'static,
            capsules_core::rng::Entropy32ToRandom<
                'static,
                capsules_extra::atecc508a::Atecc508a<'static>,
            >,
        >,
    >,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
    kv_driver: &'static capsules_extra::kv_driver::KVStoreDriver<
        'static,
        capsules_extra::virtual_kv::VirtualKVPermissions<
            'static,
            capsules_extra::kv_store_permissions::KVStorePermissions<
                'static,
                capsules_extra::tickv_kv_store::TicKVKVStore<
                    'static,
                    capsules_extra::tickv::TicKVSystem<
                        'static,
                        capsules_core::virtualizers::virtual_flash::FlashUser<
                            'static,
                            apollo3::flashctrl::FlashCtrl<'static>,
                        >,
                        capsules_extra::sip_hash::SipHasher24<'static>,
                        { apollo3::flashctrl::PAGE_SIZE },
                    >,
                    [u8; 8],
                >,
            >,
        >,
    >,
}

#[cfg(feature = "atecc508a")]
fn atecc508a_wakeup() {
    let peripherals = (unsafe { PERIPHERALS }).unwrap();

    peripherals.gpio_port[6].make_output();
    peripherals.gpio_port[6].clear();

    // The ATECC508A requires the SDA line to be low for at least 60us
    // to wake up.
    for _i in 0..700 {
        cortexm4::support::nop();
    }

    // Enable SDA and SCL for I2C (exposed via Qwiic)
    let _ = &peripherals
        .gpio_port
        .enable_i2c(&peripherals.gpio_port[6], &peripherals.gpio_port[5]);
}

#[cfg(feature = "atecc508a")]
unsafe fn setup_atecc508a(
    board_kernel: &'static kernel::Kernel,
    memory_allocation_cap: &dyn capabilities::MemoryAllocationCapability,
    mux_i2c: &'static MuxI2C<'static, apollo3::iom::Iom<'static>>,
) -> &'static capsules_core::rng::RngDriver<
    'static,
    capsules_core::rng::Entropy32ToRandom<'static, capsules_extra::atecc508a::Atecc508a<'static>>,
> {
    let atecc508a = Atecc508aComponent::new(mux_i2c, 0x60, atecc508a_wakeup).finalize(
        components::atecc508a_component_static!(apollo3::iom::Iom<'static>),
    );
    ATECC508A = Some(atecc508a);

    // Convert hardware RNG to the Random interface.
    let entropy_to_random = static_init!(
        capsules_core::rng::Entropy32ToRandom<
            'static,
            capsules_extra::atecc508a::Atecc508a<'static>,
        >,
        capsules_core::rng::Entropy32ToRandom::new(atecc508a)
    );
    atecc508a.set_client(entropy_to_random);
    // Setup RNG for userspace
    let rng_local = static_init!(
        capsules_core::rng::RngDriver<
            'static,
            capsules_core::rng::Entropy32ToRandom<
                'static,
                capsules_extra::atecc508a::Atecc508a<'static>,
            >,
        >,
        capsules_core::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(capsules_core::rng::DRIVER_NUM, memory_allocation_cap)
        )
    );
    entropy_to_random.set_client(rng_local);

    rng_local
}

#[cfg(feature = "chirp_i2c_moisture")]
unsafe fn setup_chirp_i2c_moisture(
    board_kernel: &'static kernel::Kernel,
    _memory_allocation_cap: &dyn capabilities::MemoryAllocationCapability,
    mux_i2c: &'static MuxI2C<'static, apollo3::iom::Iom<'static>>,
) -> &'static components::moisture::MoistureComponentType<ChirpI2cMoistureType> {
    let chirp_moisture =
        components::chirp_i2c_moisture::ChirpI2cMoistureComponent::new(mux_i2c, 0x20).finalize(
            components::chirp_i2c_moisture_component_static!(apollo3::iom::Iom<'static>),
        );

    let moisture = components::moisture::MoistureComponent::new(
        board_kernel,
        capsules_extra::moisture::DRIVER_NUM,
        chirp_moisture,
    )
    .finalize(components::moisture_component_static!(ChirpI2cMoistureType));

    moisture
}

#[cfg(feature = "dfrobot_i2c_rainfall")]
unsafe fn setup_dfrobot_i2c_rainfall(
    board_kernel: &'static kernel::Kernel,
    _memory_allocation_cap: &dyn capabilities::MemoryAllocationCapability,
    mux_i2c: &'static MuxI2C<'static, apollo3::iom::Iom<'static>>,
    mux_alarm: &'static MuxAlarm<'static, apollo3::stimer::STimer<'static>>,
) -> &'static components::rainfall::RainFallComponentType<DFRobotRainFallType> {
    let dfrobot_rainfall =
        components::dfrobot_rainfall_sensor::DFRobotRainFallSensorComponent::new(
            mux_i2c, 0x1D, mux_alarm,
        )
        .finalize(components::dfrobot_rainfall_sensor_component_static!(
            apollo3::stimer::STimer<'static>,
            apollo3::iom::Iom<'static>
        ));

    let rainfall = components::rainfall::RainFallComponent::new(
        board_kernel,
        capsules_extra::rainfall::DRIVER_NUM,
        dfrobot_rainfall,
    )
    .finalize(components::rainfall_component_static!(DFRobotRainFallType));

    rainfall
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
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules_extra::air_quality::DRIVER_NUM => f(Some(self.air_quality)),
            capsules_extra::kv_driver::DRIVER_NUM => f(Some(self.kv_driver)),
            capsules_core::rng::DRIVER_NUM => {
                if let Some(rng) = self.rng {
                    f(Some(rng))
                } else {
                    f(None)
                }
            }
            capsules_extra::moisture::DRIVER_NUM => {
                if let Some(moisture) = self.moisture {
                    f(Some(moisture))
                } else {
                    f(None)
                }
            }
            capsules_extra::rainfall::DRIVER_NUM => {
                if let Some(rainfall) = self.rainfall {
                    f(Some(rainfall))
                } else {
                    f(None)
                }
            }
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
) {
    let peripherals = static_init!(Apollo3DefaultPeripherals, Apollo3DefaultPeripherals::new());
    PERIPHERALS = Some(peripherals);

    // No need to statically allocate mcu/pwr/clk_ctrl because they are only used in main!
    let mcu_ctrl = apollo3::mcuctrl::McuCtrl::new();
    let pwr_ctrl = apollo3::pwrctrl::PwrCtrl::new();
    let clkgen = apollo3::clkgen::ClkGen::new();

    clkgen.set_clock_frequency(apollo3::clkgen::ClockFrequency::Freq48MHz);

    // initialize capabilities
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // Power up components
    pwr_ctrl.enable_uart0();
    pwr_ctrl.enable_iom0();
    pwr_ctrl.enable_iom2();
    pwr_ctrl.enable_iom3();

    peripherals.init();

    // Enable PinCfg
    peripherals
        .gpio_port
        .enable_uart(&peripherals.gpio_port[48], &peripherals.gpio_port[49]);
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
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
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

    // Enable SDA and SCL for I2C (exposed via Qwiic)
    peripherals
        .gpio_port
        .enable_i2c(&peripherals.gpio_port[6], &peripherals.gpio_port[5]);

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

    #[cfg(feature = "chirp_i2c_moisture")]
    let moisture = Some(setup_chirp_i2c_moisture(
        board_kernel,
        &memory_allocation_cap,
        mux_i2c,
    ));
    #[cfg(not(feature = "chirp_i2c_moisture"))]
    let moisture = None;

    #[cfg(feature = "dfrobot_i2c_rainfall")]
    let rainfall = Some(setup_dfrobot_i2c_rainfall(
        board_kernel,
        &memory_allocation_cap,
        mux_i2c,
        mux_alarm,
    ));
    #[cfg(not(feature = "dfrobot_i2c_rainfall"))]
    let rainfall = None;

    #[cfg(feature = "atecc508a")]
    let rng = Some(setup_atecc508a(
        board_kernel,
        &memory_allocation_cap,
        mux_i2c,
    ));
    #[cfg(not(feature = "atecc508a"))]
    let rng = None;

    // Init the broken out SPI controller
    let external_mux_spi = components::spi::SpiMuxComponent::new(&peripherals.iom2).finalize(
        components::spi_mux_component_static!(apollo3::iom::Iom<'static>),
    );

    let external_spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        external_mux_spi,
        kernel::hil::spi::cs::IntoChipSelect::<_, kernel::hil::spi::cs::ActiveLow>::into_cs(
            &peripherals.gpio_port[11], // A5
        ),
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
        kernel::hil::spi::cs::IntoChipSelect::<_, kernel::hil::spi::cs::ActiveLow>::into_cs(
            &peripherals.gpio_port[36], // H6 - SX1262 Slave Select
        ),
        LORA_SPI_DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        apollo3::iom::Iom<'static>
    ));
    peripherals
        .iom3
        .specify_chip_select(kernel::hil::spi::cs::IntoChipSelect::<
            _,
            kernel::hil::spi::cs::ActiveLow,
        >::into_cs(
            &peripherals.gpio_port[36], // H6 - SX1262 Slave Select
        ))
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
    mcu_ctrl.disable_ble();

    // Flash
    let flash_ctrl_read_buf = static_init!(
        [u8; apollo3::flashctrl::PAGE_SIZE],
        [0; apollo3::flashctrl::PAGE_SIZE]
    );
    let page_buffer = static_init!(
        apollo3::flashctrl::Apollo3Page,
        apollo3::flashctrl::Apollo3Page::default()
    );

    let mux_flash = components::flash::FlashMuxComponent::new(&peripherals.flash_ctrl).finalize(
        components::flash_mux_component_static!(apollo3::flashctrl::FlashCtrl),
    );

    // SipHash
    let sip_hash = static_init!(
        capsules_extra::sip_hash::SipHasher24,
        capsules_extra::sip_hash::SipHasher24::new()
    );
    kernel::deferred_call::DeferredCallClient::register(sip_hash);

    // TicKV
    let tickv = components::tickv::TicKVComponent::new(
        sip_hash,
        mux_flash, // Flash controller
        core::ptr::addr_of!(_skv_data) as usize / apollo3::flashctrl::PAGE_SIZE, // Region offset (Last 0x28000 bytes of flash)
        // Region Size, the final page doens't work correctly
        core::ptr::addr_of!(_lkv_data) as usize - apollo3::flashctrl::PAGE_SIZE,
        flash_ctrl_read_buf, // Buffer used internally in TicKV
        page_buffer,         // Buffer used with the flash controller
    )
    .finalize(components::tickv_component_static!(
        apollo3::flashctrl::FlashCtrl,
        capsules_extra::sip_hash::SipHasher24,
        { apollo3::flashctrl::PAGE_SIZE }
    ));
    HasClient::set_client(&peripherals.flash_ctrl, mux_flash);
    sip_hash.set_client(tickv);

    let kv_store = components::kv::TicKVKVStoreComponent::new(tickv).finalize(
        components::tickv_kv_store_component_static!(
            capsules_extra::tickv::TicKVSystem<
                capsules_core::virtualizers::virtual_flash::FlashUser<
                    apollo3::flashctrl::FlashCtrl,
                >,
                capsules_extra::sip_hash::SipHasher24<'static>,
                { apollo3::flashctrl::PAGE_SIZE },
            >,
            capsules_extra::tickv::TicKVKeyType,
        ),
    );

    let kv_store_permissions = components::kv::KVStorePermissionsComponent::new(kv_store).finalize(
        components::kv_store_permissions_component_static!(
            capsules_extra::tickv_kv_store::TicKVKVStore<
                capsules_extra::tickv::TicKVSystem<
                    capsules_core::virtualizers::virtual_flash::FlashUser<
                        apollo3::flashctrl::FlashCtrl,
                    >,
                    capsules_extra::sip_hash::SipHasher24<'static>,
                    { apollo3::flashctrl::PAGE_SIZE },
                >,
                capsules_extra::tickv::TicKVKeyType,
            >
        ),
    );

    let mux_kv = components::kv::KVPermissionsMuxComponent::new(kv_store_permissions).finalize(
        components::kv_permissions_mux_component_static!(
            capsules_extra::kv_store_permissions::KVStorePermissions<
                capsules_extra::tickv_kv_store::TicKVKVStore<
                    capsules_extra::tickv::TicKVSystem<
                        capsules_core::virtualizers::virtual_flash::FlashUser<
                            apollo3::flashctrl::FlashCtrl,
                        >,
                        capsules_extra::sip_hash::SipHasher24<'static>,
                        { apollo3::flashctrl::PAGE_SIZE },
                    >,
                    capsules_extra::tickv::TicKVKeyType,
                >,
            >
        ),
    );

    let virtual_kv_driver = components::kv::VirtualKVPermissionsComponent::new(mux_kv).finalize(
        components::virtual_kv_permissions_component_static!(
            capsules_extra::kv_store_permissions::KVStorePermissions<
                capsules_extra::tickv_kv_store::TicKVKVStore<
                    capsules_extra::tickv::TicKVSystem<
                        capsules_core::virtualizers::virtual_flash::FlashUser<
                            apollo3::flashctrl::FlashCtrl,
                        >,
                        capsules_extra::sip_hash::SipHasher24<'static>,
                        { apollo3::flashctrl::PAGE_SIZE },
                    >,
                    capsules_extra::tickv::TicKVKeyType,
                >,
            >
        ),
    );

    let kv_driver = components::kv::KVDriverComponent::new(
        virtual_kv_driver,
        board_kernel,
        capsules_extra::kv_driver::DRIVER_NUM,
    )
    .finalize(components::kv_driver_component_static!(
        capsules_extra::virtual_kv::VirtualKVPermissions<
            capsules_extra::kv_store_permissions::KVStorePermissions<
                capsules_extra::tickv_kv_store::TicKVKVStore<
                    capsules_extra::tickv::TicKVSystem<
                        capsules_core::virtualizers::virtual_flash::FlashUser<
                            apollo3::flashctrl::FlashCtrl,
                        >,
                        capsules_extra::sip_hash::SipHasher24<'static>,
                        { apollo3::flashctrl::PAGE_SIZE },
                    >,
                    capsules_extra::tickv::TicKVKeyType,
                >,
            >,
        >
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
        /// Beginning of the RAM region containing K/V data.
        static _skv_data: u8;
        /// Length of the RAM region containing K/V data.
        static _lkv_data: u8;
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
            temperature,
            humidity,
            air_quality,
            moisture,
            rainfall,
            rng,
            scheduler,
            systick,
            kv_driver,
        }
    );

    let chip = static_init!(
        apollo3::chip::Apollo3<Apollo3DefaultPeripherals>,
        apollo3::chip::Apollo3::new(peripherals)
    );
    CHIP = Some(chip);

    let checking_policy;
    #[cfg(feature = "atecc508a")]
    {
        // Create the software-based SHA engine.
        // We could use the ATECC508a for SHA, but writing the entire
        // application to the device to compute a digtest ends up being
        // pretty slow and the ATECC508a doesn't support the DigestVerify trait
        let sha = components::sha::ShaSoftware256Component::new()
            .finalize(components::sha_software_256_component_static!());

        // These are the generated test keys used below, please do not use them
        // for anything important!!!!
        //
        // These keys are not leaked, they are only used for this test case.
        //
        // -----BEGIN PRIVATE KEY-----
        // MIGHAgEBMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgWClhguWHtAK85Kqc
        // /BucDBQMGQw6R2PEQkyISHkn5xWhRANCAAQUFMTFoNL9oFpGmg6Cp351hQMq9hol
        // KpEdQfjP1nYF1jxqz52YjPpFHvudkK/fFsik5Rd0AevNkQqjBdWEqmpW
        // -----END PRIVATE KEY-----
        //
        // -----BEGIN PUBLIC KEY-----
        // MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEFBTExaDS/aBaRpoOgqd+dYUDKvYa
        // JSqRHUH4z9Z2BdY8as+dmIz6RR77nZCv3xbIpOUXdAHrzZEKowXVhKpqVg==
        // -----END PUBLIC KEY-----
        let public_key = static_init!(
            [u8; 64],
            [
                0x14, 0x14, 0xc4, 0xc5, 0xa0, 0xd2, 0xfd, 0xa0, 0x5a, 0x46, 0x9a, 0x0e, 0x82, 0xa7,
                0x7e, 0x75, 0x85, 0x03, 0x2a, 0xf6, 0x1a, 0x25, 0x2a, 0x91, 0x1d, 0x41, 0xf8, 0xcf,
                0xd6, 0x76, 0x05, 0xd6, 0x3c, 0x6a, 0xcf, 0x9d, 0x98, 0x8c, 0xfa, 0x45, 0x1e, 0xfb,
                0x9d, 0x90, 0xaf, 0xdf, 0x16, 0xc8, 0xa4, 0xe5, 0x17, 0x74, 0x01, 0xeb, 0xcd, 0x91,
                0x0a, 0xa3, 0x05, 0xd5, 0x84, 0xaa, 0x6a, 0x56
            ]
        );

        ATECC508A.unwrap().set_public_key(Some(public_key));

        checking_policy = components::appid::checker_signature::AppCheckerSignatureComponent::new(
            sha,
            ATECC508A.unwrap(),
            tock_tbf::types::TbfFooterV2CredentialsType::EcdsaNistP256,
        )
        .finalize(components::app_checker_signature_component_static!(
            capsules_extra::atecc508a::Atecc508a<'static>,
            capsules_extra::sha256::Sha256Software<'static>,
            32,
            64,
        ));
    };
    #[cfg(not(feature = "atecc508a"))]
    {
        checking_policy = components::appid::checker_null::AppCheckerNullComponent::new()
            .finalize(components::app_checker_null_component_static!());
    }

    // Create the AppID assigner.
    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    let storage_permissions_policy =
        components::storage_permissions::tbf_header::StoragePermissionsTbfHeaderComponent::new()
            .finalize(
                components::storage_permissions_tbf_header_component_static!(
                    apollo3::chip::Apollo3<Apollo3DefaultPeripherals>,
                    kernel::process::ProcessStandardDebugFull,
                ),
            );

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
        apollo3::chip::Apollo3<Apollo3DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS,
    ));

    (board_kernel, artemis_nano, chip)
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
        let (board_kernel, sf_lora_thing_plus_board, chip) = setup();

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
        let (board_kernel, sf_lora_thing_plus_board, _chip) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&sf_lora_thing_plus_board);
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
