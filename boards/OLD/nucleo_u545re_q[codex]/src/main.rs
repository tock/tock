// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Board file for STM32 NUCLEO-U545RE-Q.
//!
//! Only LED + GPIO are exposed as syscall drivers.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::addr_of_mut;

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use capsules_system::scheduler::round_robin::RoundRobinSched;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::hil::led::LedHigh;
use kernel::hil::uart;
use kernel::hil::uart::Configure as UartConfigure;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{create_capability, debug, static_init};
use stm32u545::chip_specs::Stm32u545Specs;
use stm32u545::clocks::hsi::Hsi16;
use stm32u545::gpio::{AlternateFunction, PinId, PortId};
use stm32u545::interrupt_service::Stm32u545DefaultPeripherals;

/// Support routines for debugging I/O.
pub mod io;

#[allow(dead_code)]
mod virtual_uart_rx_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

/// Chip type alias – adjust if your chip::Stm32u5xx type signature differs.
type ChipHw = stm32u545::chip::Stm32u5xx<'static, Stm32u545DefaultPeripherals<'static>>;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

kernel::stack_size! {0x10000}

/// Platform struct – **only** LED + GPIO + alarm + console + scheduler + systick.
struct NucleoU545RE {
    console: &'static capsules_core::console::Console<'static>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, stm32u545::gpio::Pin<'static>>,
        1,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32u545::tim::Tim2<'static>>,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, stm32u545::gpio::Pin<'static>>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm33::systick::SysTick,
}

impl SyscallDriverLookup for NucleoU545RE {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

impl KernelResources<ChipHw> for NucleoU545RE {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm33::systick::SysTick;
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

/// Configure multiplexed I/O – **only what we need for LED + debug GPIOs**.
unsafe fn set_pin_primary_functions(gpio_ports: &'static stm32u545::gpio::GpioPorts<'static>) {
    // Enable clocks for the ports we actually use.
    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();

    // USART1 TX/RX on PA9/PA10 (connected to ST-LINK VCP)
    gpio_ports.get_pin(PinId::PA09).map(|pin| {
        pin.set_alternate_function(AlternateFunction::AF7);
        pin.set_speed();
    });
    gpio_ports.get_pin(PinId::PA10).map(|pin| {
        pin.set_alternate_function(AlternateFunction::AF7);
        pin.set_speed();
    });

    gpio_ports.get_pin(PinId::PA05).map(|pin| {
        pin.make_output();
        pin.set();

        let debug_gpios = static_init!([&'static dyn kernel::hil::gpio::Pin; 1], [pin]);
        kernel::debug::initialize_debug_gpio::<
            <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
        >();
        kernel::debug::assign_gpios(debug_gpios);
    });
}

unsafe fn setup_peripherals(tim2: &stm32u545::tim::Tim2) {
    cortexm33::nvic::Nvic::new(stm32u545::nvic::USART1).enable();

    tim2.enable_clock();
    tim2.start();
    cortexm33::nvic::Nvic::new(stm32u545::nvic::TIM2).enable();
}

#[inline(never)]
unsafe fn start() -> (&'static kernel::Kernel, NucleoU545RE, &'static ChipHw) {
    // Chip-level init (NVIC, vector table, etc.)
    stm32u545::init();

    // RCC + clocks for the chip.
    let rcc = static_init!(stm32u545::rcc::Rcc, stm32u545::rcc::Rcc::new());
    Hsi16::configure_as_sysclk(rcc);
    let clocks = static_init!(
        stm32u545::clocks::Clocks<Stm32u545Specs>,
        stm32u545::clocks::Clocks::new(rcc)
    );
    let peripherals = static_init!(
        stm32u545::interrupt_service::Stm32u545DefaultPeripherals<'static>,
        stm32u545::interrupt_service::Stm32u545DefaultPeripherals::new(clocks)
    );
    peripherals.init();
    let gpio_ports = &peripherals.stm32u545.gpio_ports;
    setup_peripherals(&peripherals.stm32u545.tim2);
    set_pin_primary_functions(gpio_ports);

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let chip = static_init!(ChipHw, stm32u545::chip::Stm32u5xx::new(peripherals));

    //USART
    let _usart_clock = &peripherals.stm32u545.usart1.enable_clock();
    let _ = peripherals.stm32u545.usart1.configure(uart::Parameters {
        baud_rate: 115200,
        stop_bits: uart::StopBits::One,
        parity: uart::Parity::None,
        hw_flow_control: false,
        width: uart::Width::Eight,
    });
    peripherals.stm32u545.usart1.send_byte(b'B');
    peripherals.stm32u545.usart1.send_byte(b'O');
    peripherals.stm32u545.usart1.send_byte(b'O');
    peripherals.stm32u545.usart1.send_byte(b'T');
    peripherals.stm32u545.usart1.send_byte(b'\r');
    peripherals.stm32u545.usart1.send_byte(b'\n');

    let uart_mux =
        components::console::UartMuxComponent::new(&peripherals.stm32u545.usart1, 115200)
            .finalize(components::uart_mux_component_static!());

    // Capabilities needed by the board.
    let _memory_allocation_capability =
        create_capability!(capabilities::MemoryAllocationCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // Conosle
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // LED driver
    // Clock to Port A is enabled in `set_pin_primary_functions()`.
    let led_pin = gpio_ports.get_pin(stm32u545::gpio::PinId::PA05).unwrap();
    led_pin.make_output();

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, stm32u545::gpio::Pin>,
        LedHigh::new(led_pin),
    ));

    // ALARM
    let tim2 = &peripherals.stm32u545.tim2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_static!(stm32u545::tim::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(stm32u545::tim::Tim2));

    // GPIO driver
    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            stm32u545::gpio::Pin,
            0 => gpio_ports.get_pin(PinId::PA00).unwrap(),
            1 => gpio_ports.get_pin(PinId::PA01).unwrap(),
            2 => gpio_ports.get_pin(PinId::PA02).unwrap(),
            3 => gpio_ports.get_pin(PinId::PA03).unwrap(),
            4 => gpio_ports.get_pin(PinId::PA04).unwrap(),
        ),
    )
    .finalize(components::gpio_component_static!(stm32u545::gpio::Pin));

    // Test SET FOR LED
    //led_pin.make_output();
    //led_pin.set();

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());

    // PROCESS CONSOLE
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm33::support::reset),
    )
    .finalize(components::process_console_component_static!(
        stm32u545::tim::Tim2
    ));
    let _ = process_console.start();

    // Scheduler
    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    // SysTick based on HSI frequency.
    let systick =
        cortexm33::systick::SysTick::new_with_calibration((Hsi16::FREQ_MHZ * 1_000_000) as u32);

    let platform = NucleoU545RE {
        console,
        scheduler,
        systick,
        led,
        gpio,
        alarm,
    };

    //virtual_uart_rx_test::run_virtual_uart_receive(uart_mux);

    //debug!("Initialization complete. Entering main loop");
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
            addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - addr_of_mut!(_sappmem) as usize,
        ),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<{ NUM_PROCS as u8 }>>,
        &main_loop_capability,
    );
}
