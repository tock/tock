// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Utility functions and macros provided by the kernel crate.

pub mod arch_helpers;
pub mod binary_write;
pub mod capability_ptr;
pub mod copy_slice;
pub mod helpers;
pub mod leasable_buffer;
pub mod machine_register;
pub mod math;
pub mod mut_imut_buffer;
pub mod peripheral_management;
pub mod single_thread_value;
pub mod static_init;
pub mod storage_volume;
pub mod streaming_process_slice;

mod static_ref;
pub use self::static_ref::StaticRef;

/// The Tock Register Interface.
///
/// This is a re-export of the `tock-register-interface` crate provided for
/// convenience.
///
/// The Tock Register Interface provides a mechanism for accessing hardware
/// registers and MMIO interfaces.
pub mod registers {
    pub use tock_registers::fields::{Field, FieldValue};
    pub use tock_registers::interfaces;
    pub use tock_registers::registers::InMemoryRegister;
    pub use tock_registers::registers::{Aliased, ReadOnly, ReadWrite, WriteOnly};
    pub use tock_registers::{register_bitfields, register_structs};
    pub use tock_registers::{LocalRegisterCopy, RegisterLongName};
}

/// The Tock `Cell` types.
///
/// This is a re-export of the `tock-cells` crate provided for convenience.
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
