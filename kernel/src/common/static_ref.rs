use core::ops::Deref;

/// A pointer to statically allocated mutable data such as memory mapped I/O
/// registers.
///
/// This is a simple wrapper around a raw pointer that encapsulates an unsafe
/// dereference in a safe manner. It serve the role of creating a `&'static T`
/// given a raw address and acts similarly to `extern` definitions, except
/// `StaticRef` is subject to module and crate bounderies, while `extern`
/// definitions can be imported anywhere.
///
/// TODO(alevy): move into `common` crate or replace with other mechanism.
pub struct StaticRef<T> {
    ptr: *const T,
}

impl<T> StaticRef<T> {
    /// Create a new `StaticRef` from a raw pointer
    ///
    /// ## Safety
    ///
    /// Callers must pass in a reference to statically allocated memory which
    /// does not overlap with other values.
    pub const unsafe fn new(ptr: *const T) -> StaticRef<T> {
        StaticRef { ptr: ptr }
    }
}

impl<T> Deref for StaticRef<T> {
    type Target = T;
    fn deref(&self) -> &'static T {
        unsafe { &*self.ptr }
    }
}
