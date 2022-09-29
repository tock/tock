/// Interface for interacting with debug hardware integrated in various SoCs.
/// Currently allows reading the cycle counter and can be expanded to allow
/// access to other features in the future.
use crate::ErrorCode;

pub trait PerformanceCounters {
    /// Enable the cycle counter. Returns an error, if the cycle counter is not present.
    fn enable_cycle_counter() -> Result<(), ErrorCode>;

    /// Disable the cycle counter. Returns an error, if the cycle counter is not present.
    fn disable_cycle_counter() -> Result<(), ErrorCode>;

    /// Return the current value of the cycle counter.
    fn cycle_count() -> u32;
}
