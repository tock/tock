use stm32u5xx::chip_specifics::clock_constants::{PllConstants, SystemClockConstants};
use stm32u5xx::chip_specifics::flash::{FlashChipSpecific, FlashLatency16};

pub enum Stm32u545Specs {}

impl PllConstants for Stm32u545Specs {
    const MIN_FREQ_MHZ: usize = 13;
    const MAX_FREQ_MHZ: usize = 100;
}

impl SystemClockConstants for Stm32u545Specs {
    const APB1_FREQUENCY_LIMIT_MHZ: usize = 160;
    const APB2_FREQUENCY_LIMIT_MHZ: usize = 160;
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 160;
}

impl FlashChipSpecific for Stm32u545Specs {
    type FlashLatency = FlashLatency16;

    fn get_number_wait_cycles_based_on_frequency(freq_mhz: usize) -> Self::FlashLatency {
        match freq_mhz {
            0..=30 => Self::FlashLatency::Latency0,
            31..=60 => Self::FlashLatency::Latency1,
            61..=90 => Self::FlashLatency::Latency2,
            91..=120 => Self::FlashLatency::Latency3,
            121..=150 => Self::FlashLatency::Latency4,
            _ => Self::FlashLatency::Latency5,
        }
    }
}
