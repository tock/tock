// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! TimG Group driver.

use core::marker::PhantomData;

use kernel::ErrorCode;
use kernel::hil::time::{self, Alarm, Counter, Ticks, Ticks64};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::register_bitfields;
use kernel::utilities::registers::{ReadWrite, register_structs};

pub const TIMG0_BASE: StaticRef<TimgRegisters> =
    unsafe { StaticRef::new(0x6001_F000 as *const TimgRegisters) };

pub const TIMG1_BASE: StaticRef<TimgRegisters> =
    unsafe { StaticRef::new(0x6002_0000 as *const TimgRegisters) };

/// 20MHz `Frequency`
#[derive(Debug)]
pub struct Freq20MHz;
impl time::Frequency for Freq20MHz {
    fn frequency() -> u32 {
        20_000_000
    }
}

/// 80MHz `Frequency`
#[derive(Debug)]
pub struct Freq80MHz;
impl time::Frequency for Freq80MHz {
    fn frequency() -> u32 {
        80_000_000
    }
}

register_structs! {
    pub TimgRegisters {
        (0x000 => t0config: ReadWrite<u32, CONFIG::Register>),
        (0x004 => t0lo: ReadWrite<u32>),
        (0x008 => t0hi: ReadWrite<u32>),
        (0x00C => t0update: ReadWrite<u32>),
        (0x010 => t0alarmlo: ReadWrite<u32>),
        (0x014 => t0alarmhi: ReadWrite<u32>),
        (0x018 => t0loadlo: ReadWrite<u32>),
        (0x01C => t0loadhi: ReadWrite<u32>),
        (0x020 => t0load: ReadWrite<u32>),

        (0x024 => _reserved0),

        (0x048 => wdtconfig0: ReadWrite<u32, WDTCONFIG0::Register>),
        (0x04C => wdtconfig1: ReadWrite<u32, WDTCONFIG1::Register>),
        (0x050 => wdtconfig2: ReadWrite<u32>),
        (0x054 => wdtconfig3: ReadWrite<u32>),
        (0x058 => wdtconfig4: ReadWrite<u32>),
        (0x05C => wdtconfig5: ReadWrite<u32>),
        (0x060 => wdtfeed: ReadWrite<u32>),
        (0x064 => wdtwprotect: ReadWrite<u32>),

        (0x068 => rtccalicfg: ReadWrite<u32, RTCCALICFG::Register>),
        (0x06C => rtccalicfg1: ReadWrite<u32, RTCCALICFG1::Register>),

        (0x070 => int_ena: ReadWrite<u32, INT::Register>),
        (0x074 => int_raw: ReadWrite<u32, INT::Register>),
        (0x078 => int_st: ReadWrite<u32, INT::Register>),
        (0x07C => int_clr: ReadWrite<u32, INT::Register>),
        (0x080 => @END),
    }
}

