//! Implementation of a type for accessing MCU registers.

use core::cell::UnsafeCell;
use core::ptr;

/// `VolatileCell` provides a wrapper around unsafe volatile pointer reads
/// and writes. This is particularly useful for accessing microcontroller
/// registers by (unsafely) casting a pointer to the register into a `VolatileCell`.
///
/// ```
/// use tock_cells::volatile_cell::VolatileCell;
/// let myptr: *const usize = 0xdeadbeef as *const usize;
/// let myregister: &VolatileCell<usize> = unsafe { core::mem::transmute(myptr) };
/// ```
// Originally modified from: https://github.com/hackndev/zinc/tree/master/volatile_cell
#[derive(Default)]
#[repr(transparent)]
pub struct VolatileCell<T: ?Sized + Copy> {
    value: UnsafeCell<T>,
}

impl<T: Copy> Clone for VolatileCell<T> {
    #[inline]
    fn clone(&self) -> Self {
        VolatileCell::new(self.get())
    }
}

impl<T: Copy> VolatileCell<T> {
    pub const fn new(value: T) -> Self {
        VolatileCell {
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    /// Performs a memory write with the provided value.
    ///
    /// # Side-Effects
    ///
    /// `set` _always_ performs a memory write on the underlying location. If
    /// this location is a memory-mapped I/O register, the side-effects of
    /// performing the write are register-specific.
    ///
    /// # Examples
    ///
    /// ```
    /// use tock_cells::volatile_cell::VolatileCell;
    ///
    /// let vc = VolatileCell::new(123);
    /// vc.set(432);
    /// ```
    pub fn set(&self, value: T) {
        // `set` does not read the value currently inside the `VolatileCell`
        // and, therefore, does not `drop` it, but because `T` is `Copy`, there
        // cannot be a destructor anyway.
        unsafe { ptr::write_volatile(self.value.get(), value) }
    }

    #[inline]
    /// Performs a memory read and returns a copy of the value represented by
    /// the cell.
    ///
    /// # Side-Effects
    ///
    /// `get` _always_ performs a memory read on the underlying location. If
    /// this location is a memory-mapped I/O register, the side-effects of
    /// performing the read are register-specific.
    ///
    /// # Examples
    ///
    /// ```
    /// use tock_cells::volatile_cell::VolatileCell;
    ///
    /// let vc = VolatileCell::new(5);
    /// let five = vc.get();
    /// ```
    pub fn get(&self) -> T {
        unsafe { ptr::read_volatile(self.value.get()) }
    }
}
