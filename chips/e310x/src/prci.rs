//! Power Reset Clock Interrupts

use kernel::common::StaticRef;
use kernel::common::registers::ReadWrite;
use sifive::prci::Prci;

pub static mut PRCI: Prci = Prci::new(PRCI_BASE);


const PRCI_BASE: StaticRef<PrciRegisters> =
    unsafe { StaticRef::new(0x1000_8000 as *const PrciRegisters) };

