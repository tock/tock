use returncode::ReturnCode;

/// Trait for handling callbacks from ADC module.
pub trait Client {
    /// Called when a sample is ready.
    fn sample_done(&self, sample: u16);
}

/// Simple interface for reading a single ADC sample on any channel.
pub trait AdcSingle {
    /// Initialize must be called before taking a sample.
    /// Returns true on success.
    fn initialize(&self) -> ReturnCode;

    /// Request a single ADC sample on a particular channel.
    /// Returns true on success.
    fn sample(&self, channel: u8) -> ReturnCode;
    fn cancel_sample(&self) -> ReturnCode;
}

pub trait Frequency {
    fn frequency() -> u32;
}

#[derive(Debug)]
pub struct Freq1KHz;
impl Frequency for Freq1KHz {
    fn frequency() -> u32 {
        1000
    }
}

pub trait AdcContinuous {
    type Frequency: Frequency;
    fn compute_interval(&self, interval: u32) -> u32;
    fn sample_continuous(&self, channel: u8, interval: u32) -> ReturnCode;
    fn cancel_sampling(&self) -> ReturnCode;
}
