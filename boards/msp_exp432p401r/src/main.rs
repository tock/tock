//! Board file for the MSP-EXP432P401R evaluation board from TI.
//!
//! - <https://www.ti.com/tool/MSP-EXP432P401R>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
#![deny(missing_docs)]

use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::common::dynamic_deferred_call::DynamicDeferredCallClientState;
use kernel::component::Component;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

#[allow(dead_code)]
mod multi_alarm_test;

/// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

/// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static msp432::chip::Msp432> = None;

/// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct MspExp432P401R {
    led: &'static capsules::led::LedDriver<
        'static,
        kernel::hil::led::LedHigh<'static, msp432::gpio::IntPin<'static>>,
    >,
    console: &'static capsules::console::Console<'static>,
    button: &'static capsules::button::Button<'static, msp432::gpio::IntPin<'static>>,
    gpio: &'static capsules::gpio::GPIO<'static, msp432::gpio::IntPin<'static>>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, msp432::timer::TimerA<'static>>,
    >,
    ipc: kernel::ipc::IPC,
    adc: &'static capsules::adc::AdcDedicated<'static, msp432::adc::Adc>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for MspExp432P401R {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
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
    let wdt = msp432::wdt::Wdt::new();

    msp432::init();

    // The watchdog must be disabled, because it is enabled by default on reset and has a
    // interval of approximately 10ms. See datasheet p. 759, section 17.2.2.
    // Do this in a separate function which is executed before the kernel is started in order to
    // make sure that not more than 1 watchdog instances exist at the same time.
    wdt.disable();
    msp432::sysctl::SYSCTL.enable_all_sram_banks();
    msp432::pcm::PCM.set_high_power();
    msp432::flctl::FLCTL.set_waitstates(msp432::flctl::WaitStates::_1);
    msp432::flctl::FLCTL.set_buffering(true);
}

