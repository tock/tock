#![crate_name = "process"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core_intrinsics,unique,nonzero,const_fn)]

extern crate common;
extern crate support;

pub mod callback;
pub mod container;
pub mod mem;
pub mod process;

pub use callback::{AppId, Callback};
pub use container::{Container};
pub use mem::{AppSlice, AppPtr, Private, Shared};
pub use process::{Process,State, NUM_PROCS};

