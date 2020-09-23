//! Board file for Nucleo-F429ZI development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f429zi.html>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
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

// Unit tests
#[allow(dead_code)]
mod multi_alarm_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

static mut CHIP: Option<&'static stm32f429zi::chip::Stm32f4xx> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NucleoF429ZI {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static, stm32f429zi::gpio::Pin<'static>>,
    button: &'static capsules::button::Button<'static, stm32f429zi::gpio::Pin<'static>>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f429zi::tim2::Tim2<'static>>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static, stm32f429zi::gpio::Pin<'static>>,
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
    use stm32f429zi::dma1::{Dma1Peripheral, DMA1};
    use stm32f429zi::usart;
    use stm32f429zi::usart::USART3;

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
    use stm32f429zi::exti::{LineId, EXTI};
    use stm32f429zi::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f429zi::syscfg::SYSCFG;

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
    cortexm4::nvic::Nvic::new(stm32f429zi::nvic::EXTI15_10).enable();

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
    use stm32f429zi::tim2::TIM2;

    // USART3 IRQn is 39
    cortexm4::nvic::Nvic::new(stm32f429zi::nvic::USART3).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f429zi::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f429zi::init();

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
        stm32f429zi::chip::Stm32f4xx,
        stm32f429zi::chip::Stm32f4xx::new()
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f429zi::usart::USART3.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &stm32f429zi::usart::USART3,
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

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        stm32f429zi::gpio::Pin,
        (
            stm32f429zi::gpio::PinId::PB00.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f429zi::gpio::PinId::PB07.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f429zi::gpio::PinId::PB14.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ))
    .finalize(components::led_component_buf!(stm32f429zi::gpio::Pin));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            stm32f429zi::gpio::Pin,
            (
                stm32f429zi::gpio::PinId::PC13.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_buf!(stm32f429zi::gpio::Pin));

    // ALARM

    let tim2 = &stm32f429zi::tim2::TIM2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f429zi::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(stm32f429zi::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            stm32f429zi::gpio::Pin,
            // Arduino like RX/TX
            0 => stm32f429zi::gpio::PIN[6][9].as_ref().unwrap(), //D0
            1 => stm32f429zi::gpio::PIN[6][14].as_ref().unwrap(), //D1
            2 => stm32f429zi::gpio::PIN[5][15].as_ref().unwrap(), //D2
            3 => stm32f429zi::gpio::PIN[4][13].as_ref().unwrap(), //D3
            4 => stm32f429zi::gpio::PIN[5][14].as_ref().unwrap(), //D4
            5 => stm32f429zi::gpio::PIN[4][11].as_ref().unwrap(), //D5
            6 => stm32f429zi::gpio::PIN[4][9].as_ref().unwrap(), //D6
            7 => stm32f429zi::gpio::PIN[5][13].as_ref().unwrap(), //D7
            8 => stm32f429zi::gpio::PIN[5][12].as_ref().unwrap(), //D8
            9 => stm32f429zi::gpio::PIN[3][15].as_ref().unwrap(), //D9
            // SPI Pins
            10 => stm32f429zi::gpio::PIN[3][14].as_ref().unwrap(), //D10
            11 => stm32f429zi::gpio::PIN[0][7].as_ref().unwrap(),  //D11
            12 => stm32f429zi::gpio::PIN[0][6].as_ref().unwrap(),  //D12
            13 => stm32f429zi::gpio::PIN[0][5].as_ref().unwrap(),  //D13
            // I2C Pins
            14 => stm32f429zi::gpio::PIN[1][9].as_ref().unwrap(), //D14
            15 => stm32f429zi::gpio::PIN[1][8].as_ref().unwrap(), //D15
            16 => stm32f429zi::gpio::PIN[2][6].as_ref().unwrap(), //D16
            17 => stm32f429zi::gpio::PIN[1][15].as_ref().unwrap(), //D17
            18 => stm32f429zi::gpio::PIN[1][13].as_ref().unwrap(), //D18
            19 => stm32f429zi::gpio::PIN[1][12].as_ref().unwrap(), //D19
            20 => stm32f429zi::gpio::PIN[0][15].as_ref().unwrap(), //D20
            21 => stm32f429zi::gpio::PIN[2][7].as_ref().unwrap(), //D21
            // SPI B Pins
            // 22 => stm32f429zi::gpio::PIN[1][5].as_ref().unwrap(), //D22
            // 23 => stm32f429zi::gpio::PIN[1][3].as_ref().unwrap(), //D23
            // 24 => stm32f429zi::gpio::PIN[0][4].as_ref().unwrap(), //D24
            // 24 => stm32f429zi::gpio::PIN[1][4].as_ref().unwrap(), //D25
            // QSPI
            26 => stm32f429zi::gpio::PIN[1][6].as_ref().unwrap(), //D26
            27 => stm32f429zi::gpio::PIN[1][2].as_ref().unwrap(), //D27
            28 => stm32f429zi::gpio::PIN[3][13].as_ref().unwrap(), //D28
            29 => stm32f429zi::gpio::PIN[3][12].as_ref().unwrap(), //D29
            30 => stm32f429zi::gpio::PIN[3][11].as_ref().unwrap(), //D30
            31 => stm32f429zi::gpio::PIN[4][2].as_ref().unwrap(), //D31
            // Timer Pins
            32 => stm32f429zi::gpio::PIN[0][0].as_ref().unwrap(), //D32
            33 => stm32f429zi::gpio::PIN[1][0].as_ref().unwrap(), //D33
            34 => stm32f429zi::gpio::PIN[4][0].as_ref().unwrap(), //D34
            35 => stm32f429zi::gpio::PIN[1][11].as_ref().unwrap(), //D35
            36 => stm32f429zi::gpio::PIN[1][10].as_ref().unwrap(), //D36
            37 => stm32f429zi::gpio::PIN[4][15].as_ref().unwrap(), //D37
            38 => stm32f429zi::gpio::PIN[4][14].as_ref().unwrap(), //D38
            39 => stm32f429zi::gpio::PIN[4][12].as_ref().unwrap(), //D39
            40 => stm32f429zi::gpio::PIN[4][10].as_ref().unwrap(), //D40
            41 => stm32f429zi::gpio::PIN[4][7].as_ref().unwrap(), //D41
            42 => stm32f429zi::gpio::PIN[4][8].as_ref().unwrap(), //D42
            // SDMMC
            43 => stm32f429zi::gpio::PIN[2][8].as_ref().unwrap(), //D43
            44 => stm32f429zi::gpio::PIN[2][9].as_ref().unwrap(), //D44
            45 => stm32f429zi::gpio::PIN[2][10].as_ref().unwrap(), //D45
            46 => stm32f429zi::gpio::PIN[2][11].as_ref().unwrap(), //D46
            47 => stm32f429zi::gpio::PIN[2][12].as_ref().unwrap(), //D47
            48 => stm32f429zi::gpio::PIN[3][2].as_ref().unwrap(), //D48
            49 => stm32f429zi::gpio::PIN[6][2].as_ref().unwrap(), //D49
            50 => stm32f429zi::gpio::PIN[6][3].as_ref().unwrap(), //D50
            // USART
            51 => stm32f429zi::gpio::PIN[3][7].as_ref().unwrap(), //D51
            52 => stm32f429zi::gpio::PIN[3][6].as_ref().unwrap(), //D52
            53 => stm32f429zi::gpio::PIN[3][5].as_ref().unwrap(), //D53
            54 => stm32f429zi::gpio::PIN[3][4].as_ref().unwrap(), //D54
            55 => stm32f429zi::gpio::PIN[3][3].as_ref().unwrap(), //D55
            56 => stm32f429zi::gpio::PIN[4][2].as_ref().unwrap(), //D56
            57 => stm32f429zi::gpio::PIN[4][4].as_ref().unwrap(), //D57
            58 => stm32f429zi::gpio::PIN[4][5].as_ref().unwrap(), //D58
            59 => stm32f429zi::gpio::PIN[4][6].as_ref().unwrap(), //D59
            60 => stm32f429zi::gpio::PIN[4][3].as_ref().unwrap(), //D60
            61 => stm32f429zi::gpio::PIN[5][8].as_ref().unwrap(), //D61
            62 => stm32f429zi::gpio::PIN[5][7].as_ref().unwrap(), //D62
            63 => stm32f429zi::gpio::PIN[5][9].as_ref().unwrap(), //D63
            64 => stm32f429zi::gpio::PIN[6][1].as_ref().unwrap(), //D64
            65 => stm32f429zi::gpio::PIN[6][0].as_ref().unwrap(), //D65
            66 => stm32f429zi::gpio::PIN[3][1].as_ref().unwrap(), //D66
            67 => stm32f429zi::gpio::PIN[3][0].as_ref().unwrap(), //D67
            68 => stm32f429zi::gpio::PIN[5][0].as_ref().unwrap(), //D68
            69 => stm32f429zi::gpio::PIN[5][1].as_ref().unwrap(), //D69
            70 => stm32f429zi::gpio::PIN[5][2].as_ref().unwrap(), //D70
            71 => stm32f429zi::gpio::PIN[0][7].as_ref().unwrap()  //D71

            // ADC Pins
            // Enable the to use the ADC pins as GPIO
            // 72 => stm32f429zi::gpio::PIN[0][3].as_ref().unwrap(), //A0
            // 73 => stm32f429zi::gpio::PIN[2][0].as_ref().unwrap(), //A1
            // 74 => stm32f429zi::gpio::PIN[2][3].as_ref().unwrap(), //A2
            // 75 => stm32f429zi::gpio::PIN[5][3].as_ref().unwrap(), //A3
            // 76 => stm32f429zi::gpio::PIN[5][5].as_ref().unwrap(), //A4
            // 77 => stm32f429zi::gpio::PIN[5][10].as_ref().unwrap(), //A5
            // 78 => stm32f429zi::gpio::PIN[1][1].as_ref().unwrap(), //A6
            // 79 => stm32f429zi::gpio::PIN[2][2].as_ref().unwrap(), //A7
            // 80 => stm32f429zi::gpio::PIN[5][4].as_ref().unwrap()  //A8
        ),
    )
    .finalize(components::gpio_component_buf!(stm32f429zi::gpio::Pin));

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
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    //Uncomment to run multi alarm test
    //multi_alarm_test::run_multi_alarm(mux_alarm);

    board_kernel.kernel_loop(
        &nucleo_f429zi,
        chip,
        Some(&nucleo_f429zi.ipc),
        scheduler,
        &main_loop_capability,
    );
}
