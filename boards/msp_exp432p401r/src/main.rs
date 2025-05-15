// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for the MSP-EXP432P401R evaluation board from TI.
//!
//! - <https://www.ti.com/tool/MSP-EXP432P401R>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

/// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

/// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static msp432::chip::Msp432<msp432::chip::Msp432DefaultPeripherals>> =
    None;
// Static reference to process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct MspExp432P401R {
    led: &'static capsules_core::led::LedDriver<
        'static,
        kernel::hil::led::LedHigh<'static, msp432::gpio::IntPin<'static>>,
        3,
    >,
    console: &'static capsules_core::console::Console<'static>,
    button: &'static capsules_core::button::Button<'static, msp432::gpio::IntPin<'static>>,
    gpio: &'static capsules_core::gpio::GPIO<'static, msp432::gpio::IntPin<'static>>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            msp432::timer::TimerA<'static>,
        >,
    >,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    adc: &'static capsules_core::adc::AdcDedicated<'static, msp432::adc::Adc<'static>>,
    wdt: &'static msp432::wdt::Wdt,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl KernelResources<msp432::chip::Msp432<'static, msp432::chip::Msp432DefaultPeripherals<'static>>>
    for MspExp432P401R
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = msp432::wdt::Wdt;
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
        self.wdt
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for MspExp432P401R {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            _ => f(None),
        }
    }
}

/// Startup initialisation
///
/// This code was more or less copied from the code examples of Texas instruments.
/// It disables the watchdog, enables all RAM banks, configures the chip to the high-power mode
/// (which is necessary for 48MHz operation) and enables waitstates and buffering in a way that
/// the flash returns valid data with 48MHz CPU frequency.
unsafe fn startup_intilialisation() {
    msp432::init();

    // For now, these peripherals are only used at startup, so we do not
    // allocate them for the life of the program. If these are later used by the
    // chip crate (such as for handling interrupts), they will need to be
    // added to Msp432DefaultPeripherals
    let wdt = msp432::wdt::Wdt::new();
    let sysctl = msp432::sysctl::SysCtl::new();
    let flctl = msp432::flctl::FlCtl::new();
    let pcm = msp432::pcm::Pcm::new();

    // The watchdog must be disabled, because it is enabled by default on reset and has a
    // interval of approximately 10ms. See datasheet p. 759, section 17.2.2.
    // Do this in a separate function which is executed before the kernel is started in order to
    // make sure that not more than 1 watchdog instances exist at the same time.
    wdt.disable();
    sysctl.enable_all_sram_banks();
    pcm.set_high_power();
    flctl.set_waitstates(msp432::flctl::WaitStates::_1);
    flctl.set_buffering(true);
}

