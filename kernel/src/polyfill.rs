// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! This file helps Tock build with stale toolchains
//! The current set of toolchains that need support are:
//! The toolchain specified in rust-toolchain.toml
//! Cheri toolchain (rust 1.67)

#[cfg(target_feature = "xcheri")]
pub mod cheri_fills {
    pub mod core {
        pub mod ptr {
            #[inline(always)]
            #[must_use]
            pub const fn from_ref<T: ?Sized>(r: &T) -> *const T {
                r
            }

            #[inline(always)]
            #[must_use]
            pub fn from_mut<T: ?Sized>(r: &mut T) -> *mut T {
                r
            }
        }

        pub mod mem {
            #[macro_export]
            macro_rules! offset_of_impl {
                ($Container:ty, $($fields:ident)+ $(,)?) => {
                    // This is a builtin upstream and so the implementation here is not quite the
                    // same, and the pattern is slightly different
                    // We do not fault unsized types, for instance.
                    {
                        // Doing the logic on an "allocated" object rather than an arbitrary
                        // address makes the compiler happier in const contexts.
                        // Maybe unnit is a good choice because it is free to construct and
                        // immediatly drop.
                        let stand_in = core::mem::MaybeUninit::<$Container>::uninit();
                        let ptr_to_base = stand_in.as_ptr();

                        // Safety: this does not actualyl create a reference

                        let ptr_to_field = unsafe {
                            core::ptr::addr_of!((*ptr_to_base).$($fields)+)
                        };

                        // Safety: both these pointers are to the same allocated object, in bounds
                        // of that object.
                        unsafe {
                            (ptr_to_field as *const u8).offset_from(ptr_to_base as *const u8)
                        }
                    }
                };
            }

            pub use offset_of_impl as offset_of;
        }
    }
}

#[cfg(target_feature = "xcheri")]
pub use cheri_fills::*;

#[cfg(not(target_feature = "xcheri"))]
pub use core;
