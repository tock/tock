// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::scheduler::KernelActivity;
use kernel::{create_capability, debug};
use qemu_rv32_virt_chip::chip::{clear_irq_active, CLINT_MSIP1};
use qemu_rv32_virt_chip::lockstep::{
    dispatch_layer1_event, lockstep_barrier, SyncEntry, Transport as _,
    DRAIN_TIMEOUT_MTIME_TICKS, QEMU_TRANSPORT,
};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

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
        // Phase 1: drain Layer-1 events.
        //
        // Two exit conditions:
        //   • Sync{fp}      — normal round: hart 0 ran a non-process op and is
        //                     ready to compare fingerprints.  Run Phase 2, then
        //                     compare and reply.
        //   • SyscallDesc   — gate round: hart 0 is blocked in LockstepDriver
        //                     waiting for hart 1's process to confirm the same
        //                     descriptor before emitting to UART.  Run Phase 2
        //                     (the process sends the gate confirmation), then
        //                     wait for the Sync that hart 0 sends after the gate
        //                     and UART TX complete.
        //
        // Timeout strategy: only start the fault-detection clock AFTER the
        // first L1 event arrives. During pure idle (app sleeping on alarm)
        // hart 0 sends nothing — waiting indefinitely here is correct.
        let mut post_l1_start: Option<u32> = None;
        let is_gate_round;
        let h0_fp_from_drain;
        loop {
            if let Some(entry) = QEMU_TRANSPORT.try_pop() {
                match entry {
                    SyncEntry::Sync { fingerprint } => {
                        is_gate_round = false;
                        h0_fp_from_drain = fingerprint;
                        break;
                    }
                    SyncEntry::SyscallDesc(desc) => {
                        use qemu_rv32_virt_chip::lockstep::store_pending_syscall;
                        store_pending_syscall(desc);
                        // Drain any remaining entries (e.g. UartTxDone that
                        // arrived in the same kick) before running the process.
                        while let Some(e) = QEMU_TRANSPORT.try_pop() {
                            match e {
                                SyncEntry::SyscallDesc(d) => store_pending_syscall(d),
                                other => dispatch_layer1_event(other),
                            }
                        }
                        is_gate_round = true;
                        h0_fp_from_drain = 0; // filled in after Phase 2
                        break;
                    }
                    other => {
                        dispatch_layer1_event(other);
                        post_l1_start = Some(QEMU_TRANSPORT.now_ticks());
                    }
                }
            }
            if let Some(l1_start) = post_l1_start {
                if QEMU_TRANSPORT.now_ticks().wrapping_sub(l1_start) >= DRAIN_TIMEOUT_MTIME_TICKS {
                    panic!("lockstep: hart 1 sync-wait timeout after L1 event (divergence?)");
                }
            }
            core::hint::spin_loop();
        }

        // Phase 2: run one non-KernelWork kernel operation.
        let activity = loop {
            let a = board_kernel.kernel_loop_operation(
                &platform,
                chip,
                None::<&kernel::ipc::IPC<{ qemu_rv32_virt_lib::NUM_PROCS as u8 }>>,
                true,
                &main_loop_capability,
            );
            if !matches!(a, KernelActivity::KernelWork) {
                break a;
            }
        };
        // After a process-running round, drain any TX-done event that
        // arrived on the channel before the process called transmit_buffer.
        qemu_rv32_virt_chip::uart::replay_pending_tx_done_for_hart1();

        // For gate rounds, the Sync from hart 0 arrives after the gate passes
        // and the UART TX completes — wait for it now, draining any L1 events
        // (e.g. UartTxDone) that arrive in the interim.
        let h0_fp = if is_gate_round {
            let mut timeout_start = QEMU_TRANSPORT.now_ticks();
            loop {
                if let Some(entry) = QEMU_TRANSPORT.try_pop() {
                    match entry {
                        SyncEntry::Sync { fingerprint } => break fingerprint,
                        other => {
                            dispatch_layer1_event(other);
                            timeout_start = QEMU_TRANSPORT.now_ticks();
                        }
                    }
                }
                if QEMU_TRANSPORT.now_ticks().wrapping_sub(timeout_start) >= DRAIN_TIMEOUT_MTIME_TICKS {
                    panic!("lockstep: gate round Sync timeout (hart 0 diverged?)");
                }
                core::hint::spin_loop();
            }
        } else {
            h0_fp_from_drain
        };

        let h1_fp = activity.fingerprint();
        if h0_fp != h1_fp {
            panic!(
                "Lockstep divergence (hart 1): hart 0 {:#x}, hart 1 {:#x}",
                h0_fp, h1_fp,
            );
        }
        while !QEMU_TRANSPORT.try_push(SyncEntry::Sync { fingerprint: h1_fp }) {
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

    // Signal hart 1 it's safe to proceed and synchronize before either hart
    // enters its kernel loop.  Must happen after load_processes() so hart
    // 0's own process state is fully set up first.
    qemu_rv32_virt_lib::finish_lockstep_setup();

    // Drain any interrupts/deferred calls left over from peripheral
    // initialization (VirtIO negotiation, RNG buffer setup, etc.) to avoid
    // a spurious one-round divergence at boot.
    board_kernel.kernel_preloop_operation(&platform, chip, &main_loop_capability);

    debug!("Entering main loop.");

    loop {
        let activity = board_kernel.kernel_loop_operation(
            &platform,
            chip,
            Some(&platform.base.ipc),
            false,
            &main_loop_capability,
        );
        clear_irq_active();

        // Batch all kernel work before syncing so hart 1 has a chance to
        // drain the full set of channel events and reach the same scheduler
        // decision.
        if matches!(activity, KernelActivity::KernelWork) {
            continue;
        }

        // Both harts send their fingerprint and receive the other's.
        // Hart 1 never sends Layer-1 events (it only sends Sync), so the
        // dispatch callback is a no-op on this side.
        let theirs = lockstep_barrier(
            &QEMU_TRANSPORT,
            SyncEntry::Sync { fingerprint: activity.fingerprint() },
            |_| {},
        );
        if let SyncEntry::Sync { fingerprint: h1_fp } = theirs {
            let h0_fp = activity.fingerprint();
            if h1_fp != h0_fp {
                panic!(
                    "Lockstep divergence: hart 0 {:#x}, hart 1 {:#x}",
                    h0_fp, h1_fp,
                );
            }
        }
    }
}
