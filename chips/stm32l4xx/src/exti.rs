// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use cortexm4f::support::with_interrupts_disabled;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

use crate::gpio;
use crate::syscfg;

/// External interrupt/event controller (STM32L476)
/// Reference: RM0351, EXTI chapter.
#[repr(C)]
struct ExtiRegisters {
    /// Interrupt mask register 1 (lines 0..31)
    imr1: ReadWrite<u32, IMR1::Register>,
    /// Event mask register 1 (lines 0..31)
    emr1: ReadWrite<u32, EMR1::Register>,
    /// Rising trigger selection register 1 (lines 0..31)
    rtsr1: ReadWrite<u32, RTSR1::Register>,
    /// Falling trigger selection register 1 (lines 0..31)
    ftsr1: ReadWrite<u32, FTSR1::Register>,
    /// Software interrupt event register 1 (lines 0..31)
    swier1: ReadWrite<u32, SWIER1::Register>,
    /// Pending register 1 (lines 0..31)
    pr1: ReadWrite<u32, PR1::Register>,
    /// Interrupt mask register 2 (higher lines)
    imr2: ReadWrite<u32, IMR2::Register>,
    /// Event mask register 2 (higher lines)
    emr2: ReadWrite<u32, EMR2::Register>,
    /// Rising trigger selection register 2 (higher lines)
    rtsr2: ReadWrite<u32, RTSR2::Register>,
    /// Falling trigger selection register 2 (higher lines)
    ftsr2: ReadWrite<u32, FTSR2::Register>,
    /// Software interrupt event register 2 (higher lines)
    swier2: ReadWrite<u32, SWIER2::Register>,
    /// Pending register 2 (higher lines)
    pr2: ReadWrite<u32, PR2::Register>,
}

