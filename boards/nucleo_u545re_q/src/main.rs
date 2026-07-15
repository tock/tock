// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::deferred_call::DeferredCallClient;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, static_init};

use stm32u545::gpio::PinId;

pub mod io;

extern "C" {
    static _sappmem: u8;
    static _eappmem: u8;
}

const NUM_PROCS: usize = 4;

type GpioHw = stm32u545::gpio::Pin<'static>;
type ChipHw =
    stm32u545::chip::Stm32u5xx<'static, stm32u545::chip::Stm32u5xxDefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

type GpioDriver = components::gpio::GpioComponentType<GpioHw>;

static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x2000}

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
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    dac: &'static capsules_extra::dac::Dac<'static>,
    gpio: &'static GpioDriver,
    date_time:
        &'static capsules_extra::date_time::DateTimeCapsule<'static, stm32u545::rtc::Rtc<'static>>,
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
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::dac::DRIVER_NUM => f(Some(self.dac)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_extra::date_time::DRIVER_NUM => f(Some(self.date_time)),
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
unsafe fn set_pin_primary_functions(periphs: &stm32u545::chip::Stm32u5xxDefaultPeripherals) {
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

    // Arduino A0 (PA_0 = ADC1_IN5 - Channel5)
    periphs
        .gpio_a
        .pin(PinId::Pin00)
        .set_mode(stm32u545::gpio::Mode::Analog);
    // Arduino A1 (PA_1 = ADC1_IN6 - Channel6)
    periphs
        .gpio_a
        .pin(PinId::Pin01)
        .set_mode(stm32u545::gpio::Mode::Analog);
    //DAC pin (PA4) A2 on the board
    periphs
        .gpio_a
        .pin(PinId::Pin04)
        .set_mode(stm32u545::gpio::Mode::Analog);
    // Arduino A3 (PB_0 = ADC1_IN15 - Channel15)
    periphs
        .gpio_b
        .pin(PinId::Pin00)
        .set_mode(stm32u545::gpio::Mode::Analog);
    // Arduino A4 (PC_1 = ADC1_IN2 - Channel2)
    periphs
        .gpio_c
        .pin(PinId::Pin01)
        .set_mode(stm32u545::gpio::Mode::Analog);
    // Arduino A5 (PC_0 = ADC1_IN1 - Channel1)
    periphs
        .gpio_c
        .pin(PinId::Pin00)
        .set_mode(stm32u545::gpio::Mode::Analog);
}

#[inline(never)]
#[allow(clippy::large_stack_arrays)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    &'static NucleoU545RE,
    &'static ChipHw,
) {
    ChipHw::init();

    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Create Individual Drivers
    let exti = static_init!(
        stm32u545::exti::Exti<'static>,
        stm32u545::exti::Exti::new(stm32u545::exti::EXTI_BASE)
    );
    let dma1 = static_init!(
        stm32u545::dma::Dma,
        stm32u545::dma::Dma::new(stm32u545::dma::DMA1_BASE)
    );
    let usart1 = static_init!(
        stm32u545::usart::Usart<'static>,
        stm32u545::usart::Usart::new(stm32u545::usart::USART1_BASE)
    );

    usart1.register();

    let rcc = static_init!(
        stm32u545::rcc::Rcc,
        stm32u545::rcc::Rcc::new(stm32u545::rcc::RCC_BASE)
    );

    // Load Peripherals Bundle
    let periphs = static_init!(
        stm32u545::chip::Stm32u5xxDefaultPeripherals<'static>,
        stm32u545::chip::Stm32u5xxDefaultPeripherals::new(usart1, exti, dma1, rcc)
    );

    // Initialize wiring (DMA, clocks)
    periphs.init();

    // Register the RTC to the deferred call client
    periphs.rtc.register();

    // Board specific wiring
    periphs.tim2.start();
    set_pin_primary_functions(periphs);

    // Kernel and Muxes
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let uart_mux = components::console::UartMuxComponent::new(periphs.usart1, 115200)
        .finalize(components::uart_mux_component_static!());

    let alarm_mux = components::alarm::AlarmMuxComponent::new(&periphs.tim2).finalize(
        components::alarm_mux_component_static!(stm32u545::tim::Tim2),
    );

    // Capsules
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

    let date_time = components::date_time::DateTimeComponent::new(
        board_kernel,
        capsules_extra::date_time::DRIVER_NUM,
        &periphs.rtc,
    )
    .finalize(components::date_time_component_static!(
        stm32u545::rtc::Rtc<'static>
    ));

    let led_pin = static_init!(stm32u545::gpio::Pin, periphs.gpio_a.pin(PinId::Pin05));
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        kernel::hil::led::LedHigh<'static, stm32u545::gpio::Pin>,
        kernel::hil::led::LedHigh::new(led_pin)
    ));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            stm32u545::gpio::Pin,
            (
                static_init!(stm32u545::gpio::Pin, periphs.gpio_c.pin(PinId::Pin13)),
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullDown
            )
        ),
    )
    .finalize(components::button_component_static!(stm32u545::gpio::Pin));

    let adc_mux = components::adc::AdcMuxComponent::new(&periphs.adc1)
        .finalize(components::adc_mux_component_static!(stm32u545::adc::Adc));

    // Register the ADC channels in the same order as Arduino pins A0-A5
    let adc1_channel_5 =
        components::adc::AdcComponent::new(adc_mux, stm32u545::adc::Channel::Channel5)
            .finalize(components::adc_component_static!(stm32u545::adc::Adc));
    let adc1_channel_6 =
        components::adc::AdcComponent::new(adc_mux, stm32u545::adc::Channel::Channel6)
            .finalize(components::adc_component_static!(stm32u545::adc::Adc));
    let adc1_channel_9 =
        components::adc::AdcComponent::new(adc_mux, stm32u545::adc::Channel::Channel9)
            .finalize(components::adc_component_static!(stm32u545::adc::Adc));
    let adc1_channel_15 =
        components::adc::AdcComponent::new(adc_mux, stm32u545::adc::Channel::Channel15)
            .finalize(components::adc_component_static!(stm32u545::adc::Adc));
    let adc1_channel_2 =
        components::adc::AdcComponent::new(adc_mux, stm32u545::adc::Channel::Channel2)
            .finalize(components::adc_component_static!(stm32u545::adc::Adc));
    let adc1_channel_1 =
        components::adc::AdcComponent::new(adc_mux, stm32u545::adc::Channel::Channel1)
            .finalize(components::adc_component_static!(stm32u545::adc::Adc));

    // Applications will see 6 ADC channels available, with index 0-5 corresponding directly to Arduino pins A0-A5
    let adc_syscall =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                adc1_channel_5,
                adc1_channel_6,
                adc1_channel_9,
                adc1_channel_15,
                adc1_channel_2,
                adc1_channel_1,
            ));
    let dac = components::dac::DacComponent::new(&periphs.dac)
        .finalize(components::dac_component_static!());
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper_owned!(
            GpioHw,
            // Digital pins
            0 => periphs.gpio_a.pin(PinId::Pin03), // D0
            1 => periphs.gpio_a.pin(PinId::Pin02), // D1
            2 => periphs.gpio_c.pin(PinId::Pin08), // D2
            // D3-D6 require GPIOB
            7 => periphs.gpio_a.pin(PinId::Pin08), // D7
            8 => periphs.gpio_c.pin(PinId::Pin07), // D8
            9 => periphs.gpio_c.pin(PinId::Pin06), // D9
            10 => periphs.gpio_c.pin(PinId::Pin09), // D10
            11 => periphs.gpio_a.pin(PinId::Pin07), // D11
            12 => periphs.gpio_a.pin(PinId::Pin06), // D12
            // 13 => D13/PA5 is used by the LD2 LED capsule
            // D14-D15 require GPIOB

            // Analog pins exposed as GPIO
            16 => periphs.gpio_a.pin(PinId::Pin00), // A0
            17 => periphs.gpio_a.pin(PinId::Pin01), // A1
            18 => periphs.gpio_a.pin(PinId::Pin04), // A2
            // 19 => A3 requires GPIOB
            20 => periphs.gpio_c.pin(PinId::Pin01), // A4
            21 => periphs.gpio_c.pin(PinId::Pin00), // A5

            // ST Morpho-only GPIO pins (no D/A aliases)
            22 => periphs.gpio_c.pin(PinId::Pin10), // CN7 pin 1
            23 => periphs.gpio_c.pin(PinId::Pin11), // CN7 pin 2
            24 => periphs.gpio_c.pin(PinId::Pin12), // CN7 pin 3
            25 => periphs.gpio_a.pin(PinId::Pin15), // CN7 pin 17
            26 => periphs.gpio_c.pin(PinId::Pin03), // CN7 pin 37
        ),
    )
    .finalize(components::gpio_component_static!(GpioHw));

    // Platform and Interrupts
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
            adc: adc_syscall,
            dac,
            gpio,
            date_time,
        }
    );

    let chip = static_init!(
        stm32u545::chip::Stm32u5xx<stm32u545::chip::Stm32u5xxDefaultPeripherals>,
        stm32u545::chip::Stm32u5xx::new(periphs)
    );

    // Symbols for linker
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

    // Load processes
    let app_flash = core::slice::from_raw_parts(
        core::ptr::addr_of!(_sapps),
        core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
    );

    let app_memory = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );

    let _ = kernel::process::load_processes(
        board_kernel,
        chip,
        app_flash,
        app_memory,
        &capsules_system::process_policies::PanicFaultPolicy {},
        &create_capability!(capabilities::ProcessManagementCapability),
    );

    (board_kernel, platform, chip)
}

#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    // Hand over control to the Tock Kernel Loop
    board_kernel.kernel_loop::<NucleoU545RE, ChipHw, { NUM_PROCS as u8 }>(
        platform,
        chip,
        None,
        &main_loop_capability,
    );
}
