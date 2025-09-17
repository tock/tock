// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;
use core::mem::MaybeUninit;

use kernel::component::Component;
use kernel::platform::chip::{Chip, InterruptService};
use kernel::static_init;
use x86::mpu::PagingMPU;
use x86::registers::bits32::paging::{PD, PT};
use x86::support;
use x86::{Boundary, InterruptPoller};

use crate::keyboard::Keyboard;
use crate::pic::PIC1_OFFSET;
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

    /// Interrupt number used by the PS/2 keyboard (i8042, IRQ1).
    /// Raised when the controller’s output buffer has data ready (OB=1).
    pub(super) const KEYBOARD: u32 = (PIC1_OFFSET as u32) + 1;
}

/// Representation of a generic PC platform.
///
/// This struct serves as an implementation of Tock's [`Chip`] trait for the x86 PC platform. The
/// behavior and set of peripherals available on PCs is very heavily standardized. As a result, this
/// chip definition should be broadly compatible with most PC hardware.
///
/// Parameter `PR` is the PIT reload value. See [`Pit`] for more information.
///
/// # Interrupt Handling
///
/// This chip automatically handles interrupts for legacy PC devices which are known to be present
/// on QEMU's Q35 machine type. This includes the PIT timer and four serial ports. Other devices
/// which are conditionally present (e.g. Virtio devices specified on the QEMU command line) may be
/// handled via a board-specific implementation of [`InterruptService`].
///
/// This chip uses the legacy 8259 PIC to manage interrupts. This is relatively simple compared with
/// using the Local APIC or I/O APIC and avoids needing to interact with ACPI or MP tables.
///
/// Internally, this chip re-maps the PIC interrupt numbers to avoid conflicts with ISA-defined
/// exceptions. This remapping is fully encapsulated within the chip. **N.B.** Implementors of
/// [`InterruptService`] will be passed the physical interrupt line number, _not_ the remapped
/// number used internally by the chip. This should match the interrupt line number reported by
/// documentation or read from the PCI configuration space.
pub struct Pc<'a, I: InterruptService + 'a, const PR: u16 = RELOAD_1KHZ> {
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

    /// PS/2 Controller
    pub ps2: &'a crate::ps2::Ps2Controller,

    /// Keyboard device
    pub keyboard: &'a Keyboard<'a>,

    /// System call context
    syscall: Boundary,
    paging: PagingMPU<'a>,

    /// Board-provided interrupt service for handling non-core IRQs (e.g., PCI devices)
    int_svc: &'a I,
}

impl<'a, I: InterruptService + 'a, const PR: u16> Chip for Pc<'a, I, PR> {
    type ThreadIdProvider = x86::thread_id::X86ThreadIdProvider;

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
                let mut handled = true;
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
                    interrupt::KEYBOARD => {
                        self.ps2.handle_interrupt();
                    }
                    _ => {
                        // Convert back to physical interrupt line number before passing to
                        // board-specific handler
                        let phys_num = num - PIC1_OFFSET as u32;
                        handled = unsafe { self.int_svc.service_interrupt(phys_num) };
                    }
                }

                poller.clear_pending(num);

                // Unmask the interrupt so it can fire again, but only if we know how to handle it
                if handled {
                    unsafe {
                        crate::pic::unmask(num);
                    }
                } else {
                    kernel::debug!("Unhandled external interrupt {} left masked", num);
                }
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

/// Component helper for constructing a [`Pc`] chip instance.
///
/// During the call to `finalize()`, this helper will perform low-level initialization of the PC
/// hardware to ensure a consistent CPU state. This includes initializing memory segmentation and
/// interrupt handling. See [`x86::init`] for further details.
pub struct PcComponent<'a, I: InterruptService + 'a> {
    pd: &'a mut PD,
    pt: &'a mut PT,
    int_svc: &'a I,
}

