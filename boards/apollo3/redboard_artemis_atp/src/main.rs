// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for SparkFun Redboard Artemis ATP
//!
//! - <https://www.sparkfun.com/products/15442>

#![no_std]
#![no_main]
#![deny(missing_docs)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use apollo3::chip::Apollo3DefaultPeripherals;
use capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver;
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::i2c::I2CSlave;
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
static mut PLATFORM: Option<&'static RedboardArtemisAtp> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, apollo3::stimer::STimer<'static>>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct RedboardArtemisAtp {
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
    i2c_master_slave: &'static capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver<
        'static,
        capsules_core::i2c_master_slave_combo::I2CMasterSlaveCombo<
            'static,
            apollo3::iom::Iom<'static>,
            apollo3::ios::Ios<'static>,
        >,
    >,
    spi_controller: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            apollo3::iom::Iom<'static>,
        >,
    >,
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        apollo3::ble::Ble<'static>,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for RedboardArtemisAtp {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::i2c_master_slave_driver::DRIVER_NUM => f(Some(self.i2c_master_slave)),
            capsules_core::spi_controller::DRIVER_NUM => f(Some(self.spi_controller)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            _ => f(None),
        }
    }
}

impl KernelResources<apollo3::chip::Apollo3<Apollo3DefaultPeripherals>> for RedboardArtemisAtp {
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
    &'static RedboardArtemisAtp,
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
    pwr_ctrl.enable_iom4();
    pwr_ctrl.enable_ios();

    peripherals.init();

    // Enable PinCfg
    peripherals
        .gpio_port
        .enable_uart(&peripherals.gpio_port[48], &peripherals.gpio_port[49]);
    // Enable SDA and SCL for I2C4 (exposed via Qwiic)
    peripherals
        .gpio_port
        .enable_i2c(&peripherals.gpio_port[40], &peripherals.gpio_port[39]);
    // Enable I2C slave device
    peripherals
        .gpio_port
        .enable_i2c_slave(&peripherals.gpio_port[1], &peripherals.gpio_port[0]);
    // Enable Main SPI
    peripherals.gpio_port.enable_spi(
        &peripherals.gpio_port[5],
        &peripherals.gpio_port[7],
        &peripherals.gpio_port[6],
    );

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&peripherals.gpio_port[19]), // Blue LED
        None,
        None,
    );

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
    // These are also ADC channels, but let's expose them as GPIOs
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            apollo3::gpio::GpioPin,
            0 => &peripherals.gpio_port[2],  // D2
            1 => &peripherals.gpio_port[8],  // D8
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

    let i2c_master_slave_combo = static_init!(
        capsules_core::i2c_master_slave_combo::I2CMasterSlaveCombo<
            'static,
            apollo3::iom::Iom<'static>,
            apollo3::ios::Ios<'static>,
        >,
        capsules_core::i2c_master_slave_combo::I2CMasterSlaveCombo::new(
            &peripherals.iom4,
            &peripherals.ios
        )
    );

    let i2c_master_buffer = static_init!([u8; 32], [0; 32]);
    let i2c_slave_buffer1 = static_init!([u8; 32], [0; 32]);
    let i2c_slave_buffer2 = static_init!([u8; 32], [0; 32]);

    let i2c_master_slave = static_init!(
        I2CMasterSlaveDriver<
            capsules_core::i2c_master_slave_combo::I2CMasterSlaveCombo<
                'static,
                apollo3::iom::Iom<'static>,
                apollo3::ios::Ios<'static>,
            >,
        >,
        I2CMasterSlaveDriver::new(
            i2c_master_slave_combo,
            i2c_master_buffer,
            i2c_slave_buffer1,
            i2c_slave_buffer2,
            board_kernel.create_grant(
                capsules_core::i2c_master_slave_driver::DRIVER_NUM,
                &memory_allocation_cap
            ),
        )
    );

    i2c_master_slave_combo.set_master_client(i2c_master_slave);
    i2c_master_slave_combo.set_slave_client(i2c_master_slave);

    peripherals.iom4.enable();

    // Init the SPI controller
    let mux_spi = components::spi::SpiMuxComponent::new(&peripherals.iom0).finalize(
        components::spi_mux_component_static!(apollo3::iom::Iom<'static>),
    );

    // The IOM0 expects an auto chip select on pin D11 or D15
    // We already use manual CS control for other Apollo3 boards, so
    // let's use A13 as it's broken out next to the SPI ports
    let spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        mux_spi,
        kernel::hil::spi::cs::IntoChipSelect::<_, kernel::hil::spi::cs::ActiveLow>::into_cs(
            &peripherals.gpio_port[13], // A13
        ),
        capsules_core::spi_controller::DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        apollo3::iom::Iom<'static>
    ));

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

    let artemis_atp = static_init!(
        RedboardArtemisAtp,
        RedboardArtemisAtp {
            alarm,
            led,
            gpio,
            console,
            i2c_master_slave,
            spi_controller,
            ble_radio,
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

    (board_kernel, artemis_atp, chip, peripherals)
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
