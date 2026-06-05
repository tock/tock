// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use core::ptr::addr_of;

use kernel::collections::spsc_channel::BiChannel;
use kernel::debug;
use kernel::hil::time::Freq10MHz;
use kernel::platform::chip::{Chip, InterruptService};

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};

use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};

use crate::plic::PLIC;
use sifive::plic::Plic;

use crate::interrupts;

use virtio::transports::mmio::VirtIOMMIODevice;

/// Entry type for the inter-hart lockstep synchronization channel.
#[derive(Clone, Copy)]
pub struct SyncEntry {
    pub seq: u32,
}

/// Inter-hart synchronization channel for software lockstep.
///
/// Hart 0 (side A) sends one `SyncEntry` at the top of each kernel loop
/// iteration, runs the iteration, then waits for hart 1's ack.  Hart 1
/// (side B) receives the signal, runs its iteration, then acks.  This
/// keeps both harts advancing one loop step at a time without any
/// interrupt forwarding.
///
/// Lives in the chip crate so the board's `main.rs` loop and the
/// one-time init sync in `lib.rs` can both reach it without a circular
/// dependency.
///
/// Declared as a plain `static` in hart 0's BSS.  Because the Tock BSS is
/// far larger than the ±2 KB GP-relative window, the compiler generates
/// PC-relative addressing for all accesses, so both harts compute the same
/// absolute address from the shared `.text` — only one instance exists.
pub static LOCKSTEP_CHAN: BiChannel<32, SyncEntry> = BiChannel::new();

/// CLINT MSIP[1] register address — used by hart 0 to interrupt hart 1.
pub const CLINT_MSIP1: *mut u32 = 0x0200_0004 as *mut u32;

type QemuRv32VirtPMP = rv32i::pmp::PMPUserMPU<
    5,
    rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP<16, 5>,
>;

pub type QemuRv32VirtClint<'a> = sifive::clint::Clint<'a, Freq10MHz>;

pub struct QemuRv32VirtChip<'a, I: InterruptService + 'a> {
    userspace_kernel_boundary: rv32i::syscall::SysCall,
    pmp: QemuRv32VirtPMP,
    plic: &'a Plic,
    timer: &'a QemuRv32VirtClint<'a>,
    plic_interrupt_service: &'a I,
}

pub struct QemuRv32VirtDefaultPeripherals<'a> {
    pub uart0: crate::uart::Uart16550<'a>,
    pub virtio_mmio: [VirtIOMMIODevice; 8],
}

impl QemuRv32VirtDefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            uart0: crate::uart::Uart16550::new(crate::uart::UART0_BASE),
            virtio_mmio: [
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_0_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_1_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_2_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_3_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_4_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_5_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_6_BASE),
                VirtIOMMIODevice::new(crate::virtio_mmio::VIRTIO_MMIO_7_BASE),
            ],
        }
    }
}

