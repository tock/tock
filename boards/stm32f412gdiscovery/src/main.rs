//! Board file for STM32F412GDiscovery Discovery kit development board
//!
//! - <https://www.st.com/en/evaluation-tools/32f412gdiscovery.html>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
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
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

static mut CHIP: Option<&'static stm32f412g::chip::Stm32f4xx> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct STM32F412GDiscovery {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static, stm32f412g::gpio::Pin<'static>>,
    button: &'static capsules::button::Button<'static, stm32f412g::gpio::Pin<'static>>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f412g::tim2::Tim2<'static>>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static, stm32f412g::gpio::Pin<'static>>,
    ft6206: &'static capsules::ft6206::Ft6206<'static>,
    adc: &'static capsules::adc::Adc<'static, stm32f412g::adc::Adc>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for STM32F412GDiscovery {
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
            capsules::ft6206::DRIVER_NUM => f(Some(self.ft6206)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
unsafe fn setup_dma() {
    use stm32f412g::dma1::{Dma1Peripheral, DMA1};
    use stm32f412g::usart;
    use stm32f412g::usart::USART2;

    DMA1.enable_clock();

    let usart2_tx_stream = Dma1Peripheral::USART2_TX.get_stream();
    let usart2_rx_stream = Dma1Peripheral::USART2_RX.get_stream();

    USART2.set_dma(
        usart::TxDMA(usart2_tx_stream),
        usart::RxDMA(usart2_rx_stream),
    );

    usart2_tx_stream.set_client(&USART2);
    usart2_rx_stream.set_client(&USART2);

    usart2_tx_stream.setup(Dma1Peripheral::USART2_TX);
    usart2_rx_stream.setup(Dma1Peripheral::USART2_RX);

    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART2_TX.get_stream_irqn()).enable();
    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART2_RX.get_stream_irqn()).enable();
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use kernel::hil::gpio::Configure;
    use stm32f412g::exti::{LineId, EXTI};
    use stm32f412g::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f412g::syscfg::SYSCFG;

    SYSCFG.enable_clock();

    PORT[PortId::E as usize].enable_clock();

    // User LD3 is connected to PE02. Configure PE02 as `debug_gpio!(0, ...)`
    PinId::PE02.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    PORT[PortId::A as usize].enable_clock();

    // pa2 and pa3 (USART2) is connected to ST-LINK virtual COM port
    PinId::PA02.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    PinId::PA03.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    // uncomment this if you do not plan to use the joystick up, as they both use Exti0
    // joystick selection is connected on pa00
    // PinId::PA00.get_pin().as_ref().map(|pin| {
    //     // By default, upon reset, the pin is in input mode, with no internal
    //     // pull-up, no internal pull-down (i.e., floating).
    //     //
    //     // Only set the mapping between EXTI line and the Pin and let capsule do
    //     // the rest.
    //     EXTI.associate_line_gpiopin(LineId::Exti0, pin);
    // });
    // // EXTI0 interrupts is delivered at IRQn 6 (EXTI0)
    // cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI0).enable();

    // joystick down is connected on pg01
    PinId::PG01.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti1, pin);
    });
    // EXTI1 interrupts is delivered at IRQn 7 (EXTI1)
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI1).enable();

    // joystick left is connected on pf15
    PinId::PF15.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti15, pin);
    });
    // EXTI15_10 interrupts is delivered at IRQn 40 (EXTI15_10)
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI15_10).enable();

    // joystick right is connected on pf14
    PinId::PF14.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti14, pin);
    });
    // EXTI15_10 interrupts is delivered at IRQn 40 (EXTI15_10)
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI15_10).enable();

    // joystick up is connected on pg00
    PinId::PG00.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti0, pin);
    });
    // EXTI0 interrupts is delivered at IRQn 6 (EXTI0)
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI0).enable();

    // Enable clocks for GPIO Ports
    // Disable some of them if you don't need some of the GPIOs
    PORT[PortId::B as usize].enable_clock();
    // Ports A and E are already enabled
    PORT[PortId::C as usize].enable_clock();
    PORT[PortId::D as usize].enable_clock();
    PORT[PortId::F as usize].enable_clock();
    PORT[PortId::G as usize].enable_clock();
    PORT[PortId::H as usize].enable_clock();

    // I2C1 has the TouchPanel connected
    PinId::PB06.get_pin().as_ref().map(|pin| {
        // pin.make_output();
        pin.set_mode_output_opendrain();
        pin.set_mode(Mode::AlternateFunctionMode);
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });
    PinId::PB07.get_pin().as_ref().map(|pin| {
        // pin.make_output();
        pin.set_mode_output_opendrain();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });

    stm32f412g::i2c::I2C1.enable_clock();
    stm32f412g::i2c::I2C1.set_speed(stm32f412g::i2c::I2CSpeed::Speed100k, 16);

    // FT6206 interrupt
    PinId::PG05.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti5, pin);
    });

    // ADC

    // Arduino A0
    PinId::PA01.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A1
    PinId::PC01.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A2
    PinId::PC03.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A3
    PinId::PC04.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A4
    PinId::PC05.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A5
    PinId::PB00.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // EXTI9_5 interrupts is delivered at IRQn 23 (EXTI9_5)
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI9_5).enable();
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use stm32f412g::tim2::TIM2;

    // USART2 IRQn is 38
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::USART2).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f412g::init();

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
        stm32f412g::chip::Stm32f4xx,
        stm32f412g::chip::Stm32f4xx::new()
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f412g::usart::USART2.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &stm32f412g::usart::USART2,
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
        stm32f412g::gpio::Pin,
        (
            stm32f412g::gpio::PinId::PE00.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            stm32f412g::gpio::PinId::PE01.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            stm32f412g::gpio::PinId::PE02.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            stm32f412g::gpio::PinId::PE03.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    ))
    .finalize(components::led_component_buf!(stm32f412g::gpio::Pin));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            stm32f412g::gpio::Pin,
            // Select
            (
                stm32f412g::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Down
            (
                stm32f412g::gpio::PinId::PG01.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Left
            (
                stm32f412g::gpio::PinId::PF15.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Right
            (
                stm32f412g::gpio::PinId::PF14.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Up
            (
                stm32f412g::gpio::PinId::PG00.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_buf!(stm32f412g::gpio::Pin));

    // ALARM

    let tim2 = &stm32f412g::tim2::TIM2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f412g::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(stm32f412g::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            stm32f412g::gpio::Pin,
            // Arduino like RX/TX
            0 => stm32f412g::gpio::PinId::PG09.get_pin().as_ref().unwrap(), //D0
            1 => stm32f412g::gpio::PinId::PG14.get_pin().as_ref().unwrap(), //D1
            2 => stm32f412g::gpio::PinId::PG13.get_pin().as_ref().unwrap(), //D2
            3 => stm32f412g::gpio::PinId::PF04.get_pin().as_ref().unwrap(), //D3
            4 => stm32f412g::gpio::PinId::PG12.get_pin().as_ref().unwrap(), //D4
            5 => stm32f412g::gpio::PinId::PF10.get_pin().as_ref().unwrap(), //D5
            6 => stm32f412g::gpio::PinId::PF03.get_pin().as_ref().unwrap(), //D6
            7 => stm32f412g::gpio::PinId::PG11.get_pin().as_ref().unwrap(), //D7
            8 => stm32f412g::gpio::PinId::PG10.get_pin().as_ref().unwrap(), //D8
            9 => stm32f412g::gpio::PinId::PB08.get_pin().as_ref().unwrap(), //D9
            // SPI Pins
            10 => stm32f412g::gpio::PinId::PA15.get_pin().as_ref().unwrap(), //D10
            11 => stm32f412g::gpio::PinId::PA07.get_pin().as_ref().unwrap(),  //D11
            12 => stm32f412g::gpio::PinId::PA06.get_pin().as_ref().unwrap(),  //D12
            13 => stm32f412g::gpio::PinId::PA15.get_pin().as_ref().unwrap()  //D13

            // ADC Pins
            // Enable the to use the ADC pins as GPIO
            // 14 => stm32f412g::gpio::PinId::PA01.get_pin().as_ref().unwrap(), //A0
            // 15 => stm32f412g::gpio::PinId::PC01.get_pin().as_ref().unwrap(), //A1
            // 16 => stm32f412g::gpio::PinId::PC03.get_pin().as_ref().unwrap(), //A2
            // 17 => stm32f412g::gpio::PinId::PC04.get_pin().as_ref().unwrap(), //A3
            // 19 => stm32f412g::gpio::PinId::PC05.get_pin().as_ref().unwrap(), //A4
            // 20 => stm32f412g::gpio::PinId::PB00.get_pin().as_ref().unwrap() //A5
        ),
    )
    .finalize(components::gpio_component_buf!(stm32f412g::gpio::Pin));

    // FT6206

    let mux_i2c = components::i2c::I2CMuxComponent::new(
        &stm32f412g::i2c::I2C1,
        None,
        dynamic_deferred_caller,
    )
    .finalize(components::i2c_mux_component_helper!());

    let ft6206 = components::ft6206::Ft6206Component::new(
        stm32f412g::gpio::PinId::PG05.get_pin().as_ref().unwrap(),
    )
    .finalize(components::ft6206_i2c_component_helper!(mux_i2c));

    ft6206.is_present();

    let adc_channels = static_init!(
        [&'static stm32f412g::adc::Channel; 6],
        [
            &stm32f412g::adc::Channel::Channel1,
            &stm32f412g::adc::Channel::Channel11,
            &stm32f412g::adc::Channel::Channel13,
            &stm32f412g::adc::Channel::Channel14,
            &stm32f412g::adc::Channel::Channel15,
            &stm32f412g::adc::Channel::Channel8,
        ]
    );
    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let grant_adc = board_kernel.create_grant(&grant_cap);
    let adc = static_init!(
        capsules::adc::Adc<'static, stm32f412g::adc::Adc>,
        capsules::adc::Adc::new(
            &stm32f412g::adc::ADC1,
            grant_adc,
            adc_channels,
            &mut capsules::adc::ADC_BUFFER1,
            &mut capsules::adc::ADC_BUFFER2,
            &mut capsules::adc::ADC_BUFFER3
        )
    );
    stm32f412g::adc::ADC1.set_client(adc);

    let nucleo_f412g = STM32F412GDiscovery {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        led: led,
        button: button,
        alarm: alarm,
        gpio: gpio,
        ft6206: ft6206,
        adc: adc,
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

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;

        /// Beginning of the RAM region for app memory.
        ///
        /// This symbol is defined in the linker script.
        static mut _sappmem: u8;

        /// End of the RAM region for app memory.
        ///
        /// This symbol is defined in the linker script.
        static _eappmem: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &mut core::slice::from_raw_parts_mut(
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

    board_kernel.kernel_loop(
        &nucleo_f412g,
        chip,
        Some(&nucleo_f412g.ipc),
        &main_loop_capability,
    );
}
