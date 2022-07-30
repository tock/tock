//! Module containing items used by `tock-registers` macros. These must be `pub`
//! so the macros can use them, but are not intended to be part of
//! `tock-registers`' API.
//!
//! In other words: don't use any of these outside this crate.

use core::ptr::{read_volatile, write_volatile};
use crate::{TooLargeIndex, WriteSuccess};

// Reads a value from a MMIO non-array register.
//
// Safety:
//     mmio must point to the beginning of the peripheral.
//
//     REL_ADDR must be the address of the register to read, relative to the
//     beginning of the peripheral.
//
//     This register must be readable.
#[inline]
pub unsafe fn mmio_read<const REL_ADDR: usize, Mmio, T>(mmio: &Mmio) -> T
{
    // This is unsound under strict provenance. Under strict provenance, we
    // should attach mmio's provenance to the resulting pointer. For now, we
    // can't do that without enabling a nightly feature (which may be worth
    // doing in Miri) or using the `sptr` crate.
    let addr = (mmio as *const _ as usize + REL_ADDR) as *const T;
    unsafe { read_volatile(addr) }
}

// Reads a value from a MMIO array register.
//
// Safety:
//     mmio must point to the beginning of the peripheral.
//
//     REL_ADDR must be the address of the beginning of the register array,
//     relative to the beginning of the peripheral.
//
//     This register must be readable.
#[inline]
pub unsafe fn mmio_read_array<const REL_ADDR: usize, const LEN: usize, Mmio, T>(mmio: &Mmio, idx: usize)
    -> Result<T, TooLargeIndex>
{
    if idx >= LEN { return Err(TooLargeIndex); }
    // See the comment in mmio_read about strict provenance.
    let array_start = (mmio as *const _ as usize + REL_ADDR) as *const T;
    Ok(unsafe { read_volatile(array_start.add(idx)) })
}

// Writes a value to a MMIO non-array register.
//
// Safety:
//     mmio must point to the beginning of the peripheral.
//
//     REL_ADDR must be the address of the register to write, relative to the
//     beginning of the peripheral.
//
//     This register must be writable.
#[inline]
pub unsafe fn mmio_write<const REL_ADDR: usize, Mmio, T>(mmio: &Mmio, value: T)
{
    // See the comment in mmio_read about strict provenance.
    let addr = (mmio as *const _ as usize + REL_ADDR) as *mut T;
    unsafe { write_volatile(addr, value) }
}

// Writes a value to a MMIO array register.
//
// Safety:
//     mmio must point to the beginning of the peripheral.
//
//     REL_ADDR must be the address of the beginning of the register array,
//     relative to the beginning of the peripheral.
//
//     This register must be writable.
#[inline]
pub unsafe fn mmio_write_array<const REL_ADDR: usize, const LEN: usize, Mmio, T>(mmio: &Mmio, value: T, idx: usize)
    -> Result<WriteSuccess, TooLargeIndex>
{
    if idx >= LEN { return Err(TooLargeIndex); }
    // See the comment in mmio_read about strict provenance.
    let array_start = (mmio as *const _ as usize + REL_ADDR) as *mut T;
    unsafe { write_volatile(array_start.add(idx), value) }
    Ok(WriteSuccess)
}
