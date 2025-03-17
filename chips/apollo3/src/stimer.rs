// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! STimer driver for the Apollo3

use kernel::hil::time::{
    Alarm, AlarmClient, Counter, Freq16KHz, OverflowClient, Ticks, Ticks32, Time,
};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const STIMER_BASE: StaticRef<STimerRegisters> =
    unsafe { StaticRef::new(0x4000_8000 as *const STimerRegisters) };

register_structs! {
    pub STimerRegisters {
        (0x000 => _reserved0),
        (0x140 => stcfg: ReadWrite<u32, STCFG::Register>),
        (0x144 => sttmr: ReadWrite<u32, STTMR::Register>),
        (0x148 => capturecontrol: ReadWrite<u32, CAPTURECONTROL::Register>),
        (0x14C => _reserved1),
        (0x150 => scmpr: [ReadWrite<u32, SCMPR::Register>; 8]),
        (0x170 => _reserved2),
        (0x1E0 => scapt: [ReadWrite<u32, SCAPT::Register>; 4]),
        (0x1F0 => snvr: [ReadWrite<u32, SNVR::Register>; 4]),
        (0x200 => _reserved3),
        (0x300 => stminten: ReadWrite<u32, STMINT::Register>),
        (0x304 => stmintstat: ReadWrite<u32, STMINT::Register>),
        (0x308 => stmintclr: ReadWrite<u32, STMINT::Register>),
        (0x30C => stmintset: ReadWrite<u32, STMINT::Register>),
        (0x310 => @END),
    }
}

register_bitfields![u32,
    STCFG [
        CLKSEL OFFSET(0) NUMBITS(4) [
            NOCLK = 0x0,
            HRFC_DIV16 = 0x1,
            HRFC_DIV256 = 0x2,
            XTAL_DIV1 = 0x3,
            XTAL_DIV2 = 0x4,
            XTAL_DIV32 = 0x5,
            LFRC_DIV1 = 0x6,
            CTIMER0A = 0x7,
            CTIMER0B = 0x8
        ],
        COMPARE_A_EN OFFSET(8) NUMBITS(1) [],
        COMPARE_B_EN OFFSET(9) NUMBITS(1) [],
        COMPARE_C_EN OFFSET(10) NUMBITS(1) [],
        COMPARE_D_EN OFFSET(11) NUMBITS(1) [],
        COMPARE_E_EN OFFSET(12) NUMBITS(1) [],
        COMPARE_F_EN OFFSET(13) NUMBITS(1) [],
        COMPARE_G_EN OFFSET(14) NUMBITS(1) [],
        COMPARE_H_EN OFFSET(15) NUMBITS(1) [],
        CLEAR OFFSET(30) NUMBITS(1) [],
        FREEZE OFFSET(31) NUMBITS(1) []
    ],
    STTMR [
        STTMR OFFSET(0) NUMBITS(31) []
    ],
    CAPTURECONTROL [
        CAPTURE0 OFFSET(0) NUMBITS(1) [],
        CAPTURE1 OFFSET(1) NUMBITS(1) [],
        CAPTURE2 OFFSET(2) NUMBITS(1) [],
        CAPTURE3 OFFSET(3) NUMBITS(1) []
    ],
    SCMPR [
        SCMPR OFFSET(0) NUMBITS(31) []
    ],
    SCAPT [
        SCATP OFFSET(0) NUMBITS(31) []
    ],
    SNVR [
        SNVR OFFSET(0) NUMBITS(31) []
    ],
    STMINT [
        COMPAREA OFFSET(0) NUMBITS(1) [],
        COMPAREB OFFSET(1) NUMBITS(1) [],
        COMPAREC OFFSET(2) NUMBITS(1) [],
        COMPARED OFFSET(3) NUMBITS(1) [],
        COMPAREE OFFSET(4) NUMBITS(1) [],
        COMPAREF OFFSET(5) NUMBITS(1) [],
        COMPAREG OFFSET(6) NUMBITS(1) [],
        COMPAREH OFFSET(7) NUMBITS(1) [],
        OVERFLOW OFFSET(8) NUMBITS(1) [],
        CAPTUREA OFFSET(9) NUMBITS(1) [],
        CAPTUREB OFFSET(10) NUMBITS(1) [],
        CAPTUREC OFFSET(11) NUMBITS(1) [],
        CAPTURED OFFSET(12) NUMBITS(1) []
    ]
];

pub struct STimer<'a> {
    registers: StaticRef<STimerRegisters>,
    client: OptionalCell<&'a dyn AlarmClient>,
}

impl<'a> STimer<'a> {
    // Unsafe bc of use of STIMER_BASE internally
    pub fn new() -> STimer<'a> {
        let timer = STimer {
            registers: STIMER_BASE,
            client: OptionalCell::empty(),
        };

