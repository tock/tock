//! Pulse Width Modulation (PWM) Driver

use kernel::common::StaticRef;
use kernel::common::registers::ReadWrite;

#[repr(C)]
pub struct PwmRegisters {
    /// PWM Configuration Register
    cfg: ReadWrite<u32, cfg::Register>,
    /// Counter Register
    count: ReadWrite<u32>,
    _reserved0: [u8; 8],
    /// Scaled Halfword Counter Register
    pwms: ReadWrite<u32>,
    _reserved1: [u8; 12],
    /// Compare Register
    cmp0: ReadWrite<u32>,
    /// Compare Register
    cmp1: ReadWrite<u32>,
    /// Compare Register
    cmp2: ReadWrite<u32>,
    /// Compare Register
    cmp3: ReadWrite<u32>,
}

register_bitfields![u32,
    cfg [
        cmp3ip OFFSET(31) NUMBITS(1) [],
        cmp2ip OFFSET(30) NUMBITS(1) [],
        cmp1ip OFFSET(29) NUMBITS(1) [],
        cmp0ip OFFSET(28) NUMBITS(1) [],
        cmp3gang OFFSET(27) NUMBITS(1) [],
        cmp2gang OFFSET(26) NUMBITS(11) [],
        cmp1gang OFFSET(25) NUMBITS(1) [],
        cmp0gang OFFSET(24) NUMBITS(1) [],
        cmp3center OFFSET(19) NUMBITS(1) [],
        cmp2center OFFSET(18) NUMBITS(1) [],
        cmp1center OFFSET(17) NUMBITS(1) [],
        cmp0center OFFSET(16) NUMBITS(1) [],
        enoneshot OFFSET(13) NUMBITS(1) [],
        enalways OFFSET(12) NUMBITS(1) [],
        deglitch OFFSET(10) NUMBITS(1) [],
        zerocmp OFFSET(9) NUMBITS(1) [],
        sticky OFFSET(8) NUMBITS(1) [],
        scale OFFSET(0) NUMBITS(4) []
    ]
];

pub struct Pwm {
    registers: StaticRef<PwmRegisters>,
}

impl Pwm {
    pub const fn new(base: StaticRef<PwmRegisters>) -> Pwm {
        Pwm {
            registers: base,
        }
    }

    /// Disable the PWM so it does not generate interrupts.
    pub fn disable(&self) {
        let regs = self.registers;

        // Turn the interrupt compare off so we don't get any RTC interrupts.
        regs.cfg.write(cfg::enalways::CLEAR + cfg::enalways::CLEAR);
    }
}
