// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::cell::UnsafeCell;
use core::fmt::Write;
use core::ptr::addr_of;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};

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
    /// Fingerprint of the `KernelActivity` performed this iteration.
    /// Hart 1 echoes back its own fingerprint; hart 0 compares against its own.
    pub fingerprint: u32,
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

/// Maximum number of bytes that can be forwarded in one UART RX replay.
pub const UART_RX_REPLAY_MAX: usize = 256;

/// Reason bits for the MSIP kick from Hart 0 to Hart 1.
pub const MSIP_REASON_WATCHDOG: u8 = 0b01;
pub const MSIP_REASON_UART_RX: u8 = 0b10;

/// Bitmask of pending MSIP reasons. Hart 0 ORs in its reason before writing
/// CLINT_MSIP1; Hart 1's MachineSoft handler swaps this to 0 and dispatches.
#[link_section = ".bss"]
pub static MSIP_REASON: AtomicU8 = AtomicU8::new(0);

/// Bytes received by Hart 0's UART, to be replayed on Hart 1.
/// Written before MSIP_REASON/CLINT_MSIP1; read after MSIP_REASON is consumed.
pub struct UartRxReplayBuf(pub UnsafeCell<[u8; UART_RX_REPLAY_MAX]>);

// SAFETY: only Hart 0 writes the buffer (in receive()), and only after
// storing UART_RX_REPLAY_LEN then writing CLINT_MSIP1.  Hart 1 reads it
// only inside its MachineSoft handler after consuming MSIP_REASON, by which
// point Hart 0 has finished the write.  The Release/Acquire on the atomics
// provide the necessary ordering.
unsafe impl Sync for UartRxReplayBuf {}

#[link_section = ".bss"]
pub static UART_RX_REPLAY_BUF: UartRxReplayBuf =
    UartRxReplayBuf(UnsafeCell::new([0u8; UART_RX_REPLAY_MAX]));

/// Number of valid bytes in UART_RX_REPLAY_BUF. Zero means no replay pending.
#[link_section = ".bss"]
pub static UART_RX_REPLAY_LEN: AtomicU8 = AtomicU8::new(0);

/// Pointer to Hart 1's `Uart16550` instance, set during `start_secondary()`.
/// Zero until initialized; the MachineSoft handler checks this before dispatching.
#[link_section = ".bss"]
pub static HART1_UART_PTR: AtomicUsize = AtomicUsize::new(0);

/// CLINT mtime registers (read-only, shared across harts).
const CLINT_MTIME_LO: *const u32 = 0x0200_BFF8 as *const u32;
const CLINT_MTIME_HI: *const u32 = 0x0200_BFFC as *const u32;

/// CLINT mtimecmp[1] — written by hart 1's MachineSoft handler to arm the watchdog.
const CLINT_MTIMECMP1_LO: *mut u32 = 0x0200_4008 as *mut u32;
const CLINT_MTIMECMP1_HI: *mut u32 = 0x0200_400C as *mut u32;

/// Watchdog timeout: 100 ms at 10 MHz.
const WATCHDOG_TICKS: u64 = 1_000_000;

/// Set when hart 0 enters an interrupt handler; cleared when
/// `service_pending_interrupts` finishes. Hart 1's MachineSoft handler
/// watches this flag and panics if it stays set past the deadline.
///
/// Must be in .bss (not .sbss) so both harts share the same instance
/// rather than each getting a GP-relative private copy.
#[link_section = ".bss"]
static IRQ_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Watchdog deadline stored by Hart 1's MachineSoft handler.
/// Small (4 bytes each) → lives in .sbss → per-hart; only Hart 1 reads/writes.
static WATCHDOG_DEADLINE_LO: AtomicU32 = AtomicU32::new(0);
static WATCHDOG_DEADLINE_HI: AtomicU32 = AtomicU32::new(u32::MAX);

/// Called by hart 0's main loop after kernel_loop_operation returns.
/// Clears IRQ_ACTIVE to signal hart 1's watchdog that all kernel work
/// (interrupt handling + deferred calls) has completed.
pub fn clear_irq_active() {
    IRQ_ACTIVE.store(false, Ordering::Release);
}

