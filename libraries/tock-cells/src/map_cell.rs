// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock specific `MapCell` type for sharing references.

use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;
use core::ptr::drop_in_place;

#[derive(Clone, Copy, PartialEq)]
enum MapCellState {
    Uninit,
    Init,
    Borrowed,
}

#[inline(never)]
#[cold]
fn access_panic() {
    panic!("`MapCell` accessed while borrowed");
}

macro_rules! debug_assert_not_borrowed {
    ($slf:ident) => {
        if cfg!(debug_assertions) && $slf.occupied.get() == MapCellState::Borrowed {
            access_panic();
        }
    };
}

/// A mutable, possibly unset, memory location that provides checked `&mut` access
/// to its contents via a closure.
///
/// A `MapCell` provides checked shared access to its mutable memory. Borrow
/// rules are enforced by forcing clients to either move the memory out of the
/// cell or operate on a `&mut` within a closure. You can think of a `MapCell`
/// as a `Cell<Option<T>>` with an extra "in-use" state to prevent `map` from invoking
/// undefined behavior when called re-entrantly.
///
/// # Examples
/// ```
/// # use tock_cells::map_cell::MapCell;
/// let cell: MapCell<i64> = MapCell::empty();
///
/// assert!(cell.is_none());
/// cell.map(|_| unreachable!("The cell is empty; map does not call the closure"));
/// assert_eq!(cell.take(), None);
/// cell.put(10);
/// assert_eq!(cell.take(), Some(10));
/// assert_eq!(cell.replace(20), None);
/// assert_eq!(cell.get(), Some(20));
///
/// cell.map(|x| {
///     assert_eq!(x, &mut 20);
///     // `map` provides a `&mut` to the contents inside the closure
///     *x = 30;
/// });
/// assert_eq!(cell.replace(60), Some(30));
/// ```
pub struct MapCell<T> {
    // Since val is potentially uninitialized memory, we must be sure to check
    // `.occupied` before calling `.val.get()` or `.val.assume_init()`. See
    // [mem::MaybeUninit](https://doc.rust-lang.org/core/mem/union.MaybeUninit.html).
    val: UnsafeCell<MaybeUninit<T>>,

    // Safety invariants:
    // - The contents of `val` must be initialized if this is `Init` or `InsideMap`.
    // - It must be sound to mutate `val` behind a shared reference if this is `Uninit` or `Init`.
    //   No outside mutation can occur while a `&mut` to the contents of `val` exist.
    occupied: Cell<MapCellState>,
}

impl<T> Drop for MapCell<T> {
    fn drop(&mut self) {
        let state = self.occupied.get();
        debug_assert_not_borrowed!(self); // This should be impossible
        if state == MapCellState::Init {
            unsafe {
                // SAFETY:
                // - `occupied` is `Init`; `val` is initialized as an invariant.
                // - Even though this violates the `occupied` invariant, by causing `val`
                //   to be no longer valid, `self` is immediately dropped.
                drop_in_place(self.val.get_mut().as_mut_ptr())
            }
        }
    }
}

impl<T: Copy> MapCell<T> {
    /// Gets the contents of the cell, if any.
    ///
    /// Returns `None` if the cell is empty.
    ///
    /// This requires the held type be `Copy` for the same reason [`Cell::get`] does:
    /// it leaves the contents of `self` intact and so it can't have drop glue.
    ///
    /// This returns `None` in release mode if the `MapCell`'s contents are already borrowed.
    ///
    /// # Examples
    /// ```
    /// # use tock_cells::map_cell::MapCell;
    /// let cell: MapCell<u32> = MapCell::empty();
    /// assert_eq!(cell.get(), None);
    ///
    /// cell.put(20);
    /// assert_eq!(cell.get(), Some(20));
    /// ```
    ///
    /// # Panics
    /// If debug assertions are enabled, this panics if the `MapCell`'s contents are already borrowed.
    pub fn get(&self) -> Option<T> {
        debug_assert_not_borrowed!(self);
        // SAFETY:
        // - `Init` means that `val` is initialized and can be read
        // - `T: Copy` so there is no drop glue
        (self.occupied.get() == MapCellState::Init)
            .then(|| unsafe { self.val.get().read().assume_init() })
    }
}

impl<T> MapCell<T> {
    /// Creates an empty `MapCell`.
    pub const fn empty() -> MapCell<T> {
        MapCell {
            val: UnsafeCell::new(MaybeUninit::uninit()),
            occupied: Cell::new(MapCellState::Uninit),
        }
    }

    /// Creates a new `MapCell` containing `value`.
    pub const fn new(value: T) -> MapCell<T> {
        MapCell {
            val: UnsafeCell::new(MaybeUninit::new(value)),
            occupied: Cell::new(MapCellState::Init),
        }
    }

    /// Returns `true` if the `MapCell` contains no value.
    ///
    /// # Examples
    /// ```
    /// # use tock_cells::map_cell::MapCell;
    /// let x: MapCell<i32> = MapCell::empty();
    /// assert!(x.is_none());
    ///
    /// x.put(10);
    /// x.map(|_| assert!(!x.is_none()));
    /// assert!(!x.is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns `true` if the `MapCell` contains a value.
    ///
    /// # Examples
    /// ```
    /// # use tock_cells::map_cell::MapCell;
    /// let x: MapCell<i32> = MapCell::new(10);
    /// assert!(x.is_some());
    /// x.map(|_| assert!(x.is_some()));
    ///
    /// x.take();
    /// assert!(!x.is_some());
    /// ```
    pub fn is_some(&self) -> bool {
        self.occupied.get() != MapCellState::Uninit
    }

