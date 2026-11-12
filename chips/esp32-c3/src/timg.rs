// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! TimG Group driver.

use core::marker::PhantomData;

use kernel::ErrorCode;
use kernel::hil::time::{self, Alarm, Counter, Ticks, Ticks32};
use kernel::utilities::cells::OptionalCell;
use tock_registers_next::{Read, Write, register_bitfields, register_map_mmio32};

/// Base address of timer group 0 (TRM Table 3.3-3).
pub const TIMG0_BASE: usize = 0x6001_F000;

/// Base address of timer group 1 (TRM Table 3.3-3).
pub const TIMG1_BASE: usize = 0x6002_0000;

/// 20MHz `Frequency`
#[derive(Debug)]
pub struct Freq20MHz;
impl time::Frequency for Freq20MHz {
    fn frequency() -> u32 {
        20_000_000
    }
}

register_map_mmio32! {
    /// Registers of a timer group.
    pub registers {
        0x000 => t0config: CONFIG::Register { Read, Write },
        0x004 => t0lo: u32 { Read, Write },
        0x008 => t0hi: u32 { Read, Write },
        0x00C => t0update: UPDATE::Register { Read, Write },
        0x010 => t0alarmlo: u32 { Read, Write },
        0x014 => t0alarmhi: u32 { Read, Write },
        0x018 => t0loadlo: u32 { Read, Write },
        0x01C => t0loadhi: u32 { Read, Write },
        0x020 => t0load: u32 { Read, Write },
        0x024 => _,
        0x048 => wdtconfig0: WDTCONFIG0::Register { Read, Write },
        0x04C => wdtconfig1: WDTCONFIG1::Register { Read, Write },
        0x050 => wdtconfig2: u32 { Read, Write },
        0x054 => wdtconfig3: u32 { Read, Write },
        0x058 => wdtconfig4: u32 { Read, Write },
        0x05C => wdtconfig5: u32 { Read, Write },
        0x060 => wdtfeed: u32 { Read, Write },
        0x064 => wdtwprotect: u32 { Read, Write },
        0x068 => rtccalicfg: RTCCALICFG::Register { Read, Write },
        0x06C => rtccalicfg1: RTCCALICFG1::Register { Read, Write },
        0x070 => int_ena: INT::Register { Read, Write },
        0x074 => int_raw: INT::Register { Read, Write },
        0x078 => int_st: INT::Register { Read, Write },
        0x07C => int_clr: INT::Register { Write },
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
    UPDATE [
        UPDATE OFFSET(31) NUMBITS(1) [],
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

/// Clock source of a timer group counter.
#[derive(Copy, Clone)]
pub enum ClockSource {
    /// The 80 MHz APB clock, derived from the PLL.
    Pll = 0,
    /// The 40 MHz crystal oscillator clock.
    Xtal = 1,
}

impl ClockSource {
    /// Frequency of this clock source in Hz.
    const fn frequency(self) -> u32 {
        match self {
            ClockSource::Pll => 80_000_000,
            ClockSource::Xtal => 40_000_000,
        }
    }
}

/// Timer group (TIMG) peripheral, exposing its timer `T0` as a [`Counter`] and
/// [`Alarm`].
///
/// [`Counter::start`] configures the timer as a free-running up-counter. It
/// must be called before using this driver as an [`Alarm`], as
/// [`Alarm::set_alarm`] only arms the alarm and does not start the counter.
///
/// The counter runs at the frequency `F`, which is derived from the
/// [`ClockSource`] by an integer divider. The clock source frequency divided by
/// `F` must thus be an integer between 2 and 65536, or [`Counter::start`]
/// fails.
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
///
/// The driver exposes the low 32 bits of the counter as [`Ticks32`]. These wrap
/// around together with the full counter, as 2^54 is a multiple of 2^32. Every
/// alarm thus lies less than 2^32 ticks from the counter, well within the 2^53
/// ticks the hardware comparison requires to reliably fire the alarm even in
/// the face of a wrapping counter.
pub struct TimG<'a, F: time::Frequency, R: registers::Interface = registers::Real> {
    registers: R,
    clocksource: ClockSource,
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
    _phantom: PhantomData<F>,
}

impl<F: time::Frequency, R: registers::Interface> TimG<'_, F, R> {
    /// Construct a new [`TimG`] driver for the peripheral at `base`, clocked
    /// from `clocksource`.
    ///
    /// This does not access any registers. [`Counter::start`] must be called
    /// before using the returned driver as an [`Alarm`].
    pub const fn new(base: R, clocksource: ClockSource) -> Self {
        TimG {
            registers: base,
            clocksource,
            alarm_client: OptionalCell::empty(),
            _phantom: PhantomData,
        }
    }

    /// Check if the timer 0 alarm interrupt is pending.
    fn alarm_interrupt_pending(&self) -> bool {
        self.registers.int_st().is_set(INT::T0)
    }

    /// Clear a pending timer 0 alarm interrupt.
    fn clear_alarm_interrupt(&self) {
        self.registers.int_clr().write(INT::T0::SET);
    }

    /// Handle a timer group interrupt and notify the alarm client.
    ///
    /// The kernel services interrupts some time after the hardware raises them.
    /// Disarming or setting the alarm in between clears the pending interrupt,
    /// such that this returns without notifying the client.
    pub fn handle_interrupt(&self) {
        if !self.alarm_interrupt_pending() {
            // If we reach this, another alarm has been set in the meantime, so
            // the interrupt we're reacting to is stale. Avoid disarming a
            // potentially new, pending alarm.
            return;
        }

        let _ = self.disarm();
        self.alarm_client.map(|client| {
            client.alarm();
        });
    }

    pub fn disable_wdt(&self) {
        self.registers
            .wdtconfig0()
            .modify(WDTCONFIG0::EN::CLEAR + WDTCONFIG0::FLASHBOOT_MOD_EN::CLEAR);

        if self.registers.wdtconfig0().is_set(WDTCONFIG0::EN)
            || self
                .registers
                .wdtconfig0()
                .is_set(WDTCONFIG0::FLASHBOOT_MOD_EN)
        {
            panic!("Can't disable TIMG WDT");
        }
    }

    /// Read the full 54-bit value of the `T0` counter.
    fn counter(&self) -> u64 {
        // Writing to T0UPDATE latches the current counter value into T0LO and
        // T0HI (TRM Register 11.4). We write its UPDATE field (bit 31) as 1,
        // as the hardware clears this bit once the latched value is ready. We
        // can thus wait for it to read back as 0.
        self.registers.t0update().write(UPDATE::UPDATE::SET);
        while self.registers.t0update().is_set(UPDATE::UPDATE) {}

        let value =
            ((self.registers.t0hi().get() as u64) << 32) | self.registers.t0lo().get() as u64;
        value & COUNTER_MAX
    }
}

impl<F: time::Frequency, R: registers::Interface> time::Time for TimG<'_, F, R> {
    type Frequency = F;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.counter() as u32)
    }
}

