// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for STM32F412GDiscovery Discovery kit development board
//!
//! - <https://www.st.com/en/evaluation-tools/32f412gdiscovery.html>

#![no_std]
#![no_main]
#![deny(missing_docs)]
use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use components::rng::RngComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::hil::led::LedLow;
use kernel::hil::screen::ScreenRotation;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};
use stm32f412g::chip_specs::Stm32f412Specs;
use stm32f412g::clocks::hsi::HSI_FREQUENCY_MHZ;
use stm32f412g::interrupt_service::Stm32f412gDefaultPeripherals;
use stm32f412g::rcc::PllSource;

/// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None, None, None, None];

static mut CHIP: Option<&'static stm32f412g::chip::Stm32f4xx<Stm32f412gDefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

type TemperatureSTMSensor = components::temperature_stm::TemperatureSTMComponentType<
    capsules_core::virtualizers::virtual_adc::AdcDevice<'static, stm32f412g::adc::Adc<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<TemperatureSTMSensor>;
type RngDriver = components::rng::RngComponentType<stm32f412g::trng::Trng<'static>>;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct STM32F412GDiscovery {
    console: &'static capsules_core::console::Console<'static>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, stm32f412g::gpio::Pin<'static>>,
        4,
    >,
    button: &'static capsules_core::button::Button<'static, stm32f412g::gpio::Pin<'static>>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f412g::tim2::Tim2<'static>>,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, stm32f412g::gpio::Pin<'static>>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    touch: &'static capsules_extra::touch::Touch<'static>,
    screen: &'static capsules_extra::screen::Screen<'static>,
    temperature: &'static TemperatureDriver,
    rng: &'static RngDriver,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for STM32F412GDiscovery {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::touch::DRIVER_NUM => f(Some(self.touch)),
            capsules_extra::screen::DRIVER_NUM => f(Some(self.screen)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            _ => f(None),
        }
    }
}

