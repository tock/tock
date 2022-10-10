//! CAN

use stm32f4xx::can::Registers;
use kernel::utilities::StaticRef;

pub(crate) const CAN1_BASE: StaticRef<Registers> =
    unsafe { StaticRef::new(0x40006400 as *const Registers) };