impl<'a, I: InterruptService + 'a> PcComponent<'a, I> {
    /// Creates a new `PcComponent` instance.
    ///
    /// ## Safety
    ///
    /// It is unsafe to construct more than a single `PcComponent` during the entire lifetime of the
    /// kernel.
    ///
    /// Before calling, memory must be identity-mapped. Otherwise, introduction of flat segmentation
    /// will cause the kernel's code/data to move unexpectedly.
    ///
    /// See [`x86::init`] for further details.
    pub unsafe fn new(pd: &'a mut PD, pt: &'a mut PT, int_svc: &'a I) -> Self {
        Self { pd, pt, int_svc }
    }
}

impl<I: InterruptService + 'static> Component for PcComponent<'static, I> {
    type StaticInput = (
        <SerialPortComponent as Component>::StaticInput,
        <SerialPortComponent as Component>::StaticInput,
        <SerialPortComponent as Component>::StaticInput,
        <SerialPortComponent as Component>::StaticInput,
        &'static mut MaybeUninit<Pc<'static, I>>,
        &'static mut MaybeUninit<crate::keyboard::Keyboard<'static>>,
    );
    type Output = &'static Pc<'static, I>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        // Low-level hardware initialization. We do this first to guarantee the CPU is in a
        // predictable state before initializing the chip object.
        unsafe {
            x86::init();
            crate::pic::init();
            // Enable the VGA path by building or running with the feature flag, e.g.:
            //   `cargo run -- -display none`
            // A plain `make run` / `cargo run` keeps everything on COM1.
            //
            // Initialise VGA and clear BIOS text if VGA is enabled
            // Clear BIOS banner: the real-mode BIOS leaves its text (and the cursor off-screen) in
            // 0xB8000.  Wiping the full 80×25 buffer gives us a clean screen and a visible cursor
            // before the kernel prints its first message.
            // SAFETY: PAGE_DIR is identity-mapped, aligned, and unique
            let pd: &mut PD = &mut *core::ptr::from_mut(self.pd);
            crate::vga::new_text_console(pd);
        }

        let com1 = unsafe { SerialPortComponent::new(COM1_BASE).finalize(s.0) };
        let com2 = unsafe { SerialPortComponent::new(COM2_BASE).finalize(s.1) };
        let com3 = unsafe { SerialPortComponent::new(COM3_BASE).finalize(s.2) };
        let com4 = unsafe { SerialPortComponent::new(COM4_BASE).finalize(s.3) };

        let pit = unsafe { Pit::new() };

        let vga = unsafe { static_init!(VgaText, VgaText::new()) };

        kernel::deferred_call::DeferredCallClient::register(vga);

        let paging = unsafe {
            let pd_addr = core::ptr::from_ref(self.pd) as usize;
            let pt_addr = core::ptr::from_ref(self.pt) as usize;
            PagingMPU::new(self.pd, pd_addr, self.pt, pt_addr)
        };

        paging.init();

        let syscall = Boundary::new();

        // PS/2 inside the component
        let ps2 =
            unsafe { static_init!(crate::ps2::Ps2Controller, crate::ps2::Ps2Controller::new()) };
        kernel::deferred_call::DeferredCallClient::register(ps2);

        // controller bring-up owned by the chip
        let _ = ps2.init_early();

        // keyboard device
        let keyboard = s.5.write(Keyboard::new(ps2));
        // connect keyboard as the ps/2 client, controller will call `receive_scancode`
        ps2.set_client(keyboard);
        keyboard.init_device();

        // allow IRQ1 to fire
        unsafe {
            crate::pic::unmask(interrupt::KEYBOARD);
        }

        let pc = s.4.write(Pc {
            com1,
            com2,
            com3,
            com4,
            pit,
            vga,
            ps2,
            keyboard,
            syscall,
            paging,
            int_svc: self.int_svc,
        });

        pc
    }
}

/// Provides static buffers needed for `PcComponent::finalize()`.
#[macro_export]
macro_rules! x86_q35_component_static {
    ($isr_ty:ty) => {{
        (
            $crate::serial_port_component_static!(),
            $crate::serial_port_component_static!(),
            $crate::serial_port_component_static!(),
            $crate::serial_port_component_static!(),
            kernel::static_buf!($crate::Pc<'static, $isr_ty>),
            kernel::static_buf!($crate::keyboard::Keyboard<'static>),
        )
    };};
}
