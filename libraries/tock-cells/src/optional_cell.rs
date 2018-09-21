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
        OptionalCell { value: Cell::new(Some(val)) }
    }

    /// Create an empty `OptionalCell` (contains just `None`).
    pub const fn empty() -> OptionalCell<T> {
        OptionalCell { value: Cell::new(None) }
    }

    /// Update the stored value.
    pub fn set(&self, val: T) {
        self.value.set(Some(val));
    }

    /// Insert the value of the supplied `Option`, or `None` if the supplied
    /// `Option` is `None`.
    pub fn insert(&self, opt: Option<T>) {
        match opt {
            Some(v) => self.set(v),
            None => self.clear(),
        }
    }

    /// Replace the contents with the supplied value.
    /// If the cell was not empty, the previous value is returned, otherwise
    /// `None` is returned.
    pub fn replace(&self, val: T) -> Option<T> {
        let prev = self.take();
        self.set(val);
        prev
    }

    /// Reset the stored value to `None`.
    pub fn clear(&self) {
        self.value.set(None);
    }

    /// Check if the cell contains something.
    pub fn is_some(&self) -> bool {
        self.value.get().is_some()
    }

    /// Check if the cell is None.
    pub fn is_none(&self) -> bool {
        self.value.get().is_none()
    }

    /// Returns the contained value or panics if contents is `None`.
    pub fn expect(&self, msg: &str) -> T {
        self.value.get().expect(msg)
    }

    // Note: Explicitly do not support unwrap, as we do not to encourage
    // panic'ing in the Tock kernel.

    /// Returns the contained value or a default.
    pub fn unwrap_or(&self, default: T) -> T {
        self.value.get().unwrap_or(default)
    }

    /// Returns the contained value or computes a default.
    pub fn unwrap_or_else<F>(&self, default: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.value.get().unwrap_or_else(default)
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
        self.value.get().map_or(
            default,
            |mut val| closure(&mut val),
        )
    }

    /// If the cell contains a value, call a closure supplied with the
    /// value of the cell. If the cell contains `None`, call the other
    /// closure to return a default value.
    pub fn map_or_else<U, D, F>(&self, default: D, closure: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&mut T) -> U,
    {
        self.value.get().map_or_else(
            default,
            |mut val| closure(&mut val),
        )
    }

    /// Transforms the contained `Option<T>` into a `Result<T, E>`, mapping
    /// `Some(v)` to `Ok(v)` and `None` to `Err(err)`.
    ///
    /// Arguments passed to `ok_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use `ok_or_else`,
    /// which is lazily evaluated.
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        self.value.get().ok_or(err)
    }

    /// Transforms the contained `Option<T>` into a `Result<T, E>`, mapping
    /// `Some(v)` to `Ok(v)` and `None` to `Err(err)`.
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        self.value.get().ok_or_else(err)
    }

    /// Returns `None` if the option is `None`, otherwise returns `optb`.
    pub fn and<U>(self, optb: Option<U>) -> Option<U> {
        self.value.get().and(optb)
    }

    /// If the cell is empty, return `None`. Otherwise, call a closure
    /// with the value of the cell and return the result.
    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(&self, f: F) -> Option<U> {
        self.value.get().and_then(f)
    }

    /// Returns `None` if the option is `None`, otherwise calls `predicate` with
    /// the wrapped value and returns:
    ///
    /// - `Some(t)` if `predicate` returns `true` (where `t` is the wrapped value), and
    /// - `None` if `predicate` returns `false`.
    pub fn filter<P>(self, predicate: P) -> Option<T>
    where
        P: FnOnce(&T) -> bool,
    {
        self.value.get().filter(predicate)
    }

    /// Returns the option if it contains a value, otherwise returns `optb`.
    ///
    /// Arguments passed to or are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use `or_else`, which
    /// is lazily evaluated.
    pub fn or(self, optb: Option<T>) -> Option<T> {
        self.value.get().or(optb)
    }

    /// Returns the option if it contains a value, otherwise calls `f` and
    /// returns the result.
    pub fn or_else<F>(self, f: F) -> Option<T>
    where
        F: FnOnce() -> Option<T>,
    {
        self.value.get().or_else(f)
    }

    /// Return the contained value and replace it with None.
    pub fn take(&self) -> Option<T> {
        self.value.take()
    }

    /// Returns the contained value or a default
    ///
    /// Consumes the `self` argument then, if `Some`, returns the contained
    /// value, otherwise if `None`, returns the default value for that type.
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.value.get().unwrap_or_default()
    }
}
