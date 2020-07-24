//! Create a timer using the Machine Timer registers.

use crate::csr;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time::{self, Ticks};
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

impl time::Time for MachineTimer<'_> {
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
