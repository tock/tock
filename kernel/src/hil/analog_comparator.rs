//! Interface for direct control of the analog comparators.
//!
//! Author: Danilo Verhaert <verhaert@cs.stanford.edu>

use returncode::ReturnCode;

pub trait AnalogComparator {
    /// Initialize the ACIFC by enabling the clock, activating the ACs (Analog
    /// Comparators). Currently in initialization always-on mode is enabled,
    /// allowing a measurement on an AC to be made quickly after a measurement
    /// is triggered, without waiting for the AC startup time. The drawback is
    /// that when the AC is always on the power dissipation will be higher.
    fn enable(&self) -> ReturnCode;

    /// Do a comparison of two inputs, depending on the AC chosen. Output will
    /// be True (1) when one is higher than the other, and False (0) otherwise.
    /// Specifically, the output is True when Vp > Vn (Vin positive > Vin
    /// negative), and False if Vp < Vn.
    fn comparison(&self, usize) -> bool;

    /// Do a comparison of three input voltages. Two analog comparators, ACx and ACx+1, are
    /// grouped for this comparison depending on the window chosen. They each have a positive and negative input: we
    /// define these respectively as (Vp_x and Vn_x) for ACx and (Vp_x+1 and Vn_x+1) for ACx+1.
    /// The sources of the negative input of ACx
    /// (Vn_x) and the positive input of ACx+1 (Vp_x+1) must be connected
    /// together externally as a prerequisite to use the windowed mode. These
    /// then together form the common voltage Vcommon.
    /// The way the windowed mode works is then as follows. The two remaining sources, being the positive input of ACx (Vp_x) and
    /// negative input of ACx+1 (Vn_x+1) define an upper and a lower bound
    /// of a window. The result of the comparison then depends on Vcommon lying inside of outside
    /// of this window.
    /// When the value of Vcommon lies inside this window,
    /// the output will be True (1); it will be False (0) if it lies outside of the window.
    /// Specifically, the output will be True when Vn_x+1 < Vcommon < Vp_x, and False
    /// if Vcommon < Vn_x+1 or Vcommon > Vp_x.
    fn window_comparison(&self, usize) -> bool;

    /// Enable interrupts. This will make the AC corresponding to the variable
    /// passed (e.g. AC1) send an interrupt when Vp > Vn.
    fn enable_interrupts(&self, usize) -> ReturnCode;
}

pub trait Client {
    /// Called when an interrupt occurs.
    fn fired(&self);
}
