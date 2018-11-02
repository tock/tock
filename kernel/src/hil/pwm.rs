//! Interface for PWM

use returncode::ReturnCode;

pub trait Signal {
    // Configure with 16-bit period length and on_period
    // Duty-cycle  = on_period/period
    fn configure(&self, period: u16, on_period: u16) -> ReturnCode;
}
