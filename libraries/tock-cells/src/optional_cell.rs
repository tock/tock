//! `OptionalCell` convenience type

use core::cell::Cell;

/// `OptionalCell` is a `Cell` that wraps an `Option`. This is helper type
/// that makes keeping types that can be `None` a little cleaner.
#[derive(Default)]
pub struct OptionalCell<T> {
    value: Cell<Option<T>>,
}

impl<T> OptionalCell<T> {
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

    /// Insert the value of the supplied `Option`, or `None` if the supplied
    /// `Option` is `None`.
    pub fn insert(&self, opt: Option<T>) {
        self.value.set(opt);
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
        let value = self.value.take();
        let out = value.is_some();
        self.value.set(value);
        out
    }

    /// Check if the cell is None.
    pub fn is_none(&self) -> bool {
        let value = self.value.take();
        let out = value.is_none();
        self.value.set(value);
        out
    }

    /// Returns true if the option is a Some value containing the given value.
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        let value = self.value.take();
        let out = value.contains(x);
        self.value.set(value);
        out
    }

    /// Returns the contained value or panics if contents is `None`.
    pub fn expect(&self, msg: &str) -> T
    where
        T: Copy,
    {
        self.value.get().expect(msg)
    }

    // Note: Explicitly do not support unwrap, as we do not to encourage
    // panic'ing in the Tock kernel.

    /// Returns the contained value or a default.
    pub fn unwrap_or(&self, default: T) -> T
    where
        T: Copy,
    {
        self.value.get().unwrap_or(default)
    }

    /// Returns the contained value or computes a default.
    pub fn unwrap_or_else<F>(&self, default: F) -> T
    where
        T: Copy,
        F: FnOnce() -> T,
    {
        self.value.get().unwrap_or_else(default)
    }

    /// Call a closure on the value if the value exists.
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        T: Copy,
        F: FnOnce(&mut T) -> R,
    {
        self.value.get().map(|mut val| closure(&mut val))
    }

    /// Call a closure on the value if the value exists, or return the
    /// default if the value is `None`.
    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
    where
        T: Copy,
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
        T: Copy,
        D: FnOnce() -> U,
        F: FnOnce(&mut T) -> U,
    {
        self.value
            .get()
            .map_or_else(default, |mut val| closure(&mut val))
    }

    /// Transforms the contained `Option<T>` into a `Result<T, E>`, mapping
    /// `Some(v)` to `Ok(v)` and `None` to `Err(err)`.
    ///
    /// Arguments passed to `ok_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use `ok_or_else`,
    /// which is lazily evaluated.
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        self.value.into_inner().ok_or(err)
    }

    /// Transforms the contained `Option<T>` into a `Result<T, E>`, mapping
    /// `Some(v)` to `Ok(v)` and `None` to `Err(err)`.
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        self.value.into_inner().ok_or_else(err)
    }

    /// Returns `None` if the option is `None`, otherwise returns `optb`.
    pub fn and<U>(self, optb: Option<U>) -> Option<U> {
        self.value.into_inner().and(optb)
    }

    /// If the cell is empty, return `None`. Otherwise, call a closure
    /// with the value of the cell and return the result.
    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(&self, f: F) -> Option<U>
    where
        T: Copy,
    {
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
        self.value.into_inner().filter(predicate)
    }

    /// Returns the option if it contains a value, otherwise returns `optb`.
    ///
    /// Arguments passed to or are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use `or_else`, which
    /// is lazily evaluated.
    pub fn or(self, optb: Option<T>) -> Option<T> {
        self.value.into_inner().or(optb)
    }

    /// Returns the option if it contains a value, otherwise calls `f` and
    /// returns the result.
    pub fn or_else<F>(self, f: F) -> Option<T>
    where
        F: FnOnce() -> Option<T>,
    {
        self.value.into_inner().or_else(f)
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
        self.value.into_inner().unwrap_or_default()
    }
}