impl InterruptService for QemuRv32VirtDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART0 => self.uart0.handle_interrupt(),
            interrupts::VIRTIO_MMIO_0 => self.virtio_mmio[0].handle_interrupt(),
            interrupts::VIRTIO_MMIO_1 => self.virtio_mmio[1].handle_interrupt(),
            interrupts::VIRTIO_MMIO_2 => self.virtio_mmio[2].handle_interrupt(),
            interrupts::VIRTIO_MMIO_3 => self.virtio_mmio[3].handle_interrupt(),
            interrupts::VIRTIO_MMIO_4 => self.virtio_mmio[4].handle_interrupt(),
            interrupts::VIRTIO_MMIO_5 => self.virtio_mmio[5].handle_interrupt(),
            interrupts::VIRTIO_MMIO_6 => self.virtio_mmio[6].handle_interrupt(),
            interrupts::VIRTIO_MMIO_7 => self.virtio_mmio[7].handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> QemuRv32VirtChip<'a, I> {
    pub unsafe fn new(
        plic_interrupt_service: &'a I,
        timer: &'a QemuRv32VirtClint<'a>,
        pmp: rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP<16, 5>,
    ) -> Self {
        Self {
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
            pmp: rv32i::pmp::PMPUserMPU::new(pmp),
            plic: &*addr_of!(PLIC),
            timer,
            plic_interrupt_service,
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        self.plic.disable_all();
        self.plic.clear_all_pending();
        self.plic.enable_all();
    }

    unsafe fn handle_plic_interrupts(&self) {
        while let Some(interrupt) = self.plic.get_saved_interrupts() {
            if !self.plic_interrupt_service.service_interrupt(interrupt) {
                debug!("Pidx {}", interrupt);
            }
            self.with_interrupts_disabled(|| {
                self.plic.complete(interrupt);
            });
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for QemuRv32VirtChip<'a, I> {
    type MPU = QemuRv32VirtPMP;
    type UserspaceKernelBoundary = rv32i::syscall::SysCall;
    type ThreadIdProvider = rv32i::thread_id::RiscvThreadIdProvider;

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn userspace_kernel_boundary(&self) -> &rv32i::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        // On harts that have not enabled mext (e.g. hart 1 in lockstep mode),
        // PLIC interrupts saved by other harts must not be processed here —
        // this hart's peripheral structs may be uninitialized.
        let mext_enabled = CSR.mie.is_set(mie::mext);
        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.handle_interrupt();
            }
            if mext_enabled && self.plic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if !mip.any_matching_bits_set(mip::mtimer::SET)
                && (!mext_enabled || self.plic.get_saved_interrupts().is_none())
            {
                break;
            }
        }

        // Re-enable MIE bits for this hart. Hart 1 (lockstep) does not enable
        // mext, so we only set the bits that were originally requested.
        if mext_enabled {
            CSR.mie.modify(mie::mext::SET + mie::mtimer::SET);
        } else {
            CSR.mie.modify(mie::mtimer::SET);
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        // First check if the global machine timer interrupt is set.
        // We would also need to check for additional global interrupt bits
        // if there were to be used for anything in the future.
        if CSR.mip.is_set(mip::mtimer) {
            return true;
        }

        // Then we can check the PLIC.
        self.plic.get_saved_interrupts().is_some()
    }

    fn sleep(&self) {
        unsafe {
            rv32i::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(this: Option<&Self>, writer: &mut dyn Write) {
        rv32i::print_riscv_state(writer);
        if let Some(t) = this {
            let _ = writer.write_fmt(format_args!("{}", t.pmp.pmp));
        }
    }
}

fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall | mcause::Exception::SupervisorEnvCall => (),

        mcause::Exception::InstructionMisaligned
        | mcause::Exception::InstructionFault
        | mcause::Exception::IllegalInstruction
        | mcause::Exception::Breakpoint
        | mcause::Exception::LoadMisaligned
        | mcause::Exception::LoadFault
        | mcause::Exception::StoreMisaligned
        | mcause::Exception::StoreFault
        | mcause::Exception::MachineEnvCall
        | mcause::Exception::InstructionPageFault
        | mcause::Exception::LoadPageFault
        | mcause::Exception::StorePageFault
        | mcause::Exception::Unknown => {
            panic!("fatal exception");
        }
    }
}

unsafe fn handle_interrupt(intr: mcause::Interrupt) {
    match intr {
        mcause::Interrupt::UserSoft
        | mcause::Interrupt::UserTimer
        | mcause::Interrupt::UserExternal => {
            panic!("unexpected user-mode interrupt");
        }
        mcause::Interrupt::SupervisorExternal
        | mcause::Interrupt::SupervisorTimer
        | mcause::Interrupt::SupervisorSoft => {
            panic!("unexpected supervisor-mode interrupt");
        }

        mcause::Interrupt::MachineSoft => {
            CSR.mie.modify(mie::msoft::CLEAR);
        }
        mcause::Interrupt::MachineTimer => {
            CSR.mie.modify(mie::mtimer::CLEAR);
        }
        mcause::Interrupt::MachineExternal => {
            // We received an interrupt, disable interrupts while we handle them
            CSR.mie.modify(mie::mext::CLEAR);

            // Claim the interrupt, unwrap() as we know an interrupt exists
            // Once claimed this interrupt won't fire until it's completed
            // NOTE: The interrupt is no longer pending in the PLIC
            loop {
                let interrupt = (*addr_of!(PLIC)).next_pending();

                match interrupt {
                    Some(irq) => {
                        // Safe as interrupts are disabled
                        (*addr_of!(PLIC)).save_interrupt(irq);
                    }
                    None => {
                        // Enable generic interrupts
                        CSR.mie.modify(mie::mext::SET);

                        break;
                    }
                }
            }
        }

        mcause::Interrupt::Unknown(_) => {
            panic!("interrupt of unknown cause");
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// For the qemu-system-riscv32 virt machine this gets called when an
/// interrupt occurs while the chip is in kernel mode.
#[export_name = "_start_trap_rust_from_kernel"]
pub unsafe extern "C" fn start_trap_rust() {
    match mcause::Trap::from(CSR.mcause.extract()) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        mcause::Trap::Exception(exception) => {
            handle_exception(exception);
        }
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
///
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_rust_from_app"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: u32) {
    match mcause::Trap::from(mcause_val as usize) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}

/// Array used to track the "trap handler active" state per hart.
///
/// The `riscv` crate requires chip crates to allocate an array to
/// track whether any given hart is currently in a trap handler. The
/// array must be zero-initialized.
///
/// The QEMU rv32 virt target is configured with two harts (IDs 0 and 1)
/// for software lockstep. Hart 0 runs the kernel; hart 1 idles in WFI
/// until brought up for lockstep execution. We therefore allocate two
/// entries, one per hart.
#[export_name = "_trap_handler_active"]
static mut TRAP_HANDLER_ACTIVE: [usize; 2] = [0; 2];
