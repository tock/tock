//! Documentation crate for the Tock Operating System.
//!
//! This crate contains markdown documentation rendered into Rustdoc comments on
//! modules, mirroring the documentation of the Tock kernel repositories `doc/`
//! subdiretory. It is intended for non-code documentation, documentation that
//! spans multiple components, and official reference documents.
#![no_std]

#[doc = include_str!("../reference/README.md")]
pub mod reference {
    #[doc = include_str!("../reference/trd1-trds.md")]
    pub mod trd1_trds {}

    #[doc = include_str!("../reference/trd101-time.md")]
    pub mod trd101_time {}

    // ...

    #[doc = include_str!("../reference/trd104-syscalls.md")]
    pub mod trd104_syscalls {}
}

#[doc = include_str!("../ExternalDependencies.md")]
pub mod external_dependencies {}

#[doc = include_str!("../syscalls/README.md")]
pub mod syscalls {
    #[doc = include_str!("../syscalls/90001_screen.md")]
    pub mod syscall_90001_screen {}
}