register_bitfields![u32,
    CONFIG [
        USE_XTAL OFFSET(9) NUMBITS(1) [],
        ALARM_EN OFFSET(10) NUMBITS(1) [],
        DIVCNT_RST OFFSET(12) NUMBITS(1) [],
        DIVIDER OFFSET(13) NUMBITS(16) [],
        AUTORELOAD OFFSET(29) NUMBITS(1) [],
        INCREASE OFFSET(30) NUMBITS(1) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    WDTCONFIG0 [
        APP_CPU_RESET_EN OFFSET(12) NUMBITS(1) [],
        PROC_CPU_RESET_EN OFFSET(13) NUMBITS(1) [],
        FLASHBOOT_MOD_EN OFFSET(14) NUMBITS(1) [],
        SYS_RESET_LENGTH OFFSET(15) NUMBITS(3) [],
        CPU_RESET_LENGTH OFFSET(18) NUMBITS(3) [],
        USE_XTAL OFFSET(21) NUMBITS(1) [],
        CONF_UPDATE_EN OFFSET(22) NUMBITS(1) [],
        STG3 OFFSET(23) NUMBITS(2) [],
        STG2 OFFSET(25) NUMBITS(2) [],
        STG1 OFFSET(27) NUMBITS(2) [],
        STG0 OFFSET(29) NUMBITS(2) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    WDTCONFIG1 [
        DIVCNT_RST OFFSET(0) NUMBITS(1) [],
        CLK_PRESCALE OFFSET(16) NUMBITS(16) [],
    ],
    RTCCALICFG [
        START_CYCLING OFFSET(12) NUMBITS(1) [],
        CLK_SEL OFFSET(13) NUMBITS(2) [],
        RDY OFFSET(15) NUMBITS(1) [],
        MAX OFFSET(16) NUMBITS(15) [],
        START OFFSET(31) NUMBITS(1) [],
    ],
    RTCCALICFG1 [
        CYCLING_DATA_VLD OFFSET(0) NUMBITS(1) [],
        VALUE OFFSET(7) NUMBITS(25) [],
    ],
    INT [
        T0 OFFSET(0) NUMBITS(1) [],
        WDT OFFSET(1) NUMBITS(1) [],
    ],
];

/// Largest value of the 54-bit `T0` counter, after which it wraps around to
/// zero.
const COUNTER_MAX: u64 = (1 << 54) - 1;

#[derive(Copy, Clone)]
pub enum ClockSource {
    Pll = 0,
    Xtal = 1,
}

/// Timer group (TIMG) peripheral, exposing its timer `T0` as a [`Counter`] and
/// [`Alarm`].
///
/// [`Counter::start`] configures the timer as a free-running up-counter. It
/// must be called before using this driver as an [`Alarm`], as
/// [`Alarm::set_alarm`] only arms the alarm and does not start the counter.
///
/// # Alarm Generation
///
/// The counter and the alarm value are 54 bits wide. Once the alarm is enabled,
/// the hardware compares them as specified in the ESP32-C3 Technical Reference
/// Manual (henceforth TRM, version 1.4), Section 11.2.3 "Alarm Generation",
/// Table 11.2-1 "Alarm Generation When Up-Down Counter Increments". There,
/// `TIMG_VALUE` is the counter value and `ALARM_VALUE` is the alarm value:
///
/// | Scenario | Range                                   | Alarm                                                                                                                                               |
/// |----------|-----------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 1        | `ALARM_VALUE - TIMG_VALUE > 2^53`       | Triggered                                                                                                                                           |
/// | 2        | `0 < ALARM_VALUE - TIMG_VALUE <= 2^53`  | Triggered when the up-down counter counts `TIMG_VALUE` up to `ALARM_VALUE`                                                                          |
/// | 3        | `0 <= TIMG_VALUE - ALARM_VALUE < 2^53`  | Triggered                                                                                                                                           |
/// | 4        | `TIMG_VALUE - ALARM_VALUE >= 2^53`      | Triggered when the up-down counter restarts counting up from 0 after reaching the timer's maximum value and counts `TIMG_VALUE` up to `ALARM_VALUE` |
///
/// Taken together, these scenarios trigger the alarm immediately if
/// `TIMG_VALUE.wrapping_sub(ALARM_VALUE) & COUNTER_MAX < 2^53`, i.e., if the
/// 54-bit unsigned difference has bit 53 cleared, and otherwise once the
/// counter reaches the alarm value. The hardware thus treats an alarm value up
/// to 2^53 ticks behind the counter as already passed, including across the
/// counter's wrap-around. The TRM introduces this behavior to avoid alarms
/// which are enabled after the counter has passed their value.
pub struct TimG<'a, F: time::Frequency> {
    registers: StaticRef<TimgRegisters>,
    clocksource: ClockSource,
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
    _phantom: PhantomData<F>,
}

impl<F: time::Frequency> TimG<'_, F> {
    /// Construct a new [`TimG`] driver for the peripheral at `base`, clocked
    /// from `clocksource`.
    ///
    /// This does not access any registers. [`Counter::start`] must be called
    /// before using the returned driver as an [`Alarm`].
    pub const fn new(base: StaticRef<TimgRegisters>, clocksource: ClockSource) -> Self {
        TimG {
            registers: base,
            clocksource,
            alarm_client: OptionalCell::empty(),
            _phantom: PhantomData,
        }
    }

    pub fn handle_interrupt(&self) {
        self.registers.int_clr.modify(INT::T0::SET);
        let _ = self.disarm();
        self.alarm_client.map(|client| {
            client.alarm();
        });
    }

    pub fn disable_wdt(&self) {
        self.registers
            .wdtconfig0
            .modify(WDTCONFIG0::EN::CLEAR + WDTCONFIG0::FLASHBOOT_MOD_EN::CLEAR);

        if self.registers.wdtconfig0.is_set(WDTCONFIG0::EN)
            || self
                .registers
                .wdtconfig0
                .is_set(WDTCONFIG0::FLASHBOOT_MOD_EN)
        {
            panic!("Can't disable TIMG WDT");
        }
    }
}

