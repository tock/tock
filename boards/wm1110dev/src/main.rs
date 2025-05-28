// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Tock kernel for the Wio WM1110 Development Board.
//!
//! It is based on nRF52840 SoC and Semtech LR1110.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::hil::led::LedHigh;
use kernel::hil::spi::SpiMaster;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// Three-color LED.
const LED_RED_PIN: Pin = Pin::P0_14;
const LED_GREEN_PIN: Pin = Pin::P0_13;

const BUTTON_RST_PIN: Pin = Pin::P0_18;

const GPIO_D2: Pin = Pin::P0_17;
const GPIO_D3: Pin = Pin::P0_16;
const GPIO_D4: Pin = Pin::P0_15;
const GPIO_D5: Pin = Pin::P1_09;
const GPIO_D6: Pin = Pin::P1_04;
const GPIO_D7: Pin = Pin::P1_03;

const UART_TX_PIN: Pin = Pin::P0_24;
const UART_RX_PIN: Pin = Pin::P0_22;

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_27;
const I2C_SCL_PIN: Pin = Pin::P0_26;

// Pins for communicating with LR1110
const SPI_CS_PIN: Pin = Pin::P1_12;
const SPI_SCK_PIN: Pin = Pin::P1_13;
const SPI_MOSI_PIN: Pin = Pin::P1_14;
const SPI_MISO_PIN: Pin = Pin::P1_15;
const RADIO_BUSY_PIN: Pin = Pin::P1_11;
const RADIO_RESET_PIN: Pin = Pin::P1_10;

const LR_DIO9: Pin = Pin::P1_08;

/// GPIO pin that controls VCC for the I2C bus and sensors.
const I2C_PWR: Pin = Pin::P0_07;

const LORA_SPI_DRIVER_NUM: usize = capsules_core::driver::NUM::LoRaPhySPI as usize;
const LORA_GPIO_DRIVER_NUM: usize = capsules_core::driver::NUM::LoRaPhyGPIO as usize;

/// UART Writer for panic!()s.
pub mod io;

// How should the kernel respond when a process faults. For this board we choose
// to stop the app and print a notice, but not immediately panic.
const FAULT_RESPONSE: capsules_system::process_policies::StopWithDebugFaultPolicy =
    capsules_system::process_policies::StopWithDebugFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

// State for loading and holding applications.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

type SHT4xSensor = components::sht4x::SHT4xComponentType<
    capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, nrf52840::i2c::TWI<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<SHT4xSensor>;
type HumidityDriver = components::humidity::HumidityComponentType<SHT4xSensor>;
type RngDriver = components::rng::RngComponentType<nrf52840::trng::Trng<'static>>;

type NonvolatileDriver = components::nonvolatile_storage::NonvolatileStorageComponentType;

/// Supported drivers by the platform
pub struct Platform {
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, nrf52::gpio::GPIOPin<'static>>,
        2,
    >,
    rng: &'static RngDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    nonvolatile_storage: &'static NonvolatileDriver,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
    >,
    temperature: &'static TemperatureDriver,
    humidity: &'static HumidityDriver,
    lr1110_gpio: &'static capsules_core::gpio::GPIO<'static, nrf52840::gpio::GPIOPin<'static>>,
    lr1110_spi: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            nrf52840::spi::SPIM<'static>,
        >,
    >,
    scheduler: &'static RoundRobinSched<'static>,
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
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(self.nonvolatile_storage))
            }
            LORA_SPI_DRIVER_NUM => f(Some(self.lr1110_spi)),
            LORA_GPIO_DRIVER_NUM => f(Some(self.lr1110_gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
            _ => f(None),
        }
    }
}

