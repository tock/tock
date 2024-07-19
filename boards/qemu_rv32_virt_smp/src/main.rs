// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![feature(generic_const_exprs)]
#![feature(naked_functions)]
#![feature(asm_const)]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

mod threads;

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::scheduler::cooperative::CooperativeSched;
use kernel::threadlocal;
use kernel::threadlocal::ConstThreadId;
use kernel::threadlocal::ThreadId;
use kernel::threadlocal::ThreadLocalAccessStatic;
use kernel::threadlocal::ThreadLocalDynInit;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::{create_capability, debug, static_init, static_init_once};
use kernel::{thread_local_static_init, thread_local_static_finalize, thread_local_static, thread_local_static_access};
use qemu_rv32_virt_chip::chip::{QemuRv32VirtChip, QemuRv32VirtDefaultPeripherals};
use qemu_rv32_virt_chip::plic::PLIC;
use qemu_rv32_virt_chip::plic::PLIC_BASE;
use rv32i::csr;

use kernel::utilities::registers::interfaces::Readable;
use kernel::threadlocal::DynThreadId;
use kernel::platform::chip::{Chip, InterruptService};

use qemu_rv32_virt_chip::MAX_THREADS;
use qemu_rv32_virt_chip::QemuRv32VirtThreadLocal;

pub mod io;

// TODO: This should be moved to thread-specific mod
pub const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip for panic dumps.
// static mut CHIP: Option<&'static QemuRv32VirtChip<QemuRv32VirtDefaultPeripherals>> = None;
thread_local_static!(
    MAX_THREADS,
    // CHIP: Option<&'static QemuRv32VirtChip<'static, dyn InterruptService + 'static>> = None
    CHIP: Option<&'static dyn Chip<
        MPU = qemu_rv32_virt_chip::chip::QemuRv32VirtPMP,
        UserspaceKernelBoundary = rv32i::syscall::SysCall,
    >> = None
);

// Reference to the process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

const STACK_SIZE: usize = 0x8000;

static mut DEBUG_COUNTER: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; STACK_SIZE] = [0; STACK_SIZE];


#[repr(C)]
enum ThreadType {
    Main,
    Application,
}

/// Main function.
///
/// This function is called from the arch crate after some very basic
/// RISC-V setup and RAM initialization.
#[no_mangle]
pub unsafe fn main(thread_type: ThreadType) {

    let channel = static_init_once!(
        qemu_rv32_virt_chip::channel::QemuRv32VirtChannel,
        qemu_rv32_virt_chip::channel::QemuRv32VirtChannel::new(&mut *core::ptr::addr_of_mut!(qemu_rv32_virt_chip::channel::CHANNEL_BUFFER)),
    );

    use ThreadType as T;
    match thread_type {
        T::Main => threads::main_thread::spawn::<{T::Main as usize}>(channel),
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
