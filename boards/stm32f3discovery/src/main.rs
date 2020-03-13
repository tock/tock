//! Board file for Nucleo-F446RE development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f446re.html>

#![no_std]
#![no_main]
#![feature(asm, core_intrinsics)]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

// Unit Tests for drivers.
#[allow(dead_code)]
mod virtual_uart_rx_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static stm32f3xx::chip::Stm32f3xx> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

// Force the emission of the `.apps` segment in the kernel elf image
// NOTE: This will cause the kernel to overwrite any existing apps when flashed!
#[used]
#[link_section = ".app.hack"]
static APP_HACK: u8 = 0;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct STM32F3Discovery {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    button: &'static capsules::button::Button<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f3xx::tim2::Tim2<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for STM32F3Discovery {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    // use stm32f3xx::exti::{LineId, EXTI};
    use stm32f3xx::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f3xx::syscfg::SYSCFG;

    SYSCFG.enable_clock();

    PORT[PortId::A as usize].enable_clock();
    PORT[PortId::B as usize].enable_clock();
    PORT[PortId::C as usize].enable_clock();
    PORT[PortId::D as usize].enable_clock();
    PORT[PortId::E as usize].enable_clock();
    PORT[PortId::F as usize].enable_clock();

    PinId::PE14.get_pin().as_ref().map(|pin| {
        pin.make_output();
        pin.set();
    });

    // User LD3 is connected to PE09. Configure PE09 as `debug_gpio!(0, ...)`
    PinId::PE09.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    // // pc4 and pc5 (USART1) is connected to ST-LINK virtual COM port
    PinId::PC04.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    PinId::PA05.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use stm32f3xx::tim2::TIM2;

    // USART2 IRQn is 38
    cortexm4::nvic::Nvic::new(stm32f3xx::nvic::USART1).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f3xx::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f3xx::init();

    // We use the default HSI 8Mhz clock

    set_pin_primary_functions();

    setup_peripherals();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let chip = static_init!(
        stm32f3xx::chip::Stm32f3xx,
        stm32f3xx::chip::Stm32f3xx::new()
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f3xx::usart::USART1.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &stm32f3xx::usart::USART1,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // `finalize()` configures the underlying USART, so we need to
    // tell `send_byte()` not to configure the USART again.
    io::WRITER.set_initialized();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // // Setup the process inspection console
    // let process_console_uart = static_init!(UartDevice, UartDevice::new(mux_uart, true));
    // process_console_uart.setup();
    // pub struct ProcessConsoleCapability;
    // unsafe impl capabilities::ProcessManagementCapability for ProcessConsoleCapability {}
    // let process_console = static_init!(
    //     capsules::process_console::ProcessConsole<'static, ProcessConsoleCapability>,
    //     capsules::process_console::ProcessConsole::new(
    //         process_console_uart,
    //         &mut capsules::process_console::WRITE_BUF,
    //         &mut capsules::process_console::READ_BUF,
    //         &mut capsules::process_console::COMMAND_BUF,
    //         board_kernel,
    //         ProcessConsoleCapability,
    //     )
    // );
    // hil::uart::Transmit::set_transmit_client(process_console_uart, process_console);
    // hil::uart::Receive::set_receive_client(process_console_uart, process_console);
    // process_console.start();

    // LEDs

    // Clock to Port E is enabled in `set_pin_primary_functions()`

    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
        (
            stm32f3xx::gpio::PinId::PE09.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE08.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE10.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE15.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE14.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE12.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!((
            stm32f3xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow,
            kernel::hil::gpio::FloatingState::PullNone
        )),
    );

    // ALARM

    let tim2 = &stm32f3xx::tim2::TIM2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f3xx::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(stm32f3xx::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(board_kernel).finalize(components::gpio_component_helper!(
        // Left outer connector
        stm32f3xx::gpio::PinId::PC01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC06.get_pin().as_ref().unwrap(),
        // Left inner connector
        stm32f3xx::gpio::PinId::PC00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC07.get_pin().as_ref().unwrap(),
        // Right inner connector
        stm32f3xx::gpio::PinId::PF09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC08.get_pin().as_ref().unwrap(),
        // Right outer connector
        stm32f3xx::gpio::PinId::PF10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC09.get_pin().as_ref().unwrap()
    ));

    let stm32f3discovery = STM32F3Discovery {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        gpio: gpio,
        led: led,
        button: button,
        alarm: alarm,
    };

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

    // hprintln!("Initialization complete. Entering main loop").unwrap ();
    debug!("Initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    board_kernel.kernel_loop(
        &stm32f3discovery,
        chip,
        Some(&stm32f3discovery.ipc),
        &main_loop_capability,
    );
}
