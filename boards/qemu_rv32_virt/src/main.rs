// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::platform::chip::Chip;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::{create_capability, debug};
use qemu_rv32_virt_chip::chip::{
    clear_irq_active, read_mtime_low, SyncEntry, CLINT_MSIP1, LOCKSTEP_CHAN,
};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// How long hart 0 will wait for hart 1's per-iteration lockstep sync ack
/// before treating it as a fault (e.g. an SEU leaving hart 1 unable to make
/// progress). 100 ms at 10 MHz.
const SYNC_TIMEOUT_MTIME_TICKS: u32 = 1_000_000;

type ScreenDriver = capsules_extra::screen::screen::Screen<'static>;

struct Platform {
    base: qemu_rv32_virt_lib::QemuRv32VirtPlatform,
    screen: Option<&'static ScreenDriver>,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::screen::screen::DRIVER_NUM => {
                if let Some(screen_driver) = self.screen {
                    f(Some(screen_driver))
                } else {
                    f(None)
                }
            }

            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<qemu_rv32_virt_lib::ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::SyscallFilter;
    type ProcessFault = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::ProcessFault;
    type Scheduler = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::Scheduler;
    type SchedulerTimer = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::SchedulerTimer;
    type WatchDog = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::WatchDog;
    type ContextSwitchCallback = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        self.base.syscall_filter()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        self.base.process_fault()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.base.scheduler()
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.base.scheduler_timer()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        self.base.watchdog()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        self.base.context_switch_callback()
    }
}

// ---------------------------------------------------------------------------
// Hart 1 entry — runs instead of main() on secondary harts
// ---------------------------------------------------------------------------

