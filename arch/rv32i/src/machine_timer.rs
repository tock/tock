//! Create a timer using the Machine Timer registers.

use crate::csr;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time::{Alarm, AlarmClient, Freq32KHz, Ticks, Ticks32Bits, Time};

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
    client: OptionalCell<&'a dyn AlarmClient>,
}

impl MachineTimer<'a> {
    pub const fn new(base: StaticRef<MachineTimerRegisters>) -> MachineTimer<'a> {
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

impl Time for MachineTimer<'a> {
    type Ticks = Ticks32Bits;
    type Frequency = Freq32KHz;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.mtime.get() as u32)
    }
}

impl Alarm<'a> for MachineTimer<'a> {
    fn set_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: Self::Ticks) {
        self.registers
            .mtimecmp
            .write(MTimeCmp::MTIMECMP.val(tics.into_u32() as u64));
        csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.mtimecmp.get() as u32)
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
