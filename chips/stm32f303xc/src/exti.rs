use cortexm4::support::atomic;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

use crate::gpio;
use crate::syscfg;

/// External interrupt/event controller
#[repr(C)]
struct ExtiRegisters {
    /// Interrupt mask register (EXTI_IMR1)
    imr1: ReadWrite<u32, IMR1::Register>,
    /// Event mask register (EXTI_EMR1)
    emr1: ReadWrite<u32, EMR1::Register>,
    /// Rising Trigger selection register (EXTI_RTSR1)
    rtsr1: ReadWrite<u32, RTSR1::Register>,
    /// Falling Trigger selection register (EXTI_FTSR1)
    ftsr1: ReadWrite<u32, FTSR1::Register>,
    /// Software interrupt event register (EXTI_SWIER1)
    swier1: ReadWrite<u32, SWIER1::Register>,
    /// Pending register (EXTI_PR1)
    pr1: ReadWrite<u32, PR1::Register>,

    /// Interrupt mask register (EXTI_IMR1)
    imr2: ReadWrite<u32, IMR2::Register>,
    /// Event mask register (EXTI_EMR1)
    emr2: ReadWrite<u32, EMR2::Register>,
    /// Rising Trigger selection register (EXTI_RTSR1)
    rtsr2: ReadWrite<u32, RTSR2::Register>,
    /// Falling Trigger selection register (EXTI_FTSR1)
    ftsr2: ReadWrite<u32, FTSR2::Register>,
    /// Software interrupt event register (EXTI_SWIER1)
    swier2: ReadWrite<u32, SWIER2::Register>,
    /// Pending register (EXTI_PR1)
    pr2: ReadWrite<u32, PR2::Register>,
}

