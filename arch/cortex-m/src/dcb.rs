//! ARM Debug Control Block
//!
//! <https://developer.arm.com/documentation/ddi0403/latest>
//! Implementation matches `ARM DDI 0403E.e`

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

//TODO: Rest of the Registers
register_structs! {
    DcbRegisters{
        (0x00 => dhcsr: ReadWrite<u32, DebugHaltingControlAndStatus::Register>),

        (0x04 => dhrsr: Read<u32, DebugCoreSelectorData::Register>),

        (0x08 => dcrdr: ReadWrite<u32, DebugCoreRegisterData::Register>),

        (0x12 => demcr: ReadWrite<u32, DebugExceptionAndMonitorControl::Register>),

        (0x16 => @END),
    }
}

register_bitfields![u32,
    DebugHaltingControlAndStatus [
        /// Debug key. 0xA05F must be written to enable write access to bits 15 through 0.
        /// WO.
        DBGKEY          OFFSET(16)  NUMBITS(16),

        /// Is 1 if at least one reset happend since last read of this register. Is cleared to 0 on
        /// read.
        /// RO.
        S_RESET_ST      OFFSET(25)  NUMBITS(1),

        /// Is 1 if at least one instruction was retired since last read of this register.
        /// It is cleared to 0 after a read of this register.
        /// RO.
        S_RETIRE_ST     OFFSET(24)  NUMBITS(1),

        /// Is 1 when the processor is locked up doe tu an unrecoverable instruction.
        /// RO.
        S_LOCKUP        OFFSET(20)  NUMBITS(4),

        /// Is 1 if processor is in debug state.
        /// RO.
        S_SLEEP         OFFSET(18)  NUMBITS(1),

        /// Is used as a handshake flag for transfers through DCRDR. Writing to DCRSR clears this
        /// bit to 0. Is 0 if there is a transfer that has not completed and 1 on completion of the DCRSR transfer.
        ///
        /// RW.
        S_REGREADY      OFFSET(16)  NUMBITS(1),
    ],
    DebugExceptionAndMonitorControl [
        /// Write 1 to globally enable all DWT and ITM features.
        TRCENA          OFFSET(24)  NUMBITS(1),

        /// Debug monitor semaphore bit.
        /// Monitor software defined.
        MON_REQ         OFFSET(19)  NUMBITS(1),

        /// Write 1 to make step request pending.
        MON_STEP        OFFSET(18)  NUMBITS(1),
/// Write 0 to clear the pending state of the DebugMonitor exception.
        /// Writing 1 pends the exception.
        MON_PEND        OFFSET(17)  NUMBITS(1),

        /// Write 1 to enable DebugMonitor exception.
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