        // Reset so that time starts at 0
        let _ = timer.reset();

        timer
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;

        // Disable timer
        regs.stcfg
            .modify(STCFG::COMPARE_A_EN::CLEAR + STCFG::COMPARE_B_EN::CLEAR);

        // Disable interrupt
        regs.stminten
            .modify(STMINT::COMPAREA::CLEAR + STMINT::COMPAREB::CLEAR);

        // Clear interrupt
        regs.stmintclr
            .modify(STMINT::COMPAREA::SET + STMINT::COMPAREB::SET);

        self.client.map(|client| client.alarm());
    }
}

impl Time for STimer<'_> {
    type Frequency = Freq16KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.sttmr.get())
    }
}

impl<'a> Counter<'a> for STimer<'a> {
    fn set_overflow_client(&self, _client: &'a dyn OverflowClient) {
        //self.overflow_client.set(client);
    }

    fn start(&self) -> Result<(), ErrorCode> {
        // Set the clock source
        self.registers.stcfg.write(STCFG::CLKSEL::XTAL_DIV2);
        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::BUSY)
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        self.registers.stcfg.write(STCFG::CLEAR::SET);
        Ok(())
    }

    fn is_running(&self) -> bool {
        let regs = self.registers;
        regs.stcfg.matches_any(&[STCFG::CLKSEL::XTAL_DIV2])
    }
}

impl<'a> Alarm<'a> for STimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let regs = self.registers;
        let now = self.now();
        // Errata 4.22: Sometimes the clock can increment twice
        // This means the timer occurs earlier then actually requested
        // From testing this scaling results in the correct time, so we
        // scale the requested ticks to give us an accurate alarm.
        let scaled_time = Self::Ticks::from(((dt.into_u32() as u64 * 1000) / (1000 - 32)) as u32);
        let expire = reference.wrapping_add(scaled_time);

        // Disable the compare
        regs.stcfg
            .modify(STCFG::COMPARE_A_EN::CLEAR + STCFG::COMPARE_B_EN::CLEAR);

        // Enable interrupts
        regs.stminten
            .modify(STMINT::COMPAREA::SET + STMINT::COMPAREB::SET);

        // Check if the alarm has already expired or if it will expire before we set
        // the compare.
        if !now.within_range(reference, expire) || expire.wrapping_sub(now) < self.minimum_dt() {
            // The alarm has already expired!
            // Let's set the interrupt manually
            regs.stcfg.modify(STCFG::COMPARE_A_EN::SET);
            regs.stmintset.modify(STMINT::COMPAREA::SET);
            return;
        }

        // Set the delta, this can take a few goes
        // See Errata 4.14 at at https://ambiq.com/wp-content/uploads/2022/01/Apollo3-Blue-Errata-List.pdf
        let mut timer_delta = expire.wrapping_sub(now);
        let mut tries = 0;

        // Apollo3 Blue Datasheet 14.1: 'Only offsets from "NOW" are written to
        // comparator registers.'
        while Self::Ticks::from(regs.scmpr[0].get()) != expire && tries < 5 {
            regs.scmpr[0].set(timer_delta.into_u32());
            tries += 1;
        }

        // Timers can be missed, so set a second one a little larger
        // See Errata 4.22 at at https://ambiq.com/wp-content/uploads/2022/01/Apollo3-Blue-Errata-List.pdf
        timer_delta = timer_delta.wrapping_add(1.into());
        tries = 0;

        while Self::Ticks::from(regs.scmpr[1].get()) != expire && tries < 5 {
            regs.scmpr[1].set(timer_delta.into_u32());
            tries += 1;
        }

        // Enable the compare
        regs.stcfg
            .modify(STCFG::COMPARE_A_EN::SET + STCFG::COMPARE_B_EN::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        let regs = self.registers;
        Self::Ticks::from(regs.scmpr[0].get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        let regs = self.registers;

        regs.stcfg.modify(
            STCFG::COMPARE_A_EN::CLEAR
                + STCFG::COMPARE_B_EN::CLEAR
                + STCFG::COMPARE_C_EN::CLEAR
                + STCFG::COMPARE_D_EN::CLEAR
                + STCFG::COMPARE_E_EN::CLEAR
                + STCFG::COMPARE_F_EN::CLEAR
                + STCFG::COMPARE_G_EN::CLEAR
                + STCFG::COMPARE_H_EN::CLEAR,
        );
        Ok(())
    }

    fn is_armed(&self) -> bool {
        let regs = self.registers;

        regs.stcfg.read(STCFG::COMPARE_A_EN) != 0
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(5)
    }
}
