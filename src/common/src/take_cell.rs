use core::cell::UnsafeCell;
use core::ptr;

/// A mutable memory location that enforces borrow rules at runtime without
/// possible panics.
///
/// A `TakeCell` is a potential reference to mutable memory. Borrow rules are
/// enforced by forcing clients to either move the memory out of the cell or
/// operate on a borrow within a closure. You can think of a `TakeCell` as a
/// between an `Option` wrapped in a `RefCell` --- attempts to take the value
/// from inside a `TakeCell` may fail by returning `None`.
pub struct TakeCell<T> {
    val: UnsafeCell<Option<T>>,
}

impl<T> TakeCell<T> {
    pub const fn empty() -> TakeCell<T> {
        TakeCell { val: UnsafeCell::new(None) }
    }

    /// Creates a new `TakeCell` containing `value`
    pub const fn new(value: T) -> TakeCell<T> {
        TakeCell { val: UnsafeCell::new(Some(value)) }
    }

    pub fn is_none(&self) -> bool {
        unsafe { (&*self.val.get()).is_none() }
    }

    /// Takes the value out of the `TakeCell` leaving a `None` in it's place. If
    /// the value has already been taken elsewhere (and not `replace`ed), the
    /// returned `Option` will be empty.
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
    pub fn take(&self) -> Option<T> {
        unsafe {
            let inner = &mut *self.val.get();
            inner.take()
        }
    }

    pub fn put(&self, val: Option<T>) {
        let _ = self.take();
        let ptr = self.val.get();
        unsafe {
            ptr::replace(ptr, val);
        }
    }

    /// Replaces the contents of the `TakeCell` with `val`. If the cell was not
    /// empty, the previous value is returned, otherwise `None` is returned.
    pub fn replace(&self, val: T) -> Option<T> {
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
        where F: FnOnce(&mut T) -> R
    {
        let maybe_val = self.take();
        maybe_val.map(|mut val| {
            let res = closure(&mut val);
            self.replace(val);
            res
        })
    }

    pub fn modify_or_replace<F, G>(&self, modify: F, mkval: G)
        where F: FnOnce(&mut T),
              G: FnOnce() -> T
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
