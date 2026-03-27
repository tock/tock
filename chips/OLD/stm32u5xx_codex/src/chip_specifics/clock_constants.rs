//! Clock-related constants for STM32U5xx chips.

/// PLL-related constants for specific for a specific chip
pub trait PllConstants {
    /// PLL minimum frequency in MHz
    const MIN_FREQ_MHZ: usize;
    /// PLL maximum frequency in MHz
    const MAX_FREQ_MHZ: usize;
}

/// Generic clock constants for a specific chip
pub trait SystemClockConstants {
    /// Maximum allowed APB1 frequency in MHz
    const APB1_FREQUENCY_LIMIT_MHZ: usize;
    /// Maximum allowed APB2 frequency in MHz
    ///
    /// On STM32U5, APB2 has the same limit as APB1, so this is
    /// typically set explicitly in the impl rather than relying
    /// on a generic 2x relationship.
    const APB2_FREQUENCY_LIMIT_MHZ: usize;
    /// Maximum allowed system clock frequency in MHz
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize;
}

/// Clock constants for a specific chip
pub trait ClockConstants: SystemClockConstants + PllConstants {}

impl<T: SystemClockConstants + PllConstants> ClockConstants for T {}

/// STM32U5xx clock limits (range 1, performance mode).
///
/// - Max SYSCLK / HCLK: 160 MHz
/// - Max APB1 / APB2 / APB3: 160 MHz
/// - PLL input: 4–16 MHz (not reflected here)
/// - PLL system-clock output: up to 160 MHz
pub struct Stm32u5ClockConstants;

impl PllConstants for Stm32u5ClockConstants {
    /// Minimum useful PLL system-clock frequency in MHz.
    /// (Datasheet allows down to 1 MHz, but anything lower than that
    /// is not practical for a Tock kernel.)
    const MIN_FREQ_MHZ: usize = 1;

    /// Maximum PLL frequency used for the system clock in MHz.
    const MAX_FREQ_MHZ: usize = 100;
}

impl SystemClockConstants for Stm32u5ClockConstants {
    /// APB1 max frequency in MHz
    const APB1_FREQUENCY_LIMIT_MHZ: usize = 160;

    /// APB2 max frequency in MHz (same as APB1 on STM32U5)
    const APB2_FREQUENCY_LIMIT_MHZ: usize = 160;

    /// System clock (SYSCLK/HCLK) max frequency in MHz
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 160;
}
