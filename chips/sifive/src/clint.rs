// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Create a timer using the Machine Timer registers.

use core::marker::PhantomData;
use core::num::NonZeroU32;

use kernel::hil::time::{self, Alarm, ConvertTicks, Frequency, Ticks, Ticks64, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use rv32i::machine_timer::{MachineTimer, MachineTimerCompareRegister};
use rv32i::csr;

register_structs! {
    pub ClintRegisters {
        (0x0000 => msip: [ReadWrite<u32>; 4095]),
        (0x3FFC => _reserved),
        (0x4000 => compare: [MachineTimerCompareRegister; 4095]),
        (0xBFF8 => value_low: ReadWrite<u32>),
        (0xBFFC => value_high: ReadWrite<u32>),
        (0xC000 => @END),
    }
}

pub struct Clint<'a, F: Frequency> {
    registers: StaticRef<ClintRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
    mtimer: MachineTimer<'a>,
    _freq: PhantomData<F>,
}

impl<'a, F: Frequency> Clint<'a, F> {
    pub fn new(base: &'a StaticRef<ClintRegisters>) -> Self {
        Self {
            registers: *base,
            client: OptionalCell::empty(),
            mtimer: MachineTimer::new(
                &base.compare,
                &base.value_low,
                &base.value_high,
            ),
            _freq: PhantomData,
        }
    }

    pub fn handle_interrupt(&self) {
        let hart_id = csr::CSR.mhartid.extract().get();
        self.disable_machine_timer(hart_id);

        self.client.map(|client| {
            client.alarm();
        });
    }

    pub fn disable_machine_timer(&self, hart_id: usize) {
        self.mtimer.disable_machine_timer(hart_id);
    }
}

impl<F: Frequency> Time for Clint<'_, F> {
    type Frequency = F;
    type Ticks = Ticks64;

    fn now(&self) -> Ticks64 {
        self.mtimer.now()
    }
}

impl<'a, F: Frequency> time::Alarm<'a> for Clint<'a, F> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let hart_id = csr::CSR.mhartid.extract().get();
        self.mtimer.set_alarm(hart_id, reference, dt)
    }

    fn get_alarm(&self) -> Self::Ticks {
        let hart_id = csr::CSR.mhartid.extract().get();
        self.mtimer.get_alarm(hart_id)
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        let hart_id = csr::CSR.mhartid.extract().get();
        self.mtimer.disarm(hart_id)
    }

    fn is_armed(&self) -> bool {
        let hart_id = csr::CSR.mhartid.extract().get();
        self.mtimer.is_armed(hart_id)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        self.mtimer.minimum_dt()
    }
}

/// SchedulerTimer Implementation for RISC-V mtimer. Notably, this implementation should only be
/// used by a chip if that chip has multiple hardware timer peripherals such that a different
/// hardware timer can be used to provide alarms to capsules and userspace. This
/// implementation will not work alongside other uses of the machine timer.
impl<F: Frequency> kernel::platform::scheduler_timer::SchedulerTimer for Clint<'_, F> {
    fn start(&self, us: NonZeroU32) {
        let now = self.now();
        let tics = self.ticks_from_us(us.get());
        self.set_alarm(now, tics);
    }

    fn get_remaining_us(&self) -> Option<NonZeroU32> {
        // We need to convert from native tics to us, multiplication could overflow in 32-bit
        // arithmetic. So we convert to 64-bit.
        let diff = self.get_alarm().wrapping_sub(self.now()).into_u64();

        // If next alarm is more than one second away from now, alarm must have expired.
        // Use this formulation to protect against errors when the alarm has passed.
        // 1 second was chosen because it is significantly greater than the 400ms max value allowed
        // by start(), and requires no computational overhead (e.g. using 500ms would require
        // dividing the returned ticks by 2)
        // However, if the alarm frequency is slow enough relative to the cpu frequency, it is
        // possible this will be evaluated while now() == get_alarm(), so we special case that
        // result where the alarm has fired but the subtraction has not overflowed
        if diff >= <Self as Time>::Frequency::frequency() as u64 {
            None
        } else {
            let hertz = <Self as Time>::Frequency::frequency() as u64;
            NonZeroU32::new(((diff * 1_000_000) / hertz) as u32)
        }
    }

    fn reset(&self) {
        let hart_id = csr::CSR.mhartid.extract().get();
        self.disable_machine_timer(hart_id);
    }

    fn arm(&self) {
        // Arm and disarm are optional, but controlling the mtimer interrupt
        // should be re-enabled if Tock moves to a design that allows direct control of
        // interrupt enables
        //csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn disarm(&self) {
        //csr::CSR.mie.modify(csr::mie::mie::mtimer::CLEAR);
    }
}
