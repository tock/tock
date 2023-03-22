use crate::clk::pll::Pll;
use crate::clk::hsi::Hsi;
use crate::rcc::Rcc;

pub struct Clocks<'a> {
    pub hsi: Hsi<'a>,
    pub pll: Pll<'a>,
}

impl<'a> Clocks<'a> {
    pub fn new(rcc: &'a Rcc) -> Self {
        Self {
            hsi: Hsi::new(rcc),
            pll: Pll::new(rcc),
        }
    }
}
