//! Components extend the functionality of the Tock kernel through a
//! simple factory method interface.

/// Encapsulates peripheral-specific and capsule-specific
/// initialization for the Tock OS kernel in a factory method,
/// reducing repeated code and simplifying the boot sequence.
///
/// The Component trait encapsulates all of the initialization and
/// configuration of a kernel extension inside the finalize function
/// call. The Output type defines what type this component generates.
/// Note that instantiating a component does not necessarily
/// instantiate the underlying Output type; instead, it is typically
/// instantiated in the call to finalize() is called. If instantiating
/// and initializing the Output type requires parameters, these should
/// be passed in the Component's new() function.

pub trait Component {
    /// The type (e.g., capsule, peripheral) that this implementation
    /// of Component produces via finalize. This is typically a
    /// static reference (`&'static`).
    type Output;

    /// A factory method that returns an instance of the Output type of
    /// this Component implementation. Used in the boot sequence to
    /// instantiate and initalize part of the Tock kernel.
    unsafe fn finalize(&mut self) -> Self::Output;
}
