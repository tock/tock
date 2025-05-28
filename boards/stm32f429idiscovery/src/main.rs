// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for STM32F429I Discovery development board
//!
//! - <https://www.st.com/en/evaluation-tools/32f429idiscovery.html>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};

use stm32f429zi::chip_specs::Stm32f429Specs;
use stm32f429zi::clocks::hsi::HSI_FREQUENCY_MHZ;
use stm32f429zi::gpio::{AlternateFunction, Mode, PinId, PortId};
use stm32f429zi::interrupt_service::Stm32f429ziDefaultPeripherals;

/// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None, None, None, None];

static mut CHIP: Option<&'static stm32f429zi::chip::Stm32f4xx<Stm32f429ziDefaultPeripherals>> =
    None;
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
    capsules_core::virtualizers::virtual_adc::AdcDevice<'static, stm32f429zi::adc::Adc<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<TemperatureSTMSensor>;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct STM32F429IDiscovery {
    console: &'static capsules_core::console::Console<'static>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, stm32f429zi::gpio::Pin<'static>>,
        4,
    >,
    button: &'static capsules_core::button::Button<'static, stm32f429zi::gpio::Pin<'static>>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f429zi::tim2::Tim2<'static>>,
    >,
    temperature: &'static TemperatureDriver,
    gpio: &'static capsules_core::gpio::GPIO<'static, stm32f429zi::gpio::Pin<'static>>,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for STM32F429IDiscovery {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

impl
    KernelResources<
        stm32f429zi::chip::Stm32f4xx<
            'static,
            stm32f429zi::interrupt_service::Stm32f429ziDefaultPeripherals<'static>,
        >,
    > for STM32F429IDiscovery
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
    dma: &stm32f429zi::dma::Dma2,
    dma_streams: &'static [stm32f429zi::dma::Stream<'static, stm32f429zi::dma::Dma2>; 8],
    usart1: &'static stm32f429zi::usart::Usart<stm32f429zi::dma::Dma2>,
) {
    use stm32f429zi::dma::Dma2Peripheral;
    use stm32f429zi::usart;

    dma.enable_clock();

    let usart1_tx_stream = &dma_streams[Dma2Peripheral::USART1_TX.get_stream_idx()];
    let usart1_rx_stream = &dma_streams[Dma2Peripheral::USART1_RX.get_stream_idx()];

    usart1.set_dma(
        usart::TxDMA(usart1_tx_stream),
        usart::RxDMA(usart1_rx_stream),
    );

    usart1_tx_stream.set_client(usart1);
    usart1_rx_stream.set_client(usart1);

    usart1_tx_stream.setup(Dma2Peripheral::USART1_TX);
    usart1_rx_stream.setup(Dma2Peripheral::USART1_RX);

    cortexm4::nvic::Nvic::new(Dma2Peripheral::USART1_TX.get_stream_irqn()).enable();
    cortexm4::nvic::Nvic::new(Dma2Peripheral::USART1_RX.get_stream_irqn()).enable();
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions(
    syscfg: &stm32f429zi::syscfg::Syscfg,
    gpio_ports: &'static stm32f429zi::gpio::GpioPorts<'static>,
) {
    use kernel::hil::gpio::Configure;

    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::G).enable_clock();

    // User LD4 (red) is connected to PG14. Configure PG14 as `debug_gpio!(0, ...)`
    gpio_ports.get_pin(PinId::PG14).map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();

    // Configure USART1 on Pins PA09 and PA10.
    // USART1 is connected to ST-LINK virtual COM port on Rev.1 of the Stm32f429i Discovery board
    gpio_ports.get_pin(PinId::PA09).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    gpio_ports.get_pin(PinId::PA10).map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    // User button B1 is connected on pa00
    gpio_ports.get_pin(PinId::PA00).map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        pin.enable_interrupt();
    });
    // EXTI0 interrupts is delivered at IRQn 6 (EXTI0)
    cortexm4::nvic::Nvic::new(stm32f429zi::nvic::EXTI0).enable(); // TODO check if this is still necessary!

    // Enable clocks for GPIO Ports
    // Disable some of them if you don't need some of the GPIOs
    // Ports A, and B are already enabled
    //           A: already enabled
    gpio_ports.get_port_from_port_id(PortId::B).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::C).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::D).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::E).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::F).enable_clock();
    //           G: already enabled
    gpio_ports.get_port_from_port_id(PortId::H).enable_clock();

    // Arduino A0
    gpio_ports.get_pin(PinId::PA03).map(|pin| {
        pin.set_mode(stm32f429zi::gpio::Mode::AnalogMode);
    });

    // Arduino A1
    gpio_ports.get_pin(PinId::PC00).map(|pin| {
        pin.set_mode(stm32f429zi::gpio::Mode::AnalogMode);
    });

    // Arduino A2
    gpio_ports.get_pin(PinId::PC03).map(|pin| {
        pin.set_mode(stm32f429zi::gpio::Mode::AnalogMode);
    });

    // Arduino A3
    gpio_ports.get_pin(PinId::PF03).map(|pin| {
        pin.set_mode(stm32f429zi::gpio::Mode::AnalogMode);
    });

    // Arduino A4
    gpio_ports.get_pin(PinId::PF05).map(|pin| {
        pin.set_mode(stm32f429zi::gpio::Mode::AnalogMode);
    });

    // Arduino A5
    gpio_ports.get_pin(PinId::PF10).map(|pin| {
        pin.set_mode(stm32f429zi::gpio::Mode::AnalogMode);
    });
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals(tim2: &stm32f429zi::tim2::Tim2) {
    // USART1 IRQn is 37
    cortexm4::nvic::Nvic::new(stm32f429zi::nvic::USART1).enable();

    // TIM2 IRQn is 28
    tim2.enable_clock();
    tim2.start();
    cortexm4::nvic::Nvic::new(stm32f429zi::nvic::TIM2).enable();
}

