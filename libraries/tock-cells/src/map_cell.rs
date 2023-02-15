//! Tock specific `MapCell` type for sharing references.

use self::MapCellErr::AlreadyBorrowed;
use self::MapCellState::{Init, InitBorrowed, Uninit};
use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::ptr::drop_in_place;

#[derive(Clone, Copy, PartialEq)]
enum MapCellState {
    Uninit,
    Init,
    InitBorrowed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapCellErr {
    AlreadyBorrowed,
    Uninit,
}

/// Smart pointer to a T that will automatically set the MapCell back to the init state.
/// You probably want to Deref this immediately.
pub struct MapCellRef<'a, T> {
    map_cell: &'a MapCell<T>,
}

impl<'a, T> Deref for MapCellRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            // Safety: There will only ever be one MapCellRef to a MapCell as we only allow their
            // construction when the cell is in the 'Init' state, and move the MapCell to the
            // 'InitBorrowed' state for the duration of the existance of this type.
            let valref = &*self.map_cell.val.get();
            // Safety: when this MapCellRef was constructed, we checked that the MapCell was in the
            // Init state.
            valref.assume_init_ref()
        }
    }
}

impl<'a, T> DerefMut for MapCellRef<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // Safety: Same as deref, but because deref_mut requires borrowing self as mut, there
            // will also be no immutable references to the data.
            let valref = &mut *self.map_cell.val.get();
            valref.assume_init_mut()
        }
    }
}

impl<'a, T> Drop for MapCellRef<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.map_cell.occupied.set(Init)
    }
}

/// Single panic location for MapCell::replace() failures
#[inline(never)]
fn replace_panic() -> ! {
    panic!("MapCell::replace() on borrowed MapCell");
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
    // Since val is potentially uninitialized memory, we must be sure to check
    // `.occupied` before calling `.val.get()` or `.val.assume_init()`. See
    // [mem::MaybeUninit](https://doc.rust-lang.org/core/mem/union.MaybeUninit.html).
    val: UnsafeCell<MaybeUninit<T>>,
    occupied: Cell<MapCellState>,
}

impl<T> Drop for MapCell<T> {
    fn drop(&mut self) {
        let state = self.occupied.get();
        debug_assert!(state != InitBorrowed);
        if state == Init {
            unsafe {
                // Safety: state being Init means that the MaybeUninit data was initted.
                // the pointer to the data can never be used again as this
                drop_in_place(self.val.get_mut().as_mut_ptr())
            }
        }
    }
}

impl<T> MapCell<T> {
    /// Creates an empty `MapCell`
    pub const fn empty() -> MapCell<T> {
        MapCell {
            val: UnsafeCell::new(MaybeUninit::uninit()),
            occupied: Cell::new(Uninit),
        }
    }

    /// Creates a new `MapCell` containing `value`
    pub const fn new(value: T) -> MapCell<T> {
        MapCell {
            val: UnsafeCell::new(MaybeUninit::<T>::new(value)),
            occupied: Cell::new(Init),
        }
    }

