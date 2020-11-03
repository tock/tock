use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

use crate::gpio;
use crate::rcc;

/// System configuration controller
#[repr(C)]
struct SyscfgRegisters {
    /// memory remap register
    memrm: ReadWrite<u32, MEMRM::Register>,
    /// peripheral mode configuration register
    pmc: ReadWrite<u32, PMC::Register>,
    /// external interrupt configuration register 1
    exticr1: ReadWrite<u32, EXTICR1::Register>,
    /// external interrupt configuration register 2
    exticr2: ReadWrite<u32, EXTICR2::Register>,
    /// external interrupt configuration register 3
    exticr3: ReadWrite<u32, EXTICR3::Register>,
    /// external interrupt configuration register 4
    exticr4: ReadWrite<u32, EXTICR4::Register>,
    _reserved0: [u8; 8],
    /// Compensation cell control register
    cmpcr: ReadOnly<u32, CMPCR::Register>,
}

register_bitfields![u32,
    MEMRM [
        /// Memory mapping selection
        MEM_MODE OFFSET(0) NUMBITS(3) [],
        /// Flash bank mode selection
        FB_MODE OFFSET(8) NUMBITS(1) [],
        /// FMC memory mapping swap
        SWP_FMC OFFSET(10) NUMBITS(2) []
    ],
    PMC [
        /// Ethernet PHY interface selection
        MII_RMII_SEL OFFSET(23) NUMBITS(1) [],
        /// ADC1DC2
        ADC1DC2 OFFSET(16) NUMBITS(1) [],
        /// ADC2DC2
        ADC2DC2 OFFSET(17) NUMBITS(1) [],
        /// ADC3DC2
        ADC3DC2 OFFSET(18) NUMBITS(1) []
    ],
    EXTICR1 [
        /// EXTI x configuration (x = 0 to 3)
        EXTI3 OFFSET(12) NUMBITS(4) [],
        /// EXTI x configuration (x = 0 to 3)
        EXTI2 OFFSET(8) NUMBITS(4) [],
        /// EXTI x configuration (x = 0 to 3)
        EXTI1 OFFSET(4) NUMBITS(4) [],
        /// EXTI x configuration (x = 0 to 3)
        EXTI0 OFFSET(0) NUMBITS(4) []
    ],
    EXTICR2 [
        /// EXTI x configuration (x = 4 to 7)
        EXTI7 OFFSET(12) NUMBITS(4) [],
        /// EXTI x configuration (x = 4 to 7)
        EXTI6 OFFSET(8) NUMBITS(4) [],
        /// EXTI x configuration (x = 4 to 7)
        EXTI5 OFFSET(4) NUMBITS(4) [],
        /// EXTI x configuration (x = 4 to 7)
        EXTI4 OFFSET(0) NUMBITS(4) []
    ],
    EXTICR3 [
        /// EXTI x configuration (x = 8 to 11)
        EXTI11 OFFSET(12) NUMBITS(4) [],
        /// EXTI10
        EXTI10 OFFSET(8) NUMBITS(4) [],
        /// EXTI x configuration (x = 8 to 11)
        EXTI9 OFFSET(4) NUMBITS(4) [],
        /// EXTI x configuration (x = 8 to 11)
        EXTI8 OFFSET(0) NUMBITS(4) []
    ],
    EXTICR4 [
        /// EXTI x configuration (x = 12 to 15)
        EXTI15 OFFSET(12) NUMBITS(4) [],
        /// EXTI x configuration (x = 12 to 15)
        EXTI14 OFFSET(8) NUMBITS(4) [],
        /// EXTI x configuration (x = 12 to 15)
        EXTI13 OFFSET(4) NUMBITS(4) [],
        /// EXTI x configuration (x = 12 to 15)
        EXTI12 OFFSET(0) NUMBITS(4) []
    ],
    CMPCR [
        /// READY
        READY OFFSET(8) NUMBITS(1) [],
        /// Compensation cell power-down
        CMP_PD OFFSET(0) NUMBITS(1) []
    ]
];

const SYSCFG_BASE: StaticRef<SyscfgRegisters> =
    unsafe { StaticRef::new(0x40010000 as *const SyscfgRegisters) };

enum_from_primitive! {
    #[repr(u32)]
    /// SYSCFG EXTI configuration [^1]
    ///
    /// [^1]: Section 8.2.2, page 197 of reference manual
    enum ExtiCrId {
        PA = 0b0000,
        PB = 0b0001,
        PC = 0b0010,
        PD = 0b0011,
        PE = 0b0100,
        PF = 0b0101,
        PG = 0b0110,
        PH = 0b0111,
    }
}

pub struct Syscfg<'a> {
    registers: StaticRef<SyscfgRegisters>,
    clock: SyscfgClock<'a>,
}

impl<'a> Syscfg<'a> {
    pub const fn new(rcc: &'a rcc::Rcc) -> Syscfg {
        Syscfg {
            registers: SYSCFG_BASE,
            clock: SyscfgClock(rcc::PeripheralClock::new(
                rcc::PeripheralClockType::APB2(rcc::PCLK2::SYSCFG),
                rcc,
            )),
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

    /// Configures the SYSCFG_EXTICR{1, 2, 3, 4} registers
    pub fn configure_interrupt(&self, pinid: gpio::PinId) {
        let exticrid = self.get_exticrid_from_port_num(pinid.get_port_number());

        let pin_num = pinid.get_pin_number();
        match pin_num {
            // SYSCFG_EXTICR1
            0b0000 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI0.val(exticrid as u32)),
            0b0001 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI1.val(exticrid as u32)),
            0b0010 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI2.val(exticrid as u32)),
            0b0011 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI3.val(exticrid as u32)),
            // SYSCFG_EXTICR2
            0b0100 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI4.val(exticrid as u32)),
            0b0101 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI5.val(exticrid as u32)),
            0b0110 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI6.val(exticrid as u32)),
            0b0111 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI7.val(exticrid as u32)),
            // SYSCFG_EXTICR3
            0b1000 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI8.val(exticrid as u32)),
            0b1001 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI9.val(exticrid as u32)),
            0b1010 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI10.val(exticrid as u32)),
            0b1011 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI11.val(exticrid as u32)),
            // SYSCFG_EXTICR4
            0b1100 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI12.val(exticrid as u32)),
            0b1101 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI13.val(exticrid as u32)),
            0b1110 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI14.val(exticrid as u32)),
            0b1111 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI15.val(exticrid as u32)),
            _ => {}
        }
    }

    fn get_exticrid_from_port_num(&self, port_num: u8) -> ExtiCrId {
        ExtiCrId::from_u32(u32::from(port_num)).unwrap()
    }
}

struct SyscfgClock<'a>(rcc::PeripheralClock<'a>);

impl ClockInterface for SyscfgClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
