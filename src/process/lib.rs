#![crate_name = "process"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std,unique)]

extern crate core;
extern crate common;
extern crate support;

pub mod process;
pub mod callback;

pub use process::{Process,State};
pub use callback::{AppPtr, Callback};