register_bitfields![u32,
    IMR1 [
        MR0 OFFSET(0) NUMBITS(1) [],
        MR1 OFFSET(1) NUMBITS(1) [],
        MR2 OFFSET(2) NUMBITS(1) [],
        MR3 OFFSET(3) NUMBITS(1) [],
        MR4 OFFSET(4) NUMBITS(1) [],
        MR5 OFFSET(5) NUMBITS(1) [],
        MR6 OFFSET(6) NUMBITS(1) [],
        MR7 OFFSET(7) NUMBITS(1) [],
        MR8 OFFSET(8) NUMBITS(1) [],
        MR9 OFFSET(9) NUMBITS(1) [],
        MR10 OFFSET(10) NUMBITS(1) [],
        MR11 OFFSET(11) NUMBITS(1) [],
        MR12 OFFSET(12) NUMBITS(1) [],
        MR13 OFFSET(13) NUMBITS(1) [],
        MR14 OFFSET(14) NUMBITS(1) [],
        MR15 OFFSET(15) NUMBITS(1) []
    ],
    EMR1 [
        MR0 OFFSET(0) NUMBITS(1) [],
        MR1 OFFSET(1) NUMBITS(1) [],
        MR2 OFFSET(2) NUMBITS(1) [],
        MR3 OFFSET(3) NUMBITS(1) [],
        MR4 OFFSET(4) NUMBITS(1) [],
        MR5 OFFSET(5) NUMBITS(1) [],
        MR6 OFFSET(6) NUMBITS(1) [],
        MR7 OFFSET(7) NUMBITS(1) [],
        MR8 OFFSET(8) NUMBITS(1) [],
        MR9 OFFSET(9) NUMBITS(1) [],
        MR10 OFFSET(10) NUMBITS(1) [],
        MR11 OFFSET(11) NUMBITS(1) [],
        MR12 OFFSET(12) NUMBITS(1) [],
        MR13 OFFSET(13) NUMBITS(1) [],
        MR14 OFFSET(14) NUMBITS(1) [],
        MR15 OFFSET(15) NUMBITS(1) []
    ],
    RTSR1 [
        TR0 OFFSET(0) NUMBITS(1) [],
        TR1 OFFSET(1) NUMBITS(1) [],
        TR2 OFFSET(2) NUMBITS(1) [],
        TR3 OFFSET(3) NUMBITS(1) [],
        TR4 OFFSET(4) NUMBITS(1) [],
        TR5 OFFSET(5) NUMBITS(1) [],
        TR6 OFFSET(6) NUMBITS(1) [],
        TR7 OFFSET(7) NUMBITS(1) [],
        TR8 OFFSET(8) NUMBITS(1) [],
        TR9 OFFSET(9) NUMBITS(1) [],
        TR10 OFFSET(10) NUMBITS(1) [],
        TR11 OFFSET(11) NUMBITS(1) [],
        TR12 OFFSET(12) NUMBITS(1) [],
        TR13 OFFSET(13) NUMBITS(1) [],
        TR14 OFFSET(14) NUMBITS(1) [],
        TR15 OFFSET(15) NUMBITS(1) []
    ],
    FTSR1 [
        TR0 OFFSET(0) NUMBITS(1) [],
        TR1 OFFSET(1) NUMBITS(1) [],
        TR2 OFFSET(2) NUMBITS(1) [],
        TR3 OFFSET(3) NUMBITS(1) [],
        TR4 OFFSET(4) NUMBITS(1) [],
        TR5 OFFSET(5) NUMBITS(1) [],
        TR6 OFFSET(6) NUMBITS(1) [],
        TR7 OFFSET(7) NUMBITS(1) [],
        TR8 OFFSET(8) NUMBITS(1) [],
        TR9 OFFSET(9) NUMBITS(1) [],
        TR10 OFFSET(10) NUMBITS(1) [],
        TR11 OFFSET(11) NUMBITS(1) [],
        TR12 OFFSET(12) NUMBITS(1) [],
        TR13 OFFSET(13) NUMBITS(1) [],
        TR14 OFFSET(14) NUMBITS(1) [],
        TR15 OFFSET(15) NUMBITS(1) []
    ],
    SWIER1 [
        SWIER0 OFFSET(0) NUMBITS(1) [],
        SWIER1 OFFSET(1) NUMBITS(1) [],
        SWIER2 OFFSET(2) NUMBITS(1) [],
        SWIER3 OFFSET(3) NUMBITS(1) [],
        SWIER4 OFFSET(4) NUMBITS(1) [],
        SWIER5 OFFSET(5) NUMBITS(1) [],
        SWIER6 OFFSET(6) NUMBITS(1) [],
        SWIER7 OFFSET(7) NUMBITS(1) [],
        SWIER8 OFFSET(8) NUMBITS(1) [],
        SWIER9 OFFSET(9) NUMBITS(1) [],
        SWIER10 OFFSET(10) NUMBITS(1) [],
        SWIER11 OFFSET(11) NUMBITS(1) [],
        SWIER12 OFFSET(12) NUMBITS(1) [],
        SWIER13 OFFSET(13) NUMBITS(1) [],
        SWIER14 OFFSET(14) NUMBITS(1) [],
        SWIER15 OFFSET(15) NUMBITS(1) []
    ],
    PR1 [
        PR0 OFFSET(0) NUMBITS(1) [],
        PR1 OFFSET(1) NUMBITS(1) [],
        PR2 OFFSET(2) NUMBITS(1) [],
        PR3 OFFSET(3) NUMBITS(1) [],
        PR4 OFFSET(4) NUMBITS(1) [],
        PR5 OFFSET(5) NUMBITS(1) [],
        PR6 OFFSET(6) NUMBITS(1) [],
        PR7 OFFSET(7) NUMBITS(1) [],
        PR8 OFFSET(8) NUMBITS(1) [],
        PR9 OFFSET(9) NUMBITS(1) [],
        PR10 OFFSET(10) NUMBITS(1) [],
        PR11 OFFSET(11) NUMBITS(1) [],
        PR12 OFFSET(12) NUMBITS(1) [],
        PR13 OFFSET(13) NUMBITS(1) [],
        PR14 OFFSET(14) NUMBITS(1) [],
        PR15 OFFSET(15) NUMBITS(1) []
    ],
    // Minimal higher-line registers placeholders (not actively used yet)
    IMR2 [ MR32 OFFSET(0) NUMBITS(1) [] ],
    EMR2 [ MR32 OFFSET(0) NUMBITS(1) [] ],
    RTSR2 [ TR32 OFFSET(0) NUMBITS(1) [] ],
    FTSR2 [ TR32 OFFSET(0) NUMBITS(1) [] ],
    SWIER2 [ SWIER32 OFFSET(0) NUMBITS(1) [] ],
    PR2 [ PR32 OFFSET(0) NUMBITS(1) [] ],
];

