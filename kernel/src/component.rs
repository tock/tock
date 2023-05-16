// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components extend the functionality of the Tock kernel through a simple
//! factory method interface.

/// A component encapsulates peripheral-specific and capsule-specific
/// initialization for the Tock OS kernel in a factory method, which reduces
/// repeated code and simplifies the boot sequence.
///
/// The `Component` trait encapsulates all of the initialization and
/// configuration of a kernel extension inside the `finalize()` function call.
/// The `Output` type defines what type this component generates. Note that
/// instantiating a component does not instantiate the underlying `Output` type;
/// instead, the memory is statically allocated and provided as an argument to
/// the `finalize()` method, which correctly initializes the memory to
/// instantiate the `Output` object. If instantiating and initializing the
/// `Output` type requires parameters, these should be passed in the component's
/// `new()` function.
///
/// Using a component is as follows:
///
/// ```rust,ignore
/// let obj = CapsuleComponent::new(configuration, required_hw)
///     .finalize(capsule_component_static!());
/// ```
///
/// All required resources and configuration is passed via the constructor, and
/// all required static memory is defined by the `[name]_component_static!()`
/// macro and passed to the `finalize()` method.
pub trait Component {
    /// An optional type to specify the chip or board specific static memory
    /// that a component needs to setup the output object(s). This is the memory
    /// that `static_buf!()` would normally setup, but generic components cannot
    /// setup static buffers for types which are chip-dependent, so those
    /// buffers have to be passed in manually, and the `StaticInput` type makes
    /// this possible.
    type StaticInput;

    /// The type (e.g., capsule, peripheral) that this implementation of
    /// Component produces via `finalize()`. This is typically a static
    /// reference (`&'static`).
    type Output;

    /// A factory method that returns an instance of the Output type of this
    /// Component implementation. This is used in the boot sequence to
    /// instantiate and initialize part of the Tock kernel. This factory method
    /// may only be called once per Component instance.
    ///
    /// To enable reusable (i.e. can be used on multiple boards with different
    /// microcontrollers) and repeatable (i.e. can be instantiated multiple
    /// times on the same board) components, all components must follow this
    /// convention:
    ///
    /// - All statically allocated memory MUST be passed to `finalize()` via the
    ///   `static_memory` argument. The `finalize()` method MUST NOT use
    ///   `static_init!()` or `static_buf!()` directly. This restriction ensures
    ///   that memory is not aliased if the component is used multiple times.
    fn finalize(self, static_memory: Self::StaticInput) -> Self::Output;
}
