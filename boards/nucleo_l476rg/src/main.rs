// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Board file for Nucleo-L476RG development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-l476rg.html>

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::addr_of_mut;

use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::process::ProcessArray;
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, debug, static_init};
use stm32l476rg::chip_specs::Stm32l476Specs;
use stm32l476rg::clocks::msi::MSI_FREQUENCY_MHZ;
use stm32l476rg::gpio::{AlternateFunction, Mode, PinId, PortId};
use stm32l476rg::interrupt_service::Stm32l476rgDefaultPeripherals;

/// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

type ChipHw = stm32l476rg::chip::Stm32l4xx<'static, Stm32l476rgDefaultPeripherals<'static>>;

/// Static variables used by io.rs.
static mut PROCESSES: Option<&'static ProcessArray<NUM_PROCS>> = None;

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static ChipHw> = None;
// Static reference to process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

kernel::stack_size! {0x2000}

struct Nucleol476RG {
    console: &'static capsules_core::console::Console<'static>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, stm32l476rg::gpio::Pin<'static>>,
        1,
    >,
    button: &'static capsules_core::button::Button<'static, stm32l476rg::gpio::Pin<'static>>,
    gpio: &'static capsules_core::gpio::GPIO<'static, stm32l476rg::gpio::Pin<'static>>,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for Nucleol476RG {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl
    KernelResources<
        stm32l476rg::chip::Stm32l4xx<
            'static,
            stm32l476rg::interrupt_service::Stm32l476rgDefaultPeripherals<'static>,
        >,
    > for Nucleol476RG
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

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions(
    syscfg: &stm32l476rg::syscfg::Syscfg,
    gpio_ports: &'static stm32l476rg::gpio::GpioPorts<'static>,
) {
    syscfg.enable_clock();

    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();
    gpio_ports.get_port_from_port_id(PortId::B).enable_clock();

    // User LD2 is connected to PA05. Configure PA05 as `debug_gpio!(0, ...)`
    gpio_ports.get_pin(PinId::PA05).map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        let debug_gpios = static_init!([&'static dyn kernel::hil::gpio::Pin; 1], [pin]);
        kernel::debug::initialize_debug_gpio::<
            <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
        >();
        kernel::debug::assign_gpios(debug_gpios);
    });

    // PA2 and PA3 (USART2) is connected to ST-LINK virtual COM port
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

    // button is connected on PC13
    gpio_ports.get_pin(PinId::PC13).map(|pin| {
        pin.enable_interrupt();
    });
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    Nucleol476RG,
    &'static stm32l476rg::chip::Stm32l4xx<'static, Stm32l476rgDefaultPeripherals<'static>>,
) {
    stm32l476rg::init();

    // We use the default HSI 16Mhz clock
    let rcc = static_init!(stm32l476rg::rcc::Rcc, stm32l476rg::rcc::Rcc::new());
    let clocks = static_init!(
        stm32l476rg::clocks::Clocks<Stm32l476Specs>,
        stm32l476rg::clocks::Clocks::new(rcc)
    );

    let syscfg = static_init!(
        stm32l476rg::syscfg::Syscfg,
        stm32l476rg::syscfg::Syscfg::new(clocks)
    );
    let exti = static_init!(
        stm32l476rg::exti::Exti,
        stm32l476rg::exti::Exti::new(syscfg)
    );

    let peripherals = static_init!(
        Stm32l476rgDefaultPeripherals,
        Stm32l476rgDefaultPeripherals::new(clocks, exti)
    );
    peripherals.init();
    let base_peripherals = &peripherals.stm32l4;

    set_pin_primary_functions(syscfg, &base_peripherals.gpio_ports);

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PROCESSES = Some(processes);

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let chip = static_init!(
        stm32l476rg::chip::Stm32l4xx<Stm32l476rgDefaultPeripherals>,
        stm32l476rg::chip::Stm32l4xx::new(peripherals)
    );
    CHIP = Some(chip);

    // UART

    cortexm4::nvic::Nvic::new(stm32l476rg::nvic::USART2).enable();
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
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // LEDs
    let gpio_ports = &base_peripherals.gpio_ports;

    // Clock to Port A is enabled in `set_pin_primary_functions()`
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, stm32l476rg::gpio::Pin>,
        LedHigh::new(gpio_ports.get_pin(stm32l476rg::gpio::PinId::PA05).unwrap()),
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32l476rg::gpio::Pin,
            (
                gpio_ports.get_pin(stm32l476rg::gpio::PinId::PC13).unwrap(),
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_static!(stm32l476rg::gpio::Pin));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // GPIO
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            stm32l476rg::gpio::Pin,
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
        ),
    )
    .finalize(components::gpio_component_static!(stm32l476rg::gpio::Pin));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let nucleo_l476rg = Nucleol476RG {
        console,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        led,
        button,
        gpio,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(
            (MSI_FREQUENCY_MHZ * 1_000_000) as u32,
        ),
    };

    // Run MSI clock tests
    stm32l476rg::clocks::msi::tests::run(&clocks.msi);

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
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, nucleo_l476rg, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
