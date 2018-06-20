/// A Component extends the functionality of the Tock kernel.
/// This abstraction is intended to make the kernel boot sequence simpler:
/// without it, the reset_handler involves lots of driver-specific
/// initialization. The Component trait encapsulates all of the
/// initialziation and configuration of a kernel extension inside
/// the finalize function call.
///
/// Note that instantiating a component does not necessarily instantiate
/// the underlying Output type; this can be instantiated when finalize()
/// is called.

pub trait Component {
    type Output;
    unsafe fn finalize(&mut self) -> Self::Output;
}

/// This trait allows components to set up circular references.
/// If component A requires a reference to component B, and B
/// requires a reference to A, then one of them can implement this
/// trait. For example, A can implement ComponentWithDependency<B>.
/// When A is constructed, B does not exist yet. When B is constructed,
/// A exists, so can be passed to new(). Then, B can be passed to A
/// via a call to dependency(). After both dependencies are resolved,
/// the boot sequence can call finalize() on both of them.

pub trait ComponentWithDependency<D>: Component {
    fn dependency(&mut self, _dep: D) -> &mut Self {
        self
    }
}
