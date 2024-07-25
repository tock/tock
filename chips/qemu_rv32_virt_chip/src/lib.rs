// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chip support for the qemu-system-riscv32 virt machine

#![no_std]
#![crate_name = "qemu_rv32_virt_chip"]
#![crate_type = "rlib"]
#![feature(variant_count)]

pub const MAX_THREADS: usize = 2;
pub const MAX_CONTEXTS: usize = MAX_THREADS * 2;

pub use virtio;

pub mod interrupts;
pub mod virtio_mmio;

pub mod chip;
pub mod clint;
pub mod plic;
pub mod uart;

pub mod channel;
pub mod portal_cell;
pub mod portal;

pub struct QemuRv32VirtThreadLocal<T>(kernel::threadlocal::ThreadLocal<MAX_THREADS, T>);

impl<T: Copy> QemuRv32VirtThreadLocal<T> {
    pub const fn init(init: T) -> QemuRv32VirtThreadLocal<T> {
        QemuRv32VirtThreadLocal(kernel::threadlocal::ThreadLocal::init(init))
    }
}

impl<T> QemuRv32VirtThreadLocal<T> {
    pub const fn new(val: [T; MAX_THREADS]) -> QemuRv32VirtThreadLocal<T> {
        QemuRv32VirtThreadLocal(kernel::threadlocal::ThreadLocal::new(val))
    }
}

unsafe impl<T> kernel::threadlocal::ThreadLocalDyn<T> for QemuRv32VirtThreadLocal<T> {
    fn get_mut<'a>(&'a self) -> Option<kernel::threadlocal::NonReentrant<'a, T>> {
        use kernel::threadlocal::{ThreadLocal, ThreadLocalAccess, DynThreadId};
        use kernel::utilities::registers::interfaces::Readable;
        let id = rv32i::csr::CSR.mhartid.extract().get();
        <ThreadLocal<MAX_THREADS, T> as ThreadLocalAccess<DynThreadId, T>>::get_mut(&self.0, unsafe {
            DynThreadId::new(id)
        })
    }
}

// impl<T: Copy> kernel::threadlocal::ThreadLocalDynInit<T> for QemuRv32VirtThreadLocal<T> {
//     unsafe fn init(init: T) -> Self {
// 	    QemuRv32VirtThreadLocal(kernel::threadlocal::ThreadLocal::init(init))
//     }
// }
