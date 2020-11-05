//! Power Reset Clock Interrupt controller instantiation.

use kernel::common::StaticRef;
use sifive::prci::PrciRegisters;

pub const PRCI_BASE: StaticRef<PrciRegisters> =
    unsafe { StaticRef::new(0x1000_8000 as *const PrciRegisters) };
