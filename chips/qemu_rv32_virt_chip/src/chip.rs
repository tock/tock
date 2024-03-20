// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use core::ptr::addr_of;

use kernel::debug;
use kernel::hil::time::Freq10MHz;
use kernel::platform::chip::{Chip, InterruptService};

use kernel::threadlocal::{DynThreadId, ThreadId};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};

use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};

use sifive::plic::Plic;

use crate::interrupts;
use crate::plic::PLIC;
use crate::{MAX_THREADS, MAX_CONTEXTS, plic::PLIC_BASE};

use virtio::transports::mmio::VirtIOMMIODevice;

type QemuRv32VirtPMP = rv32i::pmp::PMPUserMPU<
    5,
    rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP<16, 5>,
>;

pub type QemuRv32VirtClint<'a> = sifive::clint::Clint<'a, Freq10MHz>;

// pub type QemuRv32VirtPlic = sifive::plic::Plic<MAX_THREADS>;

pub struct QemuRv32VirtChip<'a, I: InterruptService + 'a> {
    userspace_kernel_boundary: rv32i::syscall::SysCall,
    pmp: QemuRv32VirtPMP,
    plic: &'a Plic<MAX_CONTEXTS>,
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
        plic: &'a Plic<MAX_CONTEXTS>,
    ) -> Self {
        Self {
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
            pmp: rv32i::pmp::PMPUserMPU::new(pmp),
            plic,
            timer,
            plic_interrupt_service,
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        let hart_id = CSR.mhartid.extract().get();
        let context_id = hart_id * 2;
        self.plic.disable_all(context_id);
        self.plic.clear_all_pending(context_id);
        self.plic.enable_all(context_id);
    }

    unsafe fn handle_plic_interrupts(&self) {
        let hart_id = CSR.mhartid.extract().get();
        let context_id = hart_id * 2;
        while let Some(interrupt) = self.plic.get_saved_interrupts() {
            if !self.plic_interrupt_service.service_interrupt(interrupt) {
                debug!("Pidx {}", interrupt);
            }
            self.atomic(|| {
                self.plic.complete(context_id, interrupt);
            });
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for QemuRv32VirtChip<'a, I> {
    type MPU = QemuRv32VirtPMP;
    type UserspaceKernelBoundary = rv32i::syscall::SysCall;

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn userspace_kernel_boundary(&self) -> &rv32i::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.handle_interrupt();
            }
            if self.plic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if !mip.any_matching_bits_set(mip::mtimer::SET)
                && self.plic.get_saved_interrupts().is_none()
            {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie.modify(mie::mext::SET + mie::mtimer::SET);
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

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::atomic(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        rv32i::print_riscv_state(writer);
        let _ = writer.write_fmt(format_args!("{}", self.pmp.pmp));
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

            let hart_id = CSR.mhartid.extract().get();
            let context_id = hart_id * 2;

            // Claim the interrupt, unwrap() as we know an interrupt exists
            // Once claimed this interrupt won't fire until it's completed
            // NOTE: The interrupt is no longer pending in the PLIC
            while let Some(plic) =
                kernel::thread_local_static_access!(PLIC, DynThreadId::new(hart_id))
            {
                let interrupt = plic.next_pending(context_id);

                match interrupt {
                    Some(irq) => {
                        // Safe as interrupts are disabled
                        plic.save_interrupt(irq);
                    }
                    None => {
                        // Enable generic interrupts
                        CSR.mie.modify(mie::mext::SET);

                        break;
                    }
                }
            }
        }

        mcause::Interrupt::Unknown => {
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
