// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::debug::PanicResources;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::led::Led;
use kernel::hil::uart::Transmit;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::utilities::StaticRef;
use kernel::{create_capability, static_init};

// Use the PinId enum for board wiring
use stm32u545::gpio::PinId;

pub mod io;

extern "C" {
    static _sappmem: u8;
    static _eappmem: u8;
}

const NUM_PROCS: usize = 1;

type ChipHw =
    stm32u545::chip::Stm32u5xx<'static, stm32u545::chip::Stm32u5xxDefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new(PanicResources::new());

struct NucleoU545RE {
    console: &'static capsules_core::console::Console<'static>,
    scheduler: &'static components::sched::round_robin::RoundRobinComponentType,
    systick: cortexm33::systick::SysTick,
    led: &'static capsules_core::led::LedDriver<
        'static,
        kernel::hil::led::LedHigh<'static, stm32u545::gpio::Pin<'static>>,
        1,
    >,
    button: &'static capsules_core::button::Button<'static, stm32u545::gpio::Pin<'static>>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            stm32u545::tim::Tim2<'static>,
        >,
    >,
}

impl SyscallDriverLookup for NucleoU545RE {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

impl KernelResources<ChipHw> for NucleoU545RE {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = components::sched::round_robin::RoundRobinComponentType;
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

/// Helper function for board-specific pin muxing
unsafe fn set_pin_primary_functions(periphs: &stm32u545::Stm32u5xxPeripherals) {
    use kernel::hil::gpio::Configure;

    // USART1 Pins (PA9/10)
    let pin9 = periphs.gpio_a.pin(PinId::Pin09);
    let pin10 = periphs.gpio_a.pin(PinId::Pin10);
    pin9.set_mode(stm32u545::gpio::Mode::AlternateFunction);
    pin9.set_alternate_function(7);
    pin9.set_speed_high();
    pin10.set_mode(stm32u545::gpio::Mode::AlternateFunction);
    pin10.set_alternate_function(7);
    pin10.set_speed_high();

    // LED Pin (PA5)
    periphs.gpio_a.pin(PinId::Pin05).make_output();

    // Button Pin (PC13) - Hardware is Active High
    let btn = periphs.gpio_c.pin(PinId::Pin13);
    btn.make_input();
    btn.set_floating_state(kernel::hil::gpio::FloatingState::PullDown);
}

#[no_mangle]
pub unsafe fn main() {
    stm32u545::init();

    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // 1. Create Individual Drivers
    let exti = static_init!(
        stm32u545::exti::Exti<'static>,
        stm32u545::exti::Exti::new(StaticRef::new(0x56022000 as *const stm32u545::exti::ExtiRegisters))
    );

    let dma1 = static_init!(
        stm32u545::dma::Dma<'static>,
        stm32u545::dma::Dma::new(StaticRef::new(0x50020000 as *const stm32u545::dma::DmaRegisters))
    );

    let usart1 = static_init!(
        stm32u545::usart::Usart<'static>,
        stm32u545::usart::Usart::new(StaticRef::new(0x50013800 as *const stm32u545::usart::UsartRegisters))
    );

    // Link DMA to USART1
    usart1.set_dma(dma1, 0, 1);

    // 2. Load Peripherals Bundle
    let periphs = static_init!(
        stm32u545::Stm32u5xxPeripherals<'static>,
        stm32u545::Stm32u5xxPeripherals::new(exti, dma1, usart1)
    );

    // 3. Power and Wires
    periphs.rcc.enable_dma1();
    periphs.rcc.enable_gpioa();
    periphs.rcc.enable_gpioc();
    periphs.rcc.enable_usart1();
    periphs.rcc.enable_tim2();
    periphs.rcc.enable_syscfg();
    periphs.rcc.set_usart1_source_pclk();

    for _ in 0..1000 {
        core::arch::asm!("nop");
    }

    set_pin_primary_functions(periphs);

    // 4. Driver Config
    use kernel::hil::uart::Configure;
    let _ = periphs.usart1.configure(kernel::hil::uart::Parameters {
        baud_rate: 115200,
        stop_bits: kernel::hil::uart::StopBits::One,
        parity: kernel::hil::uart::Parity::None,
        hw_flow_control: false,
        width: kernel::hil::uart::Width::Eight,
    });
    periphs.usart1.register();

    // 5. Kernel and Muxes
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let uart_mux = components::console::UartMuxComponent::new(periphs.usart1, 115200)
        .finalize(components::uart_mux_component_static!());

    let alarm_mux = components::alarm::AlarmMuxComponent::new(&periphs.tim2).finalize(
        components::alarm_mux_component_static!(stm32u545::tim::Tim2),
    );

    // 6. Capsules
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    let _debug_writer = components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        alarm_mux,
        components::process_printer::ProcessPrinterTextComponent::new()
            .finalize(components::process_printer_text_component_static!()),
        None,
    )
    .finalize(components::process_console_component_static!(
        stm32u545::tim::Tim2
    ));
    let _ = process_console.start();

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        alarm_mux,
    )
    .finalize(components::alarm_component_static!(stm32u545::tim::Tim2));

