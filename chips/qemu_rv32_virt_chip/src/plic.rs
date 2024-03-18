// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Instantiation of the sifive Platform Level Interrupt Controller

use kernel::utilities::StaticRef;
use sifive::plic::{Plic, PlicRegisters};
use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::threadlocal::ThreadLocal;
use kernel::thread_local_static;

use crate::MAX_THREADS;

pub const PLIC_BASE: StaticRef<PlicRegisters<MAX_THREADS>> =
    unsafe { StaticRef::new(0x0c00_0000 as *const PlicRegisters<MAX_THREADS>) };

thread_local_static!(
    MAX_THREADS,
    pub PLIC: Plic<MAX_THREADS> = Plic::new(PLIC_BASE)
);

// pub static mut PLIC: Plic<MAX_THREADS> = Plic::new(PLIC_BASE);
