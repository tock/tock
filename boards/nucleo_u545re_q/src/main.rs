// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::utilities::StaticRef;
use kernel::{create_capability, static_init};

pub mod io;

const NUM_PROCS: usize = 0;

const USART1_BASE: StaticRef<stm32u545::usart::UsartRegisters> =
    unsafe { StaticRef::new(0x50013800 as *const stm32u545::usart::UsartRegisters) };

type ChipHw =
    stm32u545::chip::Stm32u5xx<'static, stm32u545::chip::Stm32u5xxDefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new(PanicResources::new());

struct NucleoU545RE {
    scheduler: &'static components::sched::round_robin::RoundRobinComponentType,
    systick: cortexm33::systick::SysTick,
}

impl SyscallDriverLookup for NucleoU545RE {
    fn with_driver<F, R>(&self, _driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        f(None)
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

    // 2. Enable FPU (Crucial for Cortex-M33 to prevent UsageFault)
    let scb_cpacr = 0xE000ED88 as *mut u32;
    *scb_cpacr |= 0xF << 20;
    core::arch::asm!("isb");

    // 3. Manual Hardware Override (Exactly like your C code)
    // We do this manually here to bypass any potential issues in Tock's drivers for now.
    let rcc_ahb2enr1 = 0x46020C8C as *mut u32;
    let rcc_apb2enr = 0x46020CA4 as *mut u32;
    let rcc_ccipr1 = 0x46020CE0 as *mut u32;
    let gpioa_base = 0x52020000 as *mut u32;

    unsafe {
        *rcc_ahb2enr1 |= 1; // Enable GPIOA Clock
        *rcc_apb2enr |= 1 << 14; // Enable USART1 Clock
        *rcc_ccipr1 &= !3; // USART1 source = PCLK (00)

        for _ in 0..1000 {
            core::arch::asm!("nop");
        }

        let moder = gpioa_base.offset(0);
        let ospeedr = gpioa_base.offset(0x08 / 4);
        let afrh = gpioa_base.offset(0x24 / 4);
        let odr = gpioa_base.offset(0x14 / 4);

        // PA5 (LED), PA9/10 (USART1)
        *moder &= !((3 << 10) | (3 << 18) | (3 << 20));
        *moder |= ((1 << 10) | (2 << 18) | (2 << 20));
        *ospeedr |= (3 << 18) | (3 << 20);
        *afrh &= !(0xFF << 4);
        *afrh |= (0x77 << 4);

        // USART1 Configuration
        let regs = &*USART1_BASE;
        regs.cr1.modify(stm32u545::usart::CR1::UE::CLEAR);
        regs.presc.set(0);
        regs.brr.set(35); // 115,200 baud @ 4MHz MSI
        regs.icr.set(0x3F);
        regs.cr1.write(
            stm32u545::usart::CR1::TE::SET
                + stm32u545::usart::CR1::RE::SET
                + stm32u545::usart::CR1::UE::SET,
        );
    }

    // --- HEARBEAT & SERIAL TEST ---
    // If this loop works, you'll see LED blinking AND characters.
    // If only LED blinks, the Baud Rate (35) is wrong for the current clock.

    loop {
        unsafe {
            *(0x52020014 as *mut u32) ^= (1 << 5);
        } // Toggle PA5 LED

        // Print 'X' directly to test hardware
        while !((*USART1_BASE).isr.is_set(stm32u545::usart::ISR::TXE)) {}
        (*USART1_BASE).tdr.set(b'X' as u32);

        for _ in 0..1_000_000 {
            core::arch::asm!("nop");
        }
    }

    // 4. Initialize Tock Kernel Objects (KEEP THESE)
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

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = static_init!(
        NucleoU545RE,
        NucleoU545RE {
            scheduler,
            systick: cortexm33::systick::SysTick::new(),
        }
    );

    // 5. Start the Kernel Loop
    board_kernel.kernel_loop::<NucleoU545RE, ChipHw, 0>(
        platform,
        chip,
        None,
        &create_capability!(capabilities::MainLoopCapability),
    );
}