/// Function to setup all ADC-capaable pins
/// Since the chips has 100 pins, we really setup all capable pins to work as ADC-pins.
unsafe fn setup_adc_pins(gpio: &msp432::gpio::GpioManager) {
    use msp432::gpio::{IntPinNr, PinNr};
    gpio.int_pins[IntPinNr::P05_5 as usize].enable_tertiary_function(); // A0
    gpio.int_pins[IntPinNr::P05_4 as usize].enable_tertiary_function(); // A1
    gpio.int_pins[IntPinNr::P05_3 as usize].enable_tertiary_function(); // A2
    gpio.int_pins[IntPinNr::P05_2 as usize].enable_tertiary_function(); // A3
    gpio.int_pins[IntPinNr::P05_1 as usize].enable_tertiary_function(); // A4
    gpio.int_pins[IntPinNr::P05_0 as usize].enable_tertiary_function(); // A5
    gpio.int_pins[IntPinNr::P04_7 as usize].enable_tertiary_function(); // A6
    gpio.int_pins[IntPinNr::P04_6 as usize].enable_tertiary_function(); // A7
    gpio.int_pins[IntPinNr::P04_5 as usize].enable_tertiary_function(); // A8
    gpio.int_pins[IntPinNr::P04_4 as usize].enable_tertiary_function(); // A9
    gpio.int_pins[IntPinNr::P04_3 as usize].enable_tertiary_function(); // A10
    gpio.int_pins[IntPinNr::P04_2 as usize].enable_tertiary_function(); // A11
    gpio.int_pins[IntPinNr::P04_1 as usize].enable_tertiary_function(); // A12
    gpio.int_pins[IntPinNr::P04_0 as usize].enable_tertiary_function(); // A13
    gpio.int_pins[IntPinNr::P06_1 as usize].enable_tertiary_function(); // A14
    gpio.int_pins[IntPinNr::P06_0 as usize].enable_tertiary_function(); // A15
    gpio.pins[PinNr::P09_1 as usize].enable_tertiary_function(); // A16
    gpio.pins[PinNr::P09_0 as usize].enable_tertiary_function(); // A17
    gpio.pins[PinNr::P08_7 as usize].enable_tertiary_function(); // A18
    gpio.pins[PinNr::P08_6 as usize].enable_tertiary_function(); // A19
    gpio.pins[PinNr::P08_5 as usize].enable_tertiary_function(); // A20
    gpio.pins[PinNr::P08_4 as usize].enable_tertiary_function(); // A21

    // Don't configure these pins since their channels are used for the internal
    // temperature sensor (Channel 22) and the Battery Monitor (A23)
    // gpio.pins[PinNr::P08_3 as usize].enable_tertiary_function(); // A22
    // gpio.pins[PinNr::P08_2 as usize].enable_tertiary_function(); // A23
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    MspExp432P401R,
    &'static msp432::chip::Msp432<'static, msp432::chip::Msp432DefaultPeripherals<'static>>,
) {
    startup_intilialisation();

    let peripherals = static_init!(
        msp432::chip::Msp432DefaultPeripherals,
        msp432::chip::Msp432DefaultPeripherals::new()
    );
    peripherals.init();

    // Setup the GPIO pins to use the HFXT (high frequency external) oscillator (48MHz)
    peripherals.gpio.pins[msp432::gpio::PinNr::PJ_2 as usize].enable_primary_function();
    peripherals.gpio.pins[msp432::gpio::PinNr::PJ_3 as usize].enable_primary_function();

    // Setup the GPIO pins to use the LFXT (low frequency external) oscillator (32.768kHz)
    peripherals.gpio.pins[msp432::gpio::PinNr::PJ_0 as usize].enable_primary_function();
    peripherals.gpio.pins[msp432::gpio::PinNr::PJ_1 as usize].enable_primary_function();

    // Setup the clocks: MCLK: 48MHz, HSMCLK: 12MHz, SMCLK: 1.5MHz, ACLK: 32.768kHz
    peripherals.cs.setup_clocks();

    // Setup the debug GPIOs
    let dbg_gpio0 = &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_0 as usize];
    let dbg_gpio1 = &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_5 as usize];
    let dbg_gpio2 = &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_7 as usize];
    dbg_gpio0.make_output();
    dbg_gpio1.make_output();
    dbg_gpio2.make_output();
    debug::assign_gpios(
        Some(dbg_gpio0), // Red LED
        Some(dbg_gpio1),
        Some(dbg_gpio2),
    );

    // Setup pins for UART0
    peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_2 as usize].enable_primary_function();
    peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_3 as usize].enable_primary_function();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));
    let chip = static_init!(
        msp432::chip::Msp432<msp432::chip::Msp432DefaultPeripherals>,
        msp432::chip::Msp432::new(peripherals)
    );
    CHIP = Some(chip);

    // Setup buttons
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            msp432::gpio::IntPin,
            (
                &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_1 as usize],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_4 as usize],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_static!(msp432::gpio::IntPin));

    // Setup LEDs
    let leds = components::led::LedsComponent::new().finalize(components::led_component_static!(
        kernel::hil::led::LedHigh<'static, msp432::gpio::IntPin>,
        kernel::hil::led::LedHigh::new(
            &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_0 as usize]
        ),
        kernel::hil::led::LedHigh::new(
            &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_1 as usize]
        ),
        kernel::hil::led::LedHigh::new(
            &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_2 as usize]
        ),
    ));

    // Setup user-GPIOs
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            msp432::gpio::IntPin<'static>,
            // Left outer connector, top to bottom
            // 0 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P06_0 as usize], // A15
            1 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_2 as usize],
            2 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_3 as usize],
            // 3 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_1 as usize], // A12
            // 4 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_3 as usize], // A10
            5 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_5 as usize],
            // 6 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_6 as usize], // A7
            7 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P06_5 as usize],
            8 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P06_4 as usize],
            // Left inner connector, top to bottom
            // 9 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P06_1 as usize], // A14
            // 10 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_0 as usize], // A13
            // 11 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_2 as usize], // A11
            // 12 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_4 as usize], // A9
            // 13 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_5 as usize], // A8
            // 14 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P04_7 as usize], // A6
            // 15 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_4 as usize], // A1
            // 16 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_5 as usize], // A0
            // Right inner connector, top to bottom
            17 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_7 as usize],
            18 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_6 as usize],
            19 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_4 as usize],
            20 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_6 as usize],
            21 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P06_6 as usize],
            22 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P06_7 as usize],
            23 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_3 as usize],
            // 24 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_1 as usize], // A4
            // 25 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_5 as usize], // debug-gpio
            // 26 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_7 as usize], // debug-gpio
            // Right outer connector, top to bottom
            27 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P02_5 as usize],
            28 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_0 as usize],
            29 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_7 as usize],
            30 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_6 as usize],
            31 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P01_7 as usize],
            // 32 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_0 as usize], // A5
            // 33 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P05_2 as usize], // A3
            34 => &peripherals.gpio.int_pins[msp432::gpio::IntPinNr::P03_6 as usize]
        ),
    )
    .finalize(components::gpio_component_static!(
        msp432::gpio::IntPin<'static>
    ));

    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // Setup UART0
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

    // Setup alarm
    let timer0 = &peripherals.timer_a0;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(timer0).finalize(
        components::alarm_mux_component_static!(msp432::timer::TimerA),
    );
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(msp432::timer::TimerA));

    // Setup ADC
    setup_adc_pins(&peripherals.gpio);

    let adc_channels = static_init!(
        [msp432::adc::Channel; 24],
        [
            msp432::adc::Channel::Channel0,  // A0
            msp432::adc::Channel::Channel1,  // A1
            msp432::adc::Channel::Channel2,  // A2
            msp432::adc::Channel::Channel3,  // A3
            msp432::adc::Channel::Channel4,  // A4
            msp432::adc::Channel::Channel5,  // A5
            msp432::adc::Channel::Channel6,  // A6
            msp432::adc::Channel::Channel7,  // A7
            msp432::adc::Channel::Channel8,  // A8
            msp432::adc::Channel::Channel9,  // A9
            msp432::adc::Channel::Channel10, // A10
            msp432::adc::Channel::Channel11, // A11
            msp432::adc::Channel::Channel12, // A12
            msp432::adc::Channel::Channel13, // A13
            msp432::adc::Channel::Channel14, // A14
            msp432::adc::Channel::Channel15, // A15
            msp432::adc::Channel::Channel16, // A16
            msp432::adc::Channel::Channel17, // A17
            msp432::adc::Channel::Channel18, // A18
            msp432::adc::Channel::Channel19, // A19
            msp432::adc::Channel::Channel20, // A20
            msp432::adc::Channel::Channel21, // A21
            msp432::adc::Channel::Channel22, // A22
            msp432::adc::Channel::Channel23, // A23
        ]
    );
    let adc = components::adc::AdcDedicatedComponent::new(
        &peripherals.adc,
        adc_channels,
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
    )
    .finalize(components::adc_dedicated_component_static!(
        msp432::adc::Adc
    ));

    // Set the reference voltage for the ADC to 2.5V
    peripherals
        .adc_ref
        .select_ref_voltage(msp432::ref_module::ReferenceVoltage::Volt2_5);
    // Enable the internal temperature sensor on ADC Channel 22
    peripherals.adc_ref.enable_temp_sensor(true);

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    let msp_exp432p4014 = MspExp432P401R {
        led: leds,
        console,
        button,
        gpio,
        alarm,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        adc,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(48_000_000),
        wdt: &peripherals.wdt,
    };

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
    .unwrap();

    //Uncomment to run multi alarm test
    /*components::test::multi_alarm_test::MultiAlarmTestComponent::new(mux_alarm)
    .finalize(components::multi_alarm_test_component_buf!(msp432::timer::TimerA))
    .run();*/

    (board_kernel, msp_exp432p4014, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, board, chip) = start();
    board_kernel.kernel_loop(&board, chip, Some(&board.ipc), &main_loop_capability);
}
