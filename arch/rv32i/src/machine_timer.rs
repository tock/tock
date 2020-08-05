//! Create a timer using the Machine Timer registers.

use crate::csr;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time::{self, Alarm, Frequency, Ticks, Time};
use kernel::ReturnCode;

#[repr(C)]
pub struct MachineTimerRegisters {
    _reserved0: [u8; 0x4000],
    mtimecmp: ReadWrite<u64, MTimeCmp::Register>,
    _reserved1: [u8; 0x7FF0],
    mtime: ReadOnly<u64, MTime::Register>,
}

register_bitfields![u64,
    MTimeCmp [
        MTIMECMP OFFSET(0) NUMBITS(64) []
    ],
    MTime [
        MTIME OFFSET(0) NUMBITS(64) []
    ]
];

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
        // Disable by setting the mtimecmp register to its max value, which
        // we will never hit.
        self.registers
            .mtimecmp
            .write(MTimeCmp::MTIMECMP.val(0xFFFF_FFFF_FFFF_FFFF));
    }
}

impl Time for MachineTimer<'_> {
    type Frequency = time::Freq32KHz;
    type Ticks = time::Ticks64;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.mtime.get())
    }
}

impl<'a> time::Alarm<'a> for MachineTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        // TODO: Need to avoid generating spurious interrupts from
        // it taking 2 instructions to write 64 bit registers, see RISC-V Priveleged ISA
        //
        // Correct approach will require seperating MTIMECMP into separate registers. Then,
        // first, set the upper register to the largest value possible without setting
        // off the alarm; this way, we can set the lower register without setting
        // off the alarm, then set the upper register to the correct value.
        let mut expire = reference.wrapping_add(dt);
        let now = Self::Ticks::from(self.registers.mtime.get());
        if !now.within_range(reference, expire) {
            // expire has already passed, so fire immediately
            // TODO(alevy): we probably need some wiggle room, but
            //              I can't trivially figure out how much
            expire = now;
        }

        self.registers
            .mtimecmp
            .write(MTimeCmp::MTIMECMP.val(expire.into_u64()));
        csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.mtimecmp.get())
    }

    fn disarm(&self) -> ReturnCode {
        self.disable_machine_timer();
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.registers.mtimecmp.get() != 0xFFFF_FFFF_FFFF_FFFF
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
        let tics = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us as u64;
            let hertz = <Self as Time>::Frequency::frequency() as u64;

            hertz * us / 1_000_000
        };
        self.registers.mtimecmp.write(MTimeCmp::MTIMECMP.val(tics));
        csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn get_remaining_us(&self) -> u32 {
        let tics = self.get_alarm().wrapping_sub(self.now()).into_u64();
        let hertz = <Self as Time>::Frequency::frequency() as u64;
        ((tics * 1_000_000) / hertz) as u32
    }

    fn has_expired(&self) -> bool {
        self.now().into_u64() < self.get_alarm().into_u64()
    }

    fn reset(&self) {
        self.disable_machine_timer();
    }

    fn disarm(&self) {
        csr::CSR.mie.modify(csr::mie::mie::mtimer::CLEAR);
    }
}
