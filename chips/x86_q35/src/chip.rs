// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;

use kernel::platform::chip::Chip;
use x86::mpu::PagingMPU;
use x86::support;
use x86::{Boundary, InterruptPoller};

use crate::pit::{Pit, RELOAD_1KHZ};
use crate::serial::SerialPort;
use crate::vga_uart_driver::VgaText;

/// Interrupt constants for legacy PC peripherals
mod interrupt {
    use crate::pic::PIC1_OFFSET;

    /// Interrupt number used by PIT
    pub(super) const PIT: u32 = PIC1_OFFSET as u32;

    /// Interrupt number shared by COM2 and COM4 serial devices
    pub(super) const COM2_COM4: u32 = (PIC1_OFFSET as u32) + 3;

    /// Interrupt number shared by COM1 and COM3 serial devices
    pub(super) const COM1_COM3: u32 = (PIC1_OFFSET as u32) + 4;
}

/// Representation of a generic PC platform.
///
/// This struct serves as an implementation of Tock's [`Chip`] trait for the x86 PC platform. The
/// behavior and set of peripherals available on PCs is very heavily standardized. As a result, this
/// chip definition should be broadly compatible with most PC hardware.
///
/// Parameter `PR` is the PIT reload value. See [`Pit`] for more information.
pub struct Pc<'a, const PR: u16 = RELOAD_1KHZ> {
    /// Legacy COM1 serial port
    pub com1: &'a SerialPort<'a>,

    /// Legacy COM2 serial port
    pub com2: &'a SerialPort<'a>,

    /// Legacy COM3 serial port
    pub com3: &'a SerialPort<'a>,

    /// Legacy COM4 serial port
    pub com4: &'a SerialPort<'a>,

    /// Legacy PIT timer
    pub pit: Pit<'a, PR>,

    /// Vga
    pub vga: &'a VgaText<'a>,

    /// System call context
    syscall: Boundary,
    paging: PagingMPU<'a>,
}

impl<'a, const PR: u16> Pc<'a, PR> {
    /// Construct a new `Pc` chip instance from its constituent parts.
    pub fn new(
        com1: &'a SerialPort<'a>,
        com2: &'a SerialPort<'a>,
        com3: &'a SerialPort<'a>,
        com4: &'a SerialPort<'a>,
        pit: Pit<'a, PR>,
        vga: &'a VgaText<'a>,
        syscall: Boundary,
        paging: PagingMPU<'a>,
    ) -> Self {
        Self {
            com1,
            com2,
            com3,
            com4,
            pit,
            vga,
            syscall,
            paging,
        }
    }
}

impl<'a, const PR: u16> Chip for Pc<'a, PR> {
    type MPU = PagingMPU<'a>;
    fn mpu(&self) -> &Self::MPU {
        &self.paging
    }

    type UserspaceKernelBoundary = Boundary;
    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.syscall
    }

    fn service_pending_interrupts(&self) {
        InterruptPoller::access(|poller| {
            while let Some(num) = poller.next_pending() {
                match num {
                    interrupt::PIT => self.pit.handle_interrupt(),
                    interrupt::COM2_COM4 => {
                        self.com2.handle_interrupt();
                        self.com4.handle_interrupt();
                    }
                    interrupt::COM1_COM3 => {
                        self.com1.handle_interrupt();
                        self.com3.handle_interrupt();
                    }
                    _ => unimplemented!("interrupt {num}"),
                }

                poller.clear_pending(num);
            }
        })
    }

    fn has_pending_interrupts(&self) -> bool {
        InterruptPoller::access(|poller| poller.next_pending().is_some())
    }

    #[cfg(target_arch = "x86")]
    fn sleep(&self) {
        use x86::registers::bits32::eflags::{self, EFLAGS};

        // On conventional embedded architectures like ARM and RISC-V, interrupts must be disabled
        // before going to sleep. But on x86 it is the opposite; we must ensure interrupts are
        // enabled before issuing the HLT instruction. Otherwise we will never wake up.
        let eflags = unsafe { eflags::read() };
        let enabled = eflags.0.is_set(EFLAGS::FLAGS_IF);

        if enabled {
            // Interrupts are already enabled, so go ahead and HLT.
            //
            // Safety: Assume we are running in ring zero.
            unsafe {
                x86::halt();
            }
        } else {
            // We need to re-enable interrupts before HLT-ing. We use inline assembly to guarantee
            // these instructions are executed back-to-back.
            //
            // Safety:
            //
            // As above, assume we are running in ring zero.
            //
            // Strictly speaking, this could cause to a TOCTOU race condition if `sleep` is called
            // within an `atomic` block, because interrupt handlers would be executed. Solving this
            // properly would require deep changes to Tock's `Chip` trait and kernel logic.
            //
            // In practice this doesn't seem to be an issue. `sleep` is only ever called once at the
            // end of the kernel's main loop, and that code does not appear to be vulnerable to the
            // TOCTOU.
            unsafe {
                core::arch::asm!("sti; hlt; cli");
            }
        }
    }

    #[cfg(not(target_arch = "x86"))]
    fn sleep(&self) {
        unimplemented!()
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        let _ = writeln!(writer);
        let _ = writeln!(writer, "---| PC State |---");
        let _ = writeln!(writer);

        // todo: print out anything that might be useful

        let _ = writeln!(writer, "(placeholder)");
    }
}
