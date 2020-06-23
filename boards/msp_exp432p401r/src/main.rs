#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
// #![deny(missing_docs)]

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::common::dynamic_deferred_call::DynamicDeferredCallClientState;
use kernel::component::Component;
use kernel::hil::watchdog::Watchdog;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

pub mod io;

/// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

/// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static msp432::chip::Msp432> = None;

/// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct MspExp432P401R {
    led: &'static capsules::led::LED<'static, msp432::gpio::Pin>,
    console: &'static capsules::console::Console<'static>,
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
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    msp432::init();
    msp432::wdt::WATCHDOG.stop();
    msp432::sysctl::SYSCTL.enable_all_sram_banks();
    msp432::pcm::PCM.set_high_power();
    msp432::flctl::FLCTL.set_waitstates(msp432::flctl::WaitStates::_1);
    msp432::flctl::FLCTL.set_buffering(true);

    // Setup the master-clock (MCLK) to 48MHz from external oscillator
    msp432::gpio::PINS[msp432::gpio::PinNr::PJ_2 as usize].enable_primary_function();
    msp432::gpio::PINS[msp432::gpio::PinNr::PJ_3 as usize].enable_primary_function();
    msp432::cs::CS.set_mclk_48mhz();
    // Setup the Low-speed subsystem master clock (SMCLK) to 12MHz
    msp432::cs::CS.set_smclk_12mhz();

    debug::assign_gpios(
        Some(&msp432::gpio::PINS[msp432::gpio::PinNr::P01_0 as usize]), // Red LED
        Some(&msp432::gpio::PINS[msp432::gpio::PinNr::P03_5 as usize]),
        Some(&msp432::gpio::PINS[msp432::gpio::PinNr::P03_7 as usize]),
    );

    // Setup pins for UART0
    msp432::gpio::PINS[msp432::gpio::PinNr::P01_2 as usize].enable_primary_function();
    msp432::gpio::PINS[msp432::gpio::PinNr::P01_3 as usize].enable_primary_function();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let chip = static_init!(msp432::chip::Msp432, msp432::chip::Msp432::new());
    CHIP = Some(chip);

    let leds = components::led::LedsComponent::new(components::led_component_helper!(
        msp432::gpio::Pin,
        (
            &msp432::gpio::PINS[msp432::gpio::PinNr::P02_0 as usize],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &msp432::gpio::PINS[msp432::gpio::PinNr::P02_1 as usize],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &msp432::gpio::PINS[msp432::gpio::PinNr::P02_2 as usize],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ))
    .finalize(components::led_component_buf!(msp432::gpio::Pin));

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

    let msp_exp432p4014 = MspExp432P401R {
        led: leds,
        console: console,
    };

    debug!("Initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap();

    board_kernel.kernel_loop(&msp_exp432p4014, chip, None, &main_loop_capability);
}