impl
    KernelResources<
        stm32f412g::chip::Stm32f4xx<
            'static,
            stm32f412g::interrupt_service::Stm32f412gDefaultPeripherals<'static>,
        >,
    > for STM32F412GDiscovery
{
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

/// Helper function called during bring-up that configures DMA.
unsafe fn setup_dma(
    dma: &stm32f412g::dma::Dma1,
    dma_streams: &'static [stm32f412g::dma::Stream<stm32f412g::dma::Dma1>; 8],
    usart2: &'static stm32f412g::usart::Usart<stm32f412g::dma::Dma1>,
) {
    use stm32f412g::dma::Dma1Peripheral;
    use stm32f412g::usart;

    dma.enable_clock();

    let usart2_tx_stream = &dma_streams[Dma1Peripheral::USART2_TX.get_stream_idx()];
    let usart2_rx_stream = &dma_streams[Dma1Peripheral::USART2_RX.get_stream_idx()];

    usart2.set_dma(
        usart::TxDMA(usart2_tx_stream),
        usart::RxDMA(usart2_rx_stream),
    );

    usart2_tx_stream.set_client(usart2);
    usart2_rx_stream.set_client(usart2);

    usart2_tx_stream.setup(Dma1Peripheral::USART2_TX);
    usart2_rx_stream.setup(Dma1Peripheral::USART2_RX);

    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART2_TX.get_stream_irqn()).enable();
    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART2_RX.get_stream_irqn()).enable();
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions(
    syscfg: &stm32f412g::syscfg::Syscfg,
    i2c1: &stm32f412g::i2c::I2C,
    gpio_ports: &'static stm32f412g::gpio::GpioPorts<'static>,
    peripheral_clock_frequency: usize,
) {
    use kernel::hil::gpio::Configure;
    use stm32f412g::gpio::{AlternateFunction, Mode, PinId, PortId};

    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::E).enable_clock();

    // User LD3 is connected to PE02. Configure PE02 as `debug_gpio!(0, ...)`
    gpio_ports.get_pin(PinId::PE02).map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();

    // pa2 and pa3 (USART2) is connected to ST-LINK virtual COM port
    gpio_ports.get_pin(PinId::PA02).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    gpio_ports.get_pin(PinId::PA03).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    // uncomment this if you do not plan to use the joystick up, as they both use Exti0
    // joystick selection is connected on pa00
    // gpio_ports.get_pin(PinId::PA00).map(|pin| {
    //     pin.enable_interrupt();
    // });

    // joystick down is connected on pg01
    gpio_ports.get_pin(PinId::PG01).map(|pin| {
        pin.enable_interrupt();
    });

    // joystick left is connected on pf15
    gpio_ports.get_pin(PinId::PF15).map(|pin| {
        pin.enable_interrupt();
    });

    // joystick right is connected on pf14
    gpio_ports.get_pin(PinId::PF14).map(|pin| {
        pin.enable_interrupt();
    });

    // joystick up is connected on pg00
    gpio_ports.get_pin(PinId::PG00).map(|pin| {
        pin.enable_interrupt();
    });

    // enable interrupt for D0
    gpio_ports.get_pin(PinId::PG09).map(|pin| {
        pin.enable_interrupt();
    });

    // Enable clocks for GPIO Ports
    // Disable some of them if you don't need some of the GPIOs
    gpio_ports.get_port_from_port_id(PortId::B).enable_clock();
    // Ports A and E are already enabled
    gpio_ports.get_port_from_port_id(PortId::C).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::D).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::F).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::G).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::H).enable_clock();

    // I2C1 has the TouchPanel connected
    gpio_ports.get_pin(PinId::PB06).map(|pin| {
        // pin.make_output();
        pin.set_mode_output_opendrain();
        pin.set_mode(Mode::AlternateFunctionMode);
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });
    gpio_ports.get_pin(PinId::PB07).map(|pin| {
        // pin.make_output();
        pin.set_mode_output_opendrain();
        pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF4 is I2C
        pin.set_alternate_function(AlternateFunction::AF4);
    });

    i2c1.enable_clock();
    i2c1.set_speed(
        stm32f412g::i2c::I2CSpeed::Speed400k,
        peripheral_clock_frequency,
    );

    // FT6206 interrupt
    gpio_ports.get_pin(PinId::PG05).map(|pin| {
        pin.enable_interrupt();
    });

    // ADC

    // Arduino A0
    gpio_ports.get_pin(PinId::PA01).map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A1
    gpio_ports.get_pin(PinId::PC01).map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A2
    gpio_ports.get_pin(PinId::PC03).map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A3
    gpio_ports.get_pin(PinId::PC04).map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A4
    gpio_ports.get_pin(PinId::PC05).map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // Arduino A5
    gpio_ports.get_pin(PinId::PB00).map(|pin| {
        pin.set_mode(stm32f412g::gpio::Mode::AnalogMode);
    });

    // EXTI9_5 interrupts is delivered at IRQn 23 (EXTI9_5)
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::EXTI9_5).enable();

    // LCD

    let pins = [
        PinId::PD00,
        PinId::PD01,
        PinId::PD04,
        PinId::PD05,
        PinId::PD08,
        PinId::PD09,
        PinId::PD10,
        PinId::PD14,
        PinId::PD15,
        PinId::PD07,
        PinId::PE07,
        PinId::PE08,
        PinId::PE09,
        PinId::PE10,
        PinId::PE11,
        PinId::PE12,
        PinId::PE13,
        PinId::PE14,
        PinId::PE15,
        PinId::PF00,
    ];

    for pin in pins.iter() {
        gpio_ports.get_pin(*pin).map(|pin| {
            pin.set_mode(stm32f412g::gpio::Mode::AlternateFunctionMode);
            pin.set_floating_state(gpio::FloatingState::PullUp);
            pin.set_speed();
            pin.set_alternate_function(stm32f412g::gpio::AlternateFunction::AF12);
        });
    }

    use kernel::hil::gpio::Output;

    gpio_ports.get_pin(PinId::PF05).map(|pin| {
        pin.make_output();
        pin.set_floating_state(gpio::FloatingState::PullNone);
        pin.set();
    });

    gpio_ports.get_pin(PinId::PG04).map(|pin| {
        pin.make_input();
    });
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals(
    tim2: &stm32f412g::tim2::Tim2,
    fsmc: &stm32f412g::fsmc::Fsmc,
    trng: &stm32f412g::trng::Trng,
) {
    // USART2 IRQn is 38
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::USART2).enable();

    // TIM2 IRQn is 28
    tim2.enable_clock();
    tim2.start();
    cortexm4::nvic::Nvic::new(stm32f412g::nvic::TIM2).enable();

    // FSMC
    fsmc.enable();

    // RNG
    trng.enable_clock();
}

