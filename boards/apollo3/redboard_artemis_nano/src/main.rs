//! Board file for SparkFun Redboard Artemis Nano
//!
//! - <https://www.sparkfun.com/products/15443>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use apollo3::chip::Apollo3DefaultPeripherals;
use components::bme280::Bme280Component;
use components::ccs811::Ccs811Component;
use core_capsules::virtual_alarm::MuxAlarm;
use core_capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::dynamic_deferred_call::DynamicDeferredCallClientState;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
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
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

// Test access to the peripherals
#[cfg(test)]
static mut PERIPHERALS: Option<&'static Apollo3DefaultPeripherals> = None;
// Test access to board
#[cfg(test)]
static mut BOARD: Option<&'static kernel::Kernel> = None;
// Test access to platform
#[cfg(test)]
static mut PLATFORM: Option<&'static RedboardArtemisNano> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, apollo3::stimer::STimer<'static>>> = None;
// Test access to sensors
static mut BME280: Option<&'static extra_capsules::bme280::Bme280<'static>> = None;
static mut CCS811: Option<&'static extra_capsules::ccs811::Ccs811<'static>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct RedboardArtemisNano {
    alarm: &'static core_capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
    led: &'static core_capsules::led::LedDriver<
        'static,
        LedHigh<'static, apollo3::gpio::GpioPin<'static>>,
        1,
    >,
    gpio: &'static core_capsules::gpio::GPIO<'static, apollo3::gpio::GpioPin<'static>>,
    console: &'static core_capsules::console::Console<'static>,
    i2c_master:
        &'static core_capsules::i2c_master::I2CMasterDriver<'static, apollo3::iom::Iom<'static>>,
    spi_controller: &'static core_capsules::spi_controller::Spi<
        'static,
        core_capsules::virtual_spi::VirtualSpiMasterDevice<'static, apollo3::iom::Iom<'static>>,
    >,
    ble_radio: &'static extra_capsules::ble_advertising_driver::BLE<
        'static,
        apollo3::ble::Ble<'static>,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
    temperature: &'static extra_capsules::temperature::TemperatureSensor<'static>,
    humidity: &'static extra_capsules::humidity::HumiditySensor<'static>,
    air_quality: &'static extra_capsules::air_quality::AirQualitySensor<'static>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for RedboardArtemisNano {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            core_capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            core_capsules::led::DRIVER_NUM => f(Some(self.led)),
            core_capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            core_capsules::console::DRIVER_NUM => f(Some(self.console)),
            core_capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            core_capsules::spi_controller::DRIVER_NUM => f(Some(self.spi_controller)),
            extra_capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            extra_capsules::temperature::DRIVER_NUM => f(Some(self.temperature)),
            extra_capsules::humidity::DRIVER_NUM => f(Some(self.humidity)),
            extra_capsules::air_quality::DRIVER_NUM => f(Some(self.air_quality)),
            _ => f(None),
        }
    }
}

