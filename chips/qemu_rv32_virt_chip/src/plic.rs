//! Instantiation of the sifive Platform Level Interrupt Controller

use kernel::utilities::StaticRef;
use sifive::plic::{Plic, PlicRegisters};

pub const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(0x0c00_0000 as *const PlicRegisters) };

pub static mut PLIC: Plic = Plic::new(PLIC_BASE);
