//! Machine Timer instantiation.

use kernel::common::StaticRef;
use rv32i::machine_timer::MachineTimerRegisters;

pub const MTIME_BASE: StaticRef<MachineTimerRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const MachineTimerRegisters) };