    /// Takes the value out of the `MapCell`, leaving it empty.
    ///
    /// Returns `None` if the cell is empty.
    ///
    /// To save size, this has no effect and returns `None` in release mode
    /// if the `MapCell`'s contents are already borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tock_cells::map_cell::MapCell;
    /// let cell = MapCell::new(1234);
    /// let x = &cell;
    /// let y = &cell;
    ///
    /// assert_eq!(x.take(), Some(1234));
    /// assert_eq!(y.take(), None);
    /// ```
    ///
    /// # Panics
    /// If debug assertions are enabled, this panics if the `MapCell`'s contents are already borrowed.
    pub fn take(&self) -> Option<T> {
        debug_assert_not_borrowed!(self);
        (self.occupied.get() == MapCellState::Init).then(|| {
            // SAFETY: Since `occupied` is `Init`, `val` is initialized and can be mutated
            //         behind a shared reference. `result` is therefore initialized.
            unsafe {
                let result: MaybeUninit<T> = self.val.get().replace(MaybeUninit::uninit());
                self.occupied.set(MapCellState::Uninit);
                result.assume_init()
            }
        })
    }

    /// Puts a value into the `MapCell` without returning the old value.
    ///
    /// To save size, this has no effect in release mode if `map` is invoking
    /// a closure for this cell.
    ///
    /// # Panics
    /// If debug assertions are enabled, this panics if the `MapCell`'s contents are already borrowed.
    pub fn put(&self, val: T) {
        debug_assert_not_borrowed!(self);
        // This will ensure the value as dropped
        self.replace(val);
    }

    /// Replaces the contents of the `MapCell`, returning the old value if available.
    ///
    /// To save size, this has no effect and returns `None` in release mode
    /// if the `MapCell`'s contents are already borrowed.
    ///
    /// # Panics
    /// If debug assertions are enabled, this panics if the `MapCell`'s contents are already borrowed.
    pub fn replace(&self, val: T) -> Option<T> {
        let occupied = self.occupied.get();
        debug_assert_not_borrowed!(self);
        if occupied == MapCellState::Borrowed {
            return None;
        }
        self.occupied.set(MapCellState::Init);

        // SAFETY:
        // - Since `occupied` is `Init` or `Uninit`, no `&mut` to the `val` exists, meaning it
        //   is safe to mutate the `get` pointer.
        // - If occupied is `Init`, `maybe_uninit_val` must be initialized.
        let maybe_uninit_val = unsafe { self.val.get().replace(MaybeUninit::new(val)) };
        (occupied == MapCellState::Init).then(|| unsafe { maybe_uninit_val.assume_init() })
    }

    /// Calls `closure` with a `&mut` of the contents of the `MapCell`, if available.
    ///
    /// The closure is only called if the `MapCell` has a value.
    /// The state of the `MapCell` is unchanged after the closure completes.
    ///
    /// # Re-entrancy
    ///
    /// This borrows the contents of the cell while the closure is executing.
    /// Be careful about calling methods on `&self` inside of that closure!
    /// To save size, this has no effect in release mode, but if debug assertions
    /// are enabled, this panics to indicate a likely bug.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tock_cells::map_cell::MapCell;
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
    ///
    /// # Panics
    /// If debug assertions are enabled, this panics if the `MapCell`'s contents are already borrowed.
    #[inline(always)]
    pub fn map<F, R>(&self, closure: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        debug_assert_not_borrowed!(self);
        (self.occupied.get() == MapCellState::Init).then(move || {
            self.occupied.set(MapCellState::Borrowed);
            // `occupied` is reset to initialized at the end of scope,
            // even if a panic occurs in `closure`.
            struct ResetToInit<'a>(&'a Cell<MapCellState>);
            impl Drop for ResetToInit<'_> {
                #[inline(always)]
                fn drop(&mut self) {
                    self.0.set(MapCellState::Init);
                }
            }
            let _reset_to_init = ResetToInit(&self.occupied);
            unsafe { closure(&mut *self.val.get().cast::<T>()) }
        })
    }

    /// Behaves like `map`, but returns `default` if there is no value present.
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
        self.map(closure).flatten()
    }

    /// If a value is present `modify` is called with a borrow.
    /// Otherwise, the value is set with `G`.
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
    use super::MapCell;

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
        let mut dropped_after_drop = false;
        let mut dropped_after_put = false;
        {
            let cell = MapCell::new(DropCheck {
                flag: &mut dropped_after_put,
            });
            cell.put(DropCheck {
                flag: &mut dropped_after_drop,
            });
        }
        assert!(dropped_after_drop);
        assert!(dropped_after_put);
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
    #[should_panic = "`MapCell` accessed while borrowed"]
    fn test_map_in_borrow() {
        let cell = MapCell::new(1);
        let borrow1 = &cell;
        let borrow2 = &cell;
        borrow1.map(|_| borrow2.map(|_| ()));
    }

    #[test]
    #[should_panic = "`MapCell` accessed while borrowed"]
    fn test_replace_in_borrow() {
        let my_cell = MapCell::new(55);
        my_cell.map(|_ref1: &mut i32| {
            // Should fail
            my_cell.replace(56);
        });
    }

    #[test]
    #[should_panic = "`MapCell` accessed while borrowed"]
    fn test_put_in_borrow() {
        let my_cell = MapCell::new(55);
        my_cell.map(|_ref1: &mut i32| {
            // Should fail
            my_cell.put(56);
        });
    }

    #[test]
    #[should_panic = "`MapCell` accessed while borrowed"]
    fn test_get_in_borrow() {
        let my_cell = MapCell::new(55);
        my_cell.map(|_ref1: &mut i32| {
            // Should fail
            my_cell.get();
        });
    }
}
