use crate::rcc::{AHBPrescaler, APBPrescaler, Rcc, SysClockSource};

/// HSI16 clock helper.
pub struct Hsi16;

impl Hsi16 {
    /// HSI16 nominal frequency in MHz.
    pub const FREQ_MHZ: usize = 16;

    /// Configure HSI16 as the system clock with /1 prescalers on all buses.
    ///
    /// Assumes:
    /// - default reset state with some other SYSCLK
    /// - VCORE range 1, VDD in nominal range (so 16 MHz needs 0 WS)
    pub fn configure_as_sysclk(rcc: &Rcc) {
        // 1. Enable HSI16
        rcc.enable_hsi_clock();

        // 2. Wait for HSI16 to be ready
        let mut timeout = 100_000;
        while !rcc.is_ready_hsi_clock() && timeout > 0 {
            timeout -= 1;
        }
        if timeout == 0 {
            panic!("HSI16 failed to start");
        }

        // 3. AHB/APB prescalers: keep everything at /1 for minimal bring-up
        rcc.set_ahb_prescaler(AHBPrescaler::DivideBy1);
        rcc.set_apb1_prescaler(APBPrescaler::DivideBy1);
        rcc.set_apb2_prescaler(APBPrescaler::DivideBy1);

        // 4. Switch SYSCLK to HSI16
        rcc.set_sys_clock_source(SysClockSource::HSI16);

        // 5. Wait until the switch is effective
        let mut timeout2 = 100_000;
        while rcc.get_sys_clock_source() != SysClockSource::HSI16 && timeout2 > 0 {
            timeout2 -= 1;
        }
        if timeout2 == 0 {
            panic!("SYSCLK failed to switch to HSI16");
        }
    }

    /// Frequency in MHz (helper for other code that wants the HSI16 rate).
    pub const fn freq_mhz() -> usize {
        Self::FREQ_MHZ
    }
}
