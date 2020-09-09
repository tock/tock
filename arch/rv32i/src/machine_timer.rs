//! Create a timer using the Machine Timer registers.

use crate::csr;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time::{self, Alarm, Frequency, Freq32KHz, Ticks, Ticks64, Time};
use kernel::ReturnCode;

register_structs! {
    pub MachineTimerRegisters {
        (0x4000 => value_low: ReadWrite<u32>),
        (0x4004 => value_high: ReadWrite<u32>),
        (0xBFF8 => compare_low: ReadWrite<u32>),
        (0xBFFC => compare_high: ReadWrite<u32>),
        (0xC000 => @END),
    }
}

pub struct MachineTimer<'a> {
    registers: StaticRef<MachineTimerRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl MachineTimer<'_> {
    pub const fn new(base: StaticRef<MachineTimerRegisters>) -> Self {
        MachineTimer {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        self.disable_machine_timer();

        self.client.map(|client| {
            client.alarm();
        });
    }

    fn disable_machine_timer(&self) {
        csr::CSR.mie.modify(csr::mie::mie::mtimer::CLEAR);
    }
}

impl Time for MachineTimer<'_> {
    type Frequency = Freq32KHz;
    type Ticks = Ticks64;

    fn now(&self) -> Ticks64 {
        let first_low: u32 = self.registers.value_low.get();
        let mut high: u32 = self.registers.value_high.get();
        let second_low: u32 = self.registers.value_low.get();
        if second_low < first_low {
            // Wraparound
            high = self.registers.value_high.get();
        }
        Ticks64::from(((high as u64) << 32) | second_low as u64)
    }
}

impl<'a> time::Alarm<'a> for MachineTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        csr::CSR.mie.modify(csr::mie::mie::mtimer::CLEAR);
        // This does not handle the 64-bit wraparound case.
        // Because mtimer fires if the counter is >= the compare,
        // handling wraparound requires setting compare to the
        // maximum value, issuing a callback on the overflow client
        // if there is one, spinning until it wraps around to 0, then
        // setting the compare to the correct value.
        let regs = self.registers;
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);

        if !now.within_range(reference, expire) {
            expire = now;
        }

        let val = expire.into_u64();
        let high = (val >> 32) as u32;
        let low = (val & 0xffffffff) as u32;

        // Recommended approach for setting the two compare registers
        // (RISC-V Privileged Architectures 3.1.15) -pal 8/6/20
        regs.compare_low.set(0xffffffff);
        regs.compare_high.set(high);
        regs.compare_low.set(low);
        csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        let mut val: u64 = (self.registers.compare_high.get() as u64) << 32;
        val |= self.registers.compare_low.get() as u64;
        Ticks64::from(val)
    }

    fn disarm(&self) -> ReturnCode {
        self.disable_machine_timer();
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.registers.compare_high.get() != 0xFFFF_FFFF &&
        self.registers.compare_low.get() != 0xFFFF_FFFF
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1u64)
    }
}

/// SchedulerTimer Implementation for RISC-V mtimer. Notably, this implementation should only be
/// used by a chip if that chip has multiple hardware timer peripherals such that a different
/// hardware timer can be used to provide alarms to capsules and userspace. This
/// implementation will not work alongside other uses of the machine timer.
impl kernel::SchedulerTimer for MachineTimer<'_> {
    fn start(&self, us: u32) {
        let now = self.now();
        let tics = Self::ticks_from_us(us);
        self.set_alarm(now, tics);
    }

    fn get_remaining_us(&self) -> u32 {
        let tics = self.get_alarm().wrapping_sub(self.now()).into_u64();
        let hertz = <Self as Time>::Frequency::frequency() as u64;
        ((tics * 1_000_000) / hertz) as u32
    }

    fn has_expired(&self) -> bool {
        self.now() < self.get_alarm()
    }

    fn reset(&self) {
        self.disable_machine_timer();
    }

}
