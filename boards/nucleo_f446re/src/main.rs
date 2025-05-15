// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for Nucleo-F446RE development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f446re.html>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};
use stm32f446re::chip_specs::Stm32f446Specs;
use stm32f446re::clocks::hsi::HSI_FREQUENCY_MHZ;
use stm32f446re::gpio::{AlternateFunction, Mode, PinId, PortId};
use stm32f446re::interrupt_service::Stm32f446reDefaultPeripherals;

/// Support routines for debugging I/O.
pub mod io;

// Unit Tests for drivers.
#[allow(dead_code)]
mod virtual_uart_rx_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None, None, None, None];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static stm32f446re::chip::Stm32f4xx<Stm32f446reDefaultPeripherals>> =
    None;
// Static reference to process printer for panic dumps.
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
    capsules_core::virtualizers::virtual_adc::AdcDevice<'static, stm32f446re::adc::Adc<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<TemperatureSTMSensor>;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NucleoF446RE {
    console: &'static capsules_core::console::Console<'static>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, stm32f446re::gpio::Pin<'static>>,
        1,
    >,
    button: &'static capsules_core::button::Button<'static, stm32f446re::gpio::Pin<'static>>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f446re::tim2::Tim2<'static>>,
    >,

    temperature: &'static TemperatureDriver,
    gpio: &'static capsules_core::gpio::GPIO<'static, stm32f446re::gpio::Pin<'static>>,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for NucleoF446RE {
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
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl
    KernelResources<
        stm32f446re::chip::Stm32f4xx<
            'static,
            stm32f446re::interrupt_service::Stm32f446reDefaultPeripherals<'static>,
        >,
    > for NucleoF446RE
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
    dma: &stm32f446re::dma::Dma1,
    dma_streams: &'static [stm32f446re::dma::Stream<stm32f446re::dma::Dma1>; 8],
    usart2: &'static stm32f446re::usart::Usart<stm32f446re::dma::Dma1>,
) {
    use stm32f446re::dma::Dma1Peripheral;
    use stm32f446re::usart;

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
    syscfg: &stm32f446re::syscfg::Syscfg,
    gpio_ports: &'static stm32f446re::gpio::GpioPorts<'static>,
) {
    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::B).enable_clock();

    // User LD2 is connected to PA05. Configure PA05 as `debug_gpio!(0, ...)`
    gpio_ports.get_pin(PinId::PA05).map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

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

    gpio_ports.get_port_from_port_id(PortId::C).enable_clock();

    // button is connected on pc13
    gpio_ports.get_pin(PinId::PC13).map(|pin| {
        pin.enable_interrupt();
    });

    // enable interrupt for gpio 2
    gpio_ports.get_pin(PinId::PA10).map(|pin| {
        pin.enable_interrupt();
    });

    // Arduino A0
    gpio_ports.get_pin(PinId::PA00).map(|pin| {
        pin.set_mode(stm32f446re::gpio::Mode::AnalogMode);
    });

    // Arduino A1
    gpio_ports.get_pin(PinId::PA01).map(|pin| {
        pin.set_mode(stm32f446re::gpio::Mode::AnalogMode);
    });

    // Arduino A2
    gpio_ports.get_pin(PinId::PA04).map(|pin| {
        pin.set_mode(stm32f446re::gpio::Mode::AnalogMode);
    });

    // Arduino A3
    gpio_ports.get_pin(PinId::PB00).map(|pin| {
        pin.set_mode(stm32f446re::gpio::Mode::AnalogMode);
    });

    // Arduino A4
    gpio_ports.get_pin(PinId::PC01).map(|pin| {
        pin.set_mode(stm32f446re::gpio::Mode::AnalogMode);
    });

    // Arduino A5
    gpio_ports.get_pin(PinId::PC00).map(|pin| {
        pin.set_mode(stm32f446re::gpio::Mode::AnalogMode);
    });
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals(tim2: &stm32f446re::tim2::Tim2) {
    // USART2 IRQn is 38
    cortexm4::nvic::Nvic::new(stm32f446re::nvic::USART2).enable();

    // TIM2 IRQn is 28
    tim2.enable_clock();
    tim2.start();
    cortexm4::nvic::Nvic::new(stm32f446re::nvic::TIM2).enable();
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    NucleoF446RE,
    &'static stm32f446re::chip::Stm32f4xx<'static, Stm32f446reDefaultPeripherals<'static>>,
) {
    stm32f446re::init();

    // We use the default HSI 16Mhz clock
    let rcc = static_init!(stm32f446re::rcc::Rcc, stm32f446re::rcc::Rcc::new());
    let clocks = static_init!(
        stm32f446re::clocks::Clocks<Stm32f446Specs>,
        stm32f446re::clocks::Clocks::new(rcc)
    );

    let syscfg = static_init!(
        stm32f446re::syscfg::Syscfg,
        stm32f446re::syscfg::Syscfg::new(clocks)
    );
    let exti = static_init!(
        stm32f446re::exti::Exti,
        stm32f446re::exti::Exti::new(syscfg)
    );
    let dma1 = static_init!(stm32f446re::dma::Dma1, stm32f446re::dma::Dma1::new(clocks));
    let dma2 = static_init!(stm32f446re::dma::Dma2, stm32f446re::dma::Dma2::new(clocks));

    let peripherals = static_init!(
        Stm32f446reDefaultPeripherals,
        Stm32f446reDefaultPeripherals::new(clocks, exti, dma1, dma2)
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
        stm32f446re::chip::Stm32f4xx<Stm32f446reDefaultPeripherals>,
        stm32f446re::chip::Stm32f4xx::new(peripherals)
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    base_peripherals.usart2.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(&base_peripherals.usart2, 115200)
        .finalize(components::uart_mux_component_static!());

    // `finalize()` configures the underlying USART, so we need to
    // tell `send_byte()` not to configure the USART again.
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
    let gpio_ports = &base_peripherals.gpio_ports;

    // Clock to Port A is enabled in `set_pin_primary_functions()`
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, stm32f446re::gpio::Pin>,
        LedHigh::new(gpio_ports.get_pin(stm32f446re::gpio::PinId::PA05).unwrap()),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32f446re::gpio::Pin,
            (
                gpio_ports.get_pin(stm32f446re::gpio::PinId::PC13).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_static!(stm32f446re::gpio::Pin));

    // ALARM
    let tim2 = &base_peripherals.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_static!(stm32f446re::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(stm32f446re::tim2::Tim2));

    // ADC
    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc1)
        .finalize(components::adc_mux_component_static!(stm32f446re::adc::Adc));

    let temp_sensor = components::temperature_stm::TemperatureSTMComponent::new(
        adc_mux,
        stm32f446re::adc::Channel::Channel18,
        2.5,
        0.76,
    )
    .finalize(components::temperature_stm_adc_component_static!(
        stm32f446re::adc::Adc
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
        components::adc::AdcComponent::new(adc_mux, stm32f446re::adc::Channel::Channel0)
            .finalize(components::adc_component_static!(stm32f446re::adc::Adc));

    let adc_channel_1 =
        components::adc::AdcComponent::new(adc_mux, stm32f446re::adc::Channel::Channel1)
            .finalize(components::adc_component_static!(stm32f446re::adc::Adc));

    let adc_channel_2 =
        components::adc::AdcComponent::new(adc_mux, stm32f446re::adc::Channel::Channel4)
            .finalize(components::adc_component_static!(stm32f446re::adc::Adc));

    let adc_channel_3 =
        components::adc::AdcComponent::new(adc_mux, stm32f446re::adc::Channel::Channel8)
            .finalize(components::adc_component_static!(stm32f446re::adc::Adc));

    let adc_channel_4 =
        components::adc::AdcComponent::new(adc_mux, stm32f446re::adc::Channel::Channel11)
            .finalize(components::adc_component_static!(stm32f446re::adc::Adc));

    let adc_channel_5 =
        components::adc::AdcComponent::new(adc_mux, stm32f446re::adc::Channel::Channel10)
            .finalize(components::adc_component_static!(stm32f446re::adc::Adc));

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

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            stm32f446re::gpio::Pin,
            // Arduino like RX/TX
            // 0 => gpio_ports.get_pin(PinId::PA03).unwrap(), //D0
            // 1 => gpio_ports.get_pin(PinId::PA02).unwrap(), //D1
            2 => gpio_ports.get_pin(PinId::PA10).unwrap(), //D2
            3 => gpio_ports.get_pin(PinId::PB03).unwrap(), //D3
            4 => gpio_ports.get_pin(PinId::PB05).unwrap(), //D4
            5 => gpio_ports.get_pin(PinId::PB04).unwrap(), //D5
            6 => gpio_ports.get_pin(PinId::PB10).unwrap(), //D6
            7 => gpio_ports.get_pin(PinId::PA08).unwrap(), //D7
            8 => gpio_ports.get_pin(PinId::PA09).unwrap(), //D8
            9 => gpio_ports.get_pin(PinId::PC07).unwrap(), //D9
            10 => gpio_ports.get_pin(PinId::PB06).unwrap(), //D10
            11 => gpio_ports.get_pin(PinId::PA07).unwrap(),  //D11
            12 => gpio_ports.get_pin(PinId::PA06).unwrap(),  //D12
            13 => gpio_ports.get_pin(PinId::PA05).unwrap(),  //D13
            14 => gpio_ports.get_pin(PinId::PB09).unwrap(), //D14
            15 => gpio_ports.get_pin(PinId::PB08).unwrap(), //D15

            // ADC Pins
            // Enable the to use the ADC pins as GPIO
            // 16 => gpio_ports.get_pin(PinId::PA00).unwrap(), //A0
            // 17 => gpio_ports.get_pin(PinId::PA01).unwrap(), //A1
            // 18 => gpio_ports.get_pin(PinId::PA04).unwrap(), //A2
            // 19 => gpio_ports.get_pin(PinId::PB00).unwrap(), //A3
            // 20 => gpio_ports.get_pin(PinId::PC01).unwrap(), //A4
            // 21 => gpio_ports.get_pin(PinId::PC00).unwrap(), //A5
        ),
    )
    .finalize(components::gpio_component_static!(stm32f446re::gpio::Pin));

    // PROCESS CONSOLE
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        stm32f446re::tim2::Tim2
    ));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let nucleo_f446re = NucleoF446RE {
        console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        led,
        button,
        adc: adc_syscall,
        alarm,

        temperature: temp,
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
    .finalize(components::multi_alarm_test_component_buf!(stm32f446re::tim2::Tim2))
    .run();*/

    (board_kernel, nucleo_f446re, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
