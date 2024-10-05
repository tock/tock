// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Instantiation of the sifive Platform Level Interrupt Controller

use kernel::utilities::StaticRef;
use sifive::plic::{Plic, PlicRegisters};
use kernel::threadlocal::{ThreadLocal, ThreadLocalDyn, ThreadId};

use crate::{MAX_THREADS, MAX_CONTEXTS};

pub const PLIC_BASE: StaticRef<PlicRegisters<MAX_CONTEXTS>> =
    unsafe { StaticRef::new(0x0c00_0000 as *const PlicRegisters<MAX_CONTEXTS>) };

static NO_PLIC: ThreadLocal<0, Option<Plic<MAX_CONTEXTS>>> = ThreadLocal::new([]);

static mut PLIC: &'static dyn ThreadLocalDyn<Option<Plic<MAX_CONTEXTS>>> = &NO_PLIC;

pub unsafe fn set_global_plic(
    plic: &'static dyn ThreadLocalDyn<Option<Plic<MAX_CONTEXTS>>>
) {
    *core::ptr::addr_of_mut!(PLIC) = plic;
}

pub fn init_plic() {
    let closure = |plic: &mut Option<Plic<MAX_CONTEXTS>>| {
        let _ = plic.replace(Plic::new(PLIC_BASE));
    };

    unsafe {
        let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(PLIC);
        threadlocal
            .get_mut()
            .unwrap_or_else(|| {
                panic!("Thread {} does not have access to its local PLIC",
                       rv32i::support::current_hart_id().get_id());
            })
            .enter_nonreentrant(closure);
    }
}

unsafe fn with_plic<R, F>(f: F) -> Option<R>
where
    F: FnOnce(&mut Plic<MAX_CONTEXTS>) -> R
{
    let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(PLIC);
    threadlocal
        .get_mut().and_then(|c| c.enter_nonreentrant(|v| v.as_mut().map(f)))
}


pub unsafe fn with_plic_panic<R, F>(f: F) -> R
where
    F: FnOnce(&mut Plic<MAX_CONTEXTS>) -> R
{
    with_plic(f)
        .unwrap_or_else(|| {
            panic!("Thread {} does not have access to a valid PLIC",
                   rv32i::support::current_hart_id().get_id());
        })
}
