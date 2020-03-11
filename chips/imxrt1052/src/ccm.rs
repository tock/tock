use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::ClockInterface;

/// Clock Controller Module
// CCGR1
// CCGR4
#[repr(C)]
struct RccRegisters {
    /// CCM control register
    ccr: ReadWrite<u32, CR::Register>,
    _reserved1: [u8; 4],
    /// CCM status register
    csr: ReadOnly<u32, CR::Register>,
    /// unimplemented
    _reserved1: [u8; 92],
    // clock gating register 1
    ccgr1: [u8, CR::Register],
    _reserved2: [u8; 12],
    // clock gating register 4
    ccgr4: [u8, CR::Register],
    _reserved3: [u8; 12],
}

register_bitfields![u32,
    ccr [
    	/// Enable for REG_BYPASS_COUNTER
    	RBC_EN(27) NUMBITS(1) [],

    	/// Counter for analog_reg_bypass
    	REG_BYPASS_COUNT(21) NUMBITS(6) [],

    	/// On chip oscilator enable bit
    	COSC_EN OFFSET(12) NUMBITS(1) [],

        /// Oscilator ready counter value
        OSCNT OFFSET(0) NUMBITS(8) []
    ]
]