//! Components extend the functionality of the Tock kernel through a
//! simple factory method interface.

/// A component encapsulates peripheral-specific and capsule-specific
/// initialization for the Tock OS kernel in a factory method,
/// which reduces repeated code and simplifies the boot sequence.
///
/// The `Component` trait encapsulates all of the initialization and
/// configuration of a kernel extension inside the `finalize()` function
/// call. The `Output` type defines what type this component generates.
/// Note that instantiating a component does not necessarily
/// instantiate the underlying `Output` type; instead, it is typically
/// instantiated in the `finalize()` method. If instantiating and
/// initializing the `Output` type requires parameters, these should be
/// passed in the component's `new()` function.
pub trait Component {
    /// An optional type to specify the chip or board specific static memory
    /// that a component needs to setup the output object(s). This is the memory
    /// that `static_init!()` would normally setup, but generic components
    /// cannot setup static buffers for types which are chip-dependent, so those
    /// buffers have to be passed in manually, and the `StaticInput` type makes
    /// this possible.
    type StaticInput = ();

    /// The type (e.g., capsule, peripheral) that this implementation
    /// of Component produces via `finalize()`. This is typically a
    /// static reference (`&'static`).
    type Output;

    /// A factory method that returns an instance of the Output type of this
    /// Component implementation. This factory method may only be called once
    /// per Component instance. Used in the boot sequence to instantiate and
    /// initialize part of the Tock kernel. Some components need to use the
    /// `static_memory` argument to allow the board initialization code to pass
    /// in references to static memory that the component will use to setup the
    /// Output type object.
    unsafe fn finalize(self, static_memory: Self::StaticInput) -> Self::Output;
}

pub trait CreateComponent {
    type Input;
    type StaticInput;
    type Output;

    unsafe fn create(input: Self::Input, static_input: Self::StaticInput) -> Self::Output;
}
