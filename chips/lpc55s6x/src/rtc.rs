// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    self, register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

register_structs! {
    /// Real-Time Clock (RTC)
    RtcRegisters {
        /// RTC control register
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// RTC match register
        (0x004 => match: ReadWrite<u32>),
        /// RTC counter register
        (0x008 => count: ReadWrite<u32, COUNT::Register>),
        /// High-resolution/wake-up timer control register
        (0x00C => wake: ReadWrite<u32, WAKE::Register>),
        /// Sub-second counter register
        (0x010 => subsec: ReadWrite<u32, SUBSEC::Register>),
        (0x014 => _reserved0),
        /// General Purpose register
        (0x040 => gpreg_0: ReadWrite<u32, GPREG0::Register>),
        /// General Purpose register
        (0x044 => gpreg_1: ReadWrite<u32, GPREG1::Register>),
        /// General Purpose register
        (0x048 => gpreg_2: ReadWrite<u32, GPREG2::Register>),
        /// General Purpose register
        (0x04C => gpreg_3: ReadWrite<u32, GPREG3::Register>),
        /// General Purpose register
        (0x050 => gpreg_4: ReadWrite<u32, GPREG4::Register>),
        /// General Purpose register
        (0x054 => gpreg_5: ReadWrite<u32, GPREG5::Register>),
        /// General Purpose register
        (0x058 => gpreg_6: ReadWrite<u32, GPREG6::Register>),
        /// General Purpose register
        (0x05C => gpreg_7: ReadWrite<u32, GPREG7::Register>),
        (0x060 => @END),
    }
}
register_bitfields![u32,
CTRL [
    /// Software reset control
    SWRESET OFFSET(0) NUMBITS(1) [
        /// Not in reset. The RTC is not held in reset. This bit must be cleared prior to co
        NOT_IN_RESET = 0,
        /// In reset. The RTC is held in reset. All register bits within the RTC will be for
        IN_RESET = 1
    ],
    /// RTC 1 Hz timer alarm flag status.
    ALARM1HZ OFFSET(2) NUMBITS(1) [
        /// No match. No match has occurred on the 1 Hz RTC timer. Writing a 0 has no effect
        NoMatchNoMatchHasOccurredOnThe1HzRTCTimerWritingA0HasNoEffect = 0,
        /// Match. A match condition has occurred on the 1 Hz RTC timer. This flag generates
        MATCH = 1
    ],
    /// RTC 1 kHz timer wake-up flag status.
    WAKE1KHZ OFFSET(3) NUMBITS(1) [
        /// Run. The RTC 1 kHz timer is running. Writing a 0 has no effect.
        RunTheRTC1KHzTimerIsRunningWritingA0HasNoEffect = 0,
        /// Time-out. The 1 kHz high-resolution/wake-up timer has timed out. This flag gener
        TIMEOUT = 1
    ],
    /// RTC 1 Hz timer alarm enable for Deep power-down.
    ALARMDPD_EN OFFSET(4) NUMBITS(1) [
        /// Disable. A match on the 1 Hz RTC timer will not bring the part out of Deep power
        DisableAMatchOnThe1HzRTCTimerWillNotBringThePartOutOfDeepPowerDownMode = 0,
        /// Enable. A match on the 1 Hz RTC timer bring the part out of Deep power-down mode
        EnableAMatchOnThe1HzRTCTimerBringThePartOutOfDeepPowerDownMode = 1
    ],
    /// RTC 1 kHz timer wake-up enable for Deep power-down.
    WAKEDPD_EN OFFSET(5) NUMBITS(1) [
        /// Disable. A match on the 1 kHz RTC timer will not bring the part out of Deep powe
        DisableAMatchOnThe1KHzRTCTimerWillNotBringThePartOutOfDeepPowerDownMode = 0,
        /// Enable. A match on the 1 kHz RTC timer bring the part out of Deep power-down mod
        EnableAMatchOnThe1KHzRTCTimerBringThePartOutOfDeepPowerDownMode = 1
    ],
    /// RTC 1 kHz clock enable. This bit can be set to 0 to conserve power if the 1 kHz
    RTC1KHZ_EN OFFSET(6) NUMBITS(1) [
        /// Disable. A match on the 1 kHz RTC timer will not bring the part out of Deep powe
        DisableAMatchOnThe1KHzRTCTimerWillNotBringThePartOutOfDeepPowerDownMode = 0,
        /// Enable. The 1 kHz RTC timer is enabled.
        EnableThe1KHzRTCTimerIsEnabled = 1
    ],
    /// RTC enable.
    RTC_EN OFFSET(7) NUMBITS(1) [
        /// Disable. The RTC 1 Hz and 1 kHz clocks are shut down and the RTC operation is di
        DISABLE = 0,
        /// Enable. The 1 Hz RTC clock is running and RTC operation is enabled. This bit mus
        ENABLE = 1
    ],
    /// RTC oscillator power-down control.
    RTC_OSC_PD OFFSET(8) NUMBITS(1) [
        /// See RTC_OSC_BYPASS
        SeeRTC_OSC_BYPASS = 0,
        /// RTC oscillator is powered-down.
        RTCOscillatorIsPoweredDown = 1
    ],
    /// RTC oscillator bypass control.
    RTC_OSC_BYPASS OFFSET(9) NUMBITS(1) [
        /// The RTC Oscillator operates normally as a crystal oscillator with the crystal co
        USED = 0,
        /// The RTC Oscillator is in bypass mode. In this mode a clock can be directly input
        BYPASS = 1
    ],
    /// RTC Sub-second counter control.
    RTC_SUBSEC_ENA OFFSET(10) NUMBITS(1) [
        /// The sub-second counter (if implemented) is disabled. This bit is cleared by a sy
        POWER_UP = 0,
        /// The 32 KHz sub-second counter is enabled (if implemented). Counting commences on
        POWERED_DOWN = 1
    ]
],
MATCH [
    /// Contains the match value against which the 1 Hz RTC timer will be compared to se
    MATVAL OFFSET(0) NUMBITS(32) []
],
COUNT [
    /// A read reflects the current value of the main, 1 Hz RTC timer. A write loads a n
    VAL OFFSET(0) NUMBITS(32) []
],
WAKE [
    /// A read reflects the current value of the high-resolution/wake-up timer. A write
    VAL OFFSET(0) NUMBITS(16) []
],
SUBSEC [
    /// A read reflects the current value of the 32KHz sub-second counter. This counter
    SUBSEC OFFSET(0) NUMBITS(15) []
],
GPREG0 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG1 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG2 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG3 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG4 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG5 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG6 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
],
GPREG7 [
    /// Data retained during Deep power-down mode or loss of main power as long as VBAT
    GPDATA OFFSET(0) NUMBITS(32) []
]
];
const RTC_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x4002C000 as *const RtcRegisters) };

pub struct Rtc<'a> {
    registers: StaticRef<RtcRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> Rtc<'a> {
    pub const fn new() -> Rtc<'a> {
        Rtc {
            registers: RTC_BASE,
            client: OptionalCell::empty(),
        }
    }

    fn enable_interrupt(&self) {
        self.registers
    }
}
