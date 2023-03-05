//! ETHERNET

use kernel::utilities::StaticRef;
use stm32f4xx::ethernet::Registers;

pub(crate) const ETHERNET_MAC_BASE: StaticRef<Registers> =
    unsafe { StaticRef::new(0x40028000 as *const Registers) };
