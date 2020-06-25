//! General Purpose Input/Output driver.

use kernel::common::registers::{register_structs, ReadWrite};

register_structs! {
    pub PadCtrlRegisters {
        (0x00 => regen: ReadWrite<u32>),
        (0x04 => dio_pads: ReadWrite<u32>),
        (0x08 => mio_pads0: ReadWrite<u32>),
        (0x0c => mio_pads1: ReadWrite<u32>),
        (0x10 => mio_pads2: ReadWrite<u32>),
        (0x14 => mio_pads3: ReadWrite<u32>),
        (0x18 => @END),
    }
}
