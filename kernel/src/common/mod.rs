//! Common operations in the Tock OS.

pub mod deferred_call;
pub mod list;
pub mod map_cell;
pub mod math;
pub mod peripherals;
pub mod queue;
pub mod ring_buffer;
pub mod static_ref;
pub mod take_cell;
pub mod utils;
pub mod volatile_cell;

#[macro_use]
pub mod regs;

mod num_cell;
mod optional_cell;

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
