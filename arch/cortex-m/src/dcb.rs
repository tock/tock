//! ARM Debug Control Block
//!
//! <https://developer.arm.com/documentation/ddi0403/latest>

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

//TODO: Rest of the Registers
register_structs! {
    DcbRegisters{
        (0x00 => demcr: ReadWrite<u32, DebugExceptionAndMonitorControl::Register>),

        (0x04 => @END),
    }
}

register_bitfields![u32,
    DebugExceptionAndMonitorControl [
        /// Write 1 to globally enable all DWT and ITM features.
        /// RW.
        TRCENA          OFFSET(24)  NUMBITS(1),

        /// Debug monitor semaphore bit.
        /// Monitor software defined.
        /// RW.
        MON_REQ         OFFSET(19)  NUMBITS(1),

        /// Write 1 to make step request pending.
        /// RW.
        MON_STEP        OFFSET(18)  NUMBITS(1),
/// Write 0 to clear the pending state of the DebugMonitor exception.
        /// Writing 1 pends the exception.
        /// RW.
        MON_PEND        OFFSET(17)  NUMBITS(1),

        /// Write 1 to enable DebugMonitor exception.
        /// RW.
        MON_EN        OFFSET(16)  NUMBITS(1),

        //TODO: Rest
    ],
];

const DCB: StaticRef<DcbRegisters> = unsafe { StaticRef::new(0xE000EDFC as *const DcbRegisters) };

/// Enable the Debug and Trace unit `DWT`
/// This has to be enabled before using any feature of the `DWT`
pub fn enable_debug_and_trace() {
    DCB.demcr
        .modify(DebugExceptionAndMonitorControl::TRCENA::SET);
}

/// Disable the Debug and Trace unit `DWT`
pub fn disable_debug_and_trace() {
    DCB.demcr
        .modify(DebugExceptionAndMonitorControl::TRCENA::CLEAR);
}
