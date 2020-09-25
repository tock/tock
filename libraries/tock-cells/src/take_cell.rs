//! Tock specific `TakeCell` type for sharing references.

use core::cell::Cell;

/// A shared reference to a mutable reference.
///
/// A `TakeCell` wraps potential reference to mutable memory that may be
/// available at a given point. Rather than enforcing borrow rules at
/// compile-time, `TakeCell` enables multiple clients to hold references to it,
/// but ensures that only one referrer has access to the underlying mutable
/// reference at a time. Clients either move the memory out of the `TakeCell` or
/// operate on a borrow within a closure. Attempts to take the value from inside
/// a `TakeCell` may fail by returning `None`.
pub struct TakeCell<'a, T: 'a + ?Sized> {
    val: Cell<Option<&'a mut T>>,
}

impl<'a, T: ?Sized> TakeCell<'a, T> {
    pub const fn empty() -> TakeCell<'a, T> {
        TakeCell {
            val: Cell::new(None),
        }
    }

    /// Creates a new `TakeCell` containing `value`
    pub const fn new(value: &'a mut T) -> TakeCell<'a, T> {
        TakeCell {
            val: Cell::new(Some(value)),
        }
    }

    pub fn is_none(&self) -> bool {
        let inner = self.take();
        let return_val = inner.is_none();
        self.val.set(inner);
        return_val
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Takes the mutable reference out of the `TakeCell` leaving a `None` in
    /// it's place. If the value has already been taken elsewhere (and not
    /// `replace`ed), the returned `Option` will be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate tock_cells;
    /// use tock_cells::take_cell::TakeCell;
    ///
    /// let mut value = 1234;
    /// let cell = TakeCell::new(&mut value);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.take();
    /// assert_eq!(y.take(), None);
    /// ```
    pub fn take(&self) -> Option<&'a mut T> {
        self.val.replace(None)
    }

    /// Stores `val` in the `TakeCell`
    pub fn put(&self, val: Option<&'a mut T>) {
        self.val.replace(val);
    }

    /// Replaces the contents of the `TakeCell` with `val`. If the cell was not
    /// empty, the previous value is returned, otherwise `None` is returned.
    pub fn replace(&self, val: &'a mut T) -> Option<&'a mut T> {
        self.val.replace(Some(val))
    }

    /// Retrieves a mutable reference to the inner value that only lives as long
    /// as the reference to this does.
    ///
    /// This escapes the "take" aspect of TakeCell in a way which is guaranteed
    /// safe due to the returned reference sharing the lifetime of `&mut self`.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.val.get_mut().as_mut().map(|v| &mut **v)
    }

    /// Allows `closure` to borrow the contents of the `TakeCell` if-and-only-if
    /// it is not `take`n already. The state of the `TakeCell` is unchanged
    /// after the closure completes.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate tock_cells;
    /// use tock_cells::take_cell::TakeCell;
    ///
    /// let mut value = 1234;
    /// let cell = TakeCell::new(&mut value);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.map(|value| {
    ///     // We have mutable access to the value while in the closure
    ///     *value += 1;
    /// });
    ///
    /// // After the closure completes, the mutable memory is still in the cell,
    /// // but potentially changed.
    /// assert_eq!(y.take(), Some(&mut 1235));
    /// ```
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let maybe_val = self.take();
        maybe_val.map(|mut val| {
            let res = closure(&mut val);
            self.replace(val);
            res
        })
    }

    /// Performs a `map` or returns a default value if the `TakeCell` is empty
    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let maybe_val = self.take();
        maybe_val.map_or(default, |mut val| {
            let res = closure(&mut val);
            self.replace(val);
            res
        })
    }

    /// Performs a `map` or generates a value with the default
    /// closure if the `TakeCell` is empty
    pub fn map_or_else<U, D, F>(&self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&mut T) -> U,
    {
        let maybe_val = self.take();
        maybe_val.map_or_else(
            || default(),
            |mut val| {
                let res = f(&mut val);
                self.replace(val);
                res
            },
        )
    }

    /// Behaves the same as `map`, except the closure is allowed to return
    /// an `Option`.
    pub fn and_then<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> Option<R>,
    {
        let maybe_val = self.take();
        maybe_val.and_then(|mut val| {
            let res = closure(&mut val);
            self.replace(val);
            res
        })
    }

    /// Uses the first closure (`modify`) to modify the value in the `TakeCell`
    /// if it is present, otherwise, fills the `TakeCell` with the result of
    /// `mkval`.
    pub fn modify_or_replace<F, G>(&self, modify: F, mkval: G)
    where
        F: FnOnce(&mut T),
        G: FnOnce() -> &'a mut T,
    {
        let val = match self.take() {
            Some(mut val) => {
                modify(&mut val);
                val
            }
            None => mkval(),
        };
        self.replace(val);
    }
}
