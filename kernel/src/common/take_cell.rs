//! Tock specific `Cell` types for sharing references.

use core::{mem, ptr};
use core::cell::{Cell, UnsafeCell};

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
    val: UnsafeCell<Option<&'a mut T>>,
}

impl<'a, T: ?Sized> TakeCell<'a, T> {
    pub const fn empty() -> TakeCell<'a, T> {
        TakeCell {
            val: UnsafeCell::new(None),
        }
    }

    /// Creates a new `TakeCell` containing `value`
    pub fn new(value: &'a mut T) -> TakeCell<'a, T> {
        TakeCell {
            val: UnsafeCell::new(Some(value)),
        }
    }

    pub fn is_none(&self) -> bool {
        unsafe { (&*self.val.get()).is_none() }
    }

    pub fn is_some(&self) -> bool {
        unsafe { (&*self.val.get()).is_some() }
    }

    /// Takes the mutable reference out of the `TakeCell` leaving a `None` in
    /// it's place. If the value has already been taken elsewhere (and not
    /// `replace`ed), the returned `Option` will be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let cell = TakeCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.take();
    /// assert_eq!(y.take(), None);
    /// ```
    pub fn take(&self) -> Option<&'a mut T> {
        unsafe {
            let inner = &mut *self.val.get();
            inner.take()
        }
    }

    /// Stores `val` in the `TakeCell`
    pub fn put(&self, val: Option<&'a mut T>) {
        let _ = self.take();
        let ptr = self.val.get();
        unsafe {
            ptr::replace(ptr, val);
        }
    }

    /// Replaces the contents of the `TakeCell` with `val`. If the cell was not
    /// empty, the previous value is returned, otherwise `None` is returned.
    pub fn replace(&self, val: &'a mut T) -> Option<&'a mut T> {
        let prev = self.take();
        let ptr = self.val.get();
        unsafe {
            ptr::replace(ptr, Some(val));
        }
        prev
    }

    /// Allows `closure` to borrow the contents of the `TakeCell` if-and-only-if
    /// it is not `take`n already. The state of the `TakeCell` is unchanged
    /// after the closure completes.
    ///
    /// # Examples
    ///
    /// ```
    /// let cell = TakeCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.map(|value| {
    ///     // We have mutable access to the value while in the closure
    ///     value += 1;
    /// });
    ///
    /// // After the closure completes, the mutable memory is still in the cell,
    /// // but potentially changed.
    /// assert_eq!(y.take(), Some(1235));
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

/// A mutable memory location that enforces borrow rules at runtime without
/// possible panics.
///
/// A `MapCell` is a potential reference to mutable memory. Borrow rules are
/// enforced by forcing clients to either move the memory out of the cell or
/// operate on a borrow within a closure. You can think of a `MapCell` as an
/// `Option` wrapped in a `RefCell` --- attempts to take the value from inside a
/// `MapCell` may fail by returning `None`.
pub struct MapCell<T> {
    val: UnsafeCell<T>,
    occupied: Cell<bool>,
}

impl<T> MapCell<T> {
    pub fn empty() -> MapCell<T> {
        MapCell {
            val: unsafe { mem::uninitialized() },
            occupied: Cell::new(false),
        }
    }

    /// Creates a new `MapCell` containing `value`
    pub const fn new(value: T) -> MapCell<T> {
        MapCell {
            val: UnsafeCell::new(value),
            occupied: Cell::new(true),
        }
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn is_some(&self) -> bool {
        self.occupied.get()
    }

    /// Takes the value out of the `MapCell` leaving it empty. If
    /// the value has already been taken elsewhere (and not `replace`ed), the
    /// returned `Option` will be `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cell = MapCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// assert_eq!(x.take(), Some(1234));
    /// assert_eq!(y.take(), None);
    /// ```
    pub fn take(&self) -> Option<T> {
        if self.is_none() {
            return None;
        } else {
            self.occupied.set(false);
            unsafe { Some(ptr::replace(self.val.get(), mem::uninitialized())) }
        }
    }

    pub fn put(&self, val: T) {
        self.occupied.set(true);
        unsafe {
            ptr::write(self.val.get(), val);
        }
    }

    /// Replaces the contents of the `MapCell` with `val`. If the cell was not
    /// empty, the previous value is returned, otherwise `None` is returned.
    pub fn replace(&self, val: T) -> Option<T> {
        if self.is_some() {
            unsafe { Some(ptr::replace(self.val.get(), val)) }
        } else {
            self.put(val);
            None
        }
    }

    /// Allows `closure` to borrow the contents of the `MapCell` if-and-only-if
    /// it is not `take`n already. The state of the `MapCell` is unchanged
    /// after the closure completes.
    ///
    /// # Examples
    ///
    /// ```
    /// let cell = MapCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// x.map(|value| {
    ///     // We have mutable access to the value while in the closure
    ///     value += 1;
    /// });
    ///
    /// // After the closure completes, the mutable memory is still in the cell,
    /// // but potentially changed.
    /// assert_eq!(y.take(), Some(1235));
    /// ```
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        if self.is_some() {
            self.occupied.set(false);
            let valref = unsafe { &mut *self.val.get() };
            let res = closure(valref);
            self.occupied.set(true);
            Some(res)
        } else {
            None
        }
    }

    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.map(closure).unwrap_or(default)
    }

    /// Behaves the same as `map`, except the closure is allowed to return
    /// an `Option`.
    pub fn and_then<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> Option<R>,
    {
        if self.is_some() {
            self.occupied.set(false);
            let valref = unsafe { &mut *self.val.get() };
            let res = closure(valref);
            self.occupied.set(true);
            res
        } else {
            None
        }
    }

    pub fn modify_or_replace<F, G>(&self, modify: F, mkval: G)
    where
        F: FnOnce(&mut T),
        G: FnOnce() -> T,
    {
        if self.map(modify).is_none() {
            self.put(mkval());
        }
    }
}
