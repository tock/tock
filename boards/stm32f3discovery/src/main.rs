//! Board file for STM32F3Discovery Kit development board
//!
//! - <https://www.st.com/en/evaluation-tools/stm32f3discovery.html>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use capsules::lsm303xx;
use capsules::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::hil::led::LedHigh;
use kernel::hil::time::Counter;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use stm32f303xc::chip::Stm32f3xxDefaultPeripherals;

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
static mut CHIP: Option<&'static stm32f303xc::chip::Stm32f3xx<Stm32f3xxDefaultPeripherals>> = None;

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
    ipc: kernel::ipc::IPC<NUM_PROCS>,
    gpio: &'static capsules::gpio::GPIO<'static, stm32f303xc::gpio::Pin<'static>>,
    led: &'static capsules::led::LedDriver<
        'static,
        LedHigh<'static, stm32f303xc::gpio::Pin<'static>>,
    >,
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
unsafe fn set_pin_primary_functions(
    syscfg: &stm32f303xc::syscfg::Syscfg,
    exti: &stm32f303xc::exti::Exti,
    spi1: &stm32f303xc::spi::Spi,
    i2c1: &stm32f303xc::i2c::I2C,
    gpio_ports: &'static stm32f303xc::gpio::GpioPorts<'static>,
) {
    use stm32f303xc::exti::LineId;
    use stm32f303xc::gpio::{AlternateFunction, Mode, PinId, PortId};

    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::B).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::C).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::D).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::E).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::F).enable_clock();

    gpio_ports.get_pin(PinId::PE14).map(|pin| {
        pin.make_output();
        pin.set();
    });

    // User LD3 is connected to PE09. Configure PE09 as `debug_gpio!(0, ...)`
    gpio_ports.get_pin(PinId::PE09).map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    // pc4 and pc5 (USART1) is connected to ST-LINK virtual COM port
    gpio_ports.get_pin(PinId::PC04).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    gpio_ports.get_pin(PinId::PC05).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    // button is connected on pa00
    gpio_ports.get_pin(PinId::PA00).map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        exti.associate_line_gpiopin(LineId::Exti0, &pin);
    });
    cortexm4::nvic::Nvic::new(stm32f303xc::nvic::EXTI0).enable();

    // SPI1 has the l3gd20 sensor connected
    gpio_ports.get_pin(PinId::PA06).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        // AF5 is SPI1/SPI2
        pin.set_alternate_function(AlternateFunction::AF5);
    });
    gpio_ports.get_pin(PinId::PA07).map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF5 is SPI1/SPI2
        pin.set_alternate_function(AlternateFunction::AF5);
    });
    gpio_ports.get_pin(PinId::PA05).map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF5 is SPI1/SPI2
        pin.set_alternate_function(AlternateFunction::AF5);
    });
    // PE03 is the chip select pin from the l3gd20 sensor
    gpio_ports.get_pin(PinId::PE03).map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set();
    });

    spi1.enable_clock();

    // I2C1 has the LSM303DLHC sensor connected
    gpio_ports.get_pin(PinId::PB06).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });
    gpio_ports.get_pin(PinId::PB07).map(|pin| {
        pin.make_output();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });

    // ADC1
    gpio_ports.get_pin(PinId::PA00).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PA01).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PA02).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PA03).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PF04).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    // ADC2
    gpio_ports.get_pin(PinId::PA04).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PA05).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PA06).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PA07).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    // ADC3
    gpio_ports.get_pin(PinId::PB01).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PE09).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PE13).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PB13).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    // ADC4
    gpio_ports.get_pin(PinId::PE14).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PE15).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PB12).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PB14).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    gpio_ports.get_pin(PinId::PB15).map(|pin| {
        pin.set_mode(stm32f303xc::gpio::Mode::AnalogMode);
    });

    i2c1.enable_clock();
    i2c1.set_speed(stm32f303xc::i2c::I2CSpeed::Speed400k, 8);
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals(tim2: &stm32f303xc::tim2::Tim2) {
    // USART1 IRQn is 37
    cortexm4::nvic::Nvic::new(stm32f303xc::nvic::USART1).enable();

    // TIM2 IRQn is 28
    tim2.enable_clock();
    tim2.start();
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

    let rcc = static_init!(stm32f303xc::rcc::Rcc, stm32f303xc::rcc::Rcc::new());
    let syscfg = static_init!(
        stm32f303xc::syscfg::Syscfg,
        stm32f303xc::syscfg::Syscfg::new(rcc)
    );
    let exti = static_init!(
        stm32f303xc::exti::Exti,
        stm32f303xc::exti::Exti::new(syscfg)
    );

    let peripherals = static_init!(
        Stm32f3xxDefaultPeripherals,
        Stm32f3xxDefaultPeripherals::new(rcc, exti)
    );
    set_pin_primary_functions(
        syscfg,
        &peripherals.exti,
        &peripherals.spi1,
        &peripherals.i2c1,
        &peripherals.gpio_ports,
    );

    setup_peripherals(&peripherals.tim2);
    peripherals.setup_circular_deps();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let chip = static_init!(
        stm32f303xc::chip::Stm32f3xx<Stm32f3xxDefaultPeripherals>,
        stm32f303xc::chip::Stm32f3xx::new(peripherals, rcc)
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    peripherals.usart1.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &peripherals.usart1,
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
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM as u32,
        uart_mux,
    )
    .finalize(());
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
        LedHigh<'static, stm32f303xc::gpio::Pin<'static>>,
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE09)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE08)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE10)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE15)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE11)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE14)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE12)
                .unwrap()
        ),
        LedHigh::new(
            &peripherals
                .gpio_ports
                .get_pin(stm32f303xc::gpio::PinId::PE13)
                .unwrap()
        ),
    ))
    .finalize(components::led_component_buf!(
        LedHigh<'static, stm32f303xc::gpio::Pin<'static>>
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules::button::DRIVER_NUM as u32,
        components::button_component_helper!(
            stm32f303xc::gpio::Pin<'static>,
            (
                &peripherals
                    .gpio_ports
                    .get_pin(stm32f303xc::gpio::PinId::PA00)
                    .unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_buf!(
        stm32f303xc::gpio::Pin<'static>
    ));

    // ALARM

    let tim2 = &peripherals.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f303xc::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules::alarm::DRIVER_NUM as u32,
        mux_alarm,
    )
    .finalize(components::alarm_component_helper!(stm32f303xc::tim2::Tim2));

    let gpio_ports = &peripherals.gpio_ports;
    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        capsules::gpio::DRIVER_NUM as u32,
        components::gpio_component_helper!(
            stm32f303xc::gpio::Pin<'static>,
            // Left outer connector
            0 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC01).unwrap(),
            1 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC03).unwrap(),
            // 2 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA01).unwrap(),
            // 3 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA03).unwrap(),
            // 4 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF04).unwrap(),
            // 5 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA05).unwrap(),
            // 6 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA07).unwrap(),
            // 7 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC05).unwrap(),
            // 8 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB01).unwrap(),
            9 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE07).unwrap(),
            // 10 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE09).unwrap(),
            11 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE11).unwrap(),
            // 12 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE13).unwrap(),
            // 13 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE15).unwrap(),
            14 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB11).unwrap(),
            // 15 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB13).unwrap(),
            // 16 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB15).unwrap(),
            17 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD09).unwrap(),
            18 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD11).unwrap(),
            19 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD13).unwrap(),
            20 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD15).unwrap(),
            21 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC06).unwrap(),
            // Left inner connector
            22 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC00).unwrap(),
            23 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC02).unwrap(),
            24 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF02).unwrap(),
            // 25 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA00).unwrap(),
            // 26 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA02).unwrap(),
            // 27 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA04).unwrap(),
            // 28 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA06).unwrap(),
            // 29 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC04).unwrap(),
            30 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB00).unwrap(),
            31 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB02).unwrap(),
            32 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE08).unwrap(),
            33 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE10).unwrap(),
            34 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE12).unwrap(),
            // 35 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE14).unwrap(),
            36 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB10).unwrap(),
            // 37 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB12).unwrap(),
            // 38 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB14).unwrap(),
            39 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD08).unwrap(),
            40 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD10).unwrap(),
            41 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD12).unwrap(),
            42 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD14).unwrap(),
            43 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC07).unwrap(),
            // Right inner connector
            44 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF09).unwrap(),
            45 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF00).unwrap(),
            46 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC14).unwrap(),
            47 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE06).unwrap(),
            48 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE04).unwrap(),
            49 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE02).unwrap(),
            50 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE00).unwrap(),
            51 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB08).unwrap(),
            // 52 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB06).unwrap(),
            53 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB04).unwrap(),
            54 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD07).unwrap(),
            55 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD05).unwrap(),
            56 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD03).unwrap(),
            57 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD01).unwrap(),
            58 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC12).unwrap(),
            59 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC10).unwrap(),
            60 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA14).unwrap(),
            61 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF06).unwrap(),
            62 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA12).unwrap(),
            63 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA10).unwrap(),
            64 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA08).unwrap(),
            65 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC08).unwrap(),
            // Right outer connector
            66 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF10).unwrap(),
            67 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PF01).unwrap(),
            68 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC15).unwrap(),
            69 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC13).unwrap(),
            70 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE05).unwrap(),
            71 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE03).unwrap(),
            72 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE01).unwrap(),
            73 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB09).unwrap(),
            // 74 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB07).unwrap(),
            75 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB05).unwrap(),
            76 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PB03).unwrap(),
            77 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD06).unwrap(),
            78 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD04).unwrap(),
            79 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD02).unwrap(),
            80 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PD00).unwrap(),
            81 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC11).unwrap(),
            82 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA15).unwrap(),
            83 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA13).unwrap(),
            84 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA11).unwrap(),
            85 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PA09).unwrap(),
            86 => &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PC09).unwrap()
        ),
    )
    .finalize(components::gpio_component_buf!(
        stm32f303xc::gpio::Pin<'static>
    ));

    // L3GD20 sensor
    let spi_mux = components::spi::SpiMuxComponent::new(&peripherals.spi1)
        .finalize(components::spi_mux_component_helper!(stm32f303xc::spi::Spi));

    let l3gd20 = components::l3gd20::L3gd20SpiComponent::new().finalize(
        components::l3gd20_spi_component_helper!(
            // spi type
            stm32f303xc::spi::Spi,
            // chip select
            &gpio_ports.get_pin(stm32f303xc::gpio::PinId::PE03).unwrap(),
            // spi mux
            spi_mux
        ),
    );

    l3gd20.power_on();

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let grant_temperature =
        board_kernel.create_grant(capsules::temperature::DRIVER_NUM as u32, &grant_cap);

    // Comment this if you want to use the ADC MCU temp sensor
    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(l3gd20, grant_temperature)
    );
    kernel::hil::sensors::TemperatureDriver::set_client(l3gd20, temp);

    // LSM303DLHC

    let mux_i2c =
        components::i2c::I2CMuxComponent::new(&peripherals.i2c1, None, dynamic_deferred_caller)
            .finalize(components::i2c_mux_component_helper!());

    let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new()
        .finalize(components::lsm303dlhc_i2c_component_helper!(mux_i2c));

    lsm303dlhc.configure(
        lsm303xx::Lsm303AccelDataRate::DataRate25Hz,
        false,
        lsm303xx::Lsm303Scale::Scale2G,
        false,
        true,
        lsm303xx::Lsm303MagnetoDataRate::DataRate3_0Hz,
        lsm303xx::Lsm303Range::Range1_9G,
    );

    let ninedof = components::ninedof::NineDofComponent::new(
        board_kernel,
        capsules::ninedof::DRIVER_NUM as u32,
    )
    .finalize(components::ninedof_component_helper!(l3gd20, lsm303dlhc));

    let adc_mux = components::adc::AdcMuxComponent::new(&peripherals.adc1)
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

    let adc_syscall =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules::adc::DRIVER_NUM as u32)
            .finalize(components::adc_syscall_component_helper!(
                adc_channel_0,
                adc_channel_1,
                adc_channel_2,
                adc_channel_3,
                adc_channel_4,
                adc_channel_5
            ));

    // Kernel storage region, allocated with the storage_volume!
    // macro in common/utils.rs
    extern "C" {
        /// Beginning on the ROM region containing app images.
        static _sstorage: u8;
        static _estorage: u8;
    }

    let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
        board_kernel,
        capsules::nonvolatile_storage_driver::DRIVER_NUM as u32,
        &peripherals.flash,
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
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM as u32,
            &memory_allocation_capability,
        ),
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
