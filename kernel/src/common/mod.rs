//! Common operations and types in Tock.
//!
//! These are data types and access mechanisms that are used throughout the Tock
//! kernel. Mostly they simplify common operations and enable the other parts of
//! the kernel (chips and capsules) to be intuitive, valid Rust. In some cases
//! they provide safe wrappers around unsafe interface so that other kernel
//! crates do not need to use unsafe code.

/// Re-export the tock-register-interface library.
pub mod registers {
    pub use tock_registers::registers::InMemoryRegister;
    pub use tock_registers::registers::RegisterLongName;
    pub use tock_registers::registers::{Aliased, ReadOnly, ReadWrite, WriteOnly};
    pub use tock_registers::registers::{Field, FieldValue, LocalRegisterCopy};
    pub use tock_registers::{register_bitfields, register_structs};
}

pub mod deferred_call;
pub mod dynamic_deferred_call;
pub mod leasable_buffer;
pub mod list;
pub mod math;
pub mod peripherals;
pub mod queue;
pub mod ring_buffer;
pub mod utils;

mod static_ref;

pub use self::list::{List, ListLink, ListNode};
pub use self::queue::Queue;
pub use self::ring_buffer::RingBuffer;
pub use self::static_ref::StaticRef;

/// Create a "fake" module inside of `common` for all of the Tock `Cell` types.
///
/// To use `TakeCell`, for example, users should use:
///
///     use kernel::common::cells::TakeCell;
pub mod cells {
    pub use tock_cells::map_cell::MapCell;
    pub use tock_cells::numeric_cell_ext::NumericCellExt;
    pub use tock_cells::optional_cell::OptionalCell;
    pub use tock_cells::take_cell::TakeCell;
    pub use tock_cells::volatile_cell::VolatileCell;
}
