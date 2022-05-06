//! Board file for Nucleo-F446RE development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f446re.html>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::hil::gpio::Configure;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};
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
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NucleoF446RE {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC<NUM_PROCS>,
    led: &'static capsules::led::LedDriver<
        'static,
        LedHigh<'static, stm32f446re::gpio::Pin<'static>>,
        1,
    >,
    button: &'static capsules::button::Button<'static, stm32f446re::gpio::Pin<'static>>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f446re::tim2::Tim2<'static>>,
    >,

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
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
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
        &self
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
    use stm32f446re::gpio::{AlternateFunction, Mode, PinId, PortId};

    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();

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

/// Statically initialize the core peripherals for the chip.
///
/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn get_peripherals() -> (
    &'static mut Stm32f446reDefaultPeripherals<'static>,
    &'static stm32f446re::syscfg::Syscfg<'static>,
    &'static stm32f446re::dma::Dma1<'static>,
) {
    // We use the default HSI 16Mhz clock
    let rcc = static_init!(stm32f446re::rcc::Rcc, stm32f446re::rcc::Rcc::new());
    let syscfg = static_init!(
        stm32f446re::syscfg::Syscfg,
        stm32f446re::syscfg::Syscfg::new(rcc)
    );
    let exti = static_init!(
        stm32f446re::exti::Exti,
        stm32f446re::exti::Exti::new(syscfg)
    );
    let dma1 = static_init!(stm32f446re::dma::Dma1, stm32f446re::dma::Dma1::new(rcc));
    let dma2 = static_init!(stm32f446re::dma::Dma2, stm32f446re::dma::Dma2::new(rcc));

    let peripherals = static_init!(
        Stm32f446reDefaultPeripherals,
        Stm32f446reDefaultPeripherals::new(rcc, exti, dma1, dma2)
    );
    (peripherals, syscfg, dma1)
}

/// Main function.
///
/// This is called after RAM initialization is complete.
#[no_mangle]
pub unsafe fn main() {
    stm32f446re::init();

    let (peripherals, syscfg, dma1) = get_peripherals();
    peripherals.init();
    let base_peripherals = &peripherals.stm32f4;

    setup_peripherals(&base_peripherals.tim2);

    set_pin_primary_functions(syscfg, &base_peripherals.gpio_ports);

    setup_dma(
        dma1,
        &base_peripherals.dma1_streams,
        &base_peripherals.usart2,
    );

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let chip = static_init!(
        stm32f446re::chip::Stm32f4xx<Stm32f446reDefaultPeripherals>,
        stm32f446re::chip::Stm32f4xx::new(peripherals)
    );
    CHIP = Some(chip);

    // UART

    // Create a shared UART channel for kernel debug.
    base_peripherals.usart2.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &base_peripherals.usart2,
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
        capsules::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_helper!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // LEDs
    let gpio_ports = &base_peripherals.gpio_ports;

    // Clock to Port A is enabled in `set_pin_primary_functions()`
    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
        LedHigh<'static, stm32f446re::gpio::Pin>,
        LedHigh::new(gpio_ports.get_pin(stm32f446re::gpio::PinId::PA05).unwrap()),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32f446re::gpio::Pin,
            (
                gpio_ports.get_pin(stm32f446re::gpio::PinId::PC13).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_buf!(stm32f446re::gpio::Pin));

    // ALARM
    let tim2 = &base_peripherals.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f446re::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_helper!(stm32f446re::tim2::Tim2));

    let process_printer =
        components::process_printer::ProcessPrinterTextComponent::new().finalize(());
    PROCESS_PRINTER = Some(process_printer);

    // PROCESS CONSOLE
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
    )
    .finalize(components::process_console_component_helper!(
        stm32f446re::tim2::Tim2
    ));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    let nucleo_f446re = NucleoF446RE {
        console: console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        led: led,
        button: button,
        alarm: alarm,

        scheduler,
        systick: cortexm4::systick::SysTick::new(),
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
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
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
    board_kernel.kernel_loop(
        &nucleo_f446re,
        chip,
        Some(&nucleo_f446re.ipc),
        &main_loop_capability,
    );
}