    /// Returns a boolean which indicates if the MapCell is unoccupied.
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns a boolean which indicates if the MapCell is occupied (regardless of whether it is
    /// borrowed or not).
    pub fn is_some(&self) -> bool {
        self.occupied.get() != Uninit
    }

    /// Takes the value out of the `MapCell` leaving it empty. If
    /// the value has already been taken elsewhere (and not `replace`ed), the
    /// returned `Option` will be `None`.
    /// If the value is currently borrowed, also returns None.
    /// # Examples
    ///
    /// ```
    /// extern crate tock_cells;
    /// use tock_cells::map_cell::MapCell;
    ///
    /// let cell = MapCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// assert_eq!(x.take(), Some(1234));
    /// assert_eq!(y.take(), None);
    /// ```
    pub fn take(&self) -> Option<T> {
        if self.occupied.get() != Init {
            None
        } else {
            self.occupied.set(Uninit);
            unsafe {
                // SAFETY: not in InitBorrowed state, so we are not leaving a dangling reference
                let result: MaybeUninit<T> =
                    ptr::replace(self.val.get(), MaybeUninit::<T>::uninit());
                // SAFETY: The Init state means that the MaybeUninit is init
                // `result` is _initialized_ and now `self.val` is now a new uninitialized value
                Some(result.assume_init())
            }
        }
    }

    /// Puts a value into the `MapCell`.
    pub fn put(&self, val: T) {
        // This will ensure the value as dropped
        self.replace(val);
    }

    /// Replaces the contents of the `MapCell` with `val`. If the cell was not
    /// empty, the previous value is returned, otherwise `None` is returned.
    /// In the event the cell is currently borrowed, returns Err(AlreadyBorrowed)
    pub fn try_replace(&self, val: T) -> Result<Option<T>, MapCellErr> {
        match self.occupied.get() {
            Uninit => {
                unsafe {
                    // Safety: Because we are in the Uninit state, we are not writing over anything
                    // that needs to be dropped
                    ptr::write(self.val.get(), MaybeUninit::<T>::new(val));
                    self.occupied.set(Init)
                }
                Ok(None)
            }
            Init => unsafe {
                // Safety: Because we are not in the InitBorrowed state, nothing is currently also
                // referencing this value
                let result: MaybeUninit<T> = ptr::replace(self.val.get(), MaybeUninit::new(val));
                // `result` is _initialized_ and now `self.val` is now a new uninitialized value
                Ok(Some(result.assume_init()))
            },
            InitBorrowed => Err(AlreadyBorrowed),
        }
    }

    /// Same as try_replace but panics if the cell is already borrowed
    pub fn replace(&self, val: T) -> Option<T> {
        match self.try_replace(val) {
            Err(_) => replace_panic(),
            Ok(v) => v,
        }
    }

    /// Try borrow a mutable reference to the data contained in this cell
    /// The type wrapped in the Option is a smart pointer to a T
    /// Using this method rather than the callback based logic below will result in considerably
    /// less code noise due to nested callbacks.
    ///  # Examples
    ///
    /// ```
    /// extern crate tock_cells;
    /// use tock_cells::map_cell::MapCell;
    /// let cell = MapCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// *x.try_borrow_mut().unwrap() += 1;
    ///
    /// assert_eq!(y.take(), Some(1235));
    /// ```
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<MapCellRef<T>, MapCellErr> {
        match self.occupied.get() {
            Init => {
                self.occupied.set(InitBorrowed);
                Ok(MapCellRef { map_cell: self })
            }
            Uninit => Err(MapCellErr::Uninit),
            InitBorrowed => Err(MapCellErr::AlreadyBorrowed),
        }
    }

    /// Allows `closure` to borrow the contents of the `MapCell` if-and-only-if
    /// it is not `take`n already. The state of the `MapCell` is unchanged
    /// after the closure completes.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate tock_cells;
    /// use tock_cells::map_cell::MapCell;
    ///
    /// let cell = MapCell::new(1234);
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
    /// assert_eq!(y.take(), Some(1235));
    /// ```
    #[inline(always)]
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        Some(closure(&mut *self.try_borrow_mut().ok()?))
    }

    #[inline(always)]
    pub fn map_or<F, R>(&self, default: R, closure: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.map(closure).unwrap_or(default)
    }

    /// Behaves the same as `map`, except the closure is allowed to return
    /// an `Option`.
    #[inline(always)]
    pub fn and_then<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> Option<R>,
    {
        closure(&mut *self.try_borrow_mut().ok()?)
    }

    #[inline(always)]
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

#[cfg(test)]
mod tests {
    use map_cell::{MapCell, MapCellErr};

    struct DropCheck<'a> {
        flag: &'a mut bool,
    }
    impl<'a> Drop for DropCheck<'a> {
        fn drop(&mut self) {
            *self.flag = true;
        }
    }

    #[test]
    fn test_drop() {
        let mut dropped = false;
        {
            let _a_cell = MapCell::new(DropCheck { flag: &mut dropped });
        }
        assert!(dropped)
    }

    #[test]
    fn test_replace() {
        let a_cell = MapCell::new(1);
        let old = a_cell.replace(2);
        assert_eq!(old, Some(1));
        assert_eq!(a_cell.take(), Some(2));
        assert_eq!(a_cell.take(), None);
    }

    #[test]
    fn test_try_replace() {
        let a_cell = MapCell::new(1);
        let borrow = a_cell.try_borrow_mut().unwrap();
        assert_eq!(a_cell.try_replace(2), Err(MapCellErr::AlreadyBorrowed));
        drop(borrow);
        assert_eq!(a_cell.try_replace(1), Ok(Some(1)));
    }

    #[test]
    fn test_borrow() {
        let a_cell = MapCell::new(1);
        *a_cell.try_borrow_mut().unwrap() = 2;
        {
            let mut borrowed2 = a_cell.try_borrow_mut().unwrap();
            assert_eq!(*borrowed2, 2);
            *borrowed2 = 3;
        }
        assert_eq!(a_cell.take(), Some(3))
    }

    #[test]
    #[should_panic]
    fn test_double_borrow() {
        let a_cell = MapCell::new(1);
        let mut borrowed = a_cell.try_borrow_mut().unwrap();
        let mut borrowed2 = a_cell.try_borrow_mut().unwrap();
        *borrowed2 = 2;
        *borrowed = 3;
    }

    #[test]
    #[should_panic]
    fn test_replace_in_borrow() {
        let my_cell = MapCell::new(55);
        my_cell.map(|_ref1: &mut i32| {
            // Should fail
            my_cell.put(56);
            my_cell.map(|_ref2: &mut i32| {})
        });
    }
}
