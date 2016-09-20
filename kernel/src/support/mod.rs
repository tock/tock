//! A bare-metal library supplying functions rustc may lower code to
//!
//! This library is not intended for general use, and is superseded by a system
//! libc if one is available. In a freestanding context, however, common
//! functions such as memset, memcpy, etc are not implemented. This library
//! provides an implementation of these functions which are either required by
//! libcore or called by rustc implicitly.
//!
//! This library is never included by default, and must be manually included if
//! necessary. It is an error to include this library when also linking with
//! the system libc library.

pub use self::arm::*;

mod arm;
