#![crate_name = "process"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core_intrinsics,raw,unique,nonzero)]

extern crate common;
extern crate support;

pub mod callback;
pub mod mem;
pub mod process;

pub use callback::{AppId, Callback};
pub use mem::{AppSlice, AppPtr, Private, Shared};
pub use process::{Process,State};

