//! Board file for SparkFun Redboard Artemis Nano
//!
//! - <https://www.sparkfun.com/products/15443>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::common::dynamic_deferred_call::DynamicDeferredCallClientState;
use kernel::component::Component;
use kernel::hil::i2c::I2CMaster;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

pub mod ble;
/// Support routines for debugging I/O.
pub mod io;

#[allow(dead_code)]
mod multi_alarm_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] = [None; 4];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static apollo3::chip::Apollo3> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct RedboardArtemisNano {
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
    led: &'static capsules::led::LED<'static, apollo3::gpio::GpioPin<'static>>,
    gpio: &'static capsules::gpio::GPIO<'static, apollo3::gpio::GpioPin<'static>>,
    console: &'static capsules::console::Console<'static>,
    i2c_master: &'static capsules::i2c_master::I2CMasterDriver<apollo3::iom::Iom<'static>>,
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        apollo3::ble::Ble<'static>,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for RedboardArtemisNano {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            _ => f(None),
        }
    }
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the Apollo3 chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    apollo3::init();

    apollo3::clkgen::CLKGEN.set_clock_frequency(apollo3::clkgen::ClockFrequency::Freq48MHz);

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Power up components
    apollo3::pwrctrl::PWRCTRL.enable_uart0();
    apollo3::pwrctrl::PWRCTRL.enable_iom2();

    // Enable PinCfg
    apollo3::gpio::PORT.enable_uart(&apollo3::gpio::PORT[48], &apollo3::gpio::PORT[49]);
    // Enable SDA and SCL for I2C2 (exposed via Qwiic)
    apollo3::gpio::PORT.enable_i2c(&apollo3::gpio::PORT[25], &apollo3::gpio::PORT[27]);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&apollo3::gpio::PORT[19]), // Blue LED
        None,
        None,
    );

    let chip = static_init!(apollo3::chip::Apollo3, apollo3::chip::Apollo3::new());
    CHIP = Some(chip);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &apollo3::uart::UART0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // LEDs
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        apollo3::gpio::GpioPin,
        (
            &apollo3::gpio::PORT[19],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ))
    .finalize(components::led_component_buf!(apollo3::gpio::GpioPin));

    // GPIOs
    // These are also ADC channels, but let's expose them as GPIOs
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            apollo3::gpio::GpioPin,
            0 => &apollo3::gpio::PORT[13],  // A0
            1 => &apollo3::gpio::PORT[33],  // A1
            2 => &apollo3::gpio::PORT[11],  // A2
            3 => &apollo3::gpio::PORT[29],  // A3
            5 => &apollo3::gpio::PORT[31]  // A5
        ),
    )
    .finalize(components::gpio_component_buf!(apollo3::gpio::GpioPin));

    // Create a shared virtualisation mux layer on top of a single hardware
    // alarm.
    let alarm = &apollo3::stimer::STIMER;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(alarm).finalize(
        components::alarm_mux_component_helper!(apollo3::stimer::STimer),
    );
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(apollo3::stimer::STimer));

    // Init the I2C device attached via Qwiic
    let i2c_master = static_init!(
        capsules::i2c_master::I2CMasterDriver<apollo3::iom::Iom<'static>>,
        capsules::i2c_master::I2CMasterDriver::new(
            &apollo3::iom::IOM2,
            &mut capsules::i2c_master::BUF,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );

    apollo3::iom::IOM2.set_master_client(i2c_master);
    apollo3::iom::IOM2.enable();

    // Setup BLE
    apollo3::mcuctrl::MCUCTRL.enable_ble();
    apollo3::clkgen::CLKGEN.enable_ble();
    apollo3::pwrctrl::PWRCTRL.enable_ble();
    apollo3::ble::BLE.setup_clocks();
    apollo3::mcuctrl::MCUCTRL.reset_ble();
    apollo3::ble::BLE.power_up();
    apollo3::ble::BLE.ble_initialise();

    let ble_radio =
        ble::BLEComponent::new(board_kernel, &apollo3::ble::BLE, mux_alarm).finalize(());

    apollo3::mcuctrl::MCUCTRL.print_chip_revision();

    debug!("Initialization complete. Entering main loop");

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

    let artemis_nano = RedboardArtemisNano {
        alarm,
        console,
        gpio,
        led,
        i2c_master,
        ble_radio,
    };

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
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    //Uncomment to run multi alarm test
    multi_alarm_test::run_multi_alarm(mux_alarm);

    board_kernel.kernel_loop(&artemis_nano, chip, None, scheduler, &main_loop_cap);
}