const EXTI_BASE: StaticRef<ExtiRegisters> =
    unsafe { StaticRef::new(0x40010400 as *const ExtiRegisters) };

/// EXTI overview (STM32L476):
///
/// External interrupt/event lines 0–15 correspond directly to the 16 GPIO pins
/// on a port. They are presented to the NVIC using a mix of dedicated and
/// grouped IRQs:
///
///  - Lines 0,1,2,3,4 -> `EXTI0`..`EXTI4` (NVIC IRQs 6–10)
///  - Lines 5–9       -> grouped into `EXTI9_5` (NVIC IRQ 23)
///  - Lines 10–15     -> grouped into `EXTI15_10` (NVIC IRQ 40)
///
/// Internal (non‑GPIO) EXTI lines above 15 (e.g. PVD/PVM, RTC events, USB, TAMP,
/// etc.) are routed to other dedicated NVIC IRQs (see `nvic.rs`) and are not
/// currently modelled in this driver; we restrict `LineId` to 0–15 for GPIO use.
///
/// Pending bits are level sources: a set bit in PR1 keeps the NVIC IRQ asserted
/// until software writes 1 to clear it. The top‑half handler must therefore
/// acknowledge (clear) the source to avoid repeated interrupts.
///
/// `EXTI_EVENTS` captures which GPIO line(s) were pending before they were
/// cleared, allowing deferred processing outside the immediate ISR if desired.
///
/// Reference: RM0351, EXTI chapter (block diagram & register descriptions).
#[no_mangle]
#[used]
pub static mut EXTI_EVENTS: u32 = 0;

enum_from_primitive! {
    #[repr(u8)]
    #[derive(Copy, Clone)]
    pub enum LineId {
        Exti0 = 0,
        Exti1 = 1,
        Exti2 = 2,
        Exti3 = 3,
        Exti4 = 4,
        Exti5 = 5,
        Exti6 = 6,
        Exti7 = 7,
        Exti8 = 8,
        Exti9 = 9,
        Exti10 = 10,
        Exti11 = 11,
        Exti12 = 12,
        Exti13 = 13,
        Exti14 = 14,
        Exti15 = 15,
    }
}

// `line_gpiopin_map` is used to call `handle_interrupt()` on the pin.
pub struct Exti<'a> {
    registers: StaticRef<ExtiRegisters>,
    clock: ExtiClock<'a>,
    line_gpiopin_map: [OptionalCell<&'static gpio::Pin<'static>>; 16],
    syscfg: &'a syscfg::Syscfg<'a>,
}

impl<'a> Exti<'a> {
    pub const fn new(syscfg: &'a syscfg::Syscfg<'a>) -> Self {
        Self {
            registers: EXTI_BASE,
            clock: ExtiClock(syscfg),
            line_gpiopin_map: [
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
            ],
            syscfg,
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn associate_line_gpiopin(&self, lineid: LineId, pin: &'static gpio::Pin<'static>) {
        self.line_gpiopin_map[usize::from(lineid as u8)].set(pin);
        self.syscfg.configure_interrupt(pin.get_pinid());
        pin.set_exti_lineid(lineid);

        // By default, all interrupts are masked. But, this will ensure that it
        // is really the case.
        self.mask_interrupt(lineid);
    }

    pub fn mask_interrupt(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.imr1.modify(IMR1::MR0::CLEAR),
            LineId::Exti1 => self.registers.imr1.modify(IMR1::MR1::CLEAR),
            LineId::Exti2 => self.registers.imr1.modify(IMR1::MR2::CLEAR),
            LineId::Exti3 => self.registers.imr1.modify(IMR1::MR3::CLEAR),
            LineId::Exti4 => self.registers.imr1.modify(IMR1::MR4::CLEAR),
            LineId::Exti5 => self.registers.imr1.modify(IMR1::MR5::CLEAR),
            LineId::Exti6 => self.registers.imr1.modify(IMR1::MR6::CLEAR),
            LineId::Exti7 => self.registers.imr1.modify(IMR1::MR7::CLEAR),
            LineId::Exti8 => self.registers.imr1.modify(IMR1::MR8::CLEAR),
            LineId::Exti9 => self.registers.imr1.modify(IMR1::MR9::CLEAR),
            LineId::Exti10 => self.registers.imr1.modify(IMR1::MR10::CLEAR),
            LineId::Exti11 => self.registers.imr1.modify(IMR1::MR11::CLEAR),
            LineId::Exti12 => self.registers.imr1.modify(IMR1::MR12::CLEAR),
            LineId::Exti13 => self.registers.imr1.modify(IMR1::MR13::CLEAR),
            LineId::Exti14 => self.registers.imr1.modify(IMR1::MR14::CLEAR),
            LineId::Exti15 => self.registers.imr1.modify(IMR1::MR15::CLEAR),
        }
    }

    pub fn unmask_interrupt(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.imr1.modify(IMR1::MR0::SET),
            LineId::Exti1 => self.registers.imr1.modify(IMR1::MR1::SET),
            LineId::Exti2 => self.registers.imr1.modify(IMR1::MR2::SET),
            LineId::Exti3 => self.registers.imr1.modify(IMR1::MR3::SET),
            LineId::Exti4 => self.registers.imr1.modify(IMR1::MR4::SET),
            LineId::Exti5 => self.registers.imr1.modify(IMR1::MR5::SET),
            LineId::Exti6 => self.registers.imr1.modify(IMR1::MR6::SET),
            LineId::Exti7 => self.registers.imr1.modify(IMR1::MR7::SET),
            LineId::Exti8 => self.registers.imr1.modify(IMR1::MR8::SET),
            LineId::Exti9 => self.registers.imr1.modify(IMR1::MR9::SET),
            LineId::Exti10 => self.registers.imr1.modify(IMR1::MR10::SET),
            LineId::Exti11 => self.registers.imr1.modify(IMR1::MR11::SET),
            LineId::Exti12 => self.registers.imr1.modify(IMR1::MR12::SET),
            LineId::Exti13 => self.registers.imr1.modify(IMR1::MR13::SET),
            LineId::Exti14 => self.registers.imr1.modify(IMR1::MR14::SET),
            LineId::Exti15 => self.registers.imr1.modify(IMR1::MR15::SET),
        }
    }