impl<'a, F: time::Frequency, R: registers::Interface> Counter<'a> for TimG<'a, F, R> {
    /// Configure the counter as a free-running up-counter and start it.
    ///
    /// This disarms the alarm and discards its pending interrupt. Returns
    /// [`ErrorCode::INVAL`] if the counter cannot run at frequency `F`, see
    /// [`TimG`].
    fn start(&self) -> Result<(), ErrorCode> {
        // The prescaler divides the clock source by 2 to 65536 (TRM Section
        // 11.2.1), and must produce exactly the frequency `F`.
        let source = self.clocksource.frequency();
        let divider = match source.checked_div(F::frequency()) {
            Some(d) if (2..=65536).contains(&d) && d * F::frequency() == source => d,
            _ => return Err(ErrorCode::INVAL),
        };

        // The divider may only be changed while the counter is disabled.
        self.registers.t0config().modify(CONFIG::EN::CLEAR);
        let _ = self.disarm();

        // Count upwards without reload, dividing the clock source down to the
        // frequency `F`. A DIVIDER value of 0 selects a divider of 65536. We
        // have to disable autoreload to make the timer monotonic; once it
        // reaches its maximum value it will simply wrap around. This is
        // compatible with Tock's Alarm trait, which also doesn't use
        // auto-reload.
        self.registers.t0config().modify(
            CONFIG::USE_XTAL.val(self.clocksource as u32)
                + CONFIG::AUTORELOAD::CLEAR
                + CONFIG::INCREASE::SET
                + CONFIG::DIVIDER.val(divider % 65536),
        );

        // Reset the prescaler such that the new divider takes effect.
        self.registers.t0config().modify(CONFIG::DIVCNT_RST::SET);

        // Enable the alarm interrupt, which only fires once `set_alarm` arms
        // the alarm.
        self.registers.int_ena().modify(INT::T0::SET);

        // Start the counter with the new configuration.
        self.registers.t0config().modify(CONFIG::EN::SET);

        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.registers.t0config().modify(CONFIG::EN::CLEAR);

        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn is_running(&self) -> bool {
        self.registers.t0config().is_set(CONFIG::EN)
    }
}

