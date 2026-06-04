// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! System Timer Module (STM) for NXP S32G3.
//!
//! Register definitions and bitfields are taken from the S32G3 Reference
//! Manual, Chapter 41. The STM provides a 32-bit count-up timer with up to
//! four compare channels that assert an interrupt when the counter matches a
//! programmed value. This driver implements the Tock
//! [`Time`](kernel::hil::time::Time),
//! [`Counter`](kernel::hil::time::Counter), and
//! [`Alarm`](kernel::hil::time::Alarm) HIL traits on top of STM channel 0
//! (see RM §41.2, §41.4.1, §41.4.2).
//!
//! Per RM §41.3.1, the channel registers are spaced 16 bytes apart. Channel 0
//! is the only one wired to an alarm client in this driver; the remaining
//! channels are exposed in the register map but are unused.

use kernel::hil::time::{Alarm, AlarmClient, Counter, OverflowClient, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

// Base addresses from the S32G3 memory map (RM §41.1.1).

/// Base address of STM_0 instance.
pub const STM_0_BASE: StaticRef<StmRegisters> =
    unsafe { StaticRef::new(0x4011_C000 as *const StmRegisters) };

/// Base address of STM_1 instance.
pub const STM_1_BASE: StaticRef<StmRegisters> =
    unsafe { StaticRef::new(0x4012_0000 as *const StmRegisters) };

/// Base address of STM_2 instance.
pub const STM_2_BASE: StaticRef<StmRegisters> =
    unsafe { StaticRef::new(0x4012_4000 as *const StmRegisters) };

/// Base address of STM_3 instance.
pub const STM_3_BASE: StaticRef<StmRegisters> =
    unsafe { StaticRef::new(0x4012_8000 as *const StmRegisters) };

// Memory map from RM §41.3.1.
register_structs! {
    pub StmRegisters {
        /// Control Register
        /// RM §41.3.2.
        (0x000 => pub cr: ReadWrite<u32, CR::Register>),
        /// Count Register — 32-bit count-up timer value
        /// RM §41.3.3.
        (0x004 => pub cnt: ReadWrite<u32, CNT::Register>),
        /// Reserved 4-byte gap between CNT and the first channel register
        /// block (RM §41.3.1 memory map).
        (0x008 => _reserved0),
        /// Channel 0 Control Register
        /// RM §41.3.4.
        (0x010 => pub ccr0: ReadWrite<u32, CCR::Register>),
        /// Channel 0 Interrupt Register
        /// RM §41.3.5.
        (0x014 => pub cir0: ReadWrite<u32, CIR::Register>),
        /// Channel 0 Compare Register
        /// RM §41.3.6.
        (0x018 => pub cmp0: ReadWrite<u32, CMP::Register>),
        /// Reserved 4-byte gap between channel 0 and channel 1 register
        /// blocks (RM §41.3.1 memory map).
        (0x01C => _reserved1),
        /// Channel 1 Control Register
        /// RM §41.3.4.
        (0x020 => pub ccr1: ReadWrite<u32, CCR::Register>),
        /// Channel 1 Interrupt Register
        /// RM §41.3.5.
        (0x024 => pub cir1: ReadWrite<u32, CIR::Register>),
        /// Channel 1 Compare Register
        /// RM §41.3.6.
        (0x028 => pub cmp1: ReadWrite<u32, CMP::Register>),
        /// Reserved 4-byte gap between channel 1 and channel 2 register
        /// blocks (RM §41.3.1 memory map).
        (0x02C => _reserved2),
        /// Channel 2 Control Register
        /// RM §41.3.4.
        (0x030 => pub ccr2: ReadWrite<u32, CCR::Register>),
        /// Channel 2 Interrupt Register
        /// RM §41.3.5.
        (0x034 => pub cir2: ReadWrite<u32, CIR::Register>),
        /// Channel 2 Compare Register
        /// RM §41.3.6.
        (0x038 => pub cmp2: ReadWrite<u32, CMP::Register>),
        /// Reserved 4-byte gap between channel 2 and channel 3 register
        /// blocks (RM §41.3.1 memory map).
        (0x03C => _reserved3),
        /// Channel 3 Control Register
        /// RM §41.3.4.
        (0x040 => pub ccr3: ReadWrite<u32, CCR::Register>),
        /// Channel 3 Interrupt Register
        /// RM §41.3.5.
        (0x044 => pub cir3: ReadWrite<u32, CIR::Register>),
        /// Channel 3 Compare Register
        /// RM §41.3.6.
        (0x048 => pub cmp3: ReadWrite<u32, CMP::Register>),
        (0x04C => @END),
    }
}

register_bitfields![u32,
    /// Control Register
    /// RM §41.3.2.
    /// Contains fields for the prescale value, freeze control, and timer
    /// enable.
    CR [
        /// Reserved. Reads return 0 (RM §41.3.2 field `31-16`).
        _RSV_16_31 OFFSET(16) NUMBITS(16) [],
        /// Counter Prescaler. Selects the module clock divide value for the
        /// prescaler (1-256). `0x00` divides by 1, `0x01` by 2, ..., `0xFF` by
        /// 256 (RM §41.3.2 field `15-8 CPS`).
        CPS  OFFSET(8) NUMBITS(8) [],
        /// Reserved. Reads return 0 (RM §41.3.2 field `7-2`).
        _RSV_2_7 OFFSET(2) NUMBITS(6) [],
        /// Freeze. Stops the timer when the chip enters Debug mode
        /// (RM §41.3.2 field `1 FRZ`).
        FRZ  OFFSET(1) NUMBITS(1) [
            /// Timer runs in Debug mode.
            Run = 0,
            /// Timer stops in Debug mode.
            Stop = 1,
        ],
        /// Timer Enable. Enables the module timer
        /// (RM §41.3.2 field `0 TEN`).
        TEN  OFFSET(0) NUMBITS(1) [
            /// Timer disabled.
            Disabled = 0,
            /// Timer enabled.
            Enabled = 1,
        ]
    ],

    /// Count Register
    /// RM §41.3.3.
    /// Holds the 32-bit timer count value, which increments at the module
    /// clock frequency divided by the prescaler.
    CNT [
        /// Timer Count. The time base for all compare channels; the counter
        /// rolls over from `0xFFFF_FFFF` to `0x0000_0000` (RM §41.3.3 field
        /// `31-0 CNT`, RM §41.4.1).
        CNT  OFFSET(0) NUMBITS(32) []
    ],

    /// Channel Control Register
    /// RM §41.3.4.
    /// Enables channel n of the timer.
    CCR [
        /// Reserved. Reads return 0 (RM §41.3.4 field `31-1`).
        _RSV_1_31 OFFSET(1) NUMBITS(31) [],
        /// Channel Enable (RM §41.3.4 field `0 CEN`).
        CEN  OFFSET(0) NUMBITS(1) [
            /// Channel disabled.
            Disabled = 0,
            /// Channel enabled.
            Enabled = 1,
        ]
    ],

    /// Channel Interrupt Register
    /// RM §41.3.5.
    /// Indicates and clears the interrupt flag for channel n of the timer.
    /// The CIF bit is read/write-1-to-clear.
    CIR [
        /// Reserved. Reads return 0 (RM §41.3.5 field `31-1`).
        _RSV_1_31 OFFSET(1) NUMBITS(31) [],
        /// Channel Interrupt Flag. Read indicates whether the channel IRQ is
        /// asserted; writing 1 clears the flag (RM §41.3.5 field `0 CIF`).
        CIF  OFFSET(0) NUMBITS(1) [
            /// Read: IRQ not asserted. Write: no effect.
            NotPending = 0,
            /// Read: IRQ asserted. Write: clear the flag.
            Pending = 1,
        ]
    ],

    /// Channel Compare Register
    /// RM §41.3.6.
    /// The compare value for channel n. When the channel is enabled and CNT
    /// matches this value, STM asserts the channel IRQ and sets CIRn[CIF]
    /// (RM §41.4.2).
    CMP [
        /// Channel Compare value (RM §41.3.6 field `31-0 CMP`).
        CMP  OFFSET(0) NUMBITS(32) []
    ]
];

/// Module clock frequency: 520,833 Hz (per S32G3 clocking for the M7 cores).
/// RM §24 (clocking summary); matches the value programmed by the M7 bare
/// demo bootloader.
pub struct Freq520KHz;
impl kernel::hil::time::Frequency for Freq520KHz {
    fn frequency() -> u32 {
        520_833
    }
}

/// STM driver instance.
///
/// Holds a `StaticRef` to the STM register block and an optional alarm client.
/// Only channel 0 is wired to the alarm client; the remaining channels are
/// accessible through the register map for application use.
pub struct Stm<'a> {
    registers: StaticRef<StmRegisters>,
    client: OptionalCell<&'a dyn AlarmClient>,
}
impl Stm<'_> {
    /// Creates a new STM driver instance bound to the given register block.
    /// The timer is left disabled; call [`Counter::start`] to enable counting.
    pub const fn new(base: StaticRef<StmRegisters>) -> Self {
        Self {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    /// STM channel-0 interrupt service routine.
    ///
    /// Disables the channel, clears the channel interrupt flag (CIR0[CIF] is
    /// write-1-to-clear per RM §41.3.5), and invokes the alarm client. Must
    /// be wired to the channel-0 NVIC vector in the board's chip
    /// configuration.
    pub fn handle_interrupt(&self) {
        // Disable channel
        self.registers.ccr0.modify(CCR::CEN::CLEAR);
        // Clear interrupt flag
        self.registers.cir0.write(CIR::CIF::SET);
        // Notify client
        self.client.map(|client| {
            client.alarm();
        });
    }
}
impl Time for Stm<'_> {
    type Frequency = Freq520KHz;
    type Ticks = Ticks32;
    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.cnt.get())
    }
}
impl<'a> Counter<'a> for Stm<'a> {
    fn set_overflow_client(&self, _client: &'a dyn OverflowClient) {}
    fn start(&self) -> Result<(), kernel::ErrorCode> {
        // Prescaler value 0xFF selects /256 (RM §41.3.2 field `15-8 CPS`).
        self.registers.cr.modify(CR::CPS.val(255) + CR::TEN::SET);
        Ok(())
    }
    fn stop(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cr.modify(CR::TEN::CLEAR);
        Ok(())
    }
    fn reset(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cnt.set(0);
        Ok(())
    }
    fn is_running(&self) -> bool {
        self.registers.cr.is_set(CR::TEN)
    }
}
impl<'a> Alarm<'a> for Stm<'a> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }
    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();
        if !now.within_range(reference, expire) {
            expire = now;
        }
        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }
        // Disable channel first
        self.registers.ccr0.modify(CCR::CEN::CLEAR);
        // Clear any pending interrupt
        self.registers.cir0.write(CIR::CIF::SET);
        // Set compare value
        self.registers.cmp0.set(expire.into_u32());
        // Re-enable channel
        self.registers.ccr0.modify(CCR::CEN::SET);
    }
    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.cmp0.get())
    }
    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.ccr0.modify(CCR::CEN::CLEAR);
        self.registers.cir0.write(CIR::CIF::SET);
        Ok(())
    }
    fn minimum_dt(&self) -> Self::Ticks {
        // Two timer ticks for the compare to settle plus one for skew
        // (see RM §41.4.2, §41.7.3).
        Self::Ticks::from(3)
    }
    fn is_armed(&self) -> bool {
        self.registers.ccr0.is_set(CCR::CEN)
    }
}
