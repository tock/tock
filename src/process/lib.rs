#![crate_name = "process"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std,unique)]

extern crate core;
extern crate common;
extern crate support;

pub mod callback;
pub mod mem;
pub mod process;

pub use callback::Callback;
pub use mem::{AppSlice, AppPtr};
pub use process::{Process,State};

