use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;

// This section defines the register offsets of
// RFC_RAT component

#[repr(C)]
pub struct RfcRatRegisters {
    pub ratcnt: ReadWrite<u32, RFCoreRadioTimer::Register>, // Radio Timer Counter Value
}

register_bitfields! [
    u32,
    RFCoreRadioTimer [
        CNT    OFFSET(0) NUMBITS(32) []                       // Radio Timer Register
    ],
    Control [
        RTC_UPD_EN  OFFSET(1) NUMBITS(1) []
    ]
];

const RFC_RAT_BASE: StaticRef<RfcRatRegisters> =
    unsafe { StaticRef::new(0x4004_3004 as *const RfcRatRegisters) };

// Enable RAT interface
pub static mut RFRAT: RFRat = RFRat::new();

pub struct RFRat {
    rfc_rat: StaticRef<RfcRatRegisters>,
}

impl RFRat {
    pub const fn new() -> RFRat {
        RFRat {
            rfc_rat: RFC_RAT_BASE,
        }
    }

    pub fn read_rat(&self) -> u32 {
        let rat_regs = &*self.rfc_rat;
        let cur_rat: u32 = rat_regs.ratcnt.get();

        return cur_rat;
    }
}
