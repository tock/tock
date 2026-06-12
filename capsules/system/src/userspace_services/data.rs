// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Serialization and deserialization for data moving to and from userspace services.

use core::mem;

use kernel::process::Error;
use kernel::processbuffer::{ReadableProcessSlice, WriteableProcessSlice};

/// Data that can be serialized into a process buffer.
pub trait Serialize {
    /// Attempt to write `Self` into a process buffer.
    fn try_serialize(&self, buffer: &WriteableProcessSlice) -> Result<(), Error>;
}

/// Data that can be interpreted from a userspace service's process buffer bytes.
pub trait Deserialize: Sized {
    /// Attempt to convert the bytes of a process buffer into `Self`.
    ///
    /// Try interpreting the bytes in the buffer as having type `Self`.
    /// Returns `Ok(Self)` if successful.
    /// Returns `Err(())` if the interpretation fails.
    fn try_deserialize(buffer: &ReadableProcessSlice) -> Result<Self, Error>;
}

/// Instantiate the Serialize and Deserialize implementations for a numeric type.
///
/// Expand the trait implementations of the serialization traits.
/// Requires that the type, T, define both `T::to_ne_bytes()` and `T::from_ne_bytes()`.
macro_rules! impl_serialization_for_numerical {
    ($($t:ty),+) => {
        $(
            impl Serialize for $t {
                fn try_serialize(&self, slice: &WriteableProcessSlice) -> Result<(), Error> {
                    if slice.len() < mem::size_of::<$t>() {
                        Err(Error::OutOfMemory)
                    } else {
                        slice[0..mem::size_of::<$t>()]
                            .copy_from_slice(&self.to_ne_bytes());
                        Ok(())
                    }
                }
            }

            impl Deserialize for $t {
                fn try_deserialize(slice: &ReadableProcessSlice) -> Result<$t, Error> {
                    if slice.len() != mem::size_of::<$t>() {
                        Err(Error::OutOfMemory)
                    } else {
                        let mut val_bytes = [0; mem::size_of::<$t>()];
                        slice.copy_to_slice(&mut val_bytes[0..mem::size_of::<$t>()]);
                        Ok(<$t>::from_ne_bytes(val_bytes))
                    }
                }
            }
        )+
    };
}

impl_serialization_for_numerical!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

/// Newtype wrapper for `Serialize`ing slices into process buffers.
///
/// A wrapper type that allows slices to be passed ergonomically to [`UserspaceServiceAccess::usercall()`](super::usercall::UserspaceServiceAccess).
/// The alternative would be to use double references
/// (i.e. define `impl Serialize for &&[u8]` and use with `&&&my_data[..]`
/// to guarantee to the compiler that the `Serialize` trait object is `Sized`).
pub struct Bytes<'a>(pub &'a [u8]);

impl Serialize for Bytes<'_> {
    fn try_serialize(&self, slice: &WriteableProcessSlice) -> Result<(), Error> {
        let src = self.0;
        if slice.len() < src.len() {
            Err(Error::OutOfMemory)
        } else {
            slice[0..src.len()].copy_from_slice(src);

            Ok(())
        }
    }
}
