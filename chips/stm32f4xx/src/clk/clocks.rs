use crate::clk::pll::Pll;
use crate::clk::hsi::Hsi;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;

use kernel::debug;
use kernel::ErrorCode;

pub struct Clocks<'a> {
    rcc: &'a Rcc,
    pub hsi: Hsi<'a>,
    pub pll: Pll<'a>,
}

impl<'a> Clocks<'a> {
    pub fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
            hsi: Hsi::new(rcc),
            pll: Pll::new(rcc),
        }
    }

    pub fn set_sys_clock_source(&self, source: SysClockSource) -> Result<(), ErrorCode> {
        if source == self.get_sys_clock_source() {
            return Ok(());
        }
        if let false = match source {
            SysClockSource::HSI => self.hsi.is_enabled(),
            SysClockSource::PLLCLK => self.pll.is_enabled(),
        } {
            return Err(ErrorCode::FAIL);
        }

        let current_frequency = self.get_sys_clock_frequency();
        let alternate_frequency = match source {
            // The unwrap can't failed because the source clock status was checked before
            SysClockSource::HSI => self.hsi.get_frequency().unwrap(),
            SysClockSource::PLLCLK => self.pll.get_frequency().unwrap(),
        };

        if alternate_frequency > current_frequency {
            todo!();
        }
        //else {
            //todo!();
        //}

        self.rcc.set_sys_clock_source(source);

        Ok(())
    }

    pub fn get_sys_clock_source(&self) -> SysClockSource {
        self.rcc.get_sys_clock_source()
    }

    pub fn get_sys_clock_frequency(&self) -> usize {
        match self.get_sys_clock_source() {
            SysClockSource::HSI => self.hsi.get_frequency().unwrap(),
            SysClockSource::PLLCLK => self.pll.get_frequency().unwrap(),
        }
    }
}

pub mod tests {
    use super::*;

    pub fn run(clocks: &Clocks) {
        debug!("");
        debug!("===============================================");
        debug!("Testing clocks...");

        // By default, the HSI clock is the system clock
        assert_eq!(SysClockSource::HSI, clocks.get_sys_clock_source());

        // HSI frequency is 16MHz
        assert_eq!(16, clocks.get_sys_clock_frequency());

        // Attempting to change the system clock source with a disabled source
        assert_eq!(Err(ErrorCode::FAIL), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Attempting to set twice the same system clock source is fine
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));

        // Change the system clock source
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Attempting to disable PLL when it is configured as the system clock must fail
        //assert_eq!(Err(ErrorCode::FAIL), clocks.pll.disable());
        // Same for the HSI since it is used indirectly as a system clock through PLL
        //assert_eq!(Err(ErrorCode::FAIL), clocks.hsi.disable());

        debug!("Finished testing clocks. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
