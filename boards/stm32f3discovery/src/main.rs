//! Board file for STM32F3Discovery Kit development board
//!
//! - <https://www.st.com/en/evaluation-tools/stm32f3discovery.html>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
#![deny(missing_docs)]

use capsules::lsm303dlhc;
use capsules::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::hil::time::Counter;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

// Unit Tests for drivers.
#[allow(dead_code)]
mod multi_alarm_test;
#[allow(dead_code)]
mod virtual_uart_rx_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static stm32f303xc::chip::Stm32f3xx> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct STM32F3Discovery {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    gpio: &'static capsules::gpio::GPIO<'static, stm32f303xc::gpio::Pin<'static>>,
    led: &'static capsules::led::LED<'static, stm32f303xc::gpio::Pin<'static>>,
    button: &'static capsules::button::Button<'static, stm32f303xc::gpio::Pin<'static>>,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
    l3gd20: &'static capsules::l3gd20::L3gd20Spi<'static>,
    lsm303dlhc: &'static capsules::lsm303dlhc::Lsm303dlhcI2C<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f303xc::tim2::Tim2<'static>>,
    >,
    adc: &'static capsules::adc::AdcVirtualized<'static>,
    nonvolatile_storage: &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
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
            capsules::l3gd20::DRIVER_NUM => f(Some(self.l3gd20)),
            capsules::lsm303dlhc::DRIVER_NUM => f(Some(self.lsm303dlhc)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => f(Some(self.nonvolatile_storage)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use stm32f303xc::exti::{LineId, EXTI};
    use stm32f303xc::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f303xc::syscfg::SYSCFG;

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

    // pc4 and pc5 (USART1) is connected to ST-LINK virtual COM port
    PinId::PC04.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    PinId::PC05.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    // button is connected on pa00
    PinId::PA00.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti0, pin);
    });
    cortexm4::nvic::Nvic::new(stm32f303xc::nvic::EXTI0).enable();

    // SPI1 has the l3gd20 sensor connected
    PinId::PA06.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        // AF5 is SPI1/SPI2
        pin.set_alternate_function(AlternateFunction::AF5);
    });
    PinId::PA07.get_pin().as_ref().map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF5 is SPI1/SPI2
        pin.set_alternate_function(AlternateFunction::AF5);
    });
    PinId::PA05.get_pin().as_ref().map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF5 is SPI1/SPI2
        pin.set_alternate_function(AlternateFunction::AF5);
    });
    // PE03 is the chip select pin from the l3gd20 sensor
    PinId::PE03.get_pin().as_ref().map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set();
    });

    stm32f303xc::spi::SPI1.enable_clock();

    // I2C1 has the LSM303DLHC sensor connected
    PinId::PB06.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });
    PinId::PB07.get_pin().as_ref().map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });

    // ADC1
    PinId::PA00.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PA01.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PA02.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PA03.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PF04.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    // ADC2
    PinId::PA04.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PA05.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PA06.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PA07.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    // ADC3
    PinId::PB01.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PE09.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PE13.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PB13.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    // ADC4
    PinId::PE14.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PE15.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PB12.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PB14.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    PinId::PB15.get_pin().as_ref().map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    stm32f303xc::i2c::I2C1.enable_clock();
    stm32f303xc::i2c::I2C1.set_speed(stm32f303xc::i2c::I2CSpeed::Speed400k, 8);
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use stm32f303xc::tim2::TIM2;

    // USART1 IRQn is 37
    cortexm4::nvic::Nvic::new(stm32f303xc::nvic::USART1).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f303xc::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F303VCT6 chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f303xc::init();

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
        stm32f303xc::chip::Stm32f3xx,
        stm32f303xc::chip::Stm32f3xx::new()
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f303xc::usart::USART1.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &stm32f303xc::usart::USART1,
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

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        stm32f303xc::gpio::Pin<'static>,
        (
            stm32f303xc::gpio::PinId::PE09.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE08.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE10.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE15.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE14.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE12.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f303xc::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ))
    .finalize(components::led_component_buf!(
        stm32f303xc::gpio::Pin<'static>
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            stm32f303xc::gpio::Pin<'static>,
            (
                stm32f303xc::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_buf!(
        stm32f303xc::gpio::Pin<'static>
    ));

    // ALARM

    let tim2 = &stm32f303xc::tim2::TIM2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f303xc::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(stm32f303xc::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            stm32f303xc::gpio::Pin<'static>,
            // Left outer connector
            0 => stm32f303xc::gpio::PinId::PC01.get_pin().as_ref().unwrap(),
            1 => stm32f303xc::gpio::PinId::PC03.get_pin().as_ref().unwrap(),
            // 2 => stm32f303xc::gpio::PinId::PA01.get_pin().as_ref().unwrap(),
            // 3 => stm32f303xc::gpio::PinId::PA03.get_pin().as_ref().unwrap(),
            // 4 => stm32f303xc::gpio::PinId::PF04.get_pin().as_ref().unwrap(),
            // 5 => stm32f303xc::gpio::PinId::PA05.get_pin().as_ref().unwrap(),
            // 6 => stm32f303xc::gpio::PinId::PA07.get_pin().as_ref().unwrap(),
            // 7 => stm32f303xc::gpio::PinId::PC05.get_pin().as_ref().unwrap(),
            // 8 => stm32f303xc::gpio::PinId::PB01.get_pin().as_ref().unwrap(),
            9 => stm32f303xc::gpio::PinId::PE07.get_pin().as_ref().unwrap(),
            // 10 => stm32f303xc::gpio::PinId::PE09.get_pin().as_ref().unwrap(),
            11 => stm32f303xc::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
            // 12 => stm32f303xc::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
            // 13 => stm32f303xc::gpio::PinId::PE15.get_pin().as_ref().unwrap(),
            14 => stm32f303xc::gpio::PinId::PB11.get_pin().as_ref().unwrap(),
            // 15 => stm32f303xc::gpio::PinId::PB13.get_pin().as_ref().unwrap(),
            // 16 => stm32f303xc::gpio::PinId::PB15.get_pin().as_ref().unwrap(),
            17 => stm32f303xc::gpio::PinId::PD09.get_pin().as_ref().unwrap(),
            18 => stm32f303xc::gpio::PinId::PD11.get_pin().as_ref().unwrap(),
            19 => stm32f303xc::gpio::PinId::PD13.get_pin().as_ref().unwrap(),
            20 => stm32f303xc::gpio::PinId::PD15.get_pin().as_ref().unwrap(),
            21 => stm32f303xc::gpio::PinId::PC06.get_pin().as_ref().unwrap(),
            // Left inner connector
            22 => stm32f303xc::gpio::PinId::PC00.get_pin().as_ref().unwrap(),
            23 => stm32f303xc::gpio::PinId::PC02.get_pin().as_ref().unwrap(),
            24 => stm32f303xc::gpio::PinId::PF02.get_pin().as_ref().unwrap(),
            // 25 => stm32f303xc::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
            // 26 => stm32f303xc::gpio::PinId::PA02.get_pin().as_ref().unwrap(),
            // 27 => stm32f303xc::gpio::PinId::PA04.get_pin().as_ref().unwrap(),
            // 28 => stm32f303xc::gpio::PinId::PA06.get_pin().as_ref().unwrap(),
            // 29 => stm32f303xc::gpio::PinId::PC04.get_pin().as_ref().unwrap(),
            30 => stm32f303xc::gpio::PinId::PB00.get_pin().as_ref().unwrap(),
            31 => stm32f303xc::gpio::PinId::PB02.get_pin().as_ref().unwrap(),
            32 => stm32f303xc::gpio::PinId::PE08.get_pin().as_ref().unwrap(),
            33 => stm32f303xc::gpio::PinId::PE10.get_pin().as_ref().unwrap(),
            34 => stm32f303xc::gpio::PinId::PE12.get_pin().as_ref().unwrap(),
            // 35 => stm32f303xc::gpio::PinId::PE14.get_pin().as_ref().unwrap(),
            36 => stm32f303xc::gpio::PinId::PB10.get_pin().as_ref().unwrap(),
            // 37 => stm32f303xc::gpio::PinId::PB12.get_pin().as_ref().unwrap(),
            // 38 => stm32f303xc::gpio::PinId::PB14.get_pin().as_ref().unwrap(),
            39 => stm32f303xc::gpio::PinId::PD08.get_pin().as_ref().unwrap(),
            40 => stm32f303xc::gpio::PinId::PD10.get_pin().as_ref().unwrap(),
            41 => stm32f303xc::gpio::PinId::PD12.get_pin().as_ref().unwrap(),
            42 => stm32f303xc::gpio::PinId::PD14.get_pin().as_ref().unwrap(),
            43 => stm32f303xc::gpio::PinId::PC07.get_pin().as_ref().unwrap(),
            // Right inner connector
            44 => stm32f303xc::gpio::PinId::PF09.get_pin().as_ref().unwrap(),
            45 => stm32f303xc::gpio::PinId::PF00.get_pin().as_ref().unwrap(),
            46 => stm32f303xc::gpio::PinId::PC14.get_pin().as_ref().unwrap(),
            47 => stm32f303xc::gpio::PinId::PE06.get_pin().as_ref().unwrap(),
            48 => stm32f303xc::gpio::PinId::PE04.get_pin().as_ref().unwrap(),
            49 => stm32f303xc::gpio::PinId::PE02.get_pin().as_ref().unwrap(),
            50 => stm32f303xc::gpio::PinId::PE00.get_pin().as_ref().unwrap(),
            51 => stm32f303xc::gpio::PinId::PB08.get_pin().as_ref().unwrap(),
            // 52 => stm32f303xc::gpio::PinId::PB06.get_pin().as_ref().unwrap(),
            53 => stm32f303xc::gpio::PinId::PB04.get_pin().as_ref().unwrap(),
            54 => stm32f303xc::gpio::PinId::PD07.get_pin().as_ref().unwrap(),
            55 => stm32f303xc::gpio::PinId::PD05.get_pin().as_ref().unwrap(),
            56 => stm32f303xc::gpio::PinId::PD03.get_pin().as_ref().unwrap(),
            57 => stm32f303xc::gpio::PinId::PD01.get_pin().as_ref().unwrap(),
            58 => stm32f303xc::gpio::PinId::PC12.get_pin().as_ref().unwrap(),
            59 => stm32f303xc::gpio::PinId::PC10.get_pin().as_ref().unwrap(),
            60 => stm32f303xc::gpio::PinId::PA14.get_pin().as_ref().unwrap(),
            61 => stm32f303xc::gpio::PinId::PF06.get_pin().as_ref().unwrap(),
            62 => stm32f303xc::gpio::PinId::PA12.get_pin().as_ref().unwrap(),
            63 => stm32f303xc::gpio::PinId::PA10.get_pin().as_ref().unwrap(),
            64 => stm32f303xc::gpio::PinId::PA08.get_pin().as_ref().unwrap(),
            65 => stm32f303xc::gpio::PinId::PC08.get_pin().as_ref().unwrap(),
            // Right outer connector
            66 => stm32f303xc::gpio::PinId::PF10.get_pin().as_ref().unwrap(),
            67 => stm32f303xc::gpio::PinId::PF01.get_pin().as_ref().unwrap(),
            68 => stm32f303xc::gpio::PinId::PC15.get_pin().as_ref().unwrap(),
            69 => stm32f303xc::gpio::PinId::PC13.get_pin().as_ref().unwrap(),
            70 => stm32f303xc::gpio::PinId::PE05.get_pin().as_ref().unwrap(),
            71 => stm32f303xc::gpio::PinId::PE03.get_pin().as_ref().unwrap(),
            72 => stm32f303xc::gpio::PinId::PE01.get_pin().as_ref().unwrap(),
            73 => stm32f303xc::gpio::PinId::PB09.get_pin().as_ref().unwrap(),
            // 74 => stm32f303xc::gpio::PinId::PB07.get_pin().as_ref().unwrap(),
            75 => stm32f303xc::gpio::PinId::PB05.get_pin().as_ref().unwrap(),
            76 => stm32f303xc::gpio::PinId::PB03.get_pin().as_ref().unwrap(),
            77 => stm32f303xc::gpio::PinId::PD06.get_pin().as_ref().unwrap(),
            78 => stm32f303xc::gpio::PinId::PD04.get_pin().as_ref().unwrap(),
            79 => stm32f303xc::gpio::PinId::PD02.get_pin().as_ref().unwrap(),
            80 => stm32f303xc::gpio::PinId::PD00.get_pin().as_ref().unwrap(),
            81 => stm32f303xc::gpio::PinId::PC11.get_pin().as_ref().unwrap(),
            82 => stm32f303xc::gpio::PinId::PA15.get_pin().as_ref().unwrap(),
            83 => stm32f303xc::gpio::PinId::PA13.get_pin().as_ref().unwrap(),
            84 => stm32f303xc::gpio::PinId::PA11.get_pin().as_ref().unwrap(),
            85 => stm32f303xc::gpio::PinId::PA09.get_pin().as_ref().unwrap(),
            86 => stm32f303xc::gpio::PinId::PC09.get_pin().as_ref().unwrap()
        ),
    )
    .finalize(components::gpio_component_buf!(
        stm32f303xc::gpio::Pin<'static>
    ));

    // L3GD20 sensor
    let spi_mux = components::spi::SpiMuxComponent::new(&stm32f303xc::spi::SPI1)
        .finalize(components::spi_mux_component_helper!(stm32f303xc::spi::Spi));

    let l3gd20 = components::l3gd20::L3gd20SpiComponent::new().finalize(
        components::l3gd20_spi_component_helper!(
            // spi type
            stm32f303xc::spi::Spi,
            // chip select
            stm32f303xc::gpio::PinId::PE03,
            // spi mux
            spi_mux
        ),
    );

    l3gd20.power_on();

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let grant_temperature = board_kernel.create_grant(&grant_cap);

    // Comment this if you want to use the ADC MCU temp sensor
    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(l3gd20, grant_temperature)
    );
    kernel::hil::sensors::TemperatureDriver::set_client(l3gd20, temp);

    // LSM303DLHC

    let mux_i2c = components::i2c::I2CMuxComponent::new(
        &stm32f303xc::i2c::I2C1,
        None,
        dynamic_deferred_caller,
    )
    .finalize(components::i2c_mux_component_helper!());

    let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new()
        .finalize(components::lsm303dlhc_i2c_component_helper!(mux_i2c));

    lsm303dlhc.configure(
        lsm303dlhc::Lsm303dlhcAccelDataRate::DataRate25Hz,
        false,
        lsm303dlhc::Lsm303dlhcScale::Scale2G,
        false,
        true,
        lsm303dlhc::Lsm303dlhcMagnetoDataRate::DataRate3_0Hz,
        lsm303dlhc::Lsm303dlhcRange::Range1_9G,
    );

    let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
        .finalize(components::ninedof_component_helper!(l3gd20, lsm303dlhc));

    let adc_mux = components::adc::AdcMuxComponent::new(&stm32f303xc::adc::ADC1)
        .finalize(components::adc_mux_component_helper!(stm32f303xc::adc::Adc));

    // Uncomment this if you want to use ADC MCU temp sensor
    // let temp_sensor = components::temperature_stm::TemperatureSTMComponent::new(4.3, 1.43)
    //     .finalize(components::temperaturestm_adc_component_helper!(
    //         // spi type
    //         stm32f303xc::adc::Adc,
    //         // chip select
    //         stm32f303xc::adc::Channel::Channel18,
    //         // spi mux
    //         adc_mux
    //     ));
    // let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    // let grant_temperature = board_kernel.create_grant(&grant_cap);

    // let temp = static_init!(
    //     capsules::temperature::TemperatureSensor<'static>,
    //     capsules::temperature::TemperatureSensor::new(temp_sensor, grant_temperature)
    // );
    // kernel::hil::sensors::TemperatureDriver::set_client(temp_sensor, temp);

    let adc_channel_0 =
        components::adc::AdcComponent::new(&adc_mux, stm32f303xc::adc::Channel::Channel0)
            .finalize(components::adc_component_helper!(stm32f303xc::adc::Adc));

    let adc_channel_1 =
        components::adc::AdcComponent::new(&adc_mux, stm32f303xc::adc::Channel::Channel1)
            .finalize(components::adc_component_helper!(stm32f303xc::adc::Adc));

    let adc_channel_2 =
        components::adc::AdcComponent::new(&adc_mux, stm32f303xc::adc::Channel::Channel2)
            .finalize(components::adc_component_helper!(stm32f303xc::adc::Adc));

    let adc_channel_3 =
        components::adc::AdcComponent::new(&adc_mux, stm32f303xc::adc::Channel::Channel3)
            .finalize(components::adc_component_helper!(stm32f303xc::adc::Adc));

    let adc_channel_4 =
        components::adc::AdcComponent::new(&adc_mux, stm32f303xc::adc::Channel::Channel4)
            .finalize(components::adc_component_helper!(stm32f303xc::adc::Adc));

    let adc_channel_5 =
        components::adc::AdcComponent::new(&adc_mux, stm32f303xc::adc::Channel::Channel5)
            .finalize(components::adc_component_helper!(stm32f303xc::adc::Adc));

    let adc_syscall = components::adc::AdcVirtualComponent::new(board_kernel).finalize(
        components::adc_syscall_component_helper!(
            adc_channel_0,
            adc_channel_1,
            adc_channel_2,
            adc_channel_3,
            adc_channel_4,
            adc_channel_5
        ),
    );

    // Kernel storage region, allocated with the storage_volume!
    // macro in common/utils.rs
    extern "C" {
        /// Beginning on the ROM region containing app images.
        static _sstorage: u8;
        static _estorage: u8;
    }

    let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
        board_kernel,
        &stm32f303xc::flash::FLASH,
        0x08038000, // Start address for userspace accesible region
        0x8000,     // Length of userspace accesible region (16 pages)
        &_sstorage as *const u8 as usize,
        &_estorage as *const u8 as usize - &_sstorage as *const u8 as usize,
    )
    .finalize(components::nv_storage_component_helper!(
        stm32f303xc::flash::Flash
    ));

    let stm32f3discovery = STM32F3Discovery {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        gpio: gpio,
        led: led,
        button: button,
        alarm: alarm,
        l3gd20: l3gd20,
        lsm303dlhc: lsm303dlhc,
        ninedof: ninedof,
        temp: temp,
        adc: adc_syscall,
        nonvolatile_storage: nonvolatile_storage,
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

    // Uncomment this to enable the watchdog
    // chip.enable_watchdog();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    //Uncomment to run multi alarm test
    //multi_alarm_test::run_multi_alarm(mux_alarm);
    board_kernel.kernel_loop(
        &stm32f3discovery,
        chip,
        Some(&stm32f3discovery.ipc),
        scheduler,
        &main_loop_capability,
    );
}
