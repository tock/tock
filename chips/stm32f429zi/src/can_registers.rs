//! CAN

use kernel::utilities::StaticRef;
use stm32f4xx::can::Registers;

pub(crate) const CAN1_BASE: StaticRef<Registers> =
    unsafe { StaticRef::new(0x40006400 as *const Registers) };
