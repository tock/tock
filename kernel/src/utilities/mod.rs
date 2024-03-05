// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Utility functions and macros provided by the kernel crate.

pub mod binary_write;
pub mod copy_slice;
pub mod helpers;
pub mod leasable_buffer;
pub mod math;
pub mod mut_imut_buffer;
pub mod packet_buffer;
pub mod peripheral_management;
pub mod static_init;
pub mod storage_volume;

mod static_ref;

pub use self::static_ref::StaticRef;

/// Re-export the tock-register-interface library.
pub mod registers {
    pub use tock_registers::fields::{Field, FieldValue};
    pub use tock_registers::interfaces;
    pub use tock_registers::registers::InMemoryRegister;
    pub use tock_registers::registers::{Aliased, ReadOnly, ReadWrite, WriteOnly};
    pub use tock_registers::{register_bitfields, register_structs};
    pub use tock_registers::{LocalRegisterCopy, RegisterLongName};
}

/// Create a "fake" module inside of `common` for all of the Tock `Cell` types.
///
/// To use `TakeCell`, for example, users should use:
///
///     use kernel::utilities::cells::TakeCell;
pub mod cells {
    pub use tock_cells::map_cell::MapCell;
    pub use tock_cells::numeric_cell_ext::NumericCellExt;
    pub use tock_cells::optional_cell::OptionalCell;
    pub use tock_cells::take_cell::TakeCell;
    pub use tock_cells::volatile_cell::VolatileCell;
}