impl KernelResources<nrf52::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    Platform,
    &'static nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>,
) {
    nrf52840::init();

    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );

    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    nrf52_components::startup::NrfStartupComponent::new(
        false,
        BUTTON_RST_PIN,
        nrf52840::uicr::Regulator0Output::DEFAULT,
        &base_peripherals.nvmc,
    )
    .finalize(());

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

    // Configure kernel debug GPIOs as early as possible. These are used by the
    // `debug_gpio!(0, toggle)` macro. We configure these early so that the
    // macro is available during most of the setup code and kernel execution.
    kernel::debug::assign_gpios(
        Some(&nrf52840_peripherals.gpio_port[LED_GREEN_PIN]),
        Some(&nrf52840_peripherals.gpio_port[LED_RED_PIN]),
        None,
    );

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            2 => &nrf52840_peripherals.gpio_port[GPIO_D2],
            3 => &nrf52840_peripherals.gpio_port[GPIO_D3],
            4 => &nrf52840_peripherals.gpio_port[GPIO_D4],
            5 => &nrf52840_peripherals.gpio_port[GPIO_D5],
            6 => &nrf52840_peripherals.gpio_port[GPIO_D6],
            7 => &nrf52840_peripherals.gpio_port[GPIO_D7],
        ),
    )
    .finalize(components::gpio_component_static!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, nrf52840::gpio::GPIOPin>,
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED_GREEN_PIN]),
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED_RED_PIN]),
    ));

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(nrf52::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(nrf52::rtc::Rtc));

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
    let uart_mux = components::console::UartMuxComponent::new(&base_peripherals.uarte0, 115200)
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

    //--------------------------------------------------------------------------
    // SENSORS
    //--------------------------------------------------------------------------

    // Enable the power supply for the I2C bus and attached sensors.
    nrf52840_peripherals.gpio_port[I2C_PWR].make_output();
    nrf52840_peripherals.gpio_port[I2C_PWR].set();

    let mux_i2c = components::i2c::I2CMuxComponent::new(&base_peripherals.twi1, None)
        .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    base_peripherals.twi1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );

    let sht4x = components::sht4x::SHT4xComponent::new(
        mux_i2c,
        capsules_extra::sht4x::BASE_ADDR,
        mux_alarm,
    )
    .finalize(components::sht4x_component_static!(
        nrf52::rtc::Rtc<'static>,
        nrf52840::i2c::TWI
    ));

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        sht4x,
    )
    .finalize(components::temperature_component_static!(SHT4xSensor));

    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules_extra::humidity::DRIVER_NUM,
        sht4x,
    )
    .finalize(components::humidity_component_static!(SHT4xSensor));

    //--------------------------------------------------------------------------
    // LoRa (SPI + GPIO)
    //--------------------------------------------------------------------------

    let mux_spi = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_static!(nrf52840::spi::SPIM));

    // Create the SPI system call capsule for accessing the LoRa radio.
    let lr1110_spi = components::spi::SpiSyscallComponent::new(
        board_kernel,
        mux_spi,
        hil::spi::cs::IntoChipSelect::<_, hil::spi::cs::ActiveLow>::into_cs(
            &nrf52840_peripherals.gpio_port[SPI_CS_PIN],
        ),
        LORA_SPI_DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        nrf52840::spi::SPIM
    ));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(SPI_MOSI_PIN as u32),
        nrf52840::pinmux::Pinmux::new(SPI_MISO_PIN as u32),
        nrf52840::pinmux::Pinmux::new(SPI_SCK_PIN as u32),
    );

    base_peripherals
        .spim0
        .specify_chip_select(
            hil::spi::cs::IntoChipSelect::<_, hil::spi::cs::ActiveLow>::into_cs(
                &nrf52840_peripherals.gpio_port[SPI_CS_PIN],
            ),
        )
        .unwrap();

    // Pin mappings from the original WM1110 source code.
    let lr1110_gpio = components::gpio::GpioComponent::new(
        board_kernel,
        LORA_GPIO_DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            40 => &nrf52840_peripherals.gpio_port[LR_DIO9],
            42 => &nrf52840_peripherals.gpio_port[RADIO_RESET_PIN],
            43 => &nrf52840_peripherals.gpio_port[RADIO_BUSY_PIN],
        ),
    )
    .finalize(components::gpio_component_static!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Process Console
    //--------------------------------------------------------------------------

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    let _process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        nrf52840::rtc::Rtc
    ));

    //--------------------------------------------------------------------------
    // RANDOM NUMBERS
    //--------------------------------------------------------------------------

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &base_peripherals.trng,
    )
    .finalize(components::rng_component_static!(nrf52840::trng::Trng));

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::nonvolatile_storage_driver::DRIVER_NUM,
        &base_peripherals.nvmc,
        0xFC000,  // Start address for userspace accessible region
        4096 * 4, // Length of userspace accessible region (16 pages)
        0,        // No kernel access
        0,
    )
    .finalize(components::nonvolatile_storage_component_static!(
        nrf52840::nvmc::Nvmc
    ));

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        console,
        led,
        gpio,
        rng,
        alarm,
        nonvolatile_storage,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
        temperature,
        humidity,
        lr1110_spi,
        lr1110_gpio,
    };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    //--------------------------------------------------------------------------
    // TESTS
    //--------------------------------------------------------------------------

    //--------------------------------------------------------------------------
    // BOOT COMPLETE
    //--------------------------------------------------------------------------

    debug!("Initialization complete. Entering main loop.");
    let _ = _process_console.start();

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
        &mut *addr_of_mut!(PROCESSES),
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

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
