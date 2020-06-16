//! Hardware interface layer (HIL) for Emergency Serial
//!

use crate::returncode::ReturnCode;

/// This is a serial-like interface designated for use only in exceptional
/// situations. In contrast to other kernel interfaces, all methods here are
/// synchronous. Implementations of this HIL should be as conservative as
/// possible, use as few chip/board features as possible, and rely on as
/// little as possible from the rest of the kernel.
///
/// The primary use case for this interface is to support panic! messages.
/// Certain debug primitives may also leverage this interface, however such
/// debuggging is intended exclusively for active development work. Only
/// panic! may use this interface in mainline code.
pub trait EmergencySerial {
    /// Called once before any other methods. This is responsible for ensuring
    /// that subsequent writes will succeed. If appropriate, other users of the
    /// underlying serial interface should be shut down (e.g. cancelling DMA).
    /// This shutdown **does not** need to be graceful, nor should it trigger
    /// other kernel code to run if possible (e.g. disable DMA interrupts
    /// before cancelling transactions so the DMA handler will not run).
    fn initialize(&self);

    /// Send a byte over the serial interface. This method must block until the
    /// byte has been sent.
    fn write_byte(&self, byte: u8);
}
