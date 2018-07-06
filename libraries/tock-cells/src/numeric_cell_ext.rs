//! `NumericCellExt` extention trait for `Cell`s.
//!
//! Adds a suite of convenience functions to `Cell`s that contain numeric
//! types. Cells that contains types that can meaningfully execute arithmetic
//! operations can use mechanisms such as `cell.add(val)` rather than
//! `cell.set(cell.get() + val)`.
//!
//! To use these traits, simply pull them into scope:
//!
//! ```rust
//! use kernel::common::cells::numeric_cell_ext;
//! ```

use core::cell::Cell;
use core::marker::Copy;
use core::ops::{Add, Sub};

pub trait NumericCellExt<T>
where
    T: Copy + Add + Sub,
{
    /// Add the passed in `val` to the stored value.
    fn add(&self, val: T);

    /// Subtract the passed in `val` from the stored value.
    fn subtract(&self, val: T);

    /// Add 1 to the stored value.
    fn increment(&self);

    /// Subtract 1 from the stored value.
    fn decrement(&self);
}

impl<T> NumericCellExt<T> for Cell<T>
where
    T: Add<Output = T> + Sub<Output = T> + Copy + From<usize>,
{
    fn add(&self, val: T) {
        self.set(self.get() + val);
    }

    fn subtract(&self, val: T) {
        self.set(self.get() - val);
    }

    fn increment(&self) {
        self.set(self.get() + T::from(1 as usize));
    }

    fn decrement(&self) {
        self.set(self.get() - T::from(1 as usize));
    }
}
