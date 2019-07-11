//! Interface for external interrupt controller.
//!
//! The External Interrupt Controller (EIC) allows pins to be configured as
//! external interrupts. Each external interrupt has its own interrupt request
//! and can be individually masked. Each external interrupt can generate an
//! interrupt on rising or falling edge, or high or low level.
//! Every interrupt pin can also be configured to be asynchronous, in order to
//! wake-up the part from sleep modes where the CLK_SYNC clock has been disabled.
//!
//! A basic use case:
//! A user button is configured for falling edge trigger and async mode.

/// Enum for selecting which edge to trigger interrupts on.
#[derive(Debug)]
pub enum InterruptMode {
    RisingEdge,
    FallingEdge,
    HighLevel,
    LowLevel,
}

/// Interface for EIC.
pub trait ExternalInterruptController {
    /// The chip-dependent type of an EIC line. Number of lines available depends on the chip.
    type Line;

    /// Enables external interrupt on the given 'line'
    /// In asychronous mode, all edge interrupts will be
    /// interpreted as level interrupts and the filter is disabled.
    fn line_enable(
        &self,
        line: &Self::Line,
        interrupt_mode: InterruptMode,
    );

    /// Disables external interrupt on the given 'line'
    fn line_disable(&self, line: &Self::Line);
}

/// Interface for users of EIC. In order
/// to execute interrupts, the user must implement
/// this `Client` interface.
pub trait Client {
    /// Called when an interrupt occurs.
    fn fired(&self);
}
