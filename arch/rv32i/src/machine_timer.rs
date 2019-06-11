//! Create a timer using the Machine Timer registers.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

const MTIME_BASE: StaticRef<MachineTimerRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const MachineTimerRegisters) };

#[repr(C)]
struct MachineTimerRegisters {
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

pub static mut MACHINETIMER: MachineTimer = MachineTimer::new();

pub struct MachineTimer {
    registers: StaticRef<MachineTimerRegisters>,
    client: OptionalCell<&'static hil::time::Client>,
}

impl MachineTimer {
    const fn new() -> MachineTimer {
        MachineTimer {
            registers: MTIME_BASE,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static hil::time::Client) {
        self.client.set(client);
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

impl hil::time::Time for MachineTimer {
    type Frequency = hil::time::Freq32KHz;

    fn disable(&self) {
        self.disable_machine_timer();
    }

    fn is_armed(&self) -> bool {
        // Check if mtimecmp is the max value. If it is, then we are not armed,
        // otherwise we assume we have a value set.
        self.registers.mtimecmp.get() != 0xFFFF_FFFF_FFFF_FFFF
    }
}

impl hil::time::Alarm for MachineTimer {
    fn now(&self) -> u32 {
        self.registers.mtime.get() as u32
    }

    fn set_alarm(&self, tics: u32) {
        self.registers
            .mtimecmp
            .write(MTimeCmp::MTIMECMP.val(tics as u64));
    }

    fn get_alarm(&self) -> u32 {
        self.registers.mtimecmp.get() as u32
    }
}