register_bitfields![u32,
    IMR1 [
        /// Interrupt Mask on line 0
        MR0 OFFSET(0) NUMBITS(1) [],
        /// Interrupt Mask on line 1
        MR1 OFFSET(1) NUMBITS(1) [],
        /// Interrupt Mask on line 2
        MR2 OFFSET(2) NUMBITS(1) [],
        /// Interrupt Mask on line 3
        MR3 OFFSET(3) NUMBITS(1) [],
        /// Interrupt Mask on line 4
        MR4 OFFSET(4) NUMBITS(1) [],
        /// Interrupt Mask on line 5
        MR5 OFFSET(5) NUMBITS(1) [],
        /// Interrupt Mask on line 6
        MR6 OFFSET(6) NUMBITS(1) [],
        /// Interrupt Mask on line 7
        MR7 OFFSET(7) NUMBITS(1) [],
        /// Interrupt Mask on line 8
        MR8 OFFSET(8) NUMBITS(1) [],
        /// Interrupt Mask on line 9
        MR9 OFFSET(9) NUMBITS(1) [],
        /// Interrupt Mask on line 10
        MR10 OFFSET(10) NUMBITS(1) [],
        /// Interrupt Mask on line 11
        MR11 OFFSET(11) NUMBITS(1) [],
        /// Interrupt Mask on line 12
        MR12 OFFSET(12) NUMBITS(1) [],
        /// Interrupt Mask on line 13
        MR13 OFFSET(13) NUMBITS(1) [],
        /// Interrupt Mask on line 14
        MR14 OFFSET(14) NUMBITS(1) [],
        /// Interrupt Mask on line 15
        MR15 OFFSET(15) NUMBITS(1) [],
        /// Interrupt Mask on line 16
        MR16 OFFSET(16) NUMBITS(1) [],
        /// Interrupt Mask on line 17
        MR17 OFFSET(17) NUMBITS(1) [],
        /// Interrupt Mask on line 18
        MR18 OFFSET(18) NUMBITS(1) [],
        /// Interrupt Mask on line 19
        MR19 OFFSET(19) NUMBITS(1) [],
        /// Interrupt Mask on line 20
        MR20 OFFSET(20) NUMBITS(1) [],
        /// Interrupt Mask on line 21
        MR21 OFFSET(21) NUMBITS(1) [],
        /// Interrupt Mask on line 22
        MR22 OFFSET(22) NUMBITS(1) [],
        /// Interrupt Mask on line 23
        MR23 OFFSET(23) NUMBITS(1) [],
        /// Interrupt Mask on line 24
        MR24 OFFSET(24) NUMBITS(1) [],
        /// Interrupt Mask on line 25
        MR25 OFFSET(25) NUMBITS(1) [],
        /// Interrupt Mask on line 26
        MR26 OFFSET(26) NUMBITS(1) [],
        /// Interrupt Mask on line 27
        MR27 OFFSET(27) NUMBITS(1) [],
        /// Interrupt Mask on line 28
        MR28 OFFSET(28) NUMBITS(1) [],
        /// Interrupt Mask on line 29
        MR29 OFFSET(29) NUMBITS(1) [],
        /// Interrupt Mask on line 30
        MR30 OFFSET(30) NUMBITS(1) [],
        /// Interrupt Mask on line 31
        MR31 OFFSET(31) NUMBITS(1) []
    ],
    EMR1 [
        /// Event Mask on line 0
        MR0 OFFSET(0) NUMBITS(1) [],
        /// Event Mask on line 1
        MR1 OFFSET(1) NUMBITS(1) [],
        /// Event Mask on line 2
        MR2 OFFSET(2) NUMBITS(1) [],
        /// Event Mask on line 3
        MR3 OFFSET(3) NUMBITS(1) [],
        /// Event Mask on line 4
        MR4 OFFSET(4) NUMBITS(1) [],
        /// Event Mask on line 5
        MR5 OFFSET(5) NUMBITS(1) [],
        /// Event Mask on line 6
        MR6 OFFSET(6) NUMBITS(1) [],
        /// Event Mask on line 7
        MR7 OFFSET(7) NUMBITS(1) [],
        /// Event Mask on line 8
        MR8 OFFSET(8) NUMBITS(1) [],
        /// Event Mask on line 9
        MR9 OFFSET(9) NUMBITS(1) [],
        /// Event Mask on line 10
        MR10 OFFSET(10) NUMBITS(1) [],
        /// Event Mask on line 11
        MR11 OFFSET(11) NUMBITS(1) [],
        /// Event Mask on line 12
        MR12 OFFSET(12) NUMBITS(1) [],
        /// Event Mask on line 13
        MR13 OFFSET(13) NUMBITS(1) [],
        /// Event Mask on line 14
        MR14 OFFSET(14) NUMBITS(1) [],
        /// Event Mask on line 15
        MR15 OFFSET(15) NUMBITS(1) [],
        /// Event Mask on line 16
        MR16 OFFSET(16) NUMBITS(1) [],
        /// Event Mask on line 17
        MR17 OFFSET(17) NUMBITS(1) [],
        /// Event Mask on line 18
        MR18 OFFSET(18) NUMBITS(1) [],
        /// Event Mask on line 19
        MR19 OFFSET(19) NUMBITS(1) [],
        /// Event Mask on line 20
        MR20 OFFSET(20) NUMBITS(1) [],
        /// Event Mask on line 21
        MR21 OFFSET(21) NUMBITS(1) [],
        /// Event Mask on line 22
        MR22 OFFSET(22) NUMBITS(1) [],
        /// Event Mask on line 23
        MR23 OFFSET(23) NUMBITS(1) [],
        /// Event Mask on line 24
        MR24 OFFSET(24) NUMBITS(1) [],
        /// Event Mask on line 25
        MR25 OFFSET(25) NUMBITS(1) [],
        /// Event Mask on line 26
        MR26 OFFSET(26) NUMBITS(1) [],
        /// Event Mask on line 27
        MR27 OFFSET(27) NUMBITS(1) [],
        /// Event Mask on line 28
        MR28 OFFSET(28) NUMBITS(1) [],
        /// Event Mask on line 29
        MR29 OFFSET(29) NUMBITS(1) [],
        /// Event Mask on line 30
        MR30 OFFSET(30) NUMBITS(1) [],
        /// Event Mask on line 31
        MR31 OFFSET(31) NUMBITS(1) []
    ],
    RTSR1 [
        /// Rising trigger event configuration of line 0
        TR0 OFFSET(0) NUMBITS(1) [],
        /// Rising trigger event configuration of line 1
        TR1 OFFSET(1) NUMBITS(1) [],
        /// Rising trigger event configuration of line 2
        TR2 OFFSET(2) NUMBITS(1) [],
        /// Rising trigger event configuration of line 3
        TR3 OFFSET(3) NUMBITS(1) [],
        /// Rising trigger event configuration of line 4
        TR4 OFFSET(4) NUMBITS(1) [],
        /// Rising trigger event configuration of line 5
        TR5 OFFSET(5) NUMBITS(1) [],
        /// Rising trigger event configuration of line 6
        TR6 OFFSET(6) NUMBITS(1) [],
        /// Rising trigger event configuration of line 7
        TR7 OFFSET(7) NUMBITS(1) [],
        /// Rising trigger event configuration of line 8
        TR8 OFFSET(8) NUMBITS(1) [],
        /// Rising trigger event configuration of line 9
        TR9 OFFSET(9) NUMBITS(1) [],
        /// Rising trigger event configuration of line 10
        TR10 OFFSET(10) NUMBITS(1) [],
        /// Rising trigger event configuration of line 11
        TR11 OFFSET(11) NUMBITS(1) [],
        /// Rising trigger event configuration of line 12
        TR12 OFFSET(12) NUMBITS(1) [],
        /// Rising trigger event configuration of line 13
        TR13 OFFSET(13) NUMBITS(1) [],
        /// Rising trigger event configuration of line 14
        TR14 OFFSET(14) NUMBITS(1) [],
        /// Rising trigger event configuration of line 15
        TR15 OFFSET(15) NUMBITS(1) [],
        /// Rising trigger event configuration of line 16
        TR16 OFFSET(16) NUMBITS(1) [],
        /// Rising trigger event configuration of line 17
        TR17 OFFSET(17) NUMBITS(1) [],
        /// Rising trigger event configuration of line 18
        TR18 OFFSET(18) NUMBITS(1) [],
        /// Rising trigger event configuration of line 19
        TR19 OFFSET(19) NUMBITS(1) [],
        /// Rising trigger event configuration of line 20
        TR20 OFFSET(20) NUMBITS(1) [],
        /// Rising trigger event configuration of line 21
        TR21 OFFSET(21) NUMBITS(1) [],
        /// Rising trigger event configuration of line 22
        TR22 OFFSET(22) NUMBITS(1) [],
        /// Rising trigger event configuration of line 29
        TR29 OFFSET(29) NUMBITS(1) [],
        /// Rising trigger event configuration of line 30
        TR30 OFFSET(30) NUMBITS(1) [],
        /// Rising trigger event configuration of line 31
        TR31 OFFSET(31) NUMBITS(1) []
    ],
    FTSR1 [
        /// Falling trigger event configuration of line 0
        TR0 OFFSET(0) NUMBITS(1) [],
        /// Falling trigger event configuration of line 1
        TR1 OFFSET(1) NUMBITS(1) [],
        /// Falling trigger event configuration of line 2
        TR2 OFFSET(2) NUMBITS(1) [],
        /// Falling trigger event configuration of line 3
        TR3 OFFSET(3) NUMBITS(1) [],
        /// Falling trigger event configuration of line 4
        TR4 OFFSET(4) NUMBITS(1) [],
        /// Falling trigger event configuration of line 5
        TR5 OFFSET(5) NUMBITS(1) [],
        /// Falling trigger event configuration of line 6
        TR6 OFFSET(6) NUMBITS(1) [],
        /// Falling trigger event configuration of line 7
        TR7 OFFSET(7) NUMBITS(1) [],
        /// Falling trigger event configuration of line 8
        TR8 OFFSET(8) NUMBITS(1) [],
        /// Falling trigger event configuration of line 9
        TR9 OFFSET(9) NUMBITS(1) [],
        /// Falling trigger event configuration of line 10
        TR10 OFFSET(10) NUMBITS(1) [],
        /// Falling trigger event configuration of line 11
        TR11 OFFSET(11) NUMBITS(1) [],
        /// Falling trigger event configuration of line 12
        TR12 OFFSET(12) NUMBITS(1) [],
        /// Falling trigger event configuration of line 13
        TR13 OFFSET(13) NUMBITS(1) [],
        /// Falling trigger event configuration of line 14
        TR14 OFFSET(14) NUMBITS(1) [],
        /// Falling trigger event configuration of line 15
        TR15 OFFSET(15) NUMBITS(1) [],
        /// Falling trigger event configuration of line 16
        TR16 OFFSET(16) NUMBITS(1) [],
        /// Falling trigger event configuration of line 17
        TR17 OFFSET(17) NUMBITS(1) [],
        /// Falling trigger event configuration of line 18
        TR18 OFFSET(18) NUMBITS(1) [],
        /// Falling trigger event configuration of line 19
        TR19 OFFSET(19) NUMBITS(1) [],
        /// Falling trigger event configuration of line 20
        TR20 OFFSET(20) NUMBITS(1) [],
        /// Falling trigger event configuration of line 21
        TR21 OFFSET(21) NUMBITS(1) [],
        /// Falling trigger event configuration of line 22
        TR22 OFFSET(22) NUMBITS(1) [],
        /// Falling trigger event configuration of line 29
        TR29 OFFSET(29) NUMBITS(1) [],
        /// Falling trigger event configuration of line 30
        TR30 OFFSET(30) NUMBITS(1) [],
        /// Falling trigger event configuration of line 31
        TR31 OFFSET(31) NUMBITS(1) []
    ],
    SWIER1 [
        /// Software Interrupt on line 0
        SWIER0 OFFSET(0) NUMBITS(1) [],
        /// Software Interrupt on line 1
        SWIER1 OFFSET(1) NUMBITS(1) [],
        /// Software Interrupt on line 2
        SWIER2 OFFSET(2) NUMBITS(1) [],
        /// Software Interrupt on line 3
        SWIER3 OFFSET(3) NUMBITS(1) [],
        /// Software Interrupt on line 4
        SWIER4 OFFSET(4) NUMBITS(1) [],
        /// Software Interrupt on line 5
        SWIER5 OFFSET(5) NUMBITS(1) [],
        /// Software Interrupt on line 6
        SWIER6 OFFSET(6) NUMBITS(1) [],
        /// Software Interrupt on line 7
        SWIER7 OFFSET(7) NUMBITS(1) [],
        /// Software Interrupt on line 8
        SWIER8 OFFSET(8) NUMBITS(1) [],
        /// Software Interrupt on line 9
        SWIER9 OFFSET(9) NUMBITS(1) [],
        /// Software Interrupt on line 10
        SWIER10 OFFSET(10) NUMBITS(1) [],
        /// Software Interrupt on line 11
        SWIER11 OFFSET(11) NUMBITS(1) [],
        /// Software Interrupt on line 12
        SWIER12 OFFSET(12) NUMBITS(1) [],
        /// Software Interrupt on line 13
        SWIER13 OFFSET(13) NUMBITS(1) [],
        /// Software Interrupt on line 14
        SWIER14 OFFSET(14) NUMBITS(1) [],
        /// Software Interrupt on line 15
        SWIER15 OFFSET(15) NUMBITS(1) [],
        /// Software Interrupt on line 16
        SWIER16 OFFSET(16) NUMBITS(1) [],
        /// Software Interrupt on line 17
        SWIER17 OFFSET(17) NUMBITS(1) [],
        /// Software Interrupt on line 18
        SWIER18 OFFSET(18) NUMBITS(1) [],
        /// Software Interrupt on line 19
        SWIER19 OFFSET(19) NUMBITS(1) [],
        /// Software Interrupt on line 20
        SWIER20 OFFSET(20) NUMBITS(1) [],
        /// Software Interrupt on line 21
        SWIER21 OFFSET(21) NUMBITS(1) [],
        /// Software Interrupt on line 22
        SWIER22 OFFSET(22) NUMBITS(1) [],
        /// Software Interrupt on line 29
        SWIER29 OFFSET(29) NUMBITS(1) [],
        /// Software Interrupt on line 30
        SWIER30 OFFSET(30) NUMBITS(1) [],
        /// Software Interrupt on line 31
        SWIER31 OFFSET(31) NUMBITS(1) []
    ],
    PR1 [
        /// Pending bit 0
        PR0 OFFSET(0) NUMBITS(1) [],
        /// Pending bit 1
        PR1 OFFSET(1) NUMBITS(1) [],
        /// Pending bit 2
        PR2 OFFSET(2) NUMBITS(1) [],
        /// Pending bit 3
        PR3 OFFSET(3) NUMBITS(1) [],
        /// Pending bit 4
        PR4 OFFSET(4) NUMBITS(1) [],
        /// Pending bit 5
        PR5 OFFSET(5) NUMBITS(1) [],
        /// Pending bit 6
        PR6 OFFSET(6) NUMBITS(1) [],
        /// Pending bit 7
        PR7 OFFSET(7) NUMBITS(1) [],
        /// Pending bit 8
        PR8 OFFSET(8) NUMBITS(1) [],
        /// Pending bit 9
        PR9 OFFSET(9) NUMBITS(1) [],
        /// Pending bit 10
        PR10 OFFSET(10) NUMBITS(1) [],
        /// Pending bit 11
        PR11 OFFSET(11) NUMBITS(1) [],
        /// Pending bit 12
        PR12 OFFSET(12) NUMBITS(1) [],
        /// Pending bit 13
        PR13 OFFSET(13) NUMBITS(1) [],
        /// Pending bit 14
        PR14 OFFSET(14) NUMBITS(1) [],
        /// Pending bit 15
        PR15 OFFSET(15) NUMBITS(1) [],
        /// Pending bit 16
        PR16 OFFSET(16) NUMBITS(1) [],
        /// Pending bit 17
        PR17 OFFSET(17) NUMBITS(1) [],
        /// Pending bit 18
        PR18 OFFSET(18) NUMBITS(1) [],
        /// Pending bit 19
        PR19 OFFSET(19) NUMBITS(1) [],
        /// Pending bit 20
        PR20 OFFSET(20) NUMBITS(1) [],
        /// Pending bit 21
        PR21 OFFSET(21) NUMBITS(1) [],
        /// Pending bit 22
        PR22 OFFSET(22) NUMBITS(1) [],
        /// Pending bit 29
        PR29 OFFSET(29) NUMBITS(1) [],
        /// Pending bit 30
        PR30 OFFSET(30) NUMBITS(1) [],
        /// Pending bit 31
        PR31 OFFSET(31) NUMBITS(1) []
    ],
    IMR2 [
        /// Interrupt Mask on line 32
        MR32 OFFSET(0) NUMBITS(1) [],
        /// Interrupt Mask on line 33
        MR33 OFFSET(1) NUMBITS(1) [],
        /// Interrupt Mask on line 34
        MR34 OFFSET(2) NUMBITS(1) [],
        /// Interrupt Mask on line 35
        MR35 OFFSET(3) NUMBITS(1) []
    ],
    EMR2 [
        /// Event Mask on line 32
        MR32 OFFSET(0) NUMBITS(1) [],
        /// Event Mask on line 33
        MR33 OFFSET(1) NUMBITS(1) [],
        /// Event Mask on line 34
        MR34 OFFSET(2) NUMBITS(1) [],
        /// Event Mask on line 35
        MR35 OFFSET(3) NUMBITS(1) []
    ],
    RTSR2 [
        /// Rising trigger event configuration of line 32
        TR32 OFFSET(0) NUMBITS(1) [],
        /// Rising trigger event configuration of line 33
        TR33 OFFSET(1) NUMBITS(1) []
    ],
    FTSR2 [
        /// Falling trigger event configuration of line 32
        TR32 OFFSET(0) NUMBITS(1) [],
        /// Falling trigger event configuration of line 33
        TR33 OFFSET(1) NUMBITS(1) []
    ],
    SWIER2 [
        /// Software Interrupt on line 32
        SWIER32 OFFSET(0) NUMBITS(1) [],
        /// Software Interrupt on line 33
        SWIER33 OFFSET(1) NUMBITS(1) []
    ],
    PR2 [
        /// Pending bit 32
        PR32 OFFSET(0) NUMBITS(1) [],
        /// Pending bit 33
        PR33 OFFSET(1) NUMBITS(1) []
    ]
];

