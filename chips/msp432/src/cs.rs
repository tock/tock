//! Clock System (CS)

use kernel::common::peripherals::{PeripheralManagement, PeripheralManager};
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::NoClockControl;

pub static mut CS: ClockSystem = ClockSystem::new();

pub const MCLK_HZ: u32 = 48_000_000;
pub const HSMCLK_HZ: u32 = 12_000_000;
pub const SMCLK_HZ: u32 = 1_500_000;
pub const ACLK_HZ: u32 = 32_768;

const CS_BASE: StaticRef<CsRegisters> =
    unsafe { StaticRef::new(0x4001_0400u32 as *const CsRegisters) };

const KEY: u32 = 0x695A;

register_structs! {
    /// CS
    pub CsRegisters {
        /// Key Register
        (0x00 => key: ReadWrite<u32, CSKEY::Register>),
        /// Control 0 Register
        (0x04 => ctl0: ReadWrite<u32, CSCTL0::Register>),
        /// Control 1 Register
        (0x08 => ctl1: ReadWrite<u32, CSCTL1::Register>),
        /// Control 2 Register
        (0x0C => ctl2: ReadWrite<u32, CSCTL2::Register>),
        /// Control 3 Register
        (0x10 => ctl3: ReadWrite<u32, CSCTL3::Register>),
        (0x14 => _reserved0),
        /// Clock Enable Register
        (0x30 => clken: ReadWrite<u32, CSCLKEN::Register>),
        /// Status Register
        (0x34 => stat: ReadOnly<u32, CSSTAT::Register>),
        (0x38 => _reserved1),
        /// Interrupt Enable Register
        (0x40 => ie: ReadWrite<u32, CSIE::Register>),
        (0x44 => _reserved2),
        /// Interrupt Flag Register
        (0x48 => ifg: ReadOnly<u32, CSIFG::Register>),
        (0x4C => _reserved3),
        /// Clear Interrupt Flag Register
        (0x50 => clrifg: WriteOnly<u32, CSCLRIFG::Register>),
        (0x54 => _reserved4),
        /// Set Interrupt Flag Register
        (0x58 => setifg: WriteOnly<u32, CSSETIFG::Register>),
        (0x5C => _reserved5),
        /// DCO External Resistor Calibration 0 Register
        (0x60 => dcoercal0: ReadWrite<u32, CSDCOERCAL0::Register>),
        /// DCO External Resistor Calibration 1 Register
        (0x64 => dcoercal1: ReadWrite<u32, CSDCOERCAL1::Register>),
        (0x68 => @END),
    }
}