    let led_pin = static_init!(stm32u545::gpio::Pin, periphs.gpio_a.pin(PinId::Pin05));
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        kernel::hil::led::LedHigh<'static, stm32u545::gpio::Pin>,
        kernel::hil::led::LedHigh::new(led_pin)
    ));

    let button_pin_raw = static_init!(stm32u545::gpio::Pin, periphs.gpio_c.pin(PinId::Pin13));
    let button_pin = static_init!(
        kernel::hil::gpio::InterruptValueWrapper<stm32u545::gpio::Pin>,
        kernel::hil::gpio::InterruptValueWrapper::new(button_pin_raw)
    ).finalize();

    let button_pins = static_init!(
        [(&'static kernel::hil::gpio::InterruptValueWrapper<'static, stm32u545::gpio::Pin>, kernel::hil::gpio::ActivationMode, kernel::hil::gpio::FloatingState); 1],
        [(button_pin, kernel::hil::gpio::ActivationMode::ActiveHigh, kernel::hil::gpio::FloatingState::PullDown)]
    );

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        button_pins,
    )
    .finalize(components::button_component_static!(stm32u545::gpio::Pin));

    // 7. Platform and Interrupts
    let platform = static_init!(
        NucleoU545RE,
        NucleoU545RE {
            console,
            scheduler: components::sched::round_robin::RoundRobinComponent::new(processes)
                .finalize(components::round_robin_component_static!(NUM_PROCS)),
            systick: cortexm33::systick::SysTick::new(),
            led,
            button,
            alarm,
        }
    );

    // 8. Initialize Chip
    let default_peripherals = static_init!(
        stm32u545::chip::Stm32u5xxDefaultPeripherals,
        stm32u545::chip::Stm32u5xxDefaultPeripherals::new(&periphs.tim2, &periphs.usart1, &periphs.exti)
    );

    let chip = static_init!(
        stm32u545::chip::Stm32u5xx<stm32u545::chip::Stm32u5xxDefaultPeripherals>,
        stm32u545::chip::Stm32u5xx::new(default_peripherals)
    );

    // IRQ Targeting
    let nvic_itns0 = 0xE000E380 as *mut u32;
    let nvic_itns1 = 0xE000E384 as *mut u32;
    let mut itns0 = core::ptr::read_volatile(nvic_itns0);
    itns0 &= !(1 << 24);
    itns0 &= !(1 << 29);
    itns0 &= !(1 << 30);
    core::ptr::write_volatile(nvic_itns0, itns0);
    
    let mut itns1 = core::ptr::read_volatile(nvic_itns1);
    itns1 &= !(1 << (45 - 32));
    itns1 &= !(1 << (61 - 32));
    core::ptr::write_volatile(nvic_itns1, itns1);

    unsafe {
        cortexm33::nvic::Nvic::new(45).enable(); // TIM2
        cortexm33::nvic::Nvic::new(61).enable(); // USART1
        cortexm33::nvic::Nvic::new(24).enable(); // EXTI13 (Button)
        cortexm33::nvic::Nvic::new(29).enable(); // GPDMA1 (DMA TX)
        cortexm33::nvic::Nvic::new(30).enable(); // GPDMA1 (DMA RX)
    }

    // --- LOAD PROCESSES ---
    let app_flash = core::slice::from_raw_parts(
        &_sappmem as *const u8,
        &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
    );
    let app_memory = static_init!([u8; 65536], [0; 65536]);

    let _ = kernel::process::load_processes(
        board_kernel,
        chip,
        app_flash,
        app_memory,
        &capsules_system::process_policies::PanicFaultPolicy {},
        &create_capability!(capabilities::ProcessManagementCapability),
    );

    // 9. Hand over control to the Tock Kernel Loop
    board_kernel.kernel_loop::<NucleoU545RE, ChipHw, 1>(
        platform,
        chip,
        None,
        &create_capability!(capabilities::MainLoopCapability),
    );
}
