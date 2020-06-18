//! Wrapper around MMIO accesses.
//!
//! Passthrough implementation that performs volatile reads and writes.
use crate::registers::IntLike;

/// `read_volatile<T>` wraps `ptr::read_volatile<T>`.
pub(crate) unsafe fn read_volatile<T: IntLike>(src: *const T) -> T {
    ::core::ptr::read_volatile(src)
}

/// `write_volatile<T>` wraps `ptr::write_volatile<T>`.
pub(crate) unsafe fn write_volatile<T: IntLike>(dst: *mut T, src: T) {
    ::core::ptr::write_volatile(dst, src)
}
