// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for WeAct STM32F401CCU6 Core Board
//!
//! - <https://github.com/WeActTC/MiniF4-STM32F4x1>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::led::LedLow;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};

use stm32f401cc::chip_specs::Stm32f401Specs;
use stm32f401cc::clocks::hsi::HSI_FREQUENCY_MHZ;
use stm32f401cc::interrupt_service::Stm32f401ccDefaultPeripherals;

/// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None, None, None, None];

static mut CHIP: Option<&'static stm32f401cc::chip::Stm32f4xx<Stm32f401ccDefaultPeripherals>> =
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

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct WeactF401CC {
    console: &'static capsules_core::console::Console<'static>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, stm32f401cc::gpio::Pin<'static>>,
        1,
    >,
    button: &'static capsules_core::button::Button<'static, stm32f401cc::gpio::Pin<'static>>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f401cc::tim2::Tim2<'static>>,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, stm32f401cc::gpio::Pin<'static>>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for WeactF401CC {
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
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

impl KernelResources<stm32f401cc::chip::Stm32f4xx<'static, Stm32f401ccDefaultPeripherals<'static>>>
    for WeactF401CC
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
    dma: &stm32f401cc::dma::Dma1,
    dma_streams: &'static [stm32f401cc::dma::Stream<stm32f401cc::dma::Dma1>; 8],
    usart2: &'static stm32f401cc::usart::Usart<stm32f401cc::dma::Dma1>,
) {
    use stm32f401cc::dma::Dma1Peripheral;
    use stm32f401cc::usart;

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
    syscfg: &stm32f401cc::syscfg::Syscfg,
    gpio_ports: &'static stm32f401cc::gpio::GpioPorts<'static>,
) {
    use kernel::hil::gpio::Configure;
    use stm32f401cc::gpio::{AlternateFunction, Mode, PinId, PortId};

    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();

    // On-board KEY button is connected on PA0
    gpio_ports.get_pin(PinId::PA00).map(|pin| {
        pin.enable_interrupt();
    });

    // enable interrupt for D3
    gpio_ports.get_pin(PinId::PC14).map(|pin| {
        pin.enable_interrupt();
    });

    // PA2 (tx) and PA3 (rx) (USART2)
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

    gpio_ports.get_port_from_port_id(PortId::C).enable_clock();

    // On-board LED C13 is connected to PC13. Configure PC13 as `debug_gpio!(0, ...)`
    gpio_ports.get_pin(PinId::PC13).map(|pin| {
        pin.make_output();
        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    // Enable clocks for GPIO Ports
    // Ports A and C enabled above, Port B is the only other board-exposed port
    gpio_ports.get_port_from_port_id(PortId::B).enable_clock();
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals(tim2: &stm32f401cc::tim2::Tim2) {
    // USART2 IRQn is 37
    cortexm4::nvic::Nvic::new(stm32f401cc::nvic::USART2).enable();

    // TIM2 IRQn is 28
    tim2.enable_clock();
    tim2.start();
    cortexm4::nvic::Nvic::new(stm32f401cc::nvic::TIM2).enable();
}

/// Main function
///
/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    WeactF401CC,
    &'static stm32f401cc::chip::Stm32f4xx<'static, Stm32f401ccDefaultPeripherals<'static>>,
) {
    stm32f401cc::init();

    // We use the default HSI 16Mhz clock
    let rcc = static_init!(stm32f401cc::rcc::Rcc, stm32f401cc::rcc::Rcc::new());
    let clocks = static_init!(
        stm32f401cc::clocks::Clocks<Stm32f401Specs>,
        stm32f401cc::clocks::Clocks::new(rcc)
    );
    let syscfg = static_init!(
        stm32f401cc::syscfg::Syscfg,
        stm32f401cc::syscfg::Syscfg::new(clocks)
    );
    let exti = static_init!(
        stm32f401cc::exti::Exti,
        stm32f401cc::exti::Exti::new(syscfg)
    );
    let dma1 = static_init!(stm32f401cc::dma::Dma1, stm32f401cc::dma::Dma1::new(clocks));
    let dma2 = static_init!(stm32f401cc::dma::Dma2, stm32f401cc::dma::Dma2::new(clocks));

    let peripherals = static_init!(
        Stm32f401ccDefaultPeripherals,
        Stm32f401ccDefaultPeripherals::new(clocks, exti, dma1, dma2)
    );

    peripherals.init();
    let base_peripherals = &peripherals.stm32f4;

    setup_peripherals(&base_peripherals.tim2);

    set_pin_primary_functions(syscfg, &base_peripherals.gpio_ports);

    setup_dma(
        dma1,
        &base_peripherals.dma1_streams,
        &base_peripherals.usart2,
    );

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    let chip = static_init!(
        stm32f401cc::chip::Stm32f4xx<Stm32f401ccDefaultPeripherals>,
        stm32f401cc::chip::Stm32f4xx::new(peripherals)
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
    // Clock to Port A, B, C are enabled in `set_pin_primary_functions()`
    let gpio_ports = &base_peripherals.gpio_ports;

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, stm32f401cc::gpio::Pin>,
        LedLow::new(gpio_ports.get_pin(stm32f401cc::gpio::PinId::PC13).unwrap()),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32f401cc::gpio::Pin,
            (
                gpio_ports.get_pin(stm32f401cc::gpio::PinId::PA00).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_static!(stm32f401cc::gpio::Pin));

    // ALARM

    let tim2 = &base_peripherals.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_static!(stm32f401cc::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(stm32f401cc::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            stm32f401cc::gpio::Pin,
            // 2 => gpio_ports.pins[2][13].as_ref().unwrap(), // C13 (reserved for led)
            3 => gpio_ports.pins[2][14].as_ref().unwrap(), // C14
            4 => gpio_ports.pins[2][15].as_ref().unwrap(), // C15
            // 10 => gpio_ports.pins[0][0].as_ref().unwrap(), // A0 (reserved for button)
            11 => gpio_ports.pins[0][1].as_ref().unwrap(), // A1
            12 => gpio_ports.pins[0][2].as_ref().unwrap(), // A2
            13 => gpio_ports.pins[0][3].as_ref().unwrap(), // A3
            14 => gpio_ports.pins[0][4].as_ref().unwrap(), // A4
            15 => gpio_ports.pins[0][5].as_ref().unwrap(), // A5
            16 => gpio_ports.pins[0][6].as_ref().unwrap(), // A6
            17 => gpio_ports.pins[0][7].as_ref().unwrap(), // A7
            18 => gpio_ports.pins[1][0].as_ref().unwrap(), // B0
            19 => gpio_ports.pins[1][1].as_ref().unwrap(), // B1
            20 => gpio_ports.pins[1][2].as_ref().unwrap(), // B2
            21 => gpio_ports.pins[1][10].as_ref().unwrap(), // B10
            25 => gpio_ports.pins[1][12].as_ref().unwrap(), // B12
            26 => gpio_ports.pins[1][13].as_ref().unwrap(), // B13
            27 => gpio_ports.pins[1][14].as_ref().unwrap(), // B14
            28 => gpio_ports.pins[1][15].as_ref().unwrap(), // B15
            29 => gpio_ports.pins[0][8].as_ref().unwrap(), // A8
            30 => gpio_ports.pins[0][9].as_ref().unwrap(), // A9
            31 => gpio_ports.pins[0][10].as_ref().unwrap(), // A10
            32 => gpio_ports.pins[0][11].as_ref().unwrap(), // A11
            33 => gpio_ports.pins[0][12].as_ref().unwrap(), // A12
            34 => gpio_ports.pins[0][13].as_ref().unwrap(), // A13
            37 => gpio_ports.pins[0][14].as_ref().unwrap(), // A14
            38 => gpio_ports.pins[0][15].as_ref().unwrap(), // A15
            39 => gpio_ports.pins[1][3].as_ref().unwrap(), // B3
            40 => gpio_ports.pins[1][4].as_ref().unwrap(), // B4
            41 => gpio_ports.pins[1][5].as_ref().unwrap(), // B5
            42 => gpio_ports.pins[1][6].as_ref().unwrap(), // B6
            43 => gpio_ports.pins[1][7].as_ref().unwrap(), // B7
            45 => gpio_ports.pins[1][8].as_ref().unwrap(), // B8
            46 => gpio_ports.pins[1][9].as_ref().unwrap(), // B9
        ),
    )
    .finalize(components::gpio_component_static!(stm32f401cc::gpio::Pin));

    // ADC
    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc1)
        .finalize(components::adc_mux_component_static!(stm32f401cc::adc::Adc));

    let adc_channel_0 =
        components::adc::AdcComponent::new(adc_mux, stm32f401cc::adc::Channel::Channel3)
            .finalize(components::adc_component_static!(stm32f401cc::adc::Adc));

    let adc_channel_1 =
        components::adc::AdcComponent::new(adc_mux, stm32f401cc::adc::Channel::Channel10)
            .finalize(components::adc_component_static!(stm32f401cc::adc::Adc));

    let adc_channel_2 =
        components::adc::AdcComponent::new(adc_mux, stm32f401cc::adc::Channel::Channel13)
            .finalize(components::adc_component_static!(stm32f401cc::adc::Adc));

    let adc_channel_3 =
        components::adc::AdcComponent::new(adc_mux, stm32f401cc::adc::Channel::Channel9)
            .finalize(components::adc_component_static!(stm32f401cc::adc::Adc));

    let adc_channel_4 =
        components::adc::AdcComponent::new(adc_mux, stm32f401cc::adc::Channel::Channel15)
            .finalize(components::adc_component_static!(stm32f401cc::adc::Adc));

    let adc_channel_5 =
        components::adc::AdcComponent::new(adc_mux, stm32f401cc::adc::Channel::Channel8)
            .finalize(components::adc_component_static!(stm32f401cc::adc::Adc));

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
        stm32f401cc::tim2::Tim2
    ));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let weact_f401cc = WeactF401CC {
        console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        adc: adc_syscall,
        led,
        button,
        alarm,
        gpio,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(
            (HSI_FREQUENCY_MHZ * 1_000_000) as u32,
        ),
    };

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
    .finalize(components::multi_alarm_test_component_buf!(stm32f401cc::tim2::Tim2))
    .run();*/

    (board_kernel, weact_f401cc, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
