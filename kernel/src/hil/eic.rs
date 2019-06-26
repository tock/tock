//! Interface for external interrupt controller.

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
    /// Enables EIC module
    fn enable(&self);

    /// Disables EIC module
    fn disable(&self);

    /// Enables external interrupt on line_num
    /// In asychronous mode, all edge interrupts will be interpreted as level interrupts and the filter is disabled.
    fn line_enable(&self, line_num: usize);

    /// Disables external interrupt on line_num
    fn line_disable(&self, line_num: usize);

    /// Configure external interrupt on line_num
    fn line_configure(&self, line_num: usize, int_mode: InterruptMode, filter: FilterMode, syn_mode: SynchronizationMode);
}

/// Interface for users of EIC. In order
/// to execute interrupts, the user must implement
/// this `Client` interface.
pub trait Client {
    /// Called when an interrupt occurs.
    fn fired(&self);
}