// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;

use kernel::component::Component;
use kernel::platform::chip::{Chip, InterruptService};
use x86::mpu::PagingMPU;
use x86::registers::bits32::paging::{PD, PT};
use x86::support;
use x86::{Boundary, InterruptPoller};

use crate::pit::{Pit, RELOAD_1KHZ};
use crate::serial::{SerialPort, SerialPortComponent, COM1_BASE, COM2_BASE, COM3_BASE, COM4_BASE};
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
    pub pit: &'a Pit<'a, PR>,

    /// Vga
    pub vga: &'a VgaText<'a>,

    /// System call context
    syscall: Boundary,
    paging: PagingMPU<'a>,
    /// Interrupt service used to dispatch IRQs to peripherals
    interrupt_service: &'static dyn InterruptService,
}

impl<const PR: u16> Pc<'static, PR> {
    /// Construct `Pc` using a standard set of peripherals plus page tables.
    ///
    /// ## Safety
    /// - Must be called only once for the lifetime of the kernel.
    /// - `pd` and `pt` must be identity-mapped and unique.
    pub unsafe fn new(
        peripherals: &'static PcDefaultPeripherals<PR>,
        pd: &'static mut PD,
        pt: &'static mut PT,
    ) -> Self {
        let paging = unsafe {
            let pd_addr = core::ptr::from_ref(pd) as usize;
            let pt_addr = core::ptr::from_ref(pt) as usize;
            let mpu = PagingMPU::new(pd, pd_addr, pt, pt_addr);
            mpu.init();
            mpu
        };

        let syscall = Boundary::new();

        Self {
            com1: peripherals.com1,
            com2: peripherals.com2,
            com3: peripherals.com3,
            com4: peripherals.com4,
            pit: &peripherals.pit,
            vga: peripherals.vga,
            syscall,
            paging,
            interrupt_service: peripherals,
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
                match unsafe { self.interrupt_service.service_interrupt(num) } {
                    true => {}
                    false => panic!("unhandled interrupt"),
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

/// Default x86 PC peripherals commonly present on Q35 machines.
pub struct PcDefaultPeripherals<const PR: u16 = RELOAD_1KHZ> {
    pub com1: &'static SerialPort<'static>,
    pub com2: &'static SerialPort<'static>,
    pub com3: &'static SerialPort<'static>,
    pub com4: &'static SerialPort<'static>,
    pub pit: Pit<'static, PR>,
    pub vga: &'static VgaText<'static>,
}

impl<const PR: u16> PcDefaultPeripherals<PR> {
    /// Create and initialize default peripherals.
    ///
    /// The caller must provide statics through `x86_q35_peripherals_static!()`.
    ///
    /// ## Safety
    /// - Must be called only once per kernel lifetime.
    pub unsafe fn new(
        s: (
            (&'static mut core::mem::MaybeUninit<SerialPort<'static>>,),
            (&'static mut core::mem::MaybeUninit<SerialPort<'static>>,),
            (&'static mut core::mem::MaybeUninit<SerialPort<'static>>,),
            (&'static mut core::mem::MaybeUninit<SerialPort<'static>>,),
            &'static mut core::mem::MaybeUninit<VgaText<'static>>,
        ),
        page_dir: &mut PD,
    ) -> Self {
        // CPU/interrupt controller/VGA baseline init
        unsafe {
            x86::init();
            crate::pic::init();
            let pd_ref: &mut PD = &mut *core::ptr::from_mut(page_dir);
            crate::vga::new_text_console(pd_ref);
        }

        let com1 = unsafe { SerialPortComponent::new(COM1_BASE).finalize(s.0) };
        let com2 = unsafe { SerialPortComponent::new(COM2_BASE).finalize(s.1) };
        let com3 = unsafe { SerialPortComponent::new(COM3_BASE).finalize(s.2) };
        let com4 = unsafe { SerialPortComponent::new(COM4_BASE).finalize(s.3) };

        let pit = unsafe { Pit::new() };

        let vga = s.4.write(VgaText::new());

        Self {
            com1,
            com2,
            com3,
            com4,
            pit,
            vga,
        }
    }

    /// Finalize deferred-call registrations and any circular deps.
    pub fn setup_circular_deps(&self) {
        kernel::deferred_call::DeferredCallClient::register(self.vga);
    }
}

impl<const PR: u16> InterruptService for PcDefaultPeripherals<PR> {
    unsafe fn service_interrupt(&self, num: u32) -> bool {
        match num {
            interrupt::PIT => {
                self.pit.handle_interrupt();
                true
            }
            interrupt::COM2_COM4 => {
                self.com2.handle_interrupt();
                self.com4.handle_interrupt();
                true
            }
            interrupt::COM1_COM3 => {
                self.com1.handle_interrupt();
                self.com3.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}