const EXTI_BASE: StaticRef<ExtiRegisters> =
    unsafe { StaticRef::new(0x4001_0400 as *const ExtiRegisters) };

// /// EXTI block has 23 lines going into NVIC. This arrangement is described here
// /// [^1].
// ///
// /// The 23 lines going into NVIC, are mapped to the following NVIC IRQs. Note
// /// there is *no* one-to-one mapping between the 23 lines to NVIC IRQs. The 23
// /// lines going into NVIC translates to 14 IRQs on NVIC.
// ///
// ///  - EXTI0 (6)
// ///  - EXTI1 (7)
// ///  - EXTI2 (8)
// ///  - EXTI3 (9)
// ///  - EXTI4 (10)
// ///  - EXTI9_5 (23)
// ///  - EXTI15_10 (40)
// ///
// ///  - EXTI16 -> PVD (1)
// ///  - EXTI17 -> RTC_Alarm (41)
// ///  - EXTI18 -> OTG_FS_WKUP (42)
// ///  - EXTI19 -> <UNKNOWN>
// ///  - EXTI20 -> OTG_FS (67)
// ///  - EXTI21 -> TAMP_STAMP (2)
// ///  - EXTI22 -> RTC_WKUP (3)
// ///

/// The EXTI_PR (pending) register when set, generates a level-triggered
/// interrupt on the NVIC. This means, that its the responsibility of the IRQ
/// handler to clear the interrupt source (pending bit), in order to prevent
/// multiple interrupts from occurring.
///
/// `EXTI_EVENTS` is modeled to capture information from `EXTI_PR` register. In
/// the top half IRQ handler, prior to clearing the pending bit, we set the
/// corresponding bit in `EXTI_EVENTS`. Once the bit is set, in `EXTI_EVENTS`,
/// we clear the pending bit and exit the ISR.
///
/// [^1]: Section 10.2.2, EXTI block diagram, page 243 of reference manual.
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

// // `line_gpiopin_map` is used to call `handle_interrupt()` on the pin.
pub struct Exti<'a> {
    registers: StaticRef<ExtiRegisters>,
    clock: ExtiClock<'a>,
    line_gpiopin_map: [OptionalCell<&'static gpio::Pin<'static>>; 16],
    syscfg: &'a syscfg::Syscfg<'a>,
}

impl<'a> Exti<'a> {
    pub const fn new(syscfg: &'a syscfg::Syscfg<'a>) -> Exti<'a> {
        Exti {
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
            atomic(|| {
                exti_pr = self.registers.pr1.get();
                self.registers.pr1.set(exti_pr);
            });
        }

        // ignore the "reserved" EXTI bits. Use bits [22:0]. See `EXTI_PR` for
        // details.
        exti_pr |= 0x007fffff;

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

/// The configuration registers for Exti is in Syscfg, so we need to
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
