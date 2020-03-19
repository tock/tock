//! Board file for Nucleo-F429ZI development board.
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f429zi.html>

#![no_std]
#![no_main]
#![feature(asm)]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 20;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static stm32f4xx::chip::Stm32f4xx> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 85536] = [0; 85536];

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
struct NucleoF429ZI {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static>,
    button: &'static capsules::button::Button<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f4xx::tim2::Tim2<'static>>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for NucleoF429ZI {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
unsafe fn setup_dma() {
    use stm32f4xx::dma1::{Dma1Peripheral, DMA1};
    use stm32f4xx::usart;
    use stm32f4xx::usart::USART3;

    DMA1.enable_clock();

    let usart3_tx_stream = Dma1Peripheral::USART3_TX.get_stream();
    let usart3_rx_stream = Dma1Peripheral::USART3_RX.get_stream();

    USART3.set_dma(
        usart::TxDMA(usart3_tx_stream),
        usart::RxDMA(usart3_rx_stream),
    );

    usart3_tx_stream.set_client(&USART3);
    usart3_rx_stream.set_client(&USART3);

    usart3_tx_stream.setup(Dma1Peripheral::USART3_TX);
    usart3_rx_stream.setup(Dma1Peripheral::USART3_RX);

    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART3_TX.get_stream_irqn()).enable();
    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART3_RX.get_stream_irqn()).enable();
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use kernel::hil::gpio::Configure;
    use stm32f4xx::exti::{LineId, EXTI};
    use stm32f4xx::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f4xx::syscfg::SYSCFG;

    SYSCFG.enable_clock();

    PORT[PortId::B as usize].enable_clock();

    // User LD2 is connected to PB07. Configure PB07 as `debug_gpio!(0, ...)`
    PinId::PB07.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    PORT[PortId::D as usize].enable_clock();

    // pd8 and pd9 (USART3) is connected to ST-LINK virtual COM port
    PinId::PD08.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    PinId::PD09.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    PORT[PortId::C as usize].enable_clock();

