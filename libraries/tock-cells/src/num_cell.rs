//! NumCell convenience type

use core::cell::Cell;
use core::marker::Copy;
use core::ops::{Add, Sub};

/// `NumCell` is a simple wrapper around a `Cell` that restricts the type
/// `T` to a number. `NumCell` then provides convenient methods, like
/// `increment` and `add`. This means instead of this code:
///
///     cell_item.set(cell_item.get() + 10);
///
/// the code can look like:
///
///     cell_item.add(10);
pub struct NumCell<T: Add + Sub + Copy + From<usize>> {
    value: Cell<T>,
}

impl<T: Add<Output = T> + Sub<Output = T> + Copy + From<usize>> NumCell<T> {
    /// Create a new NumCell.
    pub const fn new(value: T) -> NumCell<T> {
        NumCell {
            value: Cell::new(value),
        }
    }

    /// Return a copy of the stored value.
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Update the stored value.
    pub fn set(&self, val: T) {
        self.value.set(val);
    }

    /// Add 1 to the stored value.
    pub fn increment(&self) {
        self.value.set(self.value.get() + T::from(1 as usize));
    }

    /// Subtract 1 from the stored value.
    pub fn decrement(&self) {
        self.value.set(self.value.get() - T::from(1 as usize));
    }

    /// Add the passed in value to the stored value.
    pub fn add(&self, val: T) {
        self.value.set(self.value.get() + val);
    }

    /// Subtract the passed in value from the stored value.
    pub fn subtract(&self, val: T) {
        self.value.set(self.value.get() - val);
    }
}
