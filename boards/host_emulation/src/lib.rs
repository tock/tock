#![feature(asm, concat_idents, const_fn, const_mut_refs, naked_functions)]
#![feature(in_band_lifetimes)]

use std::io;

pub type Result<T> = std::result::Result<T, EmulationError>;

#[derive(Debug)]
pub enum EmulationError {
    IoError(io::Error),
    ChannelError,
    PartialMessage(usize, usize),
    Custom(String),
}

impl From<io::Error> for EmulationError {
    fn from(error: io::Error) -> Self {
        EmulationError::IoError(error)
    }
}

impl std::fmt::Display for EmulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulationError::IoError(e) => write!(f, "{}", e),
            EmulationError::ChannelError => write!(f, "Channel Error"),
            EmulationError::PartialMessage(e, a) => {
                write!(f, "Unexpected message length. Expected {}, got {}.", e, a)
            }
            EmulationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

pub mod chip;
pub mod emulation_config;
pub mod ipc_syscalls;
mod log;
pub mod mpu;
pub mod process;
pub mod syscall;
pub mod syscall_transport;
pub mod systick;
pub mod uart;
