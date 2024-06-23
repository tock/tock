// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! AON/Watchdog Timer Driver

use kernel::platform;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

// Based on the latest commit of OpenTitan supported by tock:
// Refer: https://github.com/lowRISC/opentitan/blob/217a0168ba118503c166a9587819e3811eeb0c0c/hw/ip/aon_timer/rtl/aon_timer_reg_pkg.sv#L136
register_structs! {
    pub AonTimerRegisters {
        //AON_TIMER: Alert Test Register
        (0x000 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        //AON_TIMER: Wakeup Timer Control Register
        (0x004 => wkup_ctrl: ReadWrite<u32, WKUP_CTRL::Register>),
        //AON_TIMER: Wakeup Timer Threshold Register
        (0x008 => wkup_thold: ReadWrite<u32, THRESHOLD::Register>),
        //AON_TIMER: Wakeup Timer Count Register
        (0x00C => wkup_count: ReadWrite<u32, WKUP_COUNT::Register>),
        //AON_TIMER:  Watchdog Timer Write Enable Register [rw0c]
        (0x010 => wdog_regwen: ReadWrite<u32, WDOG_REGWEN::Register>),
        //AON_TIMER: Watchdog Timer Control register
        (0x014 => wdog_ctrl: ReadWrite<u32, WDOG_CTRL::Register>),
        //AON_TIMER: Watchdog Timer Bark Threshold Register
        (0x018 => wdog_bark_thold: ReadWrite<u32, THRESHOLD::Register>),
        //AON_TIMER: Watchdog Timer Bite Threshold Register
        (0x01C => wdog_bite_thold: ReadWrite<u32, THRESHOLD::Register>),
        //AON_TIMER: Watchdog Timer Count Register
        (0x020 => wdog_count: ReadWrite<u32, WDOG_COUNT::Register>),
        //AON_TIMER: Interrupt State Register [rw1c]
        (0x024 => intr_state: ReadWrite<u32, INTR::Register>),
        //AON_TIMER: Interrupt Test Reigster
        (0x028 => intr_test: WriteOnly<u32, INTR::Register>),
        //AON_TIMER: Wakeup Request Status [rw0c]
        (0x02C => wkup_cause: ReadWrite<u32, WKUP_CAUSE::Register>),
        (0x030 => @END),
    }
}

register_bitfields![u32,
    ALERT_TEST[
        FATAL_FAULT OFFSET(0) NUMBITS(1) []
    ],
    WKUP_CTRL[
        ENABLE OFFSET(0) NUMBITS(1) [],
        PRESCALER OFFSET(1) NUMBITS(12) []
    ],
    THRESHOLD[
        THRESHOLD OFFSET(0) NUMBITS(32) []
    ],
    WKUP_COUNT[
        COUNT OFFSET(0) NUMBITS(32) []
    ],
    WDOG_REGWEN[
        REGWEN OFFSET(0) NUMBITS(1) []
    ],
    WDOG_CTRL[
        ENABLE OFFSET(0) NUMBITS(1) [],
        PAUSE_IN_SLEEP OFFSET(1) NUMBITS(1) []
    ],
    WDOG_COUNT[
        COUNT OFFSET(0) NUMBITS(32) [],
    ],
    INTR[
        WKUP_TIMER_EXPIRED OFFSET(0) NUMBITS(1) [],
        WDOG_TIMER_BARK OFFSET(1) NUMBITS(1) []
    ],
    WKUP_CAUSE[
        CAUSE OFFSET(0) NUMBITS(1) [],
    ]
];

pub struct AonTimer {
    registers: StaticRef<AonTimerRegisters>,
    aon_clk_freq: u32, //Hz, this differs for FPGA/Verilator
}

impl AonTimer {
    pub const fn new(base: StaticRef<AonTimerRegisters>, aon_clk_freq: u32) -> AonTimer {
        AonTimer {
            registers: base,
            aon_clk_freq: aon_clk_freq,
        }
    }

    /// Reset both watch dog and wake up timer count values.
    fn reset_timers(&self) {
        let regs = self.registers;
        regs.wkup_count.set(0x00);
        regs.wdog_count.set(0x00);
    }

    /// Start the watchdog counter with pause in sleep
    /// i.e wdog timer is paused when system is sleeping
    fn wdog_start_count(&self) {
        self.registers
            .wdog_ctrl
            .write(WDOG_CTRL::ENABLE::SET + WDOG_CTRL::PAUSE_IN_SLEEP::SET);
    }

    /// Program the desired thresholds in WKUP_THOLD, WDOG_BARK_THOLD and WDOG_BITE_THOLD
    fn set_wdog_thresh(&self) {
        let regs = self.registers;
        // Watchdog period may need to be revised with kernel changes/updates
        // since the watchdog is `tickled()` at the start of every kernel loop
        // see: https://github.com/tock/tock/blob/eb3f7ce59434b7ac1b77ef1ab7dd2afad1a62ac5/kernel/src/kernel.rs#L448
        let bark_cycles = self.ms_to_cycles(500);
        // ~1000ms bite period
        let bite_cycles = bark_cycles.saturating_mul(2);

        regs.wdog_bark_thold
            .write(THRESHOLD::THRESHOLD.val(bark_cycles));
        regs.wdog_bite_thold
            .write(THRESHOLD::THRESHOLD.val(bite_cycles));
    }

    // Reset watch dog timer
    fn wdog_pet(&self) {
        self.registers.wdog_count.set(0x00);
    }

    fn wdog_suspend(&self) {
        self.registers.wdog_ctrl.write(WDOG_CTRL::ENABLE::CLEAR);
    }

    fn wdog_resume(&self) {
        self.registers.wdog_ctrl.write(WDOG_CTRL::ENABLE::SET);
    }

    /// Locks further config to WDOG until next system reset
    fn lock_wdog_conf(&self) {
        self.registers.wdog_regwen.write(WDOG_REGWEN::REGWEN::SET)
    }

    /// Convert microseconds to cycles
    fn ms_to_cycles(&self, ms: u32) -> u32 {
        // 250kHZ CW130 or 125kHz Verilator (as specified in chip config)
        ms.saturating_mul(self.aon_clk_freq).saturating_div(1000)
    }

    fn reset_wkup_count(&self) {
        self.registers.wkup_count.set(0x00);
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let intr = self.registers.intr_state.extract();

        if intr.is_set(INTR::WKUP_TIMER_EXPIRED) {
            // Wake up timer has expired, sw must ack and clear
            regs.wkup_cause.set(0x00);
            regs.wkup_count.set(0x00); // To avoid re-triggers
            self.reset_wkup_count();
            // RW1C, clear the interrupt
            regs.intr_state.write(INTR::WKUP_TIMER_EXPIRED::SET);
        }

        if intr.is_set(INTR::WDOG_TIMER_BARK) {
            // Clear the bark (RW1C) and pet doggo
            regs.intr_state.write(INTR::WDOG_TIMER_BARK::SET);
            self.wdog_pet();
        }
    }
}

impl platform::watchdog::WatchDog for AonTimer {
    /// The always-on timer will run on a ~125KHz (Verilator) or ~250kHz clock.
    /// The timers themselves are 32b wide, giving a maximum timeout
    /// window of roughly ~6 hours. For the wakeup timer, the pre-scaler
    /// extends the maximum timeout to ~1000 days.
    ///
    /// The AON HW_IP has a watchdog and a wake-up timer (counts independantly of eachother),
    /// although struct `AonTimer` implements the wakeup timer functionality,
    /// we only start and use the watchdog in the code below.
    fn setup(&self) {
        // 1. Clear Timers
        self.reset_timers();

        // 2. Set thresholds.
        self.set_wdog_thresh();

        // 3. Commence gaurd duty...
        self.wdog_start_count();

        // 4. Lock watchdog config
        // Preventing firmware from accidentally or maliciously disabling the watchdog,
        // until next system reset.
        self.lock_wdog_conf();
    }

    fn tickle(&self) {
        // Nothing to worry about, good dog...
        self.wdog_pet();
    }

    fn suspend(&self) {
        self.wdog_suspend();
    }

    fn resume(&self) {
        self.wdog_resume();
    }
}
