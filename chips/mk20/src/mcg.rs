//! Implementation of the Multipurpose Clock Generator
//!

use ::core::mem;

use regs::mcg::*;

pub use self::C1::CLKS::Value as OscSource;
pub use self::C1::FRDIV::Value as Frdiv;
pub use self::C2::RANGE::Value as OscRange;

pub enum State {
    Fei(Fei),
    Fee,
    Fbi,
    Fbe(Fbe),
    Pbe(Pbe),
    Pee,
    Blpi,
    Blpe,
    Stop,
}

#[derive(Copy,Clone)]
pub struct Fei;

#[derive(Copy,Clone)]
pub struct Fbe;

#[derive(Copy,Clone)]
pub struct Pbe;

pub fn state() -> State {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    let clks: OscSource = match mcg.c1.read(C1::CLKS) {
        1 => OscSource::Internal,
        2 => OscSource::External,
        _ => OscSource::LockedLoop
    };

    let irefs = mcg.c1.is_set(C1::IREFS);
    let plls = mcg.c6.is_set(C6::PLLS);
    let lp = mcg.c2.is_set(C2::LP);

    match (clks, irefs, plls, lp) {
        (OscSource::LockedLoop, true, false, _) => State::Fei(Fei),
        (OscSource::LockedLoop, false, false, _) => State::Fee,
        (OscSource::Internal, true, false, false) => State::Fbi,
        (OscSource::External, false, false, false) => State::Fbe(Fbe),
        (OscSource::LockedLoop, false, true, _) => State::Pee,
        (OscSource::External, false, true, false) => State::Pbe(Pbe),
        (OscSource::Internal, true, false, true) => State::Blpi,
        (OscSource::External, false, _, true) => State::Blpe,
        _ => panic!("Not in a recognized power mode!")
    }
}

pub struct Xtal {
    pub range: OscRange, 
    pub frdiv: Frdiv,
    pub load: ::osc::OscCapacitance
}

pub mod xtals {
    use mcg::{Xtal, OscRange, Frdiv};
    use osc::OscCapacitance;

    #[allow(non_upper_case_globals)]
    pub const Teensy16MHz: Xtal = Xtal { 
        range: OscRange::VeryHigh, 
        frdiv: Frdiv::Low16_High512,
        load: OscCapacitance::Load_10pF
    };
}

// Source: https://branan.github.io/teensy/2017/01/28/uart.html
impl Fei {
    pub fn use_xtal(self, xtal: Xtal) -> Fbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(C2::RANGE.val(xtal.range as u8) +
                      C2::EREFS::SET);

        mcg.c1.write(C1::CLKS::External +
                     C1::FRDIV.val(xtal.frdiv as u8) +
                     C1::IREFS::CLEAR);

        while !mcg.s.matches(S::OSCINIT0::SET + 
                             S::IREFST::CLEAR + 
                             S::CLKST::External) {}

        Fbe { }
    }
}

impl Fbe {
    pub fn enable_pll(self, multiplier: u8, divider: u8) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if multiplier < 16 || multiplier > 47 {
            panic!("Invalid PLL VCO divide factor: {}", multiplier);
        }
        if divider < 1 || divider > 8 {
            panic!("Invalid PLL reference divide factor: {}", divider);
        }

        mcg.c5.modify(C5::PRDIV.val(divider - 1));

        mcg.c6.modify(C6::VDIV.val(multiplier - 16) +
                      C6::PLLS::SET);

        // Wait for PLL to be selected and stable PLL lock
        while !mcg.s.matches(S::PLLST::SET + S::LOCK0::SET) {}

        Pbe { }
    }
}

impl Pbe {
    pub fn use_pll(self) {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(C1::CLKS::LockedLoop);

        while !mcg.s.matches(S::CLKST::Pll) {}
    }
}
