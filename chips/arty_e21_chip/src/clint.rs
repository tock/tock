//! Machine Timer instantiation.

use kernel::common::StaticRef;
use sifive::clint::ClintRegisters;

pub const CLINT_BASE: StaticRef<ClintRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const ClintRegisters) };
