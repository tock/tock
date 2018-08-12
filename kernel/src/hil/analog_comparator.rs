//! Interface for direct control of the analog comparators.

// Author: Danilo Verhaert <verhaert@cs.stanford.edu>
// Last modified August 9th, 2018

use returncode::ReturnCode;

pub trait AnalogComparator {
    /// The chip-dependent type of an analog comparator channel.
    type Channel;

    /// Do a single comparison of two inputs, depending on the AC chosen. Output
    /// will be True (1) when one is higher than the other, and False (0)
    /// otherwise.  Specifically, the output is True when Vp > Vn (Vin positive
    /// > Vin negative), and False if Vp < Vn.
    fn comparison(&self, channel: &Self::Channel) -> bool;

    /// Start interrupt-based comparison for the chosen channel (e.g. channel 1
    /// for AC1). This will make it listen and send an interrupt as soon as
    /// Vp > Vn.
    fn start_comparing(&self, channel: &Self::Channel) -> ReturnCode;

    /// Stop interrupt-based comparison for the chosen channel.
    fn stop_comparing(&self, channel: &Self::Channel) -> ReturnCode;
}

pub trait Client {
    /// Fires when handle_interrupt is called, returning the channel on which
    /// the interrupt occurred.
    fn fired(&self, usize);
}
