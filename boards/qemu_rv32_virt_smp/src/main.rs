// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![feature(generic_const_exprs)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(inline_const)]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use kernel::collections::atomic_ring_buffer::AtomicRingBuffer;
use kernel::platform::chip::Chip;
use kernel::static_init_once;
use kernel::threadlocal::ThreadLocalDyn;

use qemu_rv32_virt_chip::{MAX_THREADS, QemuRv32VirtThreadLocal};
use qemu_rv32_virt_chip::chip::{QemuRv32VirtChip, QemuRv32VirtDefaultPeripherals, QemuRv32VirtPMP};

pub mod io;
mod threads;

// Reference to the chip for panic dumps.
static mut CHIP: &'static dyn ThreadLocalDyn<
        Option<&'static dyn Chip<MPU=QemuRv32VirtPMP, UserspaceKernelBoundary=rv32i::syscall::SysCall>>> =
    &mut QemuRv32VirtThreadLocal::init(None);

const STACK_SIZE: usize = 0x8000;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; STACK_SIZE] = [0; STACK_SIZE];


#[repr(C)]
pub enum ThreadType {
    Main = 0,
    Application,
}

impl TryFrom<usize> for ThreadType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value == 0 {
            Ok(ThreadType::Main)
        } else if value == 1 {
            Ok(ThreadType::Application)
        } else {
            Err(())
        }
    }
}

/// Main function.
///
/// This function is called from the arch crate after some very basic
/// RISC-V setup and RAM initialization.
#[no_mangle]
pub unsafe fn main(thread_type: ThreadType) {

    const LEN: usize = 2;

    let channel_buffer = static_init_once!(
        [qemu_rv32_virt_chip::portal::QemuRv32VirtVoyagerReference; LEN],
        core::mem::MaybeUninit::uninit().assume_init(),
    );

    let channel_ready_buffer = static_init_once!(
        [core::sync::atomic::AtomicBool; LEN],
        [const { core::sync::atomic::AtomicBool::new(false) }; LEN],
    );

    let channel = static_init_once!(
        AtomicRingBuffer<qemu_rv32_virt_chip::portal::QemuRv32VirtVoyagerReference>,
        AtomicRingBuffer::new(channel_buffer, channel_ready_buffer).unwrap(),
    );

    use ThreadType as T;
    match thread_type {
        T::Main => threads::main_thread::spawn::<{T::Main as usize}>(channel, true),
        T::Application => {
            // loop {}
            threads::app_thread::spawn::<{T::Application as usize}>(channel)
            // rv32i::semihost_command(0x18, 1, 0);
        },
        _ => panic!("Invalid Thread ID")
    }
}

#[naked]
#[no_mangle]
pub unsafe fn entry(hart_id: usize) {
    extern "C" {
        static _estack: usize;
    }

    core::arch::asm!("
        // Initialize the stack pointer register. This comes directly from
        // the linker script.
        la   sp, {estack}           // Set the initial stack pointer to be the bottom
                                    // of the stack section.
        li   a1, {offset}           // a1 = single stack offset.

      000:                          // Move the stack pointer to a proper position.
        beqz a0, 001f
        sub  sp, sp, a1
        addi a0, a0, -1
        j 000b

      001:
        // Set s0 (the frame pointer) to the start of the stack.
        add  s0, sp, zero           // s0 = sp

        // Initialize mscratch to 0 so that we know that we are currently
        // in the kernel. This is used for the check in the trap handler.
        csrw 0x340, zero            // CSR=0x340=mscratch

        csrr a0, mhartid
        j main
        ",
        estack = sym _estack,
        offset = const (STACK_SIZE / MAX_THREADS),
        options(noreturn)
    );
}
