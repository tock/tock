// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! `OptionalCell` convenience type

use core::cell::Cell;

/// `OptionalCell` is a `Cell` that wraps an `Option`. This is helper type
/// that makes keeping types that can be `None` a little cleaner.
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
        let out = match &value {
            Some(y) => y == x,
            None => false,
        };
        self.value.set(value);
        out
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

impl<T: Copy> OptionalCell<T> {
    /// Returns a copy of the contained [`Option`].
    //
    // This was originally introduced in PR #2531 [1], then renamed to `extract`
    // in PR #2533 [2], and finally renamed back in PR #3536 [3].
    //
    // The rationale for including a `get` method is to allow developers to
    // treat an `OptionalCell<T>` as what it is underneath: a `Cell<Option<T>>`.
    // This is useful to be interoperable with APIs that take an `Option<T>`, or
    // to use an *if-let* or *match* expression to perform case-analysis on the
    // `OptionalCell`'s state: this avoids using a closure and can thus allow
    // Rust to deduce that only a single branch will ever be entered (either the
    // `Some(_)` or `None`) branch, avoiding lifetime & move restrictions.
    //
    // However, there was pushback for that name, as an `OptionalCell`'s `get`
    // method might indicate that it should directly return a `T` -- given that
    // `OptionalCell<T>` presents itself as to be a wrapper around
    // `T`. Furthermore, adding `.get()` might have developers use
    // `.get().map(...)` instead, which defeats the purpose of having the
    // `OptionalCell` convenience wrapper in the first place. For these reasons,
    // `get` was renamed to `extract`.
    //
    // Unfortunately, `extract` turned out to be a confusing name, as it is not
    // an idiomatic method name as found on Rust's standard library types, and
    // further suggests that it actually removes a value from the `OptionalCell`
    // (as the `take` method does). Thus, it has been renamed back to `get`.
    //
    // [1]: https://github.com/tock/tock/pull/2531
    // [2]: https://github.com/tock/tock/pull/2533
    // [3]: https://github.com/tock/tock/pull/3536
    pub fn get(&self) -> Option<T> {
        self.value.get()
    }

    /// Returns the contained value or panics if contents is `None`.
    /// We do not use the traditional name for this function -- `unwrap()`
    /// -- because the Tock kernel discourages panicking, and this name
    /// is intended to discourage users from casually adding calls to
    /// `unwrap()` without careful consideration.
    #[track_caller]
    pub fn unwrap_or_panic(&self) -> T {
        self.value.get().unwrap()
    }

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
        F: FnOnce(T) -> R,
    {
        self.value.get().map(closure)
    }

    /// Call a closure on the value if the value exists, or return the
    /// default if the value is `None`.
    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        self.value.get().map_or(default, closure)
    }

    /// If the cell contains a value, call a closure supplied with the
    /// value of the cell. If the cell contains `None`, call the other
    /// closure to return a default value.
    pub fn map_or_else<U, D, F>(&self, default: D, closure: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        self.value.get().map_or_else(default, closure)
    }

    /// If the cell is empty, return `None`. Otherwise, call a closure
    /// with the value of the cell and return the result.
    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(&self, f: F) -> Option<U> {
        self.value.get().and_then(f)
    }
}

// Manual implementation of the [`Default`] trait, as
// `#[derive(Default)]` incorrectly constraints `T: Default`.
impl<T> Default for OptionalCell<T> {
    /// Returns an empty [`OptionalCell`].
    fn default() -> Self {
        OptionalCell::empty()
    }
}