impl<'a, F: time::Frequency, R: registers::Interface> Alarm<'a> for TimG<'a, F, R> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    /// Arm the alarm to fire at `reference + dt`.
    ///
    /// If that time has already passed, the hardware fires the alarm as soon
    /// as it is enabled, see [`TimG`]. This requires the counter to be
    /// running.
    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        // We extend `reference` to the full counter width, assuming that it
        // lies less than 2^32 ticks in the past.
        let counter = self.counter();
        let elapsed = Ticks32::from(counter as u32).wrapping_sub(reference);
        let reference = counter.wrapping_sub(elapsed.into_u32() as u64);

        // The hardware decides whether the alarm value has already passed. We
        // thus do not compare it to the current time, and the counter may
        // advance or wrap around while we arm the alarm. This matches the
        // `Alarm` HIL, as both `dt` and the time elapsed since `reference` are
        // less than 2^32 ticks.
        let alarm = reference.wrapping_add(dt.into_u32() as u64) & COUNTER_MAX;

        // The alarm value only takes effect once the alarm is enabled (TRM
        // Section 11.2.3), such that we can write its two halves in any order.
        let _ = self.disarm();
        self.registers.t0alarmhi().set((alarm >> 32) as u32);
        self.registers.t0alarmlo().set(alarm as u32);
        self.registers.t0config().modify(CONFIG::ALARM_EN::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        // The low 32 bits of the alarm value equal `reference + dt`.
        Self::Ticks::from(self.registers.t0alarmlo().get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.t0config().modify(CONFIG::ALARM_EN::CLEAR);

        // The alarm may have fired already. Clearing its pending interrupt
        // ensures that `handle_interrupt` does not notify the client.
        self.clear_alarm_interrupt();

        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.registers.t0config().is_set(CONFIG::ALARM_EN)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1)
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the [`TimG`] driver, running against [`FakeTimg`], a
    //! model of a timer group's `T0` timer which follows the TRM.

    use super::*;
    use core::cell::Cell;
    use kernel::hil::time::{AlarmClient, Time};
    use tock_registers_next::{FakeRegister, Safe};

    const T0_INT: u32 = 1 << 0;
    const CONFIG_ALARM_EN: u32 = 1 << 10;
    const CONFIG_DIVCNT_RST: u32 = 1 << 12;
    const CONFIG_DIVIDER_SHIFT: u32 = 13;
    const CONFIG_AUTORELOAD: u32 = 1 << 29;
    const CONFIG_INCREASE: u32 = 1 << 30;
    const CONFIG_EN: u32 = 1 << 31;
    const UPDATE_BIT: u32 = 1 << 31;

    /// Model of a timer group's `T0` timer, following TRM Section 11.2.
    ///
    /// The model counts upwards whenever `T0_EN` is set. Its time only advances
    /// through [`FakeTimg::advance`], and by [`FakeTimg::step`] ticks on every
    /// register access. The latter models time passing while the driver runs.
    #[derive(Default)]
    struct FakeTimg {
        /// Current 54-bit counter value.
        counter: Cell<u64>,
        /// Number of ticks the counter advances on every register access.
        step: Cell<u64>,
        config: Cell<u32>,
        /// Counter value latched into T0LO and T0HI.
        latched: Cell<u64>,
        /// Number of T0UPDATE reads after which a requested latch completes.
        latch_pending_reads: Cell<u32>,
        /// Whether the pending latch was requested by setting the UPDATE bit.
        latch_update_bit: Cell<bool>,
        alarm_lo: Cell<u32>,
        alarm_hi: Cell<u32>,
        load_lo: Cell<u32>,
        load_hi: Cell<u32>,
        int_ena: Cell<u32>,
        int_raw: Cell<u32>,
        /// Number of prescaler resets through DIVCNT_RST.
        divcnt_resets: Cell<u32>,
        /// Counter value at which the alarm last fired.
        fired_at: Cell<Option<u64>>,
        /// Backing storage for registers which the driver does not model.
        other: [Cell<u32>; 10],
    }

    impl FakeTimg {
        /// Construct a model with its counter running from `counter`.
        fn running_at(counter: u64) -> Self {
            let fake = FakeTimg::default();
            fake.counter.set(counter);
            fake.config.set(CONFIG_EN | CONFIG_INCREASE);
            fake
        }

        /// Advance the counter by `ticks`, wrapping around at 2^54.
        fn advance(&self, ticks: u64) {
            for _ in 0..ticks {
                if self.config.get() & CONFIG_EN != 0 {
                    self.counter.set((self.counter.get() + 1) & COUNTER_MAX);
                }
                self.check_alarm();
            }
        }

        /// Advance the counter by `ticks` at once, checking the alarm only at
        /// the end. This skips over intervals too long for [`Self::advance`].
        fn jump(&self, ticks: u64) {
            if self.config.get() & CONFIG_EN != 0 {
                self.counter.set((self.counter.get() + ticks) & COUNTER_MAX);
            }
            self.check_alarm();
        }

        /// Fire the alarm if it is enabled and triggered, following TRM Table
        /// 11.2-1 "Alarm Generation When Up-Down Counter Increments".
        fn check_alarm(&self) {
            if self.config.get() & CONFIG_ALARM_EN == 0 {
                return;
            }

            // The TRM's ranges use integer differences of the 54-bit values,
            // which may be negative.
            let timg_value = self.counter.get() as i128;
            let alarm_value =
                (((self.alarm_hi.get() as u64) << 32) | self.alarm_lo.get() as u64) as i128;
            let triggered = if alarm_value - timg_value > 1 << 53 {
                // Scenario 1: triggered.
                true
            } else if 0 < alarm_value - timg_value && alarm_value - timg_value <= 1 << 53 {
                // Scenario 2: triggered when the up-down counter counts
                // TIMG_VALUE up to ALARM_VALUE.
                false
            } else if 0 <= timg_value - alarm_value && timg_value - alarm_value < 1 << 53 {
                // Scenario 3: triggered.
                true
            } else if timg_value - alarm_value >= 1 << 53 {
                // Scenario 4: triggered when the up-down counter restarts
                // counting up from 0 after reaching the timer's maximum value
                // and counts TIMG_VALUE up to ALARM_VALUE.
                false
            } else {
                unreachable!("the scenarios cover all ranges");
            };

            if triggered {
                // The hardware clears ALARM_EN once the alarm fires (TRM
                // Section 11.2.3).
                self.config.set(self.config.get() & !CONFIG_ALARM_EN);
                self.int_raw.set(self.int_raw.get() | T0_INT);
                self.fired_at.set(Some(self.counter.get()));
            }
        }

        /// Model the time passing during a register access.
        fn access(&self) {
            self.advance(self.step.get());
            self.check_alarm();
        }

        fn alarm_pending(&self) -> bool {
            self.int_raw.get() & T0_INT != 0
        }
    }

    /// Implement a fake register backed by one of [`FakeTimg::other`].
    macro_rules! other_register {
        ($name:ident, $type:ty, $index:literal) => {
            type $name<'s> = FakeRegister<&'s Self, $type, Safe, Safe>;
            fn $name(&self) -> Self::$name<'_> {
                FakeRegister::new(self)
                    .on_read(|f| f.other[$index].get())
                    .on_write(|f, v| f.other[$index].set(v))
            }
        };
    }

    impl registers::Interface for FakeTimg {
        type t0config<'s> = FakeRegister<&'s Self, CONFIG::Register, Safe, Safe>;
        fn t0config(&self) -> Self::t0config<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    f.config.get()
                })
                .on_write(|f, v| {
                    f.access();
                    // The divider may only be changed, and the prescaler only
                    // be reset, while the counter is disabled (TRM Section
                    // 11.2.1).
                    let divider = 0xFFFF << CONFIG_DIVIDER_SHIFT;
                    if f.config.get() & CONFIG_EN != 0 {
                        assert_eq!(
                            v & divider,
                            f.config.get() & divider,
                            "DIVIDER changed while T0_EN is set"
                        );
                        assert_eq!(
                            v & CONFIG_DIVCNT_RST,
                            0,
                            "DIVCNT_RST set while T0_EN is set"
                        );
                    }
                    // DIVCNT_RST resets the prescaler and reads back as 0.
                    if v & CONFIG_DIVCNT_RST != 0 {
                        f.divcnt_resets.set(f.divcnt_resets.get() + 1);
                    }
                    f.config.set(v & !CONFIG_DIVCNT_RST);
                    f.check_alarm();
                })
        }

        type t0lo<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0lo(&self) -> Self::t0lo<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    f.latched.get() as u32
                })
                .on_write(|_, _| {})
        }

        type t0hi<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0hi(&self) -> Self::t0hi<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    (f.latched.get() >> 32) as u32
                })
                .on_write(|_, _| {})
        }

        // Any write to T0UPDATE requests a latch of the counter, which only
        // completes after crossing into the counter's clock domain (TRM
        // Register 11.4). The model completes it on the second read of
        // T0UPDATE. Until then, the UPDATE bit reads back as set, if the
        // write set it.
        type t0update<'s> = FakeRegister<&'s Self, UPDATE::Register, Safe, Safe>;
        fn t0update(&self) -> Self::t0update<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    match f.latch_pending_reads.get() {
                        0 => 0,
                        1 => {
                            f.latch_pending_reads.set(0);
                            f.latched.set(f.counter.get());
                            0
                        }
                        n => {
                            f.latch_pending_reads.set(n - 1);
                            if f.latch_update_bit.get() {
                                UPDATE_BIT
                            } else {
                                0
                            }
                        }
                    }
                })
                .on_write(|f, v| {
                    f.access();
                    f.latch_pending_reads.set(2);
                    f.latch_update_bit.set(v & UPDATE_BIT != 0);
                })
        }

        type t0alarmlo<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0alarmlo(&self) -> Self::t0alarmlo<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    f.alarm_lo.get()
                })
                .on_write(|f, v| {
                    f.access();
                    f.alarm_lo.set(v);
                    f.check_alarm();
                })
        }

        type t0alarmhi<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0alarmhi(&self) -> Self::t0alarmhi<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    f.alarm_hi.get()
                })
                .on_write(|f, v| {
                    f.access();
                    // The alarm value's high word has 22 bits (TRM Register
                    // 11.6).
                    f.alarm_hi.set(v & ((1 << 22) - 1));
                    f.check_alarm();
                })
        }

        type t0loadlo<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0loadlo(&self) -> Self::t0loadlo<'_> {
            FakeRegister::new(self)
                .on_read(|f| f.load_lo.get())
                .on_write(|f, v| f.load_lo.set(v))
        }

        type t0loadhi<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0loadhi(&self) -> Self::t0loadhi<'_> {
            FakeRegister::new(self)
                .on_read(|f| f.load_hi.get())
                .on_write(|f, v| f.load_hi.set(v))
        }

        type t0load<'s> = FakeRegister<&'s Self, u32, Safe, Safe>;
        fn t0load(&self) -> Self::t0load<'_> {
            FakeRegister::new(self).on_read(|_| 0).on_write(|f, _| {
                let value = ((f.load_hi.get() as u64) << 32) | f.load_lo.get() as u64;
                f.counter.set(value & COUNTER_MAX);
            })
        }

        other_register!(wdtconfig0, WDTCONFIG0::Register, 0);
        other_register!(wdtconfig1, WDTCONFIG1::Register, 1);
        other_register!(wdtconfig2, u32, 2);
        other_register!(wdtconfig3, u32, 3);
        other_register!(wdtconfig4, u32, 4);
        other_register!(wdtconfig5, u32, 5);
        other_register!(wdtfeed, u32, 6);
        other_register!(wdtwprotect, u32, 7);
        other_register!(rtccalicfg, RTCCALICFG::Register, 8);
        other_register!(rtccalicfg1, RTCCALICFG1::Register, 9);

        type int_ena<'s> = FakeRegister<&'s Self, INT::Register, Safe, Safe>;
        fn int_ena(&self) -> Self::int_ena<'_> {
            FakeRegister::new(self)
                .on_read(|f| f.int_ena.get())
                .on_write(|f, v| f.int_ena.set(v))
        }

        type int_raw<'s> = FakeRegister<&'s Self, INT::Register, Safe, Safe>;
        fn int_raw(&self) -> Self::int_raw<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    f.int_raw.get()
                })
                .on_write(|_, _| {})
        }

        type int_st<'s> = FakeRegister<&'s Self, INT::Register, Safe, Safe>;
        fn int_st(&self) -> Self::int_st<'_> {
            FakeRegister::new(self)
                .on_read(|f| {
                    f.access();
                    f.int_raw.get() & f.int_ena.get()
                })
                .on_write(|_, _| {})
        }

        // INT_CLR is write-only, such that the driver cannot read it.
        type int_clr<'s> =
            FakeRegister<&'s Self, INT::Register, tock_registers_next::NoAccess, Safe>;
        fn int_clr(&self) -> Self::int_clr<'_> {
            FakeRegister::new(self).on_write(|f, v| {
                f.access();
                f.int_raw.set(f.int_raw.get() & !v);
            })
        }
    }

    /// Alarm client counting its callbacks.
    #[derive(Default)]
    struct CountingClient {
        alarms: Cell<u32>,
    }

    impl AlarmClient for CountingClient {
        fn alarm(&self) {
            self.alarms.set(self.alarms.get() + 1);
        }
    }

    /// Alarm client which sets the next alarm from within its callback, as
    /// `MuxAlarm` does, `dt` ticks after the current time.
    #[derive(Default)]
    struct ReArmingClient<'a> {
        timg: OptionalCell<&'a TimG<'a, Freq20MHz, &'a FakeTimg>>,
        dt: Cell<u32>,
        alarms: Cell<u32>,
    }

    impl AlarmClient for ReArmingClient<'_> {
        fn alarm(&self) {
            self.alarms.set(self.alarms.get() + 1);
            self.timg.map(|timg| {
                timg.set_alarm(timg.now(), Ticks32::from(self.dt.get()));
            });
        }
    }

    /// Construct a started driver for `fake`, notifying `client`.
    fn started<'a>(
        fake: &'a FakeTimg,
        client: &'a dyn AlarmClient,
    ) -> TimG<'a, Freq20MHz, &'a FakeTimg> {
        let timg = TimG::new(fake, ClockSource::Pll);
        timg.start().unwrap();
        timg.set_alarm_client(client);
        timg
    }

    /// Deliver a pending alarm interrupt to the driver, as the kernel does
    /// some time after the hardware raised it.
    fn service_interrupt(fake: &FakeTimg, timg: &TimG<Freq20MHz, &FakeTimg>) {
        if fake.alarm_pending() {
            timg.handle_interrupt();
        }
    }

    // ---------------------------------------------------------------------
    // Counter configuration and frequency
    // ---------------------------------------------------------------------

    struct Freq80MHz;
    impl time::Frequency for Freq80MHz {
        fn frequency() -> u32 {
            80_000_000
        }
    }

    struct Freq30MHz;
    impl time::Frequency for Freq30MHz {
        fn frequency() -> u32 {
            30_000_000
        }
    }

    struct Freq0Hz;
    impl time::Frequency for Freq0Hz {
        fn frequency() -> u32 {
            0
        }
    }

    struct Freq1250Hz;
    impl time::Frequency for Freq1250Hz {
        fn frequency() -> u32 {
            1250
        }
    }

    /// `start` configures a free-running up-counter, with a divider derived
    /// from the clock source and `F`.
    #[test]
    fn start_configures_free_running_counter() {
        for (source, divider) in [(ClockSource::Pll, 4), (ClockSource::Xtal, 2)] {
            let fake = FakeTimg::default();
            fake.config.set(CONFIG_AUTORELOAD);
            let timg = TimG::<Freq20MHz, _>::new(&fake, source);
            assert_eq!(timg.start(), Ok(()));

            let config = fake.config.get();
            assert_ne!(config & CONFIG_EN, 0);
            assert_ne!(config & CONFIG_INCREASE, 0);
            assert_eq!(config & CONFIG_AUTORELOAD, 0);
            assert_eq!(config & CONFIG_ALARM_EN, 0);
            assert_eq!((config >> CONFIG_DIVIDER_SHIFT) & 0xFFFF, divider);
            assert_eq!(fake.divcnt_resets.get(), 1);
            assert_eq!(fake.int_ena.get() & T0_INT, T0_INT);
            assert!(timg.is_running());
        }

        let fake = FakeTimg::default();
        let timg = TimG::<Freq1250Hz, _>::new(&fake, ClockSource::Pll);
        assert_eq!(timg.start(), Ok(()));
        assert_eq!((fake.config.get() >> CONFIG_DIVIDER_SHIFT) & 0xFFFF, 64000);
    }

    /// `start` rejects frequencies which the prescaler cannot produce exactly,
    /// without touching any register.
    #[test]
    fn start_rejects_unsupported_frequency() {
        fn check<F: time::Frequency>(source: ClockSource) {
            let fake = FakeTimg::default();
            let timg = TimG::<F, _>::new(&fake, source);
            assert_eq!(timg.start(), Err(ErrorCode::INVAL));
            assert_eq!(fake.config.get(), 0);
            assert_eq!(fake.int_ena.get(), 0);
            assert_eq!(fake.divcnt_resets.get(), 0);
        }

        // A divider of 1 is out of range, and 80 MHz / 30 MHz is no integer.
        check::<Freq80MHz>(ClockSource::Pll);
        check::<Freq80MHz>(ClockSource::Xtal);
        check::<Freq30MHz>(ClockSource::Pll);
        check::<Freq0Hz>(ClockSource::Pll);
    }

    /// Setting and handling alarms neither stops the counter nor resets its
    /// prescaler, such that time keeps advancing steadily.
    #[test]
    fn alarms_keep_counter_running() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);
        let config = fake.config.get();

        for _ in 0..3 {
            timg.set_alarm(timg.now(), Ticks32::from(100));
            fake.advance(100);
            service_interrupt(&fake, &timg);
        }

        assert_eq!(client.alarms.get(), 3);
        assert_eq!(fake.divcnt_resets.get(), 1);
        assert_eq!(fake.config.get(), config);
        assert!(timg.is_running());

        let before = timg.now();
        fake.advance(10);
        assert_eq!(timg.now().wrapping_sub(before).into_u32(), 10);
    }

    /// `start` reconfigures a counter which is already running, e.g., as left
    /// by the boot ROM, only while it is disabled.
    #[test]
    fn start_on_running_counter() {
        let fake = FakeTimg::running_at(1 << 40);
        fake.config
            .set(fake.config.get() | 1 << CONFIG_DIVIDER_SHIFT);
        let timg = TimG::<Freq20MHz, _>::new(&fake, ClockSource::Pll);

        assert_eq!(timg.start(), Ok(()));
        assert_eq!((fake.config.get() >> CONFIG_DIVIDER_SHIFT) & 0xFFFF, 4);
        assert_eq!(timg.start(), Ok(()));
        assert_eq!(fake.divcnt_resets.get(), 2);
        assert_eq!(timg.counter(), 1 << 40);
    }

    /// `start` discards an alarm interrupt which is pending from before, such
    /// that the client is not notified of an alarm it never set.
    #[test]
    fn start_discards_stale_interrupt() {
        let fake = FakeTimg::default();
        fake.int_raw.set(T0_INT);
        let client = CountingClient::default();
        let timg = TimG::<Freq20MHz, _>::new(&fake, ClockSource::Pll);
        timg.set_alarm_client(&client);

        assert_eq!(timg.start(), Ok(()));
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 0);

        // An alarm which fired before restarting the counter is discarded as
        // well.
        timg.set_alarm(timg.now(), Ticks32::from(10));
        fake.advance(10);
        assert!(fake.alarm_pending());
        assert_eq!(timg.start(), Ok(()));
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 0);
    }

    /// A stopped counter neither advances nor fires its alarm. Starting it
    /// again disarms the alarm.
    #[test]
    fn stop_freezes_counter_and_alarm() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);

        timg.set_alarm(timg.now(), Ticks32::from(100));
        assert_eq!(timg.stop(), Ok(()));
        assert!(!timg.is_running());
        let stopped_at = timg.now();
        fake.advance(1000);
        assert_eq!(timg.now().into_u32(), stopped_at.into_u32());
        assert!(!fake.alarm_pending());

        assert_eq!(timg.start(), Ok(()));
        assert!(!timg.is_armed());
        fake.advance(1000);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 0);
    }

    // ---------------------------------------------------------------------
    // Counter reads
    // ---------------------------------------------------------------------

    /// Reading the counter waits for the latch of the current counter value to
    /// complete, and never returns a previously latched value.
    #[test]
    fn counter_read_waits_for_latch() {
        let fake = FakeTimg::running_at(0);
        fake.latched.set(0x1234);
        let timg = TimG::<Freq20MHz, _>::new(&fake, ClockSource::Pll);

        fake.counter.set(0x3F_FFFF_1234_5678);
        assert_eq!(timg.counter(), 0x3F_FFFF_1234_5678);
        assert_eq!(timg.now().into_u32(), 0x1234_5678);

        fake.advance(0x10_0000);
        assert_eq!(timg.counter(), 0x3F_FFFF_1244_5678);
    }

    /// The low 32 bits of the counter wrap around consistently with the full
    /// 54-bit counter.
    #[test]
    fn now_wraps_with_counter() {
        let fake = FakeTimg::running_at(COUNTER_MAX - 1);
        let timg = TimG::<Freq20MHz, _>::new(&fake, ClockSource::Pll);

        assert_eq!(timg.now().into_u32(), u32::MAX - 1);
        fake.advance(1);
        assert_eq!(timg.now().into_u32(), u32::MAX);
        fake.advance(1);
        assert_eq!(timg.now().into_u32(), 0);
        fake.advance(1);
        assert_eq!(timg.now().into_u32(), 1);
    }

    // ---------------------------------------------------------------------
    // Alarms around the counter's wrap-around
    // ---------------------------------------------------------------------

    /// Run the counter until the alarm fires, returning the number of ticks
    /// elapsed since `start`, or `None` if it does not fire within `limit`
    /// ticks.
    fn ticks_until_alarm(fake: &FakeTimg, start: u64, limit: u64) -> Option<u64> {
        for _ in 0..limit {
            if let Some(fired_at) = fake.fired_at.get() {
                return Some(fired_at.wrapping_sub(start) & COUNTER_MAX);
            }
            fake.advance(1);
        }
        fake.fired_at
            .get()
            .map(|f| f.wrapping_sub(start) & COUNTER_MAX)
    }

    /// An alarm whose expiry lies after the counter's wrap-around fires once
    /// the wrapped counter reaches it.
    #[test]
    fn alarm_across_counter_wrap() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);
        fake.counter.set(COUNTER_MAX - 99);

        timg.set_alarm(timg.now(), Ticks32::from(200));
        assert_eq!(fake.alarm_hi.get(), 0);
        assert_eq!(fake.alarm_lo.get(), 100);

        fake.advance(199);
        assert!(!fake.alarm_pending());
        fake.advance(1);
        assert!(fake.alarm_pending());
        assert_eq!(fake.fired_at.get(), Some(100));
    }

    /// The counter may wrap around between the caller reading `reference` and
    /// calling `set_alarm`. The alarm still fires at `reference + dt`, and not
    /// immediately.
    #[test]
    fn alarm_with_reference_before_wrap() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);
        fake.counter.set(COUNTER_MAX - 99);

        let reference = timg.now();
        fake.advance(150);
        timg.set_alarm(reference, Ticks32::from(200));

        assert!(!fake.alarm_pending());
        assert_eq!(ticks_until_alarm(&fake, 50, 1000), Some(50));
    }

    /// An alarm whose expiry has already passed fires immediately, also when
    /// the counter wrapped around in between.
    #[test]
    fn passed_alarm_fires_immediately() {
        for start in [1 << 40, COUNTER_MAX - 49] {
            let fake = FakeTimg::default();
            let client = CountingClient::default();
            let timg = started(&fake, &client);
            fake.counter.set(start);

            let reference = timg.now();
            fake.advance(100);
            timg.set_alarm(reference, Ticks32::from(10));

            assert!(fake.alarm_pending());
            service_interrupt(&fake, &timg);
            assert_eq!(client.alarms.get(), 1);
        }
    }

    /// The counter may advance or wrap around while `set_alarm` runs. The alarm
    /// then neither fires early nor stalls.
    #[test]
    fn alarm_while_counter_advances_during_set_alarm() {
        for step in [0, 1, 3, 7] {
            for offset in 1..300 {
                for dt in [0, offset / 2, offset, offset + 50, offset + 5000] {
                    let fake = FakeTimg::default();
                    let client = CountingClient::default();
                    let timg = started(&fake, &client);
                    fake.counter.set((COUNTER_MAX + 1 - offset) & COUNTER_MAX);
                    fake.step.set(step);

                    let start = timg.counter();
                    let reference = Ticks32::from(start as u32);
                    timg.set_alarm(reference, Ticks32::from(dt as u32));
                    fake.step.set(0);

                    // The model fires the alarm as soon as its value has
                    // passed. Setting it takes a few register accesses, during
                    // which the counter advances by `step` each.
                    let elapsed = ticks_until_alarm(&fake, start, dt + 1000).unwrap_or_else(|| {
                        panic!("stalled: step {step}, offset {offset}, dt {dt}")
                    });
                    assert!(
                        elapsed >= dt,
                        "early: step {step}, offset {offset}, dt {dt}"
                    );
                    assert!(
                        elapsed <= dt + 100 * step,
                        "late: step {step}, offset {offset}, dt {dt}"
                    );
                }
            }
        }
    }

    /// `get_alarm` returns `reference + dt`, wrapping around at 2^32.
    #[test]
    fn get_alarm_returns_expiry() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);
        fake.counter.set(0x2_FFFF_FF00);

        let reference = timg.now();
        timg.set_alarm(reference, Ticks32::from(0x200));
        assert_eq!(timg.get_alarm().into_u32(), 0x100);
        assert_eq!(fake.alarm_hi.get(), 3);
    }

    /// An alarm with `dt` of 0 fires immediately.
    #[test]
    fn zero_dt_fires_immediately() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);

        timg.set_alarm(timg.now(), Ticks32::from(0));
        assert!(fake.alarm_pending());
    }

    /// An alarm with the largest `dt` fires exactly `dt` ticks after
    /// `reference`, also across the counter's wrap-around.
    #[test]
    fn max_dt_fires_after_full_range() {
        for start in [1 << 40, COUNTER_MAX - 1000] {
            let fake = FakeTimg::default();
            let client = CountingClient::default();
            let timg = started(&fake, &client);
            fake.counter.set(start);

            timg.set_alarm(timg.now(), Ticks32::from(u32::MAX));
            fake.jump(u32::MAX as u64 - 1);
            assert!(!fake.alarm_pending());
            fake.advance(1);
            assert!(fake.alarm_pending());
            assert_eq!(
                fake.fired_at.get(),
                Some((start + u32::MAX as u64) & COUNTER_MAX)
            );
        }
    }

    /// An alarm whose `reference` lies almost 2^32 ticks in the past fires
    /// immediately.
    #[test]
    fn reference_far_in_past_fires_immediately() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);
        fake.counter.set((1 << 40) + 12345);

        let reference = Ticks32::from(timg.now().into_u32().wrapping_add(1));
        timg.set_alarm(reference, Ticks32::from(10));
        assert!(fake.alarm_pending());
    }

    /// A `reference` in the future is indistinguishable from one almost 2^32
    /// ticks in the past. The `Alarm` HIL requires `reference` to lie in the
    /// past, and the alarm thus fires immediately.
    #[test]
    fn reference_in_future_counts_as_past() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);
        fake.counter.set(1 << 40);

        let reference = Ticks32::from(timg.now().into_u32().wrapping_add(5));
        timg.set_alarm(reference, Ticks32::from(100));
        assert!(fake.alarm_pending());
    }

    // ---------------------------------------------------------------------
    // Interrupt handling
    // ---------------------------------------------------------------------

    /// A fired alarm notifies the client exactly once, and leaves the alarm
    /// disarmed.
    #[test]
    fn fired_alarm_notifies_client_once() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);

        timg.set_alarm(timg.now(), Ticks32::from(10));
        assert!(timg.is_armed());
        fake.advance(10);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 1);
        assert!(!timg.is_armed());

        // The interrupt controller may report the interrupt again.
        timg.handle_interrupt();
        assert_eq!(client.alarms.get(), 1);
    }

    /// Setting a new alarm before the kernel services the interrupt of a fired
    /// alarm neither notifies the client nor cancels the new alarm.
    #[test]
    fn set_alarm_discards_stale_interrupt() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);

        timg.set_alarm(timg.now(), Ticks32::from(10));
        fake.advance(10);
        assert!(fake.alarm_pending());

        timg.set_alarm(timg.now(), Ticks32::from(1000));
        timg.handle_interrupt();
        assert_eq!(client.alarms.get(), 0);
        assert!(timg.is_armed());

        fake.advance(1000);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 1);
    }

    /// Disarming a fired alarm before the kernel services its interrupt
    /// prevents the client from being notified.
    #[test]
    fn disarm_discards_stale_interrupt() {
        let fake = FakeTimg::default();
        let client = CountingClient::default();
        let timg = started(&fake, &client);

        timg.set_alarm(timg.now(), Ticks32::from(10));
        fake.advance(10);
        assert!(fake.alarm_pending());

        assert_eq!(timg.disarm(), Ok(()));
        timg.handle_interrupt();
        assert_eq!(client.alarms.get(), 0);
        assert!(!timg.is_armed());
    }

    /// A client setting the next alarm from within its callback receives a
    /// callback for that alarm, also if it has already passed.
    #[test]
    fn rearm_from_callback() {
        let fake = FakeTimg::default();
        let client = ReArmingClient::default();
        let timg = started(&fake, &client);
        client.timg.set(&timg);

        timg.set_alarm(timg.now(), Ticks32::from(10));
        fake.advance(10);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 1);

        // The callback set an alarm with a `dt` of 0, which fired immediately.
        assert!(fake.alarm_pending());
        client.dt.set(100);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 2);

        // The callback set an alarm 100 ticks in the future.
        assert!(timg.is_armed());
        fake.advance(99);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 2);
        fake.advance(1);
        service_interrupt(&fake, &timg);
        assert_eq!(client.alarms.get(), 3);
    }
}
