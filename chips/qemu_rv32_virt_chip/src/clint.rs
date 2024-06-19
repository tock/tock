// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Machine Timer instantiation.

use kernel::utilities::StaticRef;
use kernel::thread_local_static;
use sifive::clint::ClintRegisters;

use crate::chip::QemuRv32VirtClint;
use crate::MAX_THREADS;

pub const CLINT_BASE: StaticRef<ClintRegisters> =
    unsafe { StaticRef::new(0x0200_0000 as *const ClintRegisters) };

thread_local_static!(
    MAX_THREADS,
    pub CLIC: QemuRv32VirtClint<'static> = QemuRv32VirtClint::new(&CLINT_BASE)
);