    // button is connected on pc13
    PinId::PC13.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti13, pin);
    });
    // EXTI13 interrupts is delivered at IRQn 40 (EXTI15_10)
    cortexm4::nvic::Nvic::new(stm32f4xx::nvic::EXTI15_10).enable();

    // Enable clocks for GPIO Ports
    // Disable some of them if you don't need some of the GPIOs
    PORT[PortId::A as usize].enable_clock();
    // Ports B, C and D are already enabled
    PORT[PortId::E as usize].enable_clock();
    PORT[PortId::F as usize].enable_clock();
    PORT[PortId::G as usize].enable_clock();
    PORT[PortId::H as usize].enable_clock();
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use stm32f4xx::tim2::TIM2;

    // USART3 IRQn is 39
    cortexm4::nvic::Nvic::new(stm32f4xx::nvic::USART3).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f4xx::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f4xx::init();

    // We use the default HSI 16Mhz clock

    set_pin_primary_functions();

    setup_dma();

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
        stm32f4xx::chip::Stm32f4xx,
        stm32f4xx::chip::Stm32f4xx::new()
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f4xx::usart::USART3.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &stm32f4xx::usart::USART3,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

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

    // Clock to Port A is enabled in `set_pin_primary_functions()`

    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
        (
            stm32f4xx::gpio::PinId::PB00.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f4xx::gpio::PinId::PB07.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f4xx::gpio::PinId::PB14.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!((
            stm32f4xx::gpio::PinId::PC13.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow,
            kernel::hil::gpio::FloatingState::PullNone
        )),
    );

    // ALARM

    let tim2 = &stm32f4xx::tim2::TIM2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f4xx::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(stm32f4xx::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(board_kernel).finalize(components::gpio_component_helper!(
        // Arduino like RX/TX
        stm32f4xx::gpio::PIN[6][9].as_ref().unwrap(), //D0
        stm32f4xx::gpio::PIN[6][14].as_ref().unwrap(), //D1
        stm32f4xx::gpio::PIN[5][15].as_ref().unwrap(), //D2
        stm32f4xx::gpio::PIN[4][13].as_ref().unwrap(), //D3
        stm32f4xx::gpio::PIN[5][14].as_ref().unwrap(), //D4
        stm32f4xx::gpio::PIN[4][11].as_ref().unwrap(), //D5
        stm32f4xx::gpio::PIN[4][9].as_ref().unwrap(), //D6
        stm32f4xx::gpio::PIN[5][13].as_ref().unwrap(), //D7
        stm32f4xx::gpio::PIN[5][12].as_ref().unwrap(), //D8
        stm32f4xx::gpio::PIN[3][15].as_ref().unwrap(), //D9
        // SPI Pins
        stm32f4xx::gpio::PIN[3][14].as_ref().unwrap(), //D10
        stm32f4xx::gpio::PIN[0][7].as_ref().unwrap(),  //D11
        stm32f4xx::gpio::PIN[0][6].as_ref().unwrap(),  //D12
        stm32f4xx::gpio::PIN[0][5].as_ref().unwrap(),  //D13
        // I2C Pins
        stm32f4xx::gpio::PIN[1][9].as_ref().unwrap(), //D14
        stm32f4xx::gpio::PIN[1][8].as_ref().unwrap(), //D15
        stm32f4xx::gpio::PIN[2][6].as_ref().unwrap(), //D16
        stm32f4xx::gpio::PIN[1][15].as_ref().unwrap(), //D17
        stm32f4xx::gpio::PIN[1][13].as_ref().unwrap(), //D18
        stm32f4xx::gpio::PIN[1][12].as_ref().unwrap(), //D19
        stm32f4xx::gpio::PIN[0][15].as_ref().unwrap(), //D20
        stm32f4xx::gpio::PIN[2][7].as_ref().unwrap(), //D21
        // SPI B Pins
        stm32f4xx::gpio::PIN[1][5].as_ref().unwrap(), //D22
        stm32f4xx::gpio::PIN[1][3].as_ref().unwrap(), //D23
        stm32f4xx::gpio::PIN[0][4].as_ref().unwrap(), //D24
        stm32f4xx::gpio::PIN[1][4].as_ref().unwrap(), //D25
        // QSPI
        stm32f4xx::gpio::PIN[1][6].as_ref().unwrap(), //D26
        stm32f4xx::gpio::PIN[1][2].as_ref().unwrap(), //D27
        stm32f4xx::gpio::PIN[3][13].as_ref().unwrap(), //D28
        stm32f4xx::gpio::PIN[3][12].as_ref().unwrap(), //D29
        stm32f4xx::gpio::PIN[3][11].as_ref().unwrap(), //D30
        stm32f4xx::gpio::PIN[4][2].as_ref().unwrap(), //D31
        // Timer Pins
        stm32f4xx::gpio::PIN[0][0].as_ref().unwrap(), //D32
        stm32f4xx::gpio::PIN[1][0].as_ref().unwrap(), //D33
        stm32f4xx::gpio::PIN[4][0].as_ref().unwrap(), //D34
        stm32f4xx::gpio::PIN[1][11].as_ref().unwrap(), //D35
        stm32f4xx::gpio::PIN[1][10].as_ref().unwrap(), //D36
        stm32f4xx::gpio::PIN[4][15].as_ref().unwrap(), //D37
        stm32f4xx::gpio::PIN[4][14].as_ref().unwrap(), //D38
        stm32f4xx::gpio::PIN[4][12].as_ref().unwrap(), //D39
        stm32f4xx::gpio::PIN[4][10].as_ref().unwrap(), //D40
        stm32f4xx::gpio::PIN[4][7].as_ref().unwrap(), //D41
        stm32f4xx::gpio::PIN[4][8].as_ref().unwrap(), //D42
        // SDMMC
        stm32f4xx::gpio::PIN[2][8].as_ref().unwrap(), //D43
        stm32f4xx::gpio::PIN[2][9].as_ref().unwrap(), //D44
        stm32f4xx::gpio::PIN[2][10].as_ref().unwrap(), //D45
        stm32f4xx::gpio::PIN[2][11].as_ref().unwrap(), //D46
        stm32f4xx::gpio::PIN[2][12].as_ref().unwrap(), //D47
        stm32f4xx::gpio::PIN[3][2].as_ref().unwrap(), //D48
        stm32f4xx::gpio::PIN[6][2].as_ref().unwrap(), //D49
        stm32f4xx::gpio::PIN[6][3].as_ref().unwrap(), //D50
        // USART
        stm32f4xx::gpio::PIN[3][7].as_ref().unwrap(), //D51
        stm32f4xx::gpio::PIN[3][6].as_ref().unwrap(), //D52
        stm32f4xx::gpio::PIN[3][5].as_ref().unwrap(), //D53
        stm32f4xx::gpio::PIN[3][4].as_ref().unwrap(), //D54
        stm32f4xx::gpio::PIN[3][3].as_ref().unwrap(), //D55
        stm32f4xx::gpio::PIN[4][2].as_ref().unwrap(), //D56
        stm32f4xx::gpio::PIN[4][4].as_ref().unwrap(), //D57
        stm32f4xx::gpio::PIN[4][5].as_ref().unwrap(), //D58
        stm32f4xx::gpio::PIN[4][6].as_ref().unwrap(), //D59
        stm32f4xx::gpio::PIN[4][3].as_ref().unwrap(), //D60
        stm32f4xx::gpio::PIN[5][8].as_ref().unwrap(), //D61
        stm32f4xx::gpio::PIN[5][7].as_ref().unwrap(), //D62
        stm32f4xx::gpio::PIN[5][9].as_ref().unwrap(), //D63
        stm32f4xx::gpio::PIN[6][1].as_ref().unwrap(), //D64
        stm32f4xx::gpio::PIN[6][0].as_ref().unwrap(), //D65
        stm32f4xx::gpio::PIN[3][1].as_ref().unwrap(), //D66
        stm32f4xx::gpio::PIN[3][0].as_ref().unwrap(), //D67
        stm32f4xx::gpio::PIN[5][0].as_ref().unwrap(), //D68
        stm32f4xx::gpio::PIN[5][1].as_ref().unwrap(), //D69
        stm32f4xx::gpio::PIN[5][2].as_ref().unwrap(), //D70
        stm32f4xx::gpio::PIN[0][7].as_ref().unwrap()  //D71

                                                      // ADC Pins
                                                      // Enable the to use the ADC pins as GPIO
                                                      // stm32f4xx::gpio::PIN[0][3].as_ref().unwrap(), //A0
                                                      // stm32f4xx::gpio::PIN[2][0].as_ref().unwrap(), //A1
                                                      // stm32f4xx::gpio::PIN[2][3].as_ref().unwrap(), //A2
                                                      // stm32f4xx::gpio::PIN[5][3].as_ref().unwrap(), //A3
                                                      // stm32f4xx::gpio::PIN[5][5].as_ref().unwrap(), //A4
                                                      // stm32f4xx::gpio::PIN[5][10].as_ref().unwrap(), //A5
                                                      // stm32f4xx::gpio::PIN[1][1].as_ref().unwrap(), //A6
                                                      // stm32f4xx::gpio::PIN[2][2].as_ref().unwrap(), //A7
                                                      // stm32f4xx::gpio::PIN[5][4].as_ref().unwrap()  //A8
    ));

    let nucleo_f429zi = NucleoF429ZI {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        led: led,
        button: button,
        alarm: alarm,
        gpio: gpio,
    };

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

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
        &nucleo_f429zi,
        chip,
        Some(&nucleo_f429zi.ipc),
        &main_loop_capability,
    );
}
