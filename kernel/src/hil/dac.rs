//! Interface for digital to analog converters.

use crate::ErrorCode;

/// Simple interface for using the DAC.
pub trait DacChannel {
    /// Initialize and enable the DAC.
    fn initialize(&self) -> Result<(), ErrorCode>;

    /// Set the DAC output value.
    fn set_value(&self, value: usize) -> Result<(), ErrorCode>;
}