// Override the weak WFI stub from the arch crate. Sets GP (same global
// pointer as hart 0 — shared binary, shared .data), then SP from the
// dedicated hart-1 stack symbol, then jumps to the Rust secondary-hart init.
#[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
core::arch::global_asm!(r#"
    .section .text._hart1_entry_board, "ax", @progbits
    .global _hart1_entry
    .type _hart1_entry, @function
    _hart1_entry:
        /* Set GP to hart 1's own data midpoint before any data access. */
        .option push
        .option norelax
        la gp, _gp_h1
        .option pop
        la sp, _estack_h1

        /* Copy .data for hart 1: flash LMA (_etext) → hart 1 VMA (_srelocate_h1.._erelocate_h1). */
        la a0, _srelocate_h1
        la a1, _erelocate_h1
        la a2, _etext
    .L_copy_data_h1:
        beq  a0, a1, .L_copy_data_h1_done
        lw   t0, 0(a2)
        sw   t0, 0(a0)
        addi a0, a0, 4
        addi a2, a2, 4
        j    .L_copy_data_h1
    .L_copy_data_h1_done:

        /* Zero .bss for hart 1: _szero_h1.._ezero_h1. */
        la a0, _szero_h1
        la a1, _ezero_h1
    .L_zero_bss_h1:
        beq  a0, a1, .L_zero_bss_h1_done
        sw   zero, 0(a0)
        addi a0, a0, 4
        j    .L_zero_bss_h1
    .L_zero_bss_h1_done:

        call main_secondary
    .L_h1_halt:
        wfi
        j .L_h1_halt
"#);

/// Secondary-hart entry point called from `_hart1_entry`.
///
/// Spins until hart 0 has finished all peripheral initialization (signalled
/// via CLINT MSIP[1]), then runs a minimal, peripheral-free Tock kernel loop.
#[no_mangle]
pub unsafe extern "C" fn main_secondary() -> ! {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // Spin until hart 0 writes 1 to CLINT MSIP[1] at the end of start(),
    // guaranteeing all shared hardware is configured.
    // No wfi here: the arch startup disables all machine interrupts (mie=0)
    // before jumping to _hart1_entry, so wfi would never wake on the pending
    // MSIP even though the signal has already been sent.
    loop {
        if core::ptr::read_volatile(CLINT_MSIP1) != 0 {
            break;
        }
    }
    core::ptr::write_volatile(CLINT_MSIP1, 0);

    let (board_kernel, platform, chip) = qemu_rv32_virt_lib::start_secondary();

    loop {
        // Drain the channel until the per-iteration Sync barrier arrives,
        // dispatching any event popped along the way. This is the only
        // point guaranteed to see every channel message in arrival order
        // without risking consuming the next Sync meant for this same loop.
        loop {
            match LOCKSTEP_CHAN.b_spin_recv() {
                SyncEntry::Sync { .. } => break,
                SyncEntry::UartRxReady { len } => {
                    qemu_rv32_virt_chip::uart::replay_rx_done_for_hart1(len)
                }
                SyncEntry::UartTxDone => qemu_rv32_virt_chip::uart::replay_tx_done_for_hart1(),
                SyncEntry::RngReady { .. } => {
                    // Not yet wired up -- see the RNG forwarding design.
                }
            }
        }
        let activity = board_kernel.kernel_loop_operation(
            &platform,
            chip,
            None::<&kernel::ipc::IPC<{ qemu_rv32_virt_lib::NUM_PROCS as u8 }>>,
            true,
            &main_loop_capability,
        );
        while !LOCKSTEP_CHAN.b_send(SyncEntry::Sync {
            fingerprint: activity.fingerprint(),
        }) {
            core::hint::spin_loop();
        }
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, base_platform, chip, _processes) = qemu_rv32_virt_lib::start();

    let screen = base_platform.virtio_gpu_screen.map(|screen| {
        components::screen::ScreenComponent::new(
            board_kernel,
            capsules_extra::screen::screen::DRIVER_NUM,
            screen,
            None,
        )
        .finalize(components::screen_component_static!(1032))
    });

    let platform = Platform {
        base: base_platform,
        screen,
    };

    // Start the process console:
    let _ = platform.base.pconsole.start();

    // These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
        /// The start of the kernel text (Included only for kernel PMP)
        static _stext: u8;
        /// The end of the kernel text (Included only for kernel PMP)
        static _etext: u8;
        /// The start of the kernel / app / storage flash (Included only for kernel PMP)
        static _sflash: u8;
        /// The end of the kernel / app / storage flash (Included only for kernel PMP)
        static _eflash: u8;
        /// The start of the kernel / app RAM (Included only for kernel PMP)
        static _ssram: u8;
        /// The end of the kernel / app RAM (Included only for kernel PMP)
        static _esram: u8;
    }
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    // Drain any interrupts/deferred calls left over from peripheral
    // initialization (VirtIO negotiation, RNG buffer setup, etc.) before
    // hart 1 is signaled to start. Otherwise hart 0's first
    // kernel_loop_operation() call reports KernelWork while hart 1
    // (peripheral-free, nothing pending) immediately reaches RanProcess --
    // a spurious one-round divergence at boot, not a real fault.
    board_kernel.kernel_preloop_operation(&platform, chip, &main_loop_capability);
    debug!(
        "TEMP DIAGNOSTIC: after preloop, has_pending_interrupts={}, has_tasks={}",
        chip.has_pending_interrupts(),
        kernel::deferred_call::DeferredCall::has_tasks()
    );

    // Signal hart 1 it's safe to proceed and synchronize before either hart
    // enters its kernel loop.  Must happen after load_processes() so hart
    // 0's own process state is fully set up first.
    qemu_rv32_virt_lib::finish_lockstep_setup();

    debug!("Entering main loop.");

    loop {
        while !LOCKSTEP_CHAN.a_send(SyncEntry::Sync { fingerprint: 0 }) {
            core::hint::spin_loop();
        }
        let activity = board_kernel.kernel_loop_operation(
            &platform,
            chip,
            Some(&platform.base.ipc),
            false,
            &main_loop_capability,
        );
        // Extend IRQ_ACTIVE coverage through deferred calls: clear only after
        // kernel_loop_operation returns so the hart-1 watchdog covers the full
        // interrupt + deferred-call window, not just the trap handler.
        clear_irq_active();
        // Drain the channel until hart 1's Sync ack arrives, dispatching
        // any event popped along the way -- mirrors hart 1's own drain
        // loop, for the same reason: hart 0 can't advance to the next
        // iteration until it sees this ack, so any hart-1-originated event
        // pushed beforehand will be interleaved ahead of it in arrival
        // order. Hart 1 doesn't push any of these back today; reserved for
        // future hart-1-originated events (the doorbell side of this is
        // already wired up -- see MachineSoft's hart 0 branch).
        let sync_wait_start = read_mtime_low();
        let mut sync_spins: u32 = 0;
        let ack_fingerprint = loop {
            if let Some(entry) = LOCKSTEP_CHAN.a_recv() {
                match entry {
                    SyncEntry::Sync { fingerprint, .. } => break fingerprint,
                    SyncEntry::UartRxReady { .. }
                    | SyncEntry::UartTxDone
                    | SyncEntry::RngReady { .. } => {
                        unreachable!("hart 1 only ever acks with SyncEntry::Sync today")
                    }
                }
            }
            sync_spins = sync_spins.wrapping_add(1);


            // Bounded by SYNC_TIMEOUT_MTIME_TICKS: an SEU (or any other fault) that
            // leaves hart 1 unable to reach its own Sync send would otherwise
            // spin hart 0 forever. The deadline is only checked every 1024
            // spins (not every iteration) since `a_recv()` is non-blocking and
            // most spins are expected to find nothing -- read_mtime_low() is a
            // single CSR read so this is cheap either way, but there's no
            // reason to pay it on every spin.
            if sync_spins & 0x3FF == 0
                && read_mtime_low().wrapping_sub(sync_wait_start) >= SYNC_TIMEOUT_MTIME_TICKS
            {
                panic!("Lockstep: hart 1 sync timeout -- no ack within {} ticks (possible SEU)", SYNC_TIMEOUT_MTIME_TICKS);
            }
            core::hint::spin_loop();
        };
        let expected = activity.fingerprint();
        // Hart 0 alone owns real I/O (process console, virtio devices), so it
        // legitimately does KernelWork/Slept transitions hart 1 never sees.
        // Only treat it as divergence when at least one hart ran a process —
        // that's the invariant lockstep actually needs to hold.
        const RAN_PROCESS_TAG: u32 = 0x0200_0000;
        let is_ran_process = |fp: u32| fp & 0xFF00_0000 == RAN_PROCESS_TAG;
        if (is_ran_process(expected) || is_ran_process(ack_fingerprint)) && ack_fingerprint != expected
        {
            panic!(
                "Lockstep divergence: hart 0 fingerprint {:#x}, hart 1 fingerprint {:#x}",
                expected, ack_fingerprint
            );
        }
    }
}
