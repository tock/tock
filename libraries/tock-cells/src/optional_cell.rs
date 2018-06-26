//! OptionalCell convenience type

use core::cell::Cell;
use core::marker::Copy;

/// `OptionalCell` is a `Cell` that wraps an `Option`. This is helper type
/// that makes keeping types that can be `None` a little cleaner.
pub struct OptionalCell<T: Copy> {
    value: Cell<Option<T>>,
}

impl<T: Copy> OptionalCell<T> {
    /// Create a new OptionalCell.
    pub const fn new(val: T) -> OptionalCell<T> {
        OptionalCell {
            value: Cell::new(Some(val)),
        }
    }

    /// Create an empty `OptionalCell` (contains just `None`).
    pub const fn empty() -> OptionalCell<T> {
        OptionalCell {
            value: Cell::new(None),
        }
    }

    /// Check if the cell is None.
    pub fn is_none(&self) -> bool {
        self.value.get().is_none()
    }

    /// Check if the cell contains something.
    pub fn is_some(&self) -> bool {
        self.value.get().is_some()
    }

    /// Update the stored value.
    pub fn set(&self, val: T) {
        self.value.set(Some(val));
    }

    /// Reset the stored value to `None`.
    pub fn clear(&self) {
        self.value.set(None);
    }

    /// Return the contained value and replace it with None.
    pub fn take(&self) -> Option<T> {
        self.value.take()
    }

    /// Call a closure on the value if the value exists.
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.value.get().map(|mut val| closure(&mut val))
    }

    /// Call a closure on the value if the value exists, or return value
    /// instead.
    pub fn map_or<F, R>(&self, value: R, closure: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.value.get().map_or(value, |mut val| closure(&mut val))
    }
}
