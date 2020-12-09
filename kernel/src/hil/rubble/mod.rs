//! Rubble interfaces for Tock.
//!
//! This module contains the interfaces for the following intended stack using
//! the external [Rubble library](https://github.com/jonas-schievink/rubble):
//!
//! ```ignore
//! +---------------------------------+
//! |                                 |
//! |  Capsules                       |
//! |                                 |
//! +---------------------------------+
//!
//!    hil::rubble::RubbleStack
//!
//! +---------------------------------+
//! |                                 |
//! |  Rubble BLE Stack (external)    |
//! |                                 |
//! +---------------------------------+
//!
//!    hil::rubble::radio::RubbleData
//!
//! +---------------------------------+
//! |                                 |
//! |  Radio Hardware                 |
//! |                                 |
//! +---------------------------------+
//! ```

pub mod radio;

pub mod types;

mod rubble;
pub use rubble::RubbleBleRadio;
pub use rubble::RubbleCmd;
pub use rubble::RubbleLinkLayer;
pub use rubble::RubblePacketQueue;
pub use rubble::RubbleResponder;
pub use rubble::RubbleStack;