/// Main function.
///
/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    STM32F412GDiscovery,
    &'static stm32f412g::chip::Stm32f4xx<'static, Stm32f412gDefaultPeripherals<'static>>,
) {
    stm32f412g::init();

    let rcc = static_init!(stm32f412g::rcc::Rcc, stm32f412g::rcc::Rcc::new());
    let clocks = static_init!(
        stm32f412g::clocks::Clocks<Stm32f412Specs>,
        stm32f412g::clocks::Clocks::new(rcc)
    );

    let syscfg = static_init!(
        stm32f412g::syscfg::Syscfg,
        stm32f412g::syscfg::Syscfg::new(clocks)
    );

    let exti = static_init!(stm32f412g::exti::Exti, stm32f412g::exti::Exti::new(syscfg));

    let dma1 = static_init!(stm32f412g::dma::Dma1, stm32f412g::dma::Dma1::new(clocks));
    let dma2 = static_init!(stm32f412g::dma::Dma2, stm32f412g::dma::Dma2::new(clocks));

    let peripherals = static_init!(
        Stm32f412gDefaultPeripherals,
        Stm32f412gDefaultPeripherals::new(clocks, exti, dma1, dma2)
    );

    peripherals.init();

    let _ = clocks.set_ahb_prescaler(stm32f412g::rcc::AHBPrescaler::DivideBy1);
    let _ = clocks.set_apb1_prescaler(stm32f412g::rcc::APBPrescaler::DivideBy4);
    let _ = clocks.set_apb2_prescaler(stm32f412g::rcc::APBPrescaler::DivideBy2);
    let _ = clocks.set_pll_frequency_mhz(PllSource::HSI, 100);
    let _ = clocks.pll.enable();
    let _ = clocks.set_sys_clock_source(stm32f412g::rcc::SysClockSource::PLL);

    let base_peripherals = &peripherals.stm32f4;
    setup_peripherals(
        &base_peripherals.tim2,
        &base_peripherals.fsmc,
        &peripherals.trng,
    );

    set_pin_primary_functions(
        syscfg,
        &base_peripherals.i2c1,
        &base_peripherals.gpio_ports,
        clocks.get_apb1_frequency_mhz(),
    );

    setup_dma(
        dma1,
        &base_peripherals.dma1_streams,
        &base_peripherals.usart2,
    );

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    let chip = static_init!(
        stm32f412g::chip::Stm32f4xx<Stm32f412gDefaultPeripherals>,
        stm32f412g::chip::Stm32f4xx::new(peripherals)
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    base_peripherals.usart2.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(&base_peripherals.usart2, 115200)
        .finalize(components::uart_mux_component_static!());

    (*addr_of_mut!(io::WRITER)).set_initialized();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

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

    // Clock to Port A is enabled in `set_pin_primary_functions()`

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, stm32f412g::gpio::Pin>,
        LedLow::new(
            base_peripherals
                .gpio_ports
                .get_pin(stm32f412g::gpio::PinId::PE00)
                .unwrap()
        ),
        LedLow::new(
            base_peripherals
                .gpio_ports
                .get_pin(stm32f412g::gpio::PinId::PE01)
                .unwrap()
        ),
        LedLow::new(
            base_peripherals
                .gpio_ports
                .get_pin(stm32f412g::gpio::PinId::PE02)
                .unwrap()
        ),
        LedLow::new(
            base_peripherals
                .gpio_ports
                .get_pin(stm32f412g::gpio::PinId::PE03)
                .unwrap()
        ),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32f412g::gpio::Pin,
            // Select
            (
                base_peripherals
                    .gpio_ports
                    .get_pin(stm32f412g::gpio::PinId::PA00)
                    .unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Down
            (
                base_peripherals
                    .gpio_ports
                    .get_pin(stm32f412g::gpio::PinId::PG01)
                    .unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Left
            (
                base_peripherals
                    .gpio_ports
                    .get_pin(stm32f412g::gpio::PinId::PF15)
                    .unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Right
            (
                base_peripherals
                    .gpio_ports
                    .get_pin(stm32f412g::gpio::PinId::PF14)
                    .unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            ),
            // Up
            (
                base_peripherals
                    .gpio_ports
                    .get_pin(stm32f412g::gpio::PinId::PG00)
                    .unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_static!(stm32f412g::gpio::Pin));

    // ALARM

    let tim2 = &base_peripherals.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_static!(stm32f412g::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(stm32f412g::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            stm32f412g::gpio::Pin,
            // Arduino like RX/TX
            0 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG09).unwrap(), //D0
            1 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG14).unwrap(), //D1
            2 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG13).unwrap(), //D2
            3 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PF04).unwrap(), //D3
            4 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG12).unwrap(), //D4
            5 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PF10).unwrap(), //D5
            6 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PF03).unwrap(), //D6
            7 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG11).unwrap(), //D7
            8 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PG10).unwrap(), //D8
            9 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PB08).unwrap(), //D9
            // SPI Pins
            10 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PA15).unwrap(), //D10
            11 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PA07).unwrap(),  //D11
            12 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PA06).unwrap(),  //D12
            13 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PA15).unwrap()  //D13

            // ADC Pins
            // Enable the to use the ADC pins as GPIO
            // 14 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PA01).unwrap(), //A0
            // 15 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PC01).unwrap(), //A1
            // 16 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PC03).unwrap(), //A2
            // 17 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PC04).unwrap(), //A3
            // 19 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PC05).unwrap(), //A4
            // 20 => base_peripherals.gpio_ports.get_pin(stm32f412g::gpio::PinId::PB00).unwrap() //A5
        ),
    )
    .finalize(components::gpio_component_static!(stm32f412g::gpio::Pin));

    // RNG
    let rng = RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &peripherals.trng,
    )
    .finalize(components::rng_component_static!(stm32f412g::trng::Trng));

    // FT6206

    let mux_i2c = components::i2c::I2CMuxComponent::new(&base_peripherals.i2c1, None)
        .finalize(components::i2c_mux_component_static!(stm32f412g::i2c::I2C));

    let ft6x06 = components::ft6x06::Ft6x06Component::new(
        mux_i2c,
        0x38,
        base_peripherals
            .gpio_ports
            .get_pin(stm32f412g::gpio::PinId::PG05)
            .unwrap(),
    )
    .finalize(components::ft6x06_component_static!(stm32f412g::i2c::I2C));

    let bus = components::bus::Bus8080BusComponent::new(&base_peripherals.fsmc).finalize(
        components::bus8080_bus_component_static!(stm32f412g::fsmc::Fsmc,),
    );

    let tft = components::st77xx::ST77XXComponent::new(
        mux_alarm,
        bus,
        None,
        base_peripherals
            .gpio_ports
            .get_pin(stm32f412g::gpio::PinId::PD11),
        &capsules_extra::st77xx::ST7789H2,
    )
    .finalize(components::st77xx_component_static!(
        // bus type
        capsules_extra::bus::Bus8080Bus<'static, stm32f412g::fsmc::Fsmc>,
        // timer type
        stm32f412g::tim2::Tim2,
        // pin type
        stm32f412g::gpio::Pin,
    ));

    let _ = tft.init();

    let screen = components::screen::ScreenComponent::new(
        board_kernel,
        capsules_extra::screen::DRIVER_NUM,
        tft,
        Some(tft),
    )
    .finalize(components::screen_component_static!(1024));

    let touch = components::touch::MultiTouchComponent::new(
        board_kernel,
        capsules_extra::touch::DRIVER_NUM,
        ft6x06,
        Some(ft6x06),
        Some(tft),
    )
    .finalize(components::touch_component_static!());

    touch.set_screen_rotation_offset(ScreenRotation::Rotated90);

    // Uncomment this for multi touch support
    // let touch =
    //     components::touch::MultiTouchComponent::new(board_kernel, ft6x06, Some(ft6x06), None)
    //         .finalize(());

    // ADC
    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc1)
        .finalize(components::adc_mux_component_static!(stm32f412g::adc::Adc));

    let temp_sensor = components::temperature_stm::TemperatureSTMComponent::new(
        adc_mux,
        stm32f412g::adc::Channel::Channel18,
        2.5,
        0.76,
    )
    .finalize(components::temperature_stm_adc_component_static!(
        stm32f412g::adc::Adc
    ));

    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        temp_sensor,
    )
    .finalize(components::temperature_component_static!(
        TemperatureSTMSensor
    ));

    let adc_channel_0 =
        components::adc::AdcComponent::new(adc_mux, stm32f412g::adc::Channel::Channel1)
            .finalize(components::adc_component_static!(stm32f412g::adc::Adc));

    let adc_channel_1 =
        components::adc::AdcComponent::new(adc_mux, stm32f412g::adc::Channel::Channel11)
            .finalize(components::adc_component_static!(stm32f412g::adc::Adc));

    let adc_channel_2 =
        components::adc::AdcComponent::new(adc_mux, stm32f412g::adc::Channel::Channel13)
            .finalize(components::adc_component_static!(stm32f412g::adc::Adc));

    let adc_channel_3 =
        components::adc::AdcComponent::new(adc_mux, stm32f412g::adc::Channel::Channel14)
            .finalize(components::adc_component_static!(stm32f412g::adc::Adc));

    let adc_channel_4 =
        components::adc::AdcComponent::new(adc_mux, stm32f412g::adc::Channel::Channel15)
            .finalize(components::adc_component_static!(stm32f412g::adc::Adc));

    let adc_channel_5 =
        components::adc::AdcComponent::new(adc_mux, stm32f412g::adc::Channel::Channel8)
            .finalize(components::adc_component_static!(stm32f412g::adc::Adc));

    let adc_syscall =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                adc_channel_0,
                adc_channel_1,
                adc_channel_2,
                adc_channel_3,
                adc_channel_4,
                adc_channel_5
            ));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // PROCESS CONSOLE
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        stm32f412g::tim2::Tim2
    ));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let stm32f412g = STM32F412GDiscovery {
        console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        led,
        button,
        alarm,
        gpio,
        adc: adc_syscall,
        touch,
        screen,
        temperature: temp,
        rng,

        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(
            (HSI_FREQUENCY_MHZ * 1_000_000) as u32,
        ),
    };

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);
    // base_peripherals.fsmc.write(0x04, 120);
    // debug!("id {}", base_peripherals.fsmc.read(0x05));

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
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    //Uncomment to run multi alarm test
    /*components::test::multi_alarm_test::MultiAlarmTestComponent::new(mux_alarm)
    .finalize(components::multi_alarm_test_component_buf!(stm32f412g::tim2::Tim2))
    .run();*/

    (board_kernel, stm32f412g, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
