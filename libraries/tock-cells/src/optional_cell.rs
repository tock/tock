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

    /// Update the stored value.
    pub fn set(&self, val: T) {
        self.value.set(Some(val));
    }

    /// Reset the stored value to `None`.
    pub fn clear(&self) {
        self.value.set(None);
    }

    /// Call a closure on the value if the value exists.
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.value.get().map(|mut val| closure(&mut val))
    }
}
