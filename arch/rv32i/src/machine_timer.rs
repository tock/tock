//! Create a timer using the Machine Timer registers.

use crate::csr;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::time::{Alarm, Frequency, Time};

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
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
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
            client.fired();
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

impl hil::time::Time for MachineTimer<'_> {
    type Frequency = hil::time::Freq32KHz;

    fn now(&self) -> u32 {
        self.registers.mtime.get() as u32
    }

    fn max_tics(&self) -> u32 {
        core::u32::MAX
    }
}

impl<'a> hil::time::Alarm<'a> for MachineTimer<'a> {
    fn set_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        // TODO: Need to avoid generating spurious interrupts from
        // it taking 2 instructions to write 64 bit registers, see RISC-V Priveleged ISA
        //
        // Correct approach will require seperating MTIMECMP into separate registers. Then,
        // first, set the upper register to the largest value possible without setting
        // off the alarm; this way, we can set the lower register without setting
        // off the alarm, then set the upper register to the correct value.
        self.registers
            .mtimecmp
            .write(MTimeCmp::MTIMECMP.val(tics as u64));
        csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn get_alarm(&self) -> u32 {
        self.registers.mtimecmp.get() as u32
    }

    fn disable(&self) {
        self.disable_machine_timer();
    }

    fn is_enabled(&self) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.registers.mtimecmp.get() != 0xFFFF_FFFF_FFFF_FFFF
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
    }

    fn get_remaining_us(&self) -> Option<u32> {
        // We need to convert from native tics to us, multiplication could overflow in 32-bit
        // arithmetic. So we convert to 64-bit.

        let diff = self.get_alarm().wrapping_sub(self.now()) as u64;
        // If next alarm is more than one second away from now, alarm must have expired.
        // Use this formulation to protect against errors when systick wraps around.
        // 1 second was chosen because it is significantly greater than the 400ms max value allowed
        // by start(), and requires no computational overhead (e.g. using 500ms would require
        // dividing the returned ticks by 2)
        // However, if the alarm frequency is slow enough relative to the cpu frequency, it is
        // possible this will be evaluated while now() == get_alarm(), so we special case that
        // result where the alarm has fired but the subtraction has not overflowed
        if diff >= <Self as Time>::Frequency::frequency() as u64 || diff == 0 {
            None
        } else {
            let hertz = <Self as Time>::Frequency::frequency() as u64;
            Some(((diff * 1_000_000) / hertz) as u32)
        }
    }

    fn reset(&self) {
        self.disable_machine_timer();
    }

    fn arm(&self) {
        csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn disarm(&self) {
        csr::CSR.mie.modify(csr::mie::mie::mtimer::CLEAR);
    }
}
