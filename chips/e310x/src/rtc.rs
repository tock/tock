//! RTC instantiation.

use kernel::common::StaticRef;
use sifive::rtc::{Rtc, RtcRegisters};

pub static mut RTC: Rtc = Rtc::new(RTC_BASE);

const RTC_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x1000_0040 as *const RtcRegisters) };
