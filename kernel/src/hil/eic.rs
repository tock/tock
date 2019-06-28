//! Interface for external interrupt controller.
//! Interrupt can be configured as asynchronous to operate
//! during deep sleep mode where the EIC clock is disabled.
//!
//! A basic use case:
//! A user button is configured for falling edge trigger and async mode
//!

// Author: Josh Zhang <jiashuoz@cs.princeton.edu>
// Last modified June 26, 2019

/// Enum for selecting which edge to trigger interrupts on.
#[derive(Debug)]
pub enum InterruptMode {
    RisingEdge,
    FallingEdge,
    HighLevel,
    LowLevel,
}

/// Enum for enabling/disabling filter
pub enum FilterMode {
    FilterEnable,
    FilterDisable,
}

/// Enum for selecting syn/asyn mode
pub enum SynchronizationMode {
    Synchronous,
    Asynchronous,
}

/// Interface for EIC.
pub trait ExternalInterruptController {
    type Line;

    /// Enables external interrupt on line_num
    /// In asychronous mode, all edge interrupts will be
    /// interpreted as level interrupts and the filter is disabled.
    fn line_enable(
        &self,
        line: &Self::Line,
        int_mode: InterruptMode,
        filter: FilterMode,
        syn_mode: SynchronizationMode,
    );

    /// Disables external interrupt on line_num
    fn line_disable(&self, line: &Self::Line);
}

/// Interface for users of EIC. In order
/// to execute interrupts, the user must implement
/// this `Client` interface.
pub trait Client {
    /// Called when an interrupt occurs.
    fn fired(&self);
}