/// Read the CLINT mtime counter, handling the 32-bit lo/hi rollover.
///
/// # Safety
/// Caller must ensure MMIO is mapped and accessible.
unsafe fn read_mtime() -> u64 {
    loop {
        let hi1 = core::ptr::read_volatile(CLINT_MTIME_HI) as u64;
        let lo = core::ptr::read_volatile(CLINT_MTIME_LO) as u64;
        let hi2 = core::ptr::read_volatile(CLINT_MTIME_HI) as u64;
        if hi1 == hi2 {
            return (hi1 << 32) | lo;
        }
    }
}

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
                // On Hart 1 (lockstep): timer.handle_interrupt() may have
                // reset mtimecmp[1] to the next scheduler deadline, which
                // could be before the watchdog deadline. Re-arm to the stored
                // watchdog deadline so the watchdog still fires if Hart 0 hangs.
                if !mext_enabled && IRQ_ACTIVE.load(Ordering::Acquire) {
                    unsafe {
                        core::ptr::write_volatile(
                            CLINT_MTIMECMP1_LO,
                            0xFFFF_FFFF_u32,
                        );
                        core::ptr::write_volatile(
                            CLINT_MTIMECMP1_HI,
                            WATCHDOG_DEADLINE_HI.load(Ordering::Relaxed),
                        );
                        core::ptr::write_volatile(
                            CLINT_MTIMECMP1_LO,
                            WATCHDOG_DEADLINE_LO.load(Ordering::Relaxed),
                        );
                    }
                }
            }
            if mext_enabled && self.plic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if !mip.is_set(mip::mtimer)
                && (!mext_enabled || self.plic.get_saved_interrupts().is_none())
            {
                break;
            }
        }

        if mext_enabled {
            IRQ_ACTIVE.store(false, Ordering::Release);
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
            let hartid: u32;
            core::arch::asm!("csrr {}, mhartid", out(reg) hartid);
            if hartid == 1 {
                core::ptr::write_volatile(CLINT_MSIP1, 0); // clear MSIP[1]

                // Dispatch on the reason bitmask set by Hart 0 before kicking MSIP.
                let reason = MSIP_REASON.swap(0, Ordering::Acquire);
                if reason & MSIP_REASON_UART_RX != 0 {
                    crate::uart::replay_rx_done_for_hart1();
                }

                // Arm hart 1's hardware watchdog timer: if hart 0 doesn't
                // finish interrupt handling within WATCHDOG_TICKS, the
                // MachineTimer handler will fire and panic.
                // Store the deadline so service_pending_interrupts can
                // re-arm after a scheduler preemption resets mtimecmp[1].
                let deadline = read_mtime() + WATCHDOG_TICKS;
                WATCHDOG_DEADLINE_HI.store((deadline >> 32) as u32, Ordering::Relaxed);
                WATCHDOG_DEADLINE_LO.store(deadline as u32, Ordering::Relaxed);
                // Write LO=MAX first to prevent a spurious timer fire while
                // HI is being updated, then set both to the deadline.
                core::ptr::write_volatile(CLINT_MTIMECMP1_LO, 0xFFFF_FFFF_u32);
                core::ptr::write_volatile(CLINT_MTIMECMP1_HI, (deadline >> 32) as u32);
                core::ptr::write_volatile(CLINT_MTIMECMP1_LO, deadline as u32);
                CSR.mie.modify(mie::msoft::SET);
            }
        }
        mcause::Interrupt::MachineTimer => {
            let hartid: u32;
            core::arch::asm!("csrr {}, mhartid", out(reg) hartid);
            if hartid == 0 {
                IRQ_ACTIVE.store(true, Ordering::Release);
                core::ptr::write_volatile(CLINT_MSIP1, 1);
                CSR.mie.modify(mie::mtimer::CLEAR);
            } else {
                // Hart 1: could be the watchdog deadline or a scheduler preemption.
                // In both cases, clear mie.mtimer and leave mip.mtimer set so the
                // kernel can detect the preemption via has_pending_interrupts().
                // service_pending_interrupts will call timer.handle_interrupt() to
                // reset mtimecmp[1] and re-enable mie.mtimer.
                CSR.mie.modify(mie::mtimer::CLEAR);
                if IRQ_ACTIVE.load(Ordering::Acquire) {
                    // Check whether the watchdog deadline has actually been reached.
                    // If so, hart 0 is hung in interrupt handling → panic.
                    // If not, this was a scheduler preemption that fired while the
                    // watchdog was ticking; fall through and let the kernel preempt.
                    let now = read_mtime();
                    let deadline = ((WATCHDOG_DEADLINE_HI.load(Ordering::Relaxed) as u64) << 32)
                        | (WATCHDOG_DEADLINE_LO.load(Ordering::Relaxed) as u64);
                    if now >= deadline {
                        panic!("Lockstep watchdog: hart 0 hung in interrupt handler");
                    }
                }
                // Leave mip.mtimer set for kernel preemption detection.
            }
        }
        mcause::Interrupt::MachineExternal => {
            IRQ_ACTIVE.store(true, Ordering::Release);
            core::ptr::write_volatile(CLINT_MSIP1, 1);
            CSR.mie.modify(mie::mext::CLEAR);

            loop {
                let interrupt = (*addr_of!(PLIC)).next_pending();

                match interrupt {
                    Some(irq) => {
                        (*addr_of!(PLIC)).save_interrupt(irq);
                    }
                    None => {
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
