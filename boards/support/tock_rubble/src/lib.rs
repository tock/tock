//! Interface crate between Tock and the third party library Rubble.

#![no_std]

mod refcell_packet_queue;
mod timer_wrapper;
mod tock_rubble;

pub use tock_rubble::BleRadioWrapper;
pub use tock_rubble::PacketBuffer;
pub use tock_rubble::TockRubble;

/// Export this variable for sizing various buffers.
pub use rubble::link::MIN_PDU_BUF;