impl<F: time::Frequency> time::Time for TimG<'_, F> {
    type Frequency = F;
    type Ticks = Ticks64;

    fn now(&self) -> Self::Ticks {
        // A write (of any value) to T0UPDATE stores the current counter value
        // to T0LO and T0HI. When T0UPDATE is cleared the value is ready.
        self.registers.t0update.set(0xABC);
        while self.registers.t0update.get() != 0 {}

        Self::Ticks::from(
            self.registers.t0lo.get() as u64 + ((self.registers.t0hi.get() as u64) << 32),
        )
    }
}

impl<'a, F: time::Frequency> Counter<'a> for TimG<'a, F> {
    /// Configure the counter as a free-running up-counter and start it.
    fn start(&self) -> Result<(), ErrorCode> {
        // The divider may only be changed while the counter is disabled.
        self.registers
            .t0config
            .modify(CONFIG::EN::CLEAR + CONFIG::ALARM_EN::CLEAR);

        // Count upwards without reload, dividing the PLL clock by 4 or the XTAL
        // clock by 2. We have to disable autoreload to make the timer
        // monotonic; once it reaches its maximum value it will simply wrap
        // around. This is compatible with Tock's Alarm trait, which also
        // doesn't use auto-reload.
        self.registers.t0config.modify(
            CONFIG::USE_XTAL.val(self.clocksource as u32)
                + CONFIG::AUTORELOAD::CLEAR
                + CONFIG::INCREASE::SET
                + CONFIG::DIVIDER.val(2 * (2 - self.clocksource as u32)),
        );

        // Reset the prescaler such that the new divider takes effect.
        self.registers.t0config.modify(CONFIG::DIVCNT_RST::SET);

        // Enable the alarm interrupt, which only fires once `set_alarm` arms
        // the alarm.
        self.registers.int_ena.modify(INT::T0::SET);

        // Start the counter with the new configuration.
        self.registers.t0config.modify(CONFIG::EN::SET);

        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.registers.t0config.modify(CONFIG::EN::CLEAR);

        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn is_running(&self) -> bool {
        self.registers.t0config.is_set(CONFIG::EN)
    }
}

impl<'a, F: time::Frequency> Alarm<'a> for TimG<'a, F> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    /// Arm the alarm to fire at `reference + dt`.
    ///
    /// If that time has already passed, the hardware fires the alarm as soon
    /// as it is enabled, see [`TimG`]. This requires the counter to be
    /// running.
    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        // The hardware decides whether the alarm value has already passed. We
        // thus do not compare it to the current time, and the counter may
        // advance or wrap around while we arm the alarm. This matches the
        // `Alarm` HIL as long as both `dt` and the time elapsed since
        // `reference` are at most 2^53 ticks.
        let alarm = reference.wrapping_add(dt).into_u64() & COUNTER_MAX;

        // The alarm value only takes effect once the alarm is enabled (TRM
        // Section 11.2.3), such that we can write its two halves in any order.
        self.registers.t0config.modify(CONFIG::ALARM_EN::CLEAR);
        self.registers.t0alarmhi.set((alarm >> 32) as u32);
        self.registers.t0alarmlo.set(alarm as u32);
        self.registers.t0config.modify(CONFIG::ALARM_EN::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(
            self.registers.t0alarmlo.get() as u64 + ((self.registers.t0alarmhi.get() as u64) << 32),
        )
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.t0config.modify(CONFIG::ALARM_EN::CLEAR);

        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.registers.t0config.is_set(CONFIG::ALARM_EN)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1_u64)
    }
}