/// Function to setup all ADC-capaable pins
/// Since the chips has 100 pins, we really setup all capable pins to work as ADC-pins.
unsafe fn setup_adc_pins() {
    use msp432::gpio::{IntPinNr, PinNr, INT_PINS, PINS};
    INT_PINS[IntPinNr::P05_5 as usize].enable_tertiary_function(); // A0
    INT_PINS[IntPinNr::P05_4 as usize].enable_tertiary_function(); // A1
    INT_PINS[IntPinNr::P05_3 as usize].enable_tertiary_function(); // A2
    INT_PINS[IntPinNr::P05_2 as usize].enable_tertiary_function(); // A3
    INT_PINS[IntPinNr::P05_1 as usize].enable_tertiary_function(); // A4
    INT_PINS[IntPinNr::P05_0 as usize].enable_tertiary_function(); // A5
    INT_PINS[IntPinNr::P04_7 as usize].enable_tertiary_function(); // A6
    INT_PINS[IntPinNr::P04_6 as usize].enable_tertiary_function(); // A7
    INT_PINS[IntPinNr::P04_5 as usize].enable_tertiary_function(); // A8
    INT_PINS[IntPinNr::P04_4 as usize].enable_tertiary_function(); // A9
    INT_PINS[IntPinNr::P04_3 as usize].enable_tertiary_function(); // A10
    INT_PINS[IntPinNr::P04_2 as usize].enable_tertiary_function(); // A11
    INT_PINS[IntPinNr::P04_1 as usize].enable_tertiary_function(); // A12
    INT_PINS[IntPinNr::P04_0 as usize].enable_tertiary_function(); // A13
    INT_PINS[IntPinNr::P06_1 as usize].enable_tertiary_function(); // A14
    INT_PINS[IntPinNr::P06_0 as usize].enable_tertiary_function(); // A15
    PINS[PinNr::P09_1 as usize].enable_tertiary_function(); // A16
    PINS[PinNr::P09_0 as usize].enable_tertiary_function(); // A17
    PINS[PinNr::P08_7 as usize].enable_tertiary_function(); // A18
    PINS[PinNr::P08_6 as usize].enable_tertiary_function(); // A19
    PINS[PinNr::P08_5 as usize].enable_tertiary_function(); // A20
    PINS[PinNr::P08_4 as usize].enable_tertiary_function(); // A21

    // Don't configure these pins since their channels are used for the internal
    // temperature sensor (Channel 22) and the Battery Monitor (A23)
    // PINS[PinNr::P08_3 as usize].enable_tertiary_function(); // A22
    // PINS[PinNr::P08_2 as usize].enable_tertiary_function(); // A23
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the MSP432 chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    startup_intilialisation();

    // Setup the GPIO pins to use the HFXT (high frequency external) oscillator (48MHz)
    msp432::gpio::PINS[msp432::gpio::PinNr::PJ_2 as usize].enable_primary_function();
    msp432::gpio::PINS[msp432::gpio::PinNr::PJ_3 as usize].enable_primary_function();

    // Setup the GPIO pins to use the LFXT (low frequency external) oscillator (32.768kHz)
    msp432::gpio::PINS[msp432::gpio::PinNr::PJ_0 as usize].enable_primary_function();
    msp432::gpio::PINS[msp432::gpio::PinNr::PJ_1 as usize].enable_primary_function();

    // Setup the clocks: MCLK: 48MHz, HSMCLK: 12MHz, SMCLK: 1.5MHz, ACLK: 32.768kHz
    msp432::cs::CS.setup_clocks();

    debug::assign_gpios(
        Some(&msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_0 as usize]), // Red LED
        Some(&msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_5 as usize]),
        Some(&msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_7 as usize]),
    );

    // Setup pins for UART0
    msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_2 as usize].enable_primary_function();
    msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_3 as usize].enable_primary_function();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let chip = static_init!(msp432::chip::Msp432, msp432::chip::Msp432::new());
    CHIP = Some(chip);

    // Setup buttons
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            msp432::gpio::IntPin,
            (
                &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_1 as usize],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ),
            (
                &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_4 as usize],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_buf!(msp432::gpio::IntPin));

    // Setup LEDs
    let leds = components::led::LedsComponent::new(components::led_component_helper!(
        kernel::hil::led::LedHigh<'static, msp432::gpio::IntPin>,
        kernel::hil::led::LedHigh::new(
            &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_0 as usize]
        ),
        kernel::hil::led::LedHigh::new(
            &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_1 as usize]
        ),
        kernel::hil::led::LedHigh::new(
            &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_2 as usize]
        ),
    ))
    .finalize(components::led_component_buf!(
        kernel::hil::led::LedHigh<'static, msp432::gpio::IntPin>
    ));

    // Setup user-GPIOs
    let gpio = GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            msp432::gpio::IntPin<'static>,
            // Left outer connector, top to bottom
            // 0 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P06_0 as usize], // A15
            1 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_2 as usize],
            2 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_3 as usize],
            // 3 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_1 as usize], // A12
            // 4 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_3 as usize], // A10
            5 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_5 as usize],
            // 6 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_6 as usize], // A7
            7 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P06_5 as usize],
            8 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P06_4 as usize],
            // Left inner connector, top to bottom
            // 9 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P06_1 as usize], // A14
            // 10 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_0 as usize], // A13
            // 11 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_2 as usize], // A11
            // 12 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_4 as usize], // A9
            // 13 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_5 as usize], // A8
            // 14 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P04_7 as usize], // A6
            // 15 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_4 as usize], // A1
            // 16 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_5 as usize], // A0
            // Right inner connector, top to bottom
            17 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_7 as usize],
            18 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_6 as usize],
            19 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_4 as usize],
            20 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_6 as usize],
            21 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P06_6 as usize],
            22 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P06_7 as usize],
            23 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_3 as usize],
            // 24 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_1 as usize], // A4
            // 25 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_5 as usize], // debug-gpio
            // 26 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_7 as usize], // debug-gpio
            // Right outer connector, top to bottom
            27 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P02_5 as usize],
            28 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_0 as usize],
            29 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_7 as usize],
            30 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_6 as usize],
            31 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P01_7 as usize],
            // 32 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_0 as usize], // A5
            // 33 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P05_2 as usize], // A3
            34 => &msp432::gpio::INT_PINS[msp432::gpio::IntPinNr::P03_6 as usize]
        ),
    )
    .finalize(components::gpio_component_buf!(
        msp432::gpio::IntPin<'static>
    ));

    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Setup UART0
    let uart_mux = components::console::UartMuxComponent::new(
        &msp432::uart::UART0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // Setup alarm
    let timer0 = &msp432::timer::TIMER_A0;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(timer0).finalize(
        components::alarm_mux_component_helper!(msp432::timer::TimerA),
    );
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(msp432::timer::TimerA));

    // Setup ADC

    setup_adc_pins();

    let adc_channels = static_init!(
        [&'static msp432::adc::Channel; 24],
        [
            &msp432::adc::Channel::Channel0,  // A0
            &msp432::adc::Channel::Channel1,  // A1
            &msp432::adc::Channel::Channel2,  // A2
            &msp432::adc::Channel::Channel3,  // A3
            &msp432::adc::Channel::Channel4,  // A4
            &msp432::adc::Channel::Channel5,  // A5
            &msp432::adc::Channel::Channel6,  // A6
            &msp432::adc::Channel::Channel7,  // A7
            &msp432::adc::Channel::Channel8,  // A8
            &msp432::adc::Channel::Channel9,  // A9
            &msp432::adc::Channel::Channel10, // A10
            &msp432::adc::Channel::Channel11, // A11
            &msp432::adc::Channel::Channel12, // A12
            &msp432::adc::Channel::Channel13, // A13
            &msp432::adc::Channel::Channel14, // A14
            &msp432::adc::Channel::Channel15, // A15
            &msp432::adc::Channel::Channel16, // A16
            &msp432::adc::Channel::Channel17, // A17
            &msp432::adc::Channel::Channel18, // A18
            &msp432::adc::Channel::Channel19, // A19
            &msp432::adc::Channel::Channel20, // A20
            &msp432::adc::Channel::Channel21, // A21
            &msp432::adc::Channel::Channel22, // A22
            &msp432::adc::Channel::Channel23, // A23
        ]
    );

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let grant_adc = board_kernel.create_grant(&grant_cap);
    let adc = static_init!(
        capsules::adc::AdcDedicated<'static, msp432::adc::Adc>,
        capsules::adc::AdcDedicated::new(
            &msp432::adc::ADC,
            grant_adc,
            adc_channels,
            &mut capsules::adc::ADC_BUFFER1,
            &mut capsules::adc::ADC_BUFFER2,
            &mut capsules::adc::ADC_BUFFER3
        )
    );

    msp432::adc::ADC.set_client(adc);

    // Set the reference voltage for the ADC to 2.5V
    msp432::ref_module::REF.select_ref_voltage(msp432::ref_module::ReferenceVoltage::Volt2_5);
    // Enable the internal temperature sensor on ADC Channel 22
    msp432::ref_module::REF.enable_temp_sensor(true);

    let msp_exp432p4014 = MspExp432P401R {
        led: leds,
        console: console,
        button: button,
        gpio: gpio,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        adc: adc,
    };

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
        &process_management_capability,
    )
    .unwrap();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    //Uncomment to run multi alarm test
    //multi_alarm_test::run_multi_alarm(mux_alarm);

    board_kernel.kernel_loop(
        &msp_exp432p4014,
        chip,
        Some(&msp_exp432p4014.ipc),
        scheduler,
        &main_loop_capability,
    );
}
