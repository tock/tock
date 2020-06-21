//! Clock System (CS)

use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;

pub static mut CS: ClockSystem = ClockSystem::new();

const CS_BASE: StaticRef<CsRegisters> =
    unsafe { StaticRef::new(0x4001_0400u32 as *const CsRegisters) };

const KEY: u32 = 0x695A;

register_structs! {
    /// CS
    CsRegisters {
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
        // for accessing any other register, it must be unlocked using this key-register
        KEY OFFSET(0) NUMBITS(16)
    ],
    CSCTL0 [
        // for calibrating the DCO frequency
        DCOTUNE OFFSET(0) NUMBITS(10),
        // DCO frequency range select
        DCORSEL OFFSET(16) NUMBITS(3),
        // enable/disable DCO external resistor mode
        DCORES OFFSET(22) NUMBITS(1),
        // enable DCO
        DCOEN OFFSET(23) NUMBITS(23)
    ],
    CSCTL1 [
        // select MCLK source
        SELM OFFSET(0) NUMBITS(3),
        // select SMCLK and HSMCLK source
        SELS OFFSET(4) NUMBITS(3),
        // selects ACLK source
        SELA OFFSET(8) NUMBITS(3),
        // selects BLCK source
        SELB OFFSET(12) NUMBITS(1),
        // MCLK source divider
        DIVM OFFSET(16) NUMBITS(3),
        // HSMCLK source divider
        DIVHS OFFSET(20) NUMBITS(3),
        // ACLK source divider
        DIVA OFFSET(24) NUMBITS(3),
        // SMCLK divider
        DIVS OFFSET(28) NUMBITS(3)
    ],
    CSCTL2 [
        // set drive-strength for LXFT oscillator
        LFXTDRIVE OFFSET(0) NUMBITS(2),
        // turn on LFXT oscillator
        LFXT_EN OFFSET(8) NUMBITS(1),
        // LFXT bypass select
        LFXTBYPASS OFFSET(9) NUMBITS(1),
        // HFXT oscillator drive selection
        HFXTDRIVE OFFSET(16) NUMBITS(1),
        // HFXT frequency selection
        HFXTFREQ OFFSET(20) NUMBITS(3),
        // turn on HFXT oscillator
        HFXT_EN OFFSET(24) NUMBITS(1),
        // HFXT bypass select
        HFXTBYPASS OFFSET(25) NUMBITS(1)
    ],
    CSCTL3 [
        // start flag counter for LFXT
        FCNTLF OFFSET(0) NUMBITS(2),
        // reset start fault counter for LFXT
        RFCNTLF OFFSET(2) NUMBITS(1),
        // enable start fault counter for LFXT
        FCNTLF_EN OFFSET(0) NUMBITS(1),
        // start flag counter for HFXT
        FCNTHF OFFSET(4) NUMBITS(2),
        // reset start fault counter for HFXT
        RFCNTHF OFFSET(6) NUMBITS(1),
        // enable start fault counter for HFXT
        FCNTHF_EN OFFSET(7) NUMBITS(1)
    ],
    CSCLKEN [
        // ACLK system clock conditional request enable
        ACLK_EN OFFSET(0) NUMBITS(1),
        // MCLK system clock conditional request enable
        MCLK_EN OFFSET(1) NUMBITS(1),
        // HSMCLK system clock conditional request enable
        HSMCLK_EN OFFSET(2) NUMBITS(1),
        // SMCLK system clock conditional request enable
        SMCLK_EN OFFSET(3) NUMBITS(1),
        // turn on the VLO oscillator
        VLO_EN OFFSET(8) NUMBITS(1),
        // turn on the REFO oscillator
        REFO_EN OFFSET(9) NUMBITS(1),
        // turn on the MODOSC oscillator
        MODOSC_EN OFFSET(10) NUMBITS(1),
        // select REFO nominal frequency: 0 = 32.768kHz, 1=128kHz
        REFOFSEL OFFSET(15) NUMBITS(1)
    ],
    // status of the different clock-sources, if they are active or not
    CSSTAT [
        DCO_ON OFFSET(0) NUMBITS(1),
        DCOBIAS_ON OFFSET(1) NUMBITS(1),
        HFXT_ON OFFSET(2) NUMBITS(1),
        MODOSC_ON OFFSET(4) NUMBITS(1),
        VLO_ON OFFSET(5) NUMBITS(1),
        LFXT_ON OFFSET(6) NUMBITS(1),
        REFO_ON OFFSET(7) NUMBITS(1),
        ACLK_ON OFFSET(16) NUMBITS(1),
        MCLK_ON OFFSET(17) NUMBITS(1),
        HSMCLK_ON OFFSET(18) NUMBITS(1),
        SMCLK_ON OFFSET(19) NUMBITS(1),
        MODCLK_ON OFFSET(20) NUMBITS(1),
        VLOCLK_ON OFFSET(21) NUMBITS(1),
        LFXTCLK_ON OFFSET(22) NUMBITS(1),
        REFOCLK_ON OFFSET(23) NUMBITS(1),
        ACLK_READY OFFSET(24) NUMBITS(1),
        MCLK_READY OFFSET(25) NUMBITS(1),
        HSMCLK_READY OFFSET(26) NUMBITS(1),
        SMCLK_READY OFFSET(27) NUMBITS(1),
        BCLK_READY OFFSET(28) NUMBITS(1)
    ],
    // interrupt enable register
    CSIE [
        // LFXT oscillator fault flag
        LFXTIE OFFSET(0) NUMBITS(1),
        // HFXT oscillator fault flag
        HFXTIE OFFSET(1) NUMBITS(1),
        // DCO external resistor open circuit fault flag
        DCOR_OPNIE OFFSET(6) NUMBITS(1),
        // LFXT start fault counter
        FCNTLFIE OFFSET(8) NUMBITS(1),
        // HFXT start fault counter
        FCNTHFIE OFFSET(9) NUMBITS(1)
    ],
    // interrupt flag register
    CSIFG [
        // LFXT oscillator fault flag
        LFXTIFG OFFSET(0) NUMBITS(1),
        // HFXT oscillator fault flag
        HFXTIFG OFFSET(1) NUMBITS(1),
        // DCO external resistor open circuit fault flag
        DCOR_OPNIFG OFFSET(6) NUMBITS(1),
        // LFXT start fault counter
        FCNTLFIFG OFFSET(8) NUMBITS(1),
        // HFXT start fault counter
        FCNTHFIFG OFFSET(9) NUMBITS(1)
    ],
    // interrupt clear register
    CSCLRIFG [
        // LFXT oscillator fault flag
        LFXTIFG OFFSET(0) NUMBITS(1),
        // HFXT oscillator fault flag
        HFXTIFG OFFSET(1) NUMBITS(1),
        // DCO external resistor open circuit fault flag
        DCOR_OPNIFG OFFSET(6) NUMBITS(1),
        // LFXT start fault counter
        FCNTLFIFG OFFSET(8) NUMBITS(1),
        // HFXT start fault counter
        FCNTHFIFG OFFSET(9) NUMBITS(1)
    ],
    // interrupt set/assert register
    CSSETIFG [
        // LFXT oscillator fault flag
        SET_LFXTIFG OFFSET(0) NUMBITS(1),
        // HFXT oscillator fault flag
        SET_HFXTIFG OFFSET(1) NUMBITS(1),
        // DCO external resistor open circuit fault flag
        SET_DCOR_OPNIFG OFFSET(6) NUMBITS(1),
        // LFXT start fault counter
        SET_FCNTLFIFG OFFSET(8) NUMBITS(1),
        // HFXT start fault counter
        SET_FCNTHFIFG OFFSET(9) NUMBITS(1)
    ],
    // DCO external resistor calibration 0 register
    CSDCOERCAL0 [
        // DCO temperature compensation calibration
        DCO_TCCAL OFFSET(0) NUMBITS(1),
        // DCO frequency calibration for DCO frequency range (DCORSEL) 0 to 4
        DCO_FCAL_RSEL04 OFFSET(16) NUMBITS(10)
    ],
     // DCO external resistor calibration 1 register
    CSDCOERCAL1 [
        // DCO frequency calibration for DCO frequency range (DCORSEL) 5
        DCO_FCAL_RSEL5 OFFSET(0) NUMBITS(10)
    ]
];

