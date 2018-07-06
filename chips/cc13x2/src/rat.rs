#![allow(dead_code)]
use kernel::common::regs::{ ReadWrite, ReadOnly };
use kernel::common::StaticRef;

//*****************************************************************************
//
// This section defines the register offsets of
// RFC_RAT component
//
//*****************************************************************************

// May need to enable RTC_UPD_EN in rtc.rs for cc23xx chip 
#[repr(C)]
struct RtcRegisters {
    ctl: ReadWrite<u32, Control::Register>,
}

#[repr(C)]
pub struct RfcRatRegisters {
    _reserved: ReadOnly<u32>,
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

const RFC_RAT_BASE: StaticRef<RfcRatRegisters> = unsafe { StaticRef::new(0x4004_3000 as *const RfcRatRegisters) };
const RTC_BASE: StaticRef<RtcRegisters> = unsafe { StaticRef::new(0x40092000 as *const RtcRegisters) };

// Enable RAT interface
pub static mut RFRAT: RFRat = RFRat::new();

pub struct RFRat {
    rat_regs: StaticRef<RfcRatRegisters>,
    rtc_upd_en: StaticRef<RtcRegisters>,
}

impl RFRat {
    pub const fn new() -> RFRat {
        RFRat {
            rat_regs: RFC_RAT_BASE,
            rtc_upd_en: RTC_BASE, 
        }
    }
    
    pub fn rtc_enabled(&self) -> bool {
        let reg = &*self.rtc_upd_en;
        let enabled: bool = reg.ctl.matches_all(Control::RTC_UPD_EN::SET);
        
        return enabled;
    }
    
    pub fn enable_rtc_upd(&self) {
        let reg = &*self.rtc_upd_en;
        reg.ctl.modify(Control::RTC_UPD_EN::SET);
    }

    pub fn read_rat(&self) -> u32 {
        let rat_regs = RFC_RAT_BASE;
        let cur_rat: u32 = rat_regs.ratcnt.get();

        return cur_rat;
    }
}


