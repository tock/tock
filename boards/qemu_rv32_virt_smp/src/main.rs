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
use kernel::{create_capability, debug, static_init};
use kernel::{thread_local_static_init, thread_local_static_finalize, thread_local_static, thread_local_static_access};
use qemu_rv32_virt_chip::chip::{QemuRv32VirtChip, QemuRv32VirtDefaultPeripherals};
use qemu_rv32_virt_chip::plic::PLIC;
use qemu_rv32_virt_chip::plic::PLIC_BASE;
use rv32i::csr;

use kernel::utilities::registers::interfaces::Readable;
use kernel::threadlocal::DynThreadId;

use qemu_rv32_virt_chip::MAX_THREADS;

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
    CHIP: Option<&'static QemuRv32VirtChip<'static, QemuRv32VirtDefaultPeripherals<'static>>> = None
);

// Reference to the process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

const STACK_SIZE: usize = 0x8000;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; STACK_SIZE] = [0; STACK_SIZE];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct QemuRv32VirtPlatform {
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>,
        >,
        components::process_console::Capability,
    >,
    console: &'static capsules_core::console::Console<'static>,
    lldb: &'static capsules_core::low_level_debug::LowLevelDebug<
        'static,
        capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static CooperativeSched<'static>,
    scheduler_timer: &'static VirtualSchedulerTimer<
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >,
    virtio_rng: Option<
        &'static capsules_core::rng::RngDriver<
            'static,
            qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng<'static, 'static>,
        >,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for QemuRv32VirtPlatform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            capsules_core::rng::DRIVER_NUM => {
                if let Some(rng_driver) = self.virtio_rng {
                    f(Some(rng_driver))
                } else {
                    f(None)
                }
            }
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl
    KernelResources<
        qemu_rv32_virt_chip::chip::QemuRv32VirtChip<
            'static,
            QemuRv32VirtDefaultPeripherals<'static>,
        >,
    > for QemuRv32VirtPlatform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type CredentialsCheckingPolicy = ();
    type Scheduler = CooperativeSched<'static>;
    type SchedulerTimer = VirtualSchedulerTimer<
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn credentials_checking_policy(&self) -> &'static Self::CredentialsCheckingPolicy {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.scheduler_timer
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

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
    use ThreadType as T;
    match thread_type {
        T::Main => threads::main_thread::spawn::<{T::Main as usize}>(),
        T::Application => {
            threads::app_thread::spawn::<{T::Application as usize}>()
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
