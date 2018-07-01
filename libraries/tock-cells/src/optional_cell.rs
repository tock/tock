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

    /// Replace the contents with the value from the supplied `Option`,
    /// or empty this `OptionalCell` if the supplied `Option` is `None`.
    pub fn replace(&self, option: Option<T>) {
        match option {
            Some(v) => self.set(v),
            None => self.clear(),
        }
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

    /// Call a closure on the value if the value exists, or return the
    /// default if the value is `None`.
    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.value
            .get()
            .map_or(default, |mut val| closure(&mut val))
    }

    /// If the cell contains a value, call a closure supplied with the
    /// value of the cell. If the cell contains `None`, call the other
    /// closure to return a default value.
    pub fn map_or_else<U, D, F>(&self, default: D, closure: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&mut T) -> U,
    {
        self.value
            .get()
            .map_or_else(default, |mut val| closure(&mut val))
    }

    /// If the cell is empty, return `None`. Otherwise, call a closure
    /// with the value of the cell and return the result.
    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(&self, f: F) -> Option<U> {
        self.value.get().and_then(f)
    }
}
