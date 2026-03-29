// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::debug::PanicResources;
use kernel::hil::uart::Transmit;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::utilities::StaticRef;
use kernel::{create_capability, static_init};


pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 0;

// Hardware Constants (Secure Aliases from working C code)
const USART1_BASE: StaticRef<stm32u545::usart::UsartRegisters> =
    unsafe { StaticRef::new(0x50013800 as *const stm32u545::usart::UsartRegisters) };

const SECURE_RCC_AHB2ENR1: *mut u32 = 0x46020C8C as *mut u32;
const SECURE_RCC_APB2ENR: *mut u32 = 0x46020CA4 as *mut u32;
const SECURE_RCC_CCIPR1: *mut u32 = 0x46020CE0 as *mut u32;

const SECURE_GPIOA_MODER: *mut u32 = 0x52020000 as *mut u32;
const SECURE_GPIOA_OSPEEDR: *mut u32 = 0x52020008 as *mut u32;
const SECURE_GPIOA_AFRH: *mut u32 = 0x52020024 as *mut u32;
const SECURE_GPIOA_ODR: *mut u32 = 0x52020014 as *mut u32;

type ChipHw =
    stm32u545::chip::Stm32u5xx<'static, stm32u545::chip::Stm32u5xxDefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new(PanicResources::new());

struct NucleoU545RE {
    console: &'static capsules_core::console::Console<'static>,
    scheduler: &'static components::sched::round_robin::RoundRobinComponentType,
    systick: cortexm33::systick::SysTick,
}

impl SyscallDriverLookup for NucleoU545RE {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
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

#[no_mangle]
pub unsafe fn main() {
    // 1. Basic Core Init (Disables interrupts)
    stm32u545::init();

    // 2. Initialize Deferred Call State (Required by DebugWriter/UartMux)
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // 3. Hardware Initialization (Literal Port from working C code)
    unsafe {
        // Enable Clocks
        *SECURE_RCC_AHB2ENR1 |= 1; // Enable GPIOA Clock
        *SECURE_RCC_APB2ENR |= 1 << 14; // Enable USART1 Clock
        *SECURE_RCC_CCIPR1 &= !3; // USART1 source = PCLK (00)

        for _ in 0..1000 {
            core::arch::asm!("nop");
        }

        // GPIO Configuration: PA5=Out, PA9/10=AF7
        *SECURE_GPIOA_MODER &= !((3 << 10) | (3 << 18) | (3 << 20));
        *SECURE_GPIOA_MODER |= (1 << 10) | (2 << 18) | (2 << 20);
        *SECURE_GPIOA_OSPEEDR |= (3 << 18) | (3 << 20);
        *SECURE_GPIOA_AFRH &= !(0xFF << 4);
        *SECURE_GPIOA_AFRH |= (0x77 << 4);
    }

    // 4. Initialize Tock USART Driver
    let usart = static_init!(
        stm32u545::usart::Usart<'static>,
        stm32u545::usart::Usart::new(USART1_BASE)
    );

    // 5. Configure USART Registers (Ported from working C code)
    unsafe {
        let regs = &*USART1_BASE;
        regs.cr1.modify(stm32u545::usart::CR1::UE::CLEAR);
        regs.presc.set(0);
        regs.brr.set(35); // 115,200 baud @ 4MHz MSI
        regs.icr.set(0x3F);

        // Final Enable: TE, RE, UE
        regs.cr1.write(
            stm32u545::usart::CR1::TE::SET
                + stm32u545::usart::CR1::RE::SET
                + stm32u545::usart::CR1::UE::SET,
        );
    }

    // 6. TEST: Manual Print via Driver
    usart.transmit_byte(b'T');
    usart.transmit_byte(b'O');
    usart.transmit_byte(b'C');
    usart.transmit_byte(b'K');
    usart.transmit_byte(b'\r');
    usart.transmit_byte(b'\n');

    // 7. Early debug print (uses io::Writer directly)
    debug!("Kernel initialization complete. Entering main loop.\r\n");

    // 8. Initialize Tock Kernel Objects
    let peripherals = static_init!(
        stm32u545::chip::Stm32u5xxDefaultPeripherals,
        stm32u545::chip::Stm32u5xxDefaultPeripherals::new()
    );

    let chip = static_init!(
        stm32u545::chip::Stm32u5xx<stm32u545::chip::Stm32u5xxDefaultPeripherals>,
        stm32u545::chip::Stm32u5xx::new(peripherals)
    );

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // 9. Shared UART channel (MUX)
    let uart_mux = components::console::UartMuxComponent::new(usart, 115200)
        .finalize(components::uart_mux_component_static!());

    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    // Setup the Debug Writer
    let _debug_writer = components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // 10. Initialise Platform
    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = static_init!(
        NucleoU545RE,
        NucleoU545RE {
            console,
            scheduler,
            systick: cortexm33::systick::SysTick::new(),
        }
    );

    debug!("Final Kernel check.\r\n");

    // 11. Hand over control to the Tock Kernel Loop
    board_kernel.kernel_loop::<NucleoU545RE, ChipHw, 0>(
        platform,
        chip,
        None,
        &create_capability!(capabilities::MainLoopCapability),
    );
}
