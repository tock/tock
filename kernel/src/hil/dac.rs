//! Interface for digital to analog converters.

use crate::returncode::ReturnCode;

/// Simple interface for using the DAC.
pub trait DacChannel {
    /// Initialize and enable the DAC.
    fn initialize(&self) -> ReturnCode;

    /// Set the DAC output value.
    fn set_value(&self, value: usize) -> ReturnCode;
}
