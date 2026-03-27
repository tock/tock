// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]
#![no_main]
#![allow(missing_docs)]

use core::ptr::addr_of_mut;

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_system::scheduler::round_robin::RoundRobinSched;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil::led::LedHigh;
use kernel::hil::gpio::Configure;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::process::ProcessArray;
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

/// Chip type alias
type ChipHw = stm32u545::chip::Stm32u5xx<'static, Stm32u545DefaultPeripherals<'static>>;

/// Static variables used by io.rs.
static mut PROCESSES: Option<&'static ProcessArray<NUM_PROCS>> = None;

/// Static variable for panic resources.
pub static mut PANIC_RESOURCES: Option<
    &'static PanicResources<ChipHw, capsules_system::process_printer::ProcessPrinterText>,
> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

kernel::stack_size! {0x10000}

/// Platform struct
struct Nucleo32U545 {
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

impl SyscallDriverLookup for Nucleo32U545 {
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

impl KernelResources<ChipHw> for Nucleo32U545 {
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
    fn syscall_filter(&self) -> &Self::SyscallFilter { &() }
    fn process_fault(&self) -> &Self::ProcessFault { &() }
    fn scheduler(&self) -> &Self::Scheduler { self.scheduler }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer { &self.systick }
    fn watchdog(&self) -> &Self::WatchDog { &() }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback { &() }
}

unsafe fn set_pin_primary_functions(gpio_ports: &'static stm32u545::gpio::GpioPorts<'static>) {
    gpio_ports.get_port_from_port_id(PortId::A).enable_clock();

    // LPUART1 TX/RX on PA2/PA3 (connected to ST-LINK VCP)
    gpio_ports.get_pin(PinId::PA02).map(|pin| {
        pin.set_alternate_function(AlternateFunction::AF8);
        pin.set_speed();
    });
    gpio_ports.get_pin(PinId::PA03).map(|pin| {
        pin.set_alternate_function(AlternateFunction::AF8);
        pin.set_speed();
    });
}

unsafe fn setup_peripherals(tim2: &stm32u545::tim::Tim2) {
    tim2.enable_clock();
    tim2.start();
    cortexm33::nvic::Nvic::new(stm32u545::nvic::TIM2).enable();
}

#[inline(never)]
unsafe fn start() -> (&'static kernel::Kernel, Nucleo32U545, &'static ChipHw) {
    stm32u545::init();
    let rcc = static_init!(stm32u545::rcc::Rcc, stm32u545::rcc::Rcc::new());
    Hsi16::configure_as_sysclk(rcc);
    let clocks = static_init!(stm32u545::clocks::Clocks<Stm32u545Specs>, stm32u545::clocks::Clocks::new(rcc));
    let peripherals = static_init!(stm32u545::interrupt_service::Stm32u545DefaultPeripherals<'static>, stm32u545::interrupt_service::Stm32u545DefaultPeripherals::new(clocks));
    peripherals.init();
    let gpio_ports = &peripherals.stm32u545.gpio_ports;
    setup_peripherals(&peripherals.stm32u545.tim2);
    set_pin_primary_functions(gpio_ports);

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PROCESSES = Some(processes);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));
    let chip = static_init!(ChipHw, stm32u545::chip::Stm32u5xx::new(peripherals));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    let panic_resources = static_init!(PanicResources<ChipHw, capsules_system::process_printer::ProcessPrinterText>, PanicResources::new());
    panic_resources.processes.replace(processes.as_slice());
    panic_resources.chip.replace(chip);
    panic_resources.printer.replace(process_printer);
    PANIC_RESOURCES = Some(panic_resources);

    // Using LPUART1. If UartMuxComponent needs a Usart object, we use usart1 placeholder
    // but our debug! macro will use raw LPUART1.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.stm32u545.usart1, 115200)
            .finalize(components::uart_mux_component_static!());

    let console = components::console::ConsoleComponent::new(board_kernel, capsules_core::console::DRIVER_NUM, uart_mux)
        .finalize(components::console_component_static!());

    let led_pin = gpio_ports.get_pin(stm32u545::gpio::PinId::PA05).unwrap();
    led_pin.make_output();
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, stm32u545::gpio::Pin>, LedHigh::new(led_pin)));

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.stm32u545.tim2).finalize(
        components::alarm_mux_component_static!(stm32u545::tim::Tim2));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, capsules_core::alarm::DRIVER_NUM, mux_alarm)
        .finalize(components::alarm_component_static!(stm32u545::tim::Tim2));

    let gpio = GpioComponent::new(board_kernel, capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(stm32u545::gpio::Pin, 0 => gpio_ports.get_pin(PinId::PA00).unwrap())
    ).finalize(components::gpio_component_static!(stm32u545::gpio::Pin));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));
    let systick = cortexm33::systick::SysTick::new_with_calibration((Hsi16::FREQ_MHZ * 1_000_000) as u32);

    (board_kernel, Nucleo32U545 { console, led, alarm, gpio, scheduler, systick }, chip)
}

#[no_mangle]
pub unsafe fn main() {
    // --- Register Addresses ---
    let rcc_ahb2enr   = 0x46020C8C as *mut u32;
    let rcc_apb2enr   = 0x46020CA0 as *mut u32;
    let rcc_ccipr1    = 0x46020CE0 as *mut u32;
    
    let gpioa_moder   = 0x42020000 as *mut u32;
    let gpioa_ospeedr = 0x42020008 as *mut u32; // <--- ADDED
    let gpioa_odr     = 0x42020014 as *mut u32;
    let gpioa_afrh    = 0x42020024 as *mut u32;
    
    let uart_base = 0x40013800 as *mut u32; 
    let uart_cr1  = uart_base.wrapping_offset(0x00 / 4);
    let uart_brr  = uart_base.wrapping_offset(0x0C / 4);
    let uart_isr  = uart_base.wrapping_offset(0x1C / 4);
    let uart_tdr  = uart_base.wrapping_offset(0x28 / 4);

    // 1. Enable Peripherals
    *rcc_ahb2enr |= 0x1;           // GPIOA Clock
    *rcc_apb2enr |= (1 << 14);     // USART1 Clock
    for _ in 0..100 { cortexm33::support::nop(); }

    // 2. Set USART1 Clock Source to SYSCLK (01)
    // Put this here before enabling the UART
    *rcc_ccipr1 &= !0x3;
    *rcc_ccipr1 |= 0x1; 

    // 3. Configure GPIO (PA9=TX, PA10=RX)
    *gpioa_moder &= !( (0x3 << 10) | (0xF << 18) ); // Clear PA5, PA9, PA10
    *gpioa_moder |= (0x1 << 10) | (0xA << 18);      // PA5=Out, PA9/10=AF

    // Set High Speed for PA9/10 (Crucial for 115200+ baud)
    *gpioa_ospeedr |= (0xF << 18); 

    // AF7 for PA9 and PA10
    *gpioa_afrh &= !(0xFF << 4);
    *gpioa_afrh |= (0x77 << 4);

    // 4. USART1 Init (Assume 8MHz from ST-LINK MCO)
    // 8,000,000 / 115,200 = 69.44 -> 69 (0x45)
    *uart_cr1 &= !0x1; 
    *uart_brr = 69; 
    *uart_cr1 |= 0x1 | 0x8; // UE=1, TE=1

    loop {
        *gpioa_odr ^= 1 << 5; // Blink LED
        
        // Flood the UART with 'X'
        for _ in 0..10 {
            while (*uart_isr & (1 << 7)) == 0 {}
            *uart_tdr = b'X' as u32;
        }

        for _ in 0..1_000_000 { cortexm33::support::nop(); }
    }
}
