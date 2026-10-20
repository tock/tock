// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Dual-timer driver for the MPS3-AN536.
//!
//! Each `DualTimer` instance manages one timer. Pass the dual-timer base
//! address for timer 1, and +0x20 for timer 2.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil::time::{
    Alarm,
    AlarmClient,
    Frequency,
    Ticks,
    Ticks32,
    Time, //Counter, OverflowClient,
};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable}; //ReadWriteable,
use kernel::utilities::registers::{
    ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

register_structs! {
    DualTimerRegisters {
        (0x00 => load: ReadWrite<u32>),
        (0x04 => value: ReadOnly<u32>),
        (0x08 => control: ReadWrite<u32, Control::Register>),
        (0x0c => int_clear: WriteOnly<u32, IntClear::Register>),
        (0x10 => raw_int_status: ReadOnly<u32>),
        (0x14 => masked_int_status: ReadOnly<u32>),
        (0x18 => background_load: ReadWrite<u32>),
        (0x1c => _reserved),
        (0x20 => @END),
    }
}

register_bitfields![u32,
    IntClear [
        INTERRUPT OFFSET(0) NUMBITS(1) []
    ],
    Control [
        ONESHOT OFFSET(0) NUMBITS(1) [],
        TIMER_SIZE OFFSET(1) NUMBITS(1) [],
        TIMER_PRE OFFSET(3) NUMBITS(2) [],
        INT_ENABLE OFFSET(5) NUMBITS(1) [],
        PERIODIC OFFSET(6) NUMBITS(1) [],
        ENABLE OFFSET(7) NUMBITS(1) []
    ]
];

// Tried to implement this peripheral for Alarm, but too complicated!
// Finally used Generic timer instead. But this driver can be used
// for the scheduler or other timing purposes. It just need some
// rework as eveything here is broken or incomplete.

/// One timer from an dual-timer peripheral.
pub struct DualTimer<'a, F: Frequency> {
    registers: StaticRef<DualTimerRegisters>,
    client: OptionalCell<&'a dyn AlarmClient>,
    last_now: Cell<u32>,
    _f: core::marker::PhantomData<F>, // Need by F generic
}

impl<'a, F: Frequency> DualTimer<'a, F> {
    /// Create a timer at `base`. Timer 2 starts at the dual-timer base plus `0x20`.
    pub const fn new(base: u32) -> Self {
        let registers = unsafe { StaticRef::new(base as *const DualTimerRegisters) };

        // Free running mode
        //registers.control.set(Control::ENABLE::SET.value | Control::TIMER_SIZE::SET.value);

        Self {
            registers,
            client: OptionalCell::empty(),
            last_now: Cell::new(0),
            _f: core::marker::PhantomData,
        }
    }

    /// Handle this timer's interrupt. Call from the chip interrupt dispatcher.
    pub fn handle_interrupt(&self) {
        // Clear INT
        self.registers
            .control
            .set(Control::ENABLE::SET.value | Control::TIMER_SIZE::SET.value);
        self.registers.int_clear.write(IntClear::INTERRUPT::SET);
        self.client.map(|client| client.alarm());
    }
}

impl<F: Frequency> Time for DualTimer<'_, F> {
    type Frequency = F;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        let a = -(self.registers.value.get() as i32) as u32;
        let b = a.wrapping_add(self.last_now.get());
        Self::Ticks::from(b)
    }
}

/*
impl<'a> Counter<'a> for DualTimer<'a> {
    fn set_overflow_client(&self, client: &'a dyn OverflowClient) {
        self.overflow_client.set(client);
    }

    fn start(&self) -> Result<(), ErrorCode> {
        let now = self.now().into_u32();
        self.offset.set(now);
        self.alarm_armed.set(false);
        self.start_free_running();
        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        let now = self.now().into_u32();
        self.registers.control.modify(Control::ENABLE::CLEAR);
        self.registers.int_clear.write(IntClear::INTERRUPT::SET);
        self.offset.set(now);
        self.alarm_armed.set(false);
        self.running.set(false);
        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        let was_running = self.running.get();
        self.registers.control.modify(Control::ENABLE::CLEAR);
        self.registers.int_clear.write(IntClear::INTERRUPT::SET);
        self.offset.set(0);
        self.alarm_armed.set(false);
        self.registers.load.set(u32::MAX);
        self.registers.control.set(if was_running {
            Self::free_running_control()
        } else {
            Control::TIMER_SIZE::SET.value | Control::PERIODIC::SET.value
        });
        self.running.set(was_running);
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.get()
    }
}
*/
impl<'a, F: Frequency> Alarm<'a> for DualTimer<'a, F> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        //sh::p!("--- ALARM set client");
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        //sh::p!("--- ALARM set alarm {reference:?} {dt:?}");

        let delta = reference.wrapping_add(dt).into_u32();

        let a = -(self.registers.value.get() as i32) as u32;

        self.last_now.set(a);
        self.registers.load.set(delta);

        self.registers.control.set(
            Control::ENABLE::SET.value
                | Control::TIMER_SIZE::SET.value
                | Control::INT_ENABLE::SET.value,
        );
    }

    fn get_alarm(&self) -> Self::Ticks {
        //sh::p!("--- ALARM get alarm");
        123.into() // whatever
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.int_clear.write(IntClear::INTERRUPT::SET);
        Ok(())
    }

    fn is_armed(&self) -> bool {
        //self.registers.control.is_set(Control::INT_ENABLE)
        false
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(2)
    }
}