register_bitfields! [u32,
    CSKEY [
        /// For accessing any other register, it must be unlocked using this key-register
        KEY OFFSET(0) NUMBITS(16)
    ],
    CSCTL0 [
        /// For calibrating the DCO frequency
        DCOTUNE OFFSET(0) NUMBITS(10),
        /// DCO frequency range select
        DCORSEL OFFSET(16) NUMBITS(3),
        /// Enable/disable DCO external resistor mode
        DCORES OFFSET(22) NUMBITS(1),
        /// Enable DCO
        DCOEN OFFSET(23) NUMBITS(23)
    ],
    CSCTL1 [
        /// Select MCLK source
        SELM OFFSET(0) NUMBITS(3),
        /// Select SMCLK and HSMCLK source
        SELS OFFSET(4) NUMBITS(3),
        /// Select ACLK source
        SELA OFFSET(8) NUMBITS(3),
        /// Select BLCK source
        SELB OFFSET(12) NUMBITS(1),
        /// MCLK source divider
        DIVM OFFSET(16) NUMBITS(3),
        /// HSMCLK source divider
        DIVHS OFFSET(20) NUMBITS(3),
        /// ACLK source divider
        DIVA OFFSET(24) NUMBITS(3),
        /// SMCLK divider
        DIVS OFFSET(28) NUMBITS(3)
    ],
    CSCTL2 [
        /// Set drive-strength for LXFT oscillator
        LFXTDRIVE OFFSET(0) NUMBITS(2),
        /// Turn on LFXT oscillator
        LFXT_EN OFFSET(8) NUMBITS(1),
        /// LFXT bypass select
        LFXTBYPASS OFFSET(9) NUMBITS(1),
        /// HFXT oscillator drive selection
        HFXTDRIVE OFFSET(16) NUMBITS(1),
        /// HFXT frequency selection
        HFXTFREQ OFFSET(20) NUMBITS(3),
        /// Turn on HFXT oscillator
        HFXT_EN OFFSET(24) NUMBITS(1),
        /// HFXT bypass select
        HFXTBYPASS OFFSET(25) NUMBITS(1)
    ],
    CSCTL3 [
        /// Start flag counter for LFXT
        FCNTLF OFFSET(0) NUMBITS(2),
        /// Reset start fault counter for LFXT
        RFCNTLF OFFSET(2) NUMBITS(1),
        /// Enable start fault counter for LFXT
        FCNTLF_EN OFFSET(0) NUMBITS(1),
        /// Start flag counter for HFXT
        FCNTHF OFFSET(4) NUMBITS(2),
        /// Reset start fault counter for HFXT
        RFCNTHF OFFSET(6) NUMBITS(1),
        /// Enable start fault counter for HFXT
        FCNTHF_EN OFFSET(7) NUMBITS(1)
    ],
    CSCLKEN [
        /// ACLK system clock conditional request enable
        ACLK_EN OFFSET(0) NUMBITS(1),
        /// MCLK system clock conditional request enable
        MCLK_EN OFFSET(1) NUMBITS(1),
        /// HSMCLK system clock conditional request enable
        HSMCLK_EN OFFSET(2) NUMBITS(1),
        /// SMCLK system clock conditional request enable
        SMCLK_EN OFFSET(3) NUMBITS(1),
        /// Turn on the VLO oscillator
        VLO_EN OFFSET(8) NUMBITS(1),
        /// Turn on the REFO oscillator
        REFO_EN OFFSET(9) NUMBITS(1),
        /// Turn on the MODOSC oscillator
        MODOSC_EN OFFSET(10) NUMBITS(1),
        /// Select REFO nominal frequency: 0 = 32.768kHz, 1=128kHz
        REFOFSEL OFFSET(15) NUMBITS(1)
    ],
    /// Status of the different clock-sources
    CSSTAT [
        /// DCO status, 1=active, 0=inactive
        DCO_ON OFFSET(0) NUMBITS(1),
        /// DCO bias status, 1=active, 0=inactive
        DCOBIAS_ON OFFSET(1) NUMBITS(1),
        /// HFXT status, 1=active, 0=inactive
        HFXT_ON OFFSET(2) NUMBITS(1),
        /// MODOSC status, 1=active, 0=inactive
        MODOSC_ON OFFSET(4) NUMBITS(1),
        /// VLO status, 1=active, 0=inactive
        VLO_ON OFFSET(5) NUMBITS(1),
        /// LFXT status, 1=active, 0=inactive
        LFXT_ON OFFSET(6) NUMBITS(1),
        /// REFO status, 1=active, 0=inactive
        REFO_ON OFFSET(7) NUMBITS(1),
        /// ACLK system clock status, 1=active, 0=inactive
        ACLK_ON OFFSET(16) NUMBITS(1),
        /// MCLK system clock status, 1=active, 0=inactive
        MCLK_ON OFFSET(17) NUMBITS(1),
        /// HSMCLK system clock status, 1=active, 0=inactive
        HSMCLK_ON OFFSET(18) NUMBITS(1),
        /// SMCLK system clock status, 1=active, 0=inactive
        SMCLK_ON OFFSET(19) NUMBITS(1),
        /// MODCLK system clock status, 1=active, 0=inactive
        MODCLK_ON OFFSET(20) NUMBITS(1),
        /// VLOCLK system clock status, 1=active, 0=inactive
        VLOCLK_ON OFFSET(21) NUMBITS(1),
        /// LFXTCLK system clock status, 1=active, 0=inactive
        LFXTCLK_ON OFFSET(22) NUMBITS(1),
        /// REFOCLK system clock status, 1=active, 0=inactive
        REFOCLK_ON OFFSET(23) NUMBITS(1),
        /// ACLK ready status, indicates if the clock is stable after a change in the frequency/divider settings
        ACLK_READY OFFSET(24) NUMBITS(1),
        /// MCLK ready status, indicates if the clock is stable after a change in the frequency/divider settings
        MCLK_READY OFFSET(25) NUMBITS(1),
        /// HSMCLK ready status, indicates if the clock is stable after a change in the frequency/divider settings
        HSMCLK_READY OFFSET(26) NUMBITS(1),
        /// SMCLK ready status, indicates if the clock is stable after a change in the frequency/divider settings
        SMCLK_READY OFFSET(27) NUMBITS(1),
        /// BCLK ready status, indicates if the clock is stable after a change in the frequency/divider settings
        BCLK_READY OFFSET(28) NUMBITS(1)
    ],
    /// Interrupt enable register
    CSIE [
        /// LFXT oscillator fault flag
        LFXTIE OFFSET(0) NUMBITS(1),
        /// HFXT oscillator fault flag
        HFXTIE OFFSET(1) NUMBITS(1),
        /// DCO external resistor open circuit fault flag
        DCOR_OPNIE OFFSET(6) NUMBITS(1),
        /// LFXT start fault counter
        FCNTLFIE OFFSET(8) NUMBITS(1),
        /// HFXT start fault counter
        FCNTHFIE OFFSET(9) NUMBITS(1)
    ],
    /// Interrupt flag register
    CSIFG [
        /// LFXT oscillator fault flag
        LFXTIFG OFFSET(0) NUMBITS(1),
        /// HFXT oscillator fault flag
        HFXTIFG OFFSET(1) NUMBITS(1),
        /// DCO external resistor open circuit fault flag
        DCOR_OPNIFG OFFSET(6) NUMBITS(1),
        /// LFXT start fault counter
        FCNTLFIFG OFFSET(8) NUMBITS(1),
        /// HFXT start fault counter
        FCNTHFIFG OFFSET(9) NUMBITS(1)
    ],
    /// iIterrupt clear register
    CSCLRIFG [
        /// LFXT oscillator fault flag
        LFXTIFG OFFSET(0) NUMBITS(1),
        /// HFXT oscillator fault flag
        HFXTIFG OFFSET(1) NUMBITS(1),
        /// DCO external resistor open circuit fault flag
        DCOR_OPNIFG OFFSET(6) NUMBITS(1),
        /// LFXT start fault counter
        FCNTLFIFG OFFSET(8) NUMBITS(1),
        /// HFXT start fault counter
        FCNTHFIFG OFFSET(9) NUMBITS(1)
    ],
    /// Interrupt set/assert register
    CSSETIFG [
        /// LFXT oscillator fault flag
        SET_LFXTIFG OFFSET(0) NUMBITS(1),
        /// HFXT oscillator fault flag
        SET_HFXTIFG OFFSET(1) NUMBITS(1),
        /// DCO external resistor open circuit fault flag
        SET_DCOR_OPNIFG OFFSET(6) NUMBITS(1),
        /// LFXT start fault counter
        SET_FCNTLFIFG OFFSET(8) NUMBITS(1),
        /// HFXT start fault counter
        SET_FCNTHFIFG OFFSET(9) NUMBITS(1)
    ],
    /// DCO external resistor calibration 0 register
    CSDCOERCAL0 [
        /// DCO temperature compensation calibration
        DCO_TCCAL OFFSET(0) NUMBITS(1),
        /// DCO frequency calibration for DCO frequency range (DCORSEL) 0 to 4
        DCO_FCAL_RSEL04 OFFSET(16) NUMBITS(10)
    ],
    /// DCO external resistor calibration 1 register
    CSDCOERCAL1 [
        /// DCO frequency calibration for DCO frequency range (DCORSEL) 5
        DCO_FCAL_RSEL5 OFFSET(0) NUMBITS(10)
    ]
];

