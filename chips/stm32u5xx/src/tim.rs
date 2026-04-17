// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use cortexm33;
use kernel::hil::time::Time;
use kernel::hil::time::{self, Ticks, Ticks32};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub TimRegisters {
        /// Control register 1
        (0x000 => cr1: ReadWrite<u32, CR1::Register>),
        (0x004 => _reserved0),
        /// DMA/Interrupt enable register
        (0x00C => dier: ReadWrite<u32, DIER::Register>),
        /// Status register
        (0x010 => sr: ReadWrite<u32, SR::Register>),
        /// Event generation register
        (0x014 => egr: ReadWrite<u32>),
        (0x018 => _reserved1),
        /// Counter
        (0x024 => cnt: ReadWrite<u32>),
        /// Prescaler
        (0x028 => psc: ReadWrite<u32>),
        /// Auto-reload register
        (0x02C => arr: ReadWrite<u32>),
        (0x030 => _reserved2),
        /// Capture/compare register 1
        (0x034 => ccr1: ReadWrite<u32>),
        (0x038 => @END),
    }
}

register_bitfields![u32,
    pub CR1 [
        /// Counter enable
        CEN OFFSET(0) NUMBITS(1) []
    ],
    pub DIER [
        /// Update interrupt enable
        UIE  OFFSET(0) NUMBITS(1) [],
        /// CC1 interrupt enable
        CC1IE OFFSET(1) NUMBITS(1) []
    ],
    pub SR [
        /// Update interrupt flag
        UIF  OFFSET(0) NUMBITS(1) [],
        /// CC1 interrupt flag
        CC1IF OFFSET(1) NUMBITS(1) []
    ],
    pub EGR [
        /// Update generation
        UG OFFSET(0) NUMBITS(1) []
    ],
];

/// TIM2 hardware driver for the STM32U5.
///
/// This driver implements the Tock Alarm HIL using the 32-bit general-purpose
/// TIM2 timer. It is configured to run at 32kHz to provide high-resolution
/// timing while remaining power-efficient.
pub struct Tim2<'a> {
    registers: StaticRef<TimRegisters>,
    enable_clock: fn(),
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a> Tim2<'a> {
    /// Creates a new instance of the driver.
    ///
    /// - `base`: The StaticRef pointing to the MMIO base address of the peripheral.
    /// - `enable_clock`: (For Timers) A callback function to power on the peripheral via RCC.
    pub const fn new(base: StaticRef<TimRegisters>, enable_clock: fn()) -> Tim2<'a> {
        Tim2 {
            registers: base,
            enable_clock,
            client: OptionalCell::empty(),
        }
    }

    fn enable_clock(&self) {
        (self.enable_clock)();
    }

    /// Core interrupt handler for the peripheral.
    ///
    /// This function must be called from the chip's main interrupt service routine
    /// (located in `chip.rs`) whenever the corresponding IRQ fires. It
    /// identifies the cause of the interrupt, clears the relevant hardware
    /// pending flags, and notifies any registered clients.
    pub fn handle_interrupt(&self) {
        // Clear interrupt flag
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        self.client.map(|client| {
            client.alarm();
        });
    }

    /// Initializes and starts the timer hardware.
    ///
    /// This sets the prescaler to 124 (converting the 4MHz clock to 32kHz)
    /// and enables the 32-bit free-running counter.
    pub fn start(&self) {
        self.enable_clock();

        // 1. Set the value
        self.registers.psc.set(124);

        // 2. Force the hardware to load the value NOW
        // On STM32, the PSC is buffered. By setting the UG bit in EGR,
        self.registers.egr.set(1);

        // 3. Clear the status flag caused by the manual update
        self.registers.sr.modify(SR::UIF::CLEAR);

        self.registers.arr.set(0xFFFFFFFF);
        self.registers.cr1.modify(CR1::CEN::SET);

        unsafe {
            cortexm33::nvic::Nvic::new(crate::nvic::TIM2_IRQ).enable();
        }
    }
}

impl time::Time for Tim2<'_> {
    type Frequency = time::Freq32KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.cnt.get())
    }
}

impl<'a> time::Alarm<'a> for Tim2<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Ticks32, dt: Ticks32) {
        // 1. Calculate the raw target time
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();

        // 2. The "Past Check": If the target is behind us, clamp it to 'now'
        if !now.within_range(reference, expire) {
            expire = now;
        }

        // 3. The "Minimum Delay": If the alarm is too close to now,
        // push it forward slightly to give the CPU time to finish this function.
        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        // 4. DISARM and CLEAR FIRST
        // This stops old alarms from firing while we are setting the new one.
        let _ = self.disarm();
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        // 5. Program the hardware
        self.registers.ccr1.set(expire.into_u32());
        self.registers.dier.modify(DIER::CC1IE::SET);
    }

    fn get_alarm(&self) -> Ticks32 {
        Ticks32::from(self.registers.ccr1.get())
    }

    fn is_armed(&self) -> bool {
        self.registers.dier.is_set(DIER::CC1IE)
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.dier.modify(DIER::CC1IE::CLEAR);
        Ok(())
    }

    fn minimum_dt(&self) -> Ticks32 {
        Ticks32::from(2)
    }
}