impl KernelResources<apollo3::chip::Apollo3<Apollo3DefaultPeripherals>> for RedboardArtemisNano {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type CredentialsCheckingPolicy = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        &self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn credentials_checking_policy(&self) -> &'static Self::CredentialsCheckingPolicy {
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

unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static RedboardArtemisNano,
    &'static apollo3::chip::Apollo3<Apollo3DefaultPeripherals>,
    &'static Apollo3DefaultPeripherals,
) {
    apollo3::init();

    let peripherals = static_init!(Apollo3DefaultPeripherals, Apollo3DefaultPeripherals::new());

    // No need to statically allocate mcu/pwr/clk_ctrl because they are only used in main!
    let mcu_ctrl = apollo3::mcuctrl::McuCtrl::new();
    let pwr_ctrl = apollo3::pwrctrl::PwrCtrl::new();
    let clkgen = apollo3::clkgen::ClkGen::new();

    clkgen.set_clock_frequency(apollo3::clkgen::ClockFrequency::Freq48MHz);

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 4], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Power up components
    pwr_ctrl.enable_uart0();
    pwr_ctrl.enable_iom0();
    pwr_ctrl.enable_iom2();

    // Enable PinCfg
    let _ = &peripherals
        .gpio_port
        .enable_uart(&&peripherals.gpio_port[48], &&peripherals.gpio_port[49]);
    // Enable SDA and SCL for I2C2 (exposed via Qwiic)
    let _ = &peripherals
        .gpio_port
        .enable_i2c(&&peripherals.gpio_port[25], &&peripherals.gpio_port[27]);
    // Enable Main SPI
    let _ = &peripherals.gpio_port.enable_spi(
        &&peripherals.gpio_port[5],
        &&peripherals.gpio_port[7],
        &&peripherals.gpio_port[6],
    );

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&peripherals.gpio_port[19]), // Blue LED
        None,
        None,
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &peripherals.uart0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        core_capsules::console::DRIVER_NUM,
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
    // These are also ADC channels, but let's expose them as GPIOs
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        core_capsules::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            apollo3::gpio::GpioPin,
            0 => &&peripherals.gpio_port[13],  // A0
            1 => &&peripherals.gpio_port[33],  // A1
            2 => &&peripherals.gpio_port[11],  // A2
            3 => &&peripherals.gpio_port[29],  // A3
            5 => &&peripherals.gpio_port[31]  // A5
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
        core_capsules::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(apollo3::stimer::STimer));
    ALARM = Some(mux_alarm);

    // Create a process printer for panic.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Init the I2C device attached via Qwiic
    let i2c_master = static_init!(
        core_capsules::i2c_master::I2CMasterDriver<'static, apollo3::iom::Iom<'static>>,
        core_capsules::i2c_master::I2CMasterDriver::new(
            &peripherals.iom2,
            &mut core_capsules::i2c_master::BUF,
            board_kernel.create_grant(
                core_capsules::i2c_master::DRIVER_NUM,
                &memory_allocation_cap
            )
        )
    );

    let _ = &peripherals.iom2.set_master_client(i2c_master);
    let _ = &peripherals.iom2.enable();

    let mux_i2c =
        components::i2c::I2CMuxComponent::new(&peripherals.iom2, None, dynamic_deferred_caller)
            .finalize(components::i2c_mux_component_static!());

    let bme280 =
        Bme280Component::new(mux_i2c, 0x77).finalize(components::bme280_component_static!());
    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        extra_capsules::temperature::DRIVER_NUM,
        bme280,
    )
    .finalize(components::temperature_component_static!());
    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        extra_capsules::humidity::DRIVER_NUM,
        bme280,
    )
    .finalize(components::humidity_component_static!());
    BME280 = Some(bme280);

    let ccs811 = Ccs811Component::new(mux_i2c, 0x5B, dynamic_deferred_caller)
        .finalize(components::ccs811_component_static!());
    let air_quality = components::air_quality::AirQualityComponent::new(
        board_kernel,
        extra_capsules::temperature::DRIVER_NUM,
        ccs811,
    )
    .finalize(components::air_quality_component_static!());
    CCS811 = Some(ccs811);

    // Init the SPI controller
    let mux_spi = components::spi::SpiMuxComponent::new(&peripherals.iom0, dynamic_deferred_caller)
        .finalize(components::spi_mux_component_static!(
            apollo3::iom::Iom<'static>
        ));

    // The IOM0 expects an auto chip select on pin D11 or D15, neither of
    // which are broken out on the board. So the CS control must be manual
    let spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        mux_spi,
        &peripherals.gpio_port[35], // A14
        core_capsules::spi_controller::DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        apollo3::iom::Iom<'static>
    ));

    // Setup BLE
    mcu_ctrl.enable_ble();
    clkgen.enable_ble();
    pwr_ctrl.enable_ble();
    let _ = &peripherals.ble.setup_clocks();
    mcu_ctrl.reset_ble();
    let _ = &peripherals.ble.power_up();
    let _ = &peripherals.ble.ble_initialise();

    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        extra_capsules::ble_advertising_driver::DRIVER_NUM,
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

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let systick = cortexm4::systick::SysTick::new_with_calibration(48_000_000);

    let artemis_nano = static_init!(
        RedboardArtemisNano,
        RedboardArtemisNano {
            alarm,
            console,
            gpio,
            led,
            i2c_master,
            spi_controller,
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
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
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
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        let (board_kernel, esp32_c3_board, chip, _peripherals) = setup();

        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(
            esp32_c3_board,
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
        let (board_kernel, esp32_c3_board, _chip, peripherals) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&esp32_c3_board);
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