type CsRegisterManager<'a> = PeripheralManager<'a, ClockSystem, NoClockControl>;

pub struct ClockSystem {}

impl ClockSystem {
    const fn new() -> ClockSystem {
        ClockSystem {}
    }

    fn set_mclk_48mhz(&self) {
        let cs = CsRegisterManager::new(self);

        // Set HFXT to 40-48MHz range
        cs.registers.ctl2.modify(CSCTL2::HFXTFREQ.val(6));

        // Set HFXT (48MHz) as MCLK source
        cs.registers
            .ctl1
            .modify(CSCTL1::SELM.val(5) + CSCTL1::DIVM.val(0));

        while cs.registers.ifg.is_set(CSIFG::HFXTIFG) {
            cs.registers
                .clrifg
                .write(CSCLRIFG::HFXTIFG::SET + CSCLRIFG::FCNTHFIFG::SET);
        }
    }

    // Setup the subsystem master clock (HSMCLK) to 1/4 of the master-clock -> 12MHz
    fn set_hsmclk_12mhz(&self) {
        let cs = CsRegisterManager::new(self);

        // Set HFXT (48MHz) as clock-source for HSMCLK
        cs.registers.ctl1.modify(CSCTL1::SELS.val(5));

        // Set HSMCLK divider to 4 -> 48MHz / 4 = 12MHz
        cs.registers.ctl1.modify(CSCTL1::DIVHS.val(2));
    }

