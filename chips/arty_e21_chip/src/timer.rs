//! Machine Timer instantiation.

use kernel::common::StaticRef;
use riscv::machine_timer::{MachineTimer, MachineTimerRegisters};

pub static mut MACHINETIMER: MachineTimer = MachineTimer::new(MTIME_BASE);

const MTIME_BASE: StaticRef<MachineTimerRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const MachineTimerRegisters) };