pub struct ClockSystem {
    registers: StaticRef<CsRegisters>,
}

impl ClockSystem {
    const fn new() -> ClockSystem {
        ClockSystem { registers: CS_BASE }
    }

    fn unlock_registers(&self) {
        self.registers.key.modify(CSKEY::KEY.val(KEY));
    }

    fn lock_registers(&self) {
        // every value except KEY written to the key register will perform the lock
        self.registers.key.modify(CSKEY::KEY.val(0));
    }

    // not sure about the interface, so for testing provide a function to set
    // the clock to 48Mhz
    pub fn set_clk_48mhz(&self) {
        self.unlock_registers();

        // set HFXT to 40-48MHz range
        self.registers.ctl2.modify(CSCTL2::HFXTFREQ.val(6));

        // set HFXT as MCLK source
        self.registers
            .ctl1
            .modify(CSCTL1::SELM.val(5) + CSCTL1::DIVM.val(0));

        while self.registers.ifg.is_set(CSIFG::HFXTIFG) {
            self.registers
                .clrifg
                .write(CSCLRIFG::HFXTIFG::SET + CSCLRIFG::FCNTHFIFG::SET);
        }
        self.lock_registers();
    }

    pub fn set_smclk_12mhz(&self) {
        self.unlock_registers();

        // set HFXT as clock-source for SMCLK
        self.registers.ctl1.modify(CSCTL1::SELS.val(5));

        // set SMCLK divider to 4 -> 48MHz / 4 = 12MHz
        self.registers.ctl1.modify(CSCTL1::DIVS.val(2));

        self.lock_registers();
    }
}
