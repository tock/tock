// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

#![crate_name = "support"]
#![crate_type = "rlib"]
#![feature(asm,lang_items)]
#![no_std]

pub use arm::*;

mod arm;