    // Setup the low-speed subsystem master clock (SMCLK) to 1/64 of the master-clock -> 750kHz
    fn set_smclk_1500khz(&self) {
        let cs = CsRegisterManager::new(self);

        // Set HFXT (48MHz) as clock-source for SMCLK
        cs.registers.ctl1.modify(CSCTL1::SELS.val(5));

        // Set SMCLK divider to 32 -> 48MHz / 32 = 1.5MHz
        cs.registers.ctl1.modify(CSCTL1::DIVS.val(5));
    }

    // Setup the auxiliary clock (ACLK) to 32.768kHz
    fn set_aclk_32khz(&self) {
        let cs = CsRegisterManager::new(self);

        // Set LFXT (32.768kHz) as clock-source for ACLK
        cs.registers.ctl1.modify(CSCTL1::SELA.val(0));

        // SET ACLK divider to 1 -> 32.768kHz
        cs.registers.ctl1.modify(CSCTL1::DIVA.val(0));
    }

    pub fn setup_clocks(&self) {
        self.set_mclk_48mhz();
        self.set_hsmclk_12mhz();
        self.set_smclk_1500khz();
        self.set_aclk_32khz();
    }
}

impl<'a> PeripheralManagement<NoClockControl> for ClockSystem {
    type RegisterType = CsRegisters;

    fn get_registers(&self) -> &CsRegisters {
        &*CS_BASE
    }

    fn get_clock(&self) -> &NoClockControl {
        unsafe { &kernel::NO_CLOCK_CONTROL }
    }

    fn before_peripheral_access(&self, _c: &NoClockControl, r: &Self::RegisterType) {
        // Unlocks the registers in order to allow write accesses
        r.key.modify(CSKEY::KEY.val(KEY));
    }

    fn after_peripheral_access(&self, _c: &NoClockControl, r: &Self::RegisterType) {
        // Locks the registers in order to prevent write accesses
        // Every value except KEY written to the key register will perform the lock
        r.key.modify(CSKEY::KEY.val(0));
    }
}