    // Pending clear happens by writing 1
    pub fn clear_pending(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.pr1.write(PR1::PR0::SET),
            LineId::Exti1 => self.registers.pr1.write(PR1::PR1::SET),
            LineId::Exti2 => self.registers.pr1.write(PR1::PR2::SET),
            LineId::Exti3 => self.registers.pr1.write(PR1::PR3::SET),
            LineId::Exti4 => self.registers.pr1.write(PR1::PR4::SET),
            LineId::Exti5 => self.registers.pr1.write(PR1::PR5::SET),
            LineId::Exti6 => self.registers.pr1.write(PR1::PR6::SET),
            LineId::Exti7 => self.registers.pr1.write(PR1::PR7::SET),
            LineId::Exti8 => self.registers.pr1.write(PR1::PR8::SET),
            LineId::Exti9 => self.registers.pr1.write(PR1::PR9::SET),
            LineId::Exti10 => self.registers.pr1.write(PR1::PR10::SET),
            LineId::Exti11 => self.registers.pr1.write(PR1::PR11::SET),
            LineId::Exti12 => self.registers.pr1.write(PR1::PR12::SET),
            LineId::Exti13 => self.registers.pr1.write(PR1::PR13::SET),
            LineId::Exti14 => self.registers.pr1.write(PR1::PR14::SET),
            LineId::Exti15 => self.registers.pr1.write(PR1::PR15::SET),
        }
    }

    pub fn is_pending(&self, lineid: LineId) -> bool {
        let val = match lineid {
            LineId::Exti0 => self.registers.pr1.read(PR1::PR0),
            LineId::Exti1 => self.registers.pr1.read(PR1::PR1),
            LineId::Exti2 => self.registers.pr1.read(PR1::PR2),
            LineId::Exti3 => self.registers.pr1.read(PR1::PR3),
            LineId::Exti4 => self.registers.pr1.read(PR1::PR4),
            LineId::Exti5 => self.registers.pr1.read(PR1::PR5),
            LineId::Exti6 => self.registers.pr1.read(PR1::PR6),
            LineId::Exti7 => self.registers.pr1.read(PR1::PR7),
            LineId::Exti8 => self.registers.pr1.read(PR1::PR8),
            LineId::Exti9 => self.registers.pr1.read(PR1::PR9),
            LineId::Exti10 => self.registers.pr1.read(PR1::PR10),
            LineId::Exti11 => self.registers.pr1.read(PR1::PR11),
            LineId::Exti12 => self.registers.pr1.read(PR1::PR12),
            LineId::Exti13 => self.registers.pr1.read(PR1::PR13),
            LineId::Exti14 => self.registers.pr1.read(PR1::PR14),
            LineId::Exti15 => self.registers.pr1.read(PR1::PR15),
        };
        val > 0
    }

    pub fn select_rising_trigger(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.rtsr1.modify(RTSR1::TR0::SET),
            LineId::Exti1 => self.registers.rtsr1.modify(RTSR1::TR1::SET),
            LineId::Exti2 => self.registers.rtsr1.modify(RTSR1::TR2::SET),
            LineId::Exti3 => self.registers.rtsr1.modify(RTSR1::TR3::SET),
            LineId::Exti4 => self.registers.rtsr1.modify(RTSR1::TR4::SET),
            LineId::Exti5 => self.registers.rtsr1.modify(RTSR1::TR5::SET),
            LineId::Exti6 => self.registers.rtsr1.modify(RTSR1::TR6::SET),
            LineId::Exti7 => self.registers.rtsr1.modify(RTSR1::TR7::SET),
            LineId::Exti8 => self.registers.rtsr1.modify(RTSR1::TR8::SET),
            LineId::Exti9 => self.registers.rtsr1.modify(RTSR1::TR9::SET),
            LineId::Exti10 => self.registers.rtsr1.modify(RTSR1::TR10::SET),
            LineId::Exti11 => self.registers.rtsr1.modify(RTSR1::TR11::SET),
            LineId::Exti12 => self.registers.rtsr1.modify(RTSR1::TR12::SET),
            LineId::Exti13 => self.registers.rtsr1.modify(RTSR1::TR13::SET),
            LineId::Exti14 => self.registers.rtsr1.modify(RTSR1::TR14::SET),
            LineId::Exti15 => self.registers.rtsr1.modify(RTSR1::TR15::SET),
        }
    }

    pub fn deselect_rising_trigger(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.rtsr1.modify(RTSR1::TR0::CLEAR),
            LineId::Exti1 => self.registers.rtsr1.modify(RTSR1::TR1::CLEAR),
            LineId::Exti2 => self.registers.rtsr1.modify(RTSR1::TR2::CLEAR),
            LineId::Exti3 => self.registers.rtsr1.modify(RTSR1::TR3::CLEAR),
            LineId::Exti4 => self.registers.rtsr1.modify(RTSR1::TR4::CLEAR),
            LineId::Exti5 => self.registers.rtsr1.modify(RTSR1::TR5::CLEAR),
            LineId::Exti6 => self.registers.rtsr1.modify(RTSR1::TR6::CLEAR),
            LineId::Exti7 => self.registers.rtsr1.modify(RTSR1::TR7::CLEAR),
            LineId::Exti8 => self.registers.rtsr1.modify(RTSR1::TR8::CLEAR),
            LineId::Exti9 => self.registers.rtsr1.modify(RTSR1::TR9::CLEAR),
            LineId::Exti10 => self.registers.rtsr1.modify(RTSR1::TR10::CLEAR),
            LineId::Exti11 => self.registers.rtsr1.modify(RTSR1::TR11::CLEAR),
            LineId::Exti12 => self.registers.rtsr1.modify(RTSR1::TR12::CLEAR),
            LineId::Exti13 => self.registers.rtsr1.modify(RTSR1::TR13::CLEAR),
            LineId::Exti14 => self.registers.rtsr1.modify(RTSR1::TR14::CLEAR),
            LineId::Exti15 => self.registers.rtsr1.modify(RTSR1::TR15::CLEAR),
        }
    }

    pub fn select_falling_trigger(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.ftsr1.modify(FTSR1::TR0::SET),
            LineId::Exti1 => self.registers.ftsr1.modify(FTSR1::TR1::SET),
            LineId::Exti2 => self.registers.ftsr1.modify(FTSR1::TR2::SET),
            LineId::Exti3 => self.registers.ftsr1.modify(FTSR1::TR3::SET),
            LineId::Exti4 => self.registers.ftsr1.modify(FTSR1::TR4::SET),
            LineId::Exti5 => self.registers.ftsr1.modify(FTSR1::TR5::SET),
            LineId::Exti6 => self.registers.ftsr1.modify(FTSR1::TR6::SET),
            LineId::Exti7 => self.registers.ftsr1.modify(FTSR1::TR7::SET),
            LineId::Exti8 => self.registers.ftsr1.modify(FTSR1::TR8::SET),
            LineId::Exti9 => self.registers.ftsr1.modify(FTSR1::TR9::SET),
            LineId::Exti10 => self.registers.ftsr1.modify(FTSR1::TR10::SET),
            LineId::Exti11 => self.registers.ftsr1.modify(FTSR1::TR11::SET),
            LineId::Exti12 => self.registers.ftsr1.modify(FTSR1::TR12::SET),
            LineId::Exti13 => self.registers.ftsr1.modify(FTSR1::TR13::SET),
            LineId::Exti14 => self.registers.ftsr1.modify(FTSR1::TR14::SET),
            LineId::Exti15 => self.registers.ftsr1.modify(FTSR1::TR15::SET),
        }
    }

    pub fn deselect_falling_trigger(&self, lineid: LineId) {
        match lineid {
            LineId::Exti0 => self.registers.ftsr1.modify(FTSR1::TR0::CLEAR),
            LineId::Exti1 => self.registers.ftsr1.modify(FTSR1::TR1::CLEAR),
            LineId::Exti2 => self.registers.ftsr1.modify(FTSR1::TR2::CLEAR),
            LineId::Exti3 => self.registers.ftsr1.modify(FTSR1::TR3::CLEAR),
            LineId::Exti4 => self.registers.ftsr1.modify(FTSR1::TR4::CLEAR),
            LineId::Exti5 => self.registers.ftsr1.modify(FTSR1::TR5::CLEAR),
            LineId::Exti6 => self.registers.ftsr1.modify(FTSR1::TR6::CLEAR),
            LineId::Exti7 => self.registers.ftsr1.modify(FTSR1::TR7::CLEAR),
            LineId::Exti8 => self.registers.ftsr1.modify(FTSR1::TR8::CLEAR),
            LineId::Exti9 => self.registers.ftsr1.modify(FTSR1::TR9::CLEAR),
            LineId::Exti10 => self.registers.ftsr1.modify(FTSR1::TR10::CLEAR),
            LineId::Exti11 => self.registers.ftsr1.modify(FTSR1::TR11::CLEAR),
            LineId::Exti12 => self.registers.ftsr1.modify(FTSR1::TR12::CLEAR),
            LineId::Exti13 => self.registers.ftsr1.modify(FTSR1::TR13::CLEAR),
            LineId::Exti14 => self.registers.ftsr1.modify(FTSR1::TR14::CLEAR),
            LineId::Exti15 => self.registers.ftsr1.modify(FTSR1::TR15::CLEAR),
        }
    }

    pub fn handle_interrupt(&self) {
        let mut exti_pr: u32 = 0;

        // Read the `EXTI_PR` register and toggle the appropriate bits in
        // `exti_pr`. Once that is done, write the value of `exti_pr` back. We
        // can have a situation where memory value of `EXTI_PR` could have
        // changed due to an external interrupt. `EXTI_PR` is a read/clear write
        // 1 register (`rc_w1`). So, we only clear bits whose value has been
        // transferred to `exti_pr`.
        unsafe {
            with_interrupts_disabled(|| {
                exti_pr = self.registers.pr1.get();
                self.registers.pr1.set(exti_pr);
            });
        }

        // ignore the "reserved" EXTI bits. Use bits [22:0]. See `EXTI_PR` for
        // details.
        exti_pr &= 0x007fffff;

        let mut flagged_bit = 0;

        // stay in loop until we have processed all the flagged event bits
        while exti_pr != 0 {
            if (exti_pr & 0b1) != 0 {
                if let Some(d) = LineId::from_u8(flagged_bit) {
                    self.line_gpiopin_map[usize::from(d as u8)].map(|pin| pin.handle_interrupt());
                }
            }
            // move to next bit
            flagged_bit += 1;
            exti_pr >>= 1;
        }
    }
}

/// Exti peripheral is clocked using PCLK2. However, PCLK2 does not seem to be
/// gated. The configuration registers for Exti is in Syscfg, so we need to
/// enable clock to Syscfg, when using Exti.
struct ExtiClock<'a>(&'a syscfg::Syscfg<'a>);

impl ClockInterface for ExtiClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled_clock()
    }

    fn enable(&self) {
        self.0.enable_clock();
    }

    fn disable(&self) {
        self.0.disable_clock();
    }
}
