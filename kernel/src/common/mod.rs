//! Common operations and types in Tock.
//!
//! These are data types and access mechanisms that are used throughout the Tock
//! kernel. Mostly they simplify common operations and enable the other parts of
//! the kernel (chips and capsules) to be intuitive, valid Rust. In some cases
//! they provide safe wrappers around unsafe interface so that other kernel
//! crates do not need to use unsafe code.

pub use tock_regs::*;

pub mod deferred_call;
pub mod list;
pub mod math;
pub mod peripherals;
pub mod utils;

mod map_cell;
mod num_cell;
mod optional_cell;
mod queue;
mod ring_buffer;
mod static_ref;
mod take_cell;
mod volatile_cell;

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
    pub use common::map_cell::MapCell;
    pub use common::num_cell::NumCell;
    pub use common::optional_cell::OptionalCell;
    pub use common::take_cell::TakeCell;
    pub use common::volatile_cell::VolatileCell;
}