/// Main function
///
/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    STM32F429IDiscovery,
    &'static stm32f429zi::chip::Stm32f4xx<'static, Stm32f429ziDefaultPeripherals<'static>>,
) {
    stm32f429zi::init();

    // We use the default HSI 16Mhz clock
    let rcc = static_init!(stm32f429zi::rcc::Rcc, stm32f429zi::rcc::Rcc::new());
    let clocks = static_init!(
        stm32f429zi::clocks::Clocks<Stm32f429Specs>,
        stm32f429zi::clocks::Clocks::new(rcc)
    );
    let syscfg = static_init!(
        stm32f429zi::syscfg::Syscfg,
        stm32f429zi::syscfg::Syscfg::new(clocks)
    );
    let exti = static_init!(
        stm32f429zi::exti::Exti,
        stm32f429zi::exti::Exti::new(syscfg)
    );
    let dma1 = static_init!(stm32f429zi::dma::Dma1, stm32f429zi::dma::Dma1::new(clocks));
    let dma2 = static_init!(stm32f429zi::dma::Dma2, stm32f429zi::dma::Dma2::new(clocks));
    let peripherals = static_init!(
        Stm32f429ziDefaultPeripherals,
        Stm32f429ziDefaultPeripherals::new(clocks, exti, dma1, dma2)
    );

    peripherals.init();
    let base_peripherals = &peripherals.stm32f4;

    setup_peripherals(&base_peripherals.tim2);

    set_pin_primary_functions(syscfg, &base_peripherals.gpio_ports);

    setup_dma(
        dma2,
        &base_peripherals.dma2_streams,
        &base_peripherals.usart1,
    );

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    let chip = static_init!(
        stm32f429zi::chip::Stm32f4xx<Stm32f429ziDefaultPeripherals>,
        stm32f429zi::chip::Stm32f4xx::new(peripherals)
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    // USART1 is only connected to the ST-LINK port in the DISC1 revision of
    // the STM32F429I boards, DISC0 does not have this connection and will
    // not have USART output available!
    base_peripherals.usart1.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(&base_peripherals.usart1, 115200)
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

    // Clock to all GPIO Ports is enabled in `set_pin_primary_functions()`
    let gpio_ports = &base_peripherals.gpio_ports;

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, stm32f429zi::gpio::Pin>,
        LedHigh::new(gpio_ports.get_pin(stm32f429zi::gpio::PinId::PG13).unwrap()),
        LedHigh::new(gpio_ports.get_pin(stm32f429zi::gpio::PinId::PG14).unwrap()),
        LedHigh::new(gpio_ports.get_pin(stm32f429zi::gpio::PinId::PB13).unwrap()),
        LedHigh::new(gpio_ports.get_pin(stm32f429zi::gpio::PinId::PC05).unwrap()),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32f429zi::gpio::Pin,
            (
                gpio_ports.get_pin(stm32f429zi::gpio::PinId::PA00).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_static!(stm32f429zi::gpio::Pin));

    // ALARM

    let tim2 = &base_peripherals.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_static!(stm32f429zi::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(stm32f429zi::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            stm32f429zi::gpio::Pin,
            // Arduino like RX/TX
            0 => gpio_ports.get_pin(PinId::PG09).unwrap(), //D0
            1 => gpio_ports.pins[6][14].as_ref().unwrap(), //D1
            2 => gpio_ports.pins[5][15].as_ref().unwrap(), //D2
            3 => gpio_ports.pins[4][13].as_ref().unwrap(), //D3
            4 => gpio_ports.pins[5][14].as_ref().unwrap(), //D4
            5 => gpio_ports.pins[4][11].as_ref().unwrap(), //D5
            6 => gpio_ports.pins[4][9].as_ref().unwrap(), //D6
            7 => gpio_ports.pins[5][13].as_ref().unwrap(), //D7
            8 => gpio_ports.pins[5][12].as_ref().unwrap(), //D8
            9 => gpio_ports.pins[3][15].as_ref().unwrap(), //D9
            // SPI Pins
            10 => gpio_ports.pins[3][14].as_ref().unwrap(), //D10
            11 => gpio_ports.pins[0][7].as_ref().unwrap(),  //D11
            12 => gpio_ports.pins[0][6].as_ref().unwrap(),  //D12
            13 => gpio_ports.pins[0][5].as_ref().unwrap(),  //D13
            // I2C Pins
            14 => gpio_ports.pins[1][9].as_ref().unwrap(), //D14
            15 => gpio_ports.pins[1][8].as_ref().unwrap(), //D15
            16 => gpio_ports.pins[2][6].as_ref().unwrap(), //D16
            17 => gpio_ports.pins[1][15].as_ref().unwrap(), //D17
            18 => gpio_ports.pins[1][13].as_ref().unwrap(), //D18
            19 => gpio_ports.pins[1][12].as_ref().unwrap(), //D19
            20 => gpio_ports.pins[0][15].as_ref().unwrap(), //D20
            21 => gpio_ports.pins[2][7].as_ref().unwrap(), //D21
            // SPI B Pins
            // 22 => gpio_ports.pins[1][5].as_ref().unwrap(), //D22
            // 23 => gpio_ports.pins[1][3].as_ref().unwrap(), //D23
            // 24 => gpio_ports.pins[0][4].as_ref().unwrap(), //D24
            // 24 => gpio_ports.pins[1][4].as_ref().unwrap(), //D25
            // QSPI
            26 => gpio_ports.pins[1][6].as_ref().unwrap(), //D26
            27 => gpio_ports.pins[1][2].as_ref().unwrap(), //D27
            28 => gpio_ports.pins[3][13].as_ref().unwrap(), //D28
            29 => gpio_ports.pins[3][12].as_ref().unwrap(), //D29
            30 => gpio_ports.pins[3][11].as_ref().unwrap(), //D30
            31 => gpio_ports.pins[4][2].as_ref().unwrap(), //D31
            // Timer Pins
            // PA00 (or PIN[0][0]) is used for the button component so cannot
            // be used for this component as well, otherwise interrupts will
            // not reach the button component.
            // 32 => stm32f429zi::gpio::PIN[0][0].as_ref().unwrap(), //D32
            33 => gpio_ports.pins[1][0].as_ref().unwrap(), //D33
            34 => gpio_ports.pins[4][0].as_ref().unwrap(), //D34
            35 => gpio_ports.pins[1][11].as_ref().unwrap(), //D35
            36 => gpio_ports.pins[1][10].as_ref().unwrap(), //D36
            37 => gpio_ports.pins[4][15].as_ref().unwrap(), //D37
            38 => gpio_ports.pins[4][14].as_ref().unwrap(), //D38
            39 => gpio_ports.pins[4][12].as_ref().unwrap(), //D39
            40 => gpio_ports.pins[4][10].as_ref().unwrap(), //D40
            41 => gpio_ports.pins[4][7].as_ref().unwrap(), //D41
            42 => gpio_ports.pins[4][8].as_ref().unwrap(), //D42
            // SDMMC
            43 => gpio_ports.pins[2][8].as_ref().unwrap(), //D43
            44 => gpio_ports.pins[2][9].as_ref().unwrap(), //D44
            45 => gpio_ports.pins[2][10].as_ref().unwrap(), //D45
            46 => gpio_ports.pins[2][11].as_ref().unwrap(), //D46
            47 => gpio_ports.pins[2][12].as_ref().unwrap(), //D47
            48 => gpio_ports.pins[3][2].as_ref().unwrap(), //D48
            49 => gpio_ports.pins[6][2].as_ref().unwrap(), //D49
            50 => gpio_ports.pins[6][3].as_ref().unwrap(), //D50
            // USART
            51 => gpio_ports.pins[3][7].as_ref().unwrap(), //D51
            52 => gpio_ports.pins[3][6].as_ref().unwrap(), //D52
            53 => gpio_ports.pins[3][5].as_ref().unwrap(), //D53
            54 => gpio_ports.pins[3][4].as_ref().unwrap(), //D54
            55 => gpio_ports.pins[3][3].as_ref().unwrap(), //D55
            56 => gpio_ports.pins[4][2].as_ref().unwrap(), //D56
            57 => gpio_ports.pins[4][4].as_ref().unwrap(), //D57
            58 => gpio_ports.pins[4][5].as_ref().unwrap(), //D58
            59 => gpio_ports.pins[4][6].as_ref().unwrap(), //D59
            60 => gpio_ports.pins[4][3].as_ref().unwrap(), //D60
            61 => gpio_ports.pins[5][8].as_ref().unwrap(), //D61
            62 => gpio_ports.pins[5][7].as_ref().unwrap(), //D62
            63 => gpio_ports.pins[5][9].as_ref().unwrap(), //D63
            64 => gpio_ports.pins[6][1].as_ref().unwrap(), //D64
            65 => gpio_ports.pins[6][0].as_ref().unwrap(), //D65
            66 => gpio_ports.pins[3][1].as_ref().unwrap(), //D66
            67 => gpio_ports.pins[3][0].as_ref().unwrap(), //D67
            68 => gpio_ports.pins[5][0].as_ref().unwrap(), //D68
            69 => gpio_ports.pins[5][1].as_ref().unwrap(), //D69
            70 => gpio_ports.pins[5][2].as_ref().unwrap(), //D70
            71 => gpio_ports.pins[0][7].as_ref().unwrap()  //D71

            // ADC Pins
            // Enable the to use the ADC pins as GPIO
            // 72 => gpio_ports.pins[0][3].as_ref().unwrap(), //A0
            // 73 => gpio_ports.pins[2][0].as_ref().unwrap(), //A1
            // 74 gpio_ports.pins::PIN[2][3].as_ref().unwrap(), //A2
            // 75 gpio_ports.pins::PIN[5][3].as_ref().unwrap(), //A3
            // 76 gpio_ports.pins::PIN[5][5].as_ref().unwrap(), //A4
            // 77 gpio_ports.pins::PIN[5][10].as_ref().unwrap(), //A5
            // 78 gpio_ports.pins::PIN[1][1].as_ref().unwrap(), //A6
            // 79 gpio_ports.pins::PIN[2][2].as_ref().unwrap(), //A7
            // 80 gpio_ports.pins::PIN[5][4].as_ref().unwrap()  //A8
        ),
    )
    .finalize(components::gpio_component_static!(stm32f429zi::gpio::Pin));

    // ADC
    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc1)
        .finalize(components::adc_mux_component_static!(stm32f429zi::adc::Adc));

    let temp_sensor = components::temperature_stm::TemperatureSTMComponent::new(
        adc_mux,
        stm32f429zi::adc::Channel::Channel18,
        2.5,
        0.76,
    )
    .finalize(components::temperature_stm_adc_component_static!(
        stm32f429zi::adc::Adc
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
        components::adc::AdcComponent::new(adc_mux, stm32f429zi::adc::Channel::Channel3)
            .finalize(components::adc_component_static!(stm32f429zi::adc::Adc));

    let adc_channel_1 =
        components::adc::AdcComponent::new(adc_mux, stm32f429zi::adc::Channel::Channel10)
            .finalize(components::adc_component_static!(stm32f429zi::adc::Adc));

    let adc_channel_2 =
        components::adc::AdcComponent::new(adc_mux, stm32f429zi::adc::Channel::Channel13)
            .finalize(components::adc_component_static!(stm32f429zi::adc::Adc));

    let adc_channel_3 =
        components::adc::AdcComponent::new(adc_mux, stm32f429zi::adc::Channel::Channel9)
            .finalize(components::adc_component_static!(stm32f429zi::adc::Adc));

    let adc_channel_4 =
        components::adc::AdcComponent::new(adc_mux, stm32f429zi::adc::Channel::Channel15)
            .finalize(components::adc_component_static!(stm32f429zi::adc::Adc));

    let adc_channel_5 =
        components::adc::AdcComponent::new(adc_mux, stm32f429zi::adc::Channel::Channel8)
            .finalize(components::adc_component_static!(stm32f429zi::adc::Adc));

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
        stm32f429zi::tim2::Tim2
    ));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let stm32f429i_discovery = STM32F429IDiscovery {
        console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        adc: adc_syscall,
        led,
        temperature: temp,
        button,
        alarm,
        gpio,

        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(
            (HSI_FREQUENCY_MHZ * 1_000_000) as u32,
        ),
    };

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

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
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    //Uncomment to run multi alarm test
    /*components::test::multi_alarm_test::MultiAlarmTestComponent::new(mux_alarm)
    .finalize(components::multi_alarm_test_component_buf!(stm32f429zi::tim2::Tim2))
    .run();*/

    (board_kernel, stm32f429i_discovery, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
