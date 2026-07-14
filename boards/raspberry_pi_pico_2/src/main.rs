// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Tock kernel for the Raspberry Pi Pico 2.
//!
//! It is based on RP2350SoC SoC (Cortex M33).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use core::ptr::addr_of_mut;

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use components::led::LedsComponent;
use enum_primitive::cast::FromPrimitive;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::syscall::SyscallDriver;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{capabilities, create_capability, static_init, Kernel};

use rp2350::chip::{Rp2350, Rp2350DefaultPeripherals};
use rp2350::clocks::{
    AdcAuxiliaryClockSource, HstxAuxiliaryClockSource, PeripheralAuxiliaryClockSource, PllClock,
    ReferenceAuxiliaryClockSource, ReferenceClockSource, SystemAuxiliaryClockSource,
    SystemClockSource, UsbAuxiliaryClockSource,
};
use rp2350::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2350::lockstep::{
    dispatch_layer1_event, lockstep_barrier, DriverUpcallRules, LockstepDriver, LockstepUart,
    Rp2350UartHooks, Rp2350UpcallVerifier, SyncEntry, Transport as _, UpcallMode, UpcallRule,
    RP2350_TRANSPORT,
};
use rp2350::resets::Peripheral;
use rp2350::timer::RPTimer;
#[allow(unused)]
use rp2350::{xosc, BASE_VECTORS};

mod io;

mod flash_bootloader;

// Allocate memory for the stack
kernel::stack_size! {0x3000}

// Manually setting the boot header section that contains the FCB header
//
// When compiling for a macOS host, the `link_section` attribute is elided as
// it yields the following error: `mach-o section specifier requires a segment
// and section separated by a comma`.
#[cfg_attr(not(target_os = "macos"), link_section = ".flash_bootloader")]
#[used]
static FLASH_BOOTLOADER: [u8; 256] = flash_bootloader::FLASH_BOOTLOADER;

// When compiling for a macOS host, the `link_section` attribute is elided as
// it yields the following error: `mach-o section specifier requires a segment
// and section separated by a comma`.
#[cfg_attr(not(target_os = "macos"), link_section = ".metadata_block")]
#[used]
static METADATA_BLOCK: [u8; 28] = flash_bootloader::METADATA_BLOCK;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// ---------------------------------------------------------------------------
// Layer-2 upcall-verifier registry (compare-mode for console upcalls)
// ---------------------------------------------------------------------------
//
// Mirrors qemu_rv32_virt_lib's CONSOLE_UPCALL_RULES/UPCALL_REGISTRY exactly --
// console driver_num/subscribe_num semantics are capsule-level, not
// chip-specific. `Rp2350UpcallVerifier::on_upcall` doesn't yet act on these
// (see its doc comment), but the registry itself is live and shared by both
// cores' verifiers.
static CONSOLE_UPCALL_RULES: [UpcallRule; 2] = [
    UpcallRule {
        subscribe_num: 1, // subscribe_num 1 = WRITE_DONE in capsules_core::console
        mode: UpcallMode::Compare,
        mask: (true, false, false), // r0 = bytes written; r1/r2 unused
    },
    UpcallRule {
        subscribe_num: 2, // subscribe_num 2 = READ_DONE
        mode: UpcallMode::Compare,
        mask: (true, false, false), // r0 = bytes read; r1/r2 unused
    },
];

static UPCALL_REGISTRY: [DriverUpcallRules; 1] = [DriverUpcallRules {
    driver_num: capsules_core::console::DRIVER_NUM,
    rules: &CONSOLE_UPCALL_RULES,
}];

type ChipHw = Rp2350<'static, Rp2350DefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

// Cooperative on both cores, not this board's original RoundRobin: matches
// qemu_rv32_virt's hart 0/1 (both Cooperative), which deliberately use the
// same scheduler on both sides so fine-grained Layer-2 syscall lockstep sees
// identical scheduling decisions -- preemption timing included -- round for
// round. A RoundRobin/SysTick core racing against a Cooperative peer (or a
// mismatched pair of the two) can land a process's syscalls on different
// rounds than its peer, which is what caused "shadow ... has no matching
// leader descriptor" panics during lockstep bring-up.
type SchedulerInUse = components::sched::cooperative::CooperativeComponentType;

/// Supported drivers by the platform
pub struct RaspberryPiPico2 {
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    lockstep_console: &'static LockstepDriver<
        'static,
        rp2350::lockstep::Rp2350Transport,
        capsules_core::console::Console<'static>,
    >,
    scheduler: &'static SchedulerInUse,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rp2350::timer::RPTimer<'static>>,
    >,
    gpio: &'static capsules_core::gpio::GPIO<'static, RPGpioPin<'static>>,
    led: &'static capsules_core::led::LedDriver<'static, LedHigh<'static, RPGpioPin<'static>>, 1>,
}

impl SyscallDriverLookup for RaspberryPiPico2 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.lockstep_console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<Rp2350<'static, Rp2350DefaultPeripherals<'static>>> for RaspberryPiPico2 {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = ();
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
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

#[allow(dead_code)]
extern "C" {
    /// Entry point used for debugger
    ///
    /// When loaded using gdb, the Raspberry Pi Pico 2 is not reset
    /// by default. Without this function, gdb sets the PC to the
    /// beginning of the flash. This is not correct, as the RP2350
    /// has a more complex boot process.
    ///
    /// This function is set to be the entry point for gdb and is used
    /// to send the RP2350 back in the bootloader so that all the boot
    /// sequence is performed.
    fn jump_to_bootloader();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
    .section .jump_to_bootloader, \"ax\"
    .global jump_to_bootloader
    .thumb_func
  jump_to_bootloader:
    movs r0, #0
    ldr r1, =(0xe0000000 + 0x0000ed08)
    str r0, [r1]
    ldmia r0!, {{r1, r2}}
    msr msp, r1
    bx r2
    "
);

// ---------------------------------------------------------------------------
// Core 1 — lockstep shadow kernel (Stage A4)
// ---------------------------------------------------------------------------
//
// Core 1 runs its own, independent, peripheral-free Tock kernel instance:
// its own Clocks/SIO/TIMER0 handles (fresh value-type wrappers around the
// same MMIO -- see the module doc below for why these don't need to be
// shared with core 0's), own process array, own Cooperative scheduler (so it
// never needs a scheduler-timer alarm channel, which would otherwise contend
// with core 0's use of TIMER0's shared alarm registers), and no console/gpio/
// led/ipc drivers at all. Layer-1 peripheral-input replay and Layer-2
// syscall verification are not wired up yet (Step B) -- Stage A only proves
// the two independent kernel loops stay in fingerprint lockstep.

/// Number of concurrent processes core 1's shadow kernel supports. Kept at 1
/// (vs. core 0's `NUM_PROCS`): Stage A only needs to run a single
/// peripheral-light test app (e.g. yield-test) on both cores.
const NUM_PROCS_H1: usize = 1;

// SCRATCH DIAGNOSTIC (fail-stop verification): plain shared atomics, safe to
// write from core 1 without touching the UART directly (raw io::WRITER
// writes from core 1 race with core 0's own interrupt-driven UART use --
// confirmed by an earlier attempt that produced visibly interleaved output).
// Core 0 reports these periodically via its own, already-safe kernel::debug!
// path in its main loop below.
static CORE1_STAGE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
static CORE1_ROUND: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// Count of UartTxDone/UartRxReady/SyscallDesc Layer-1/2 events dispatched
/// on core 1 so far. Also readable from the panic handler (io.rs), which
/// reliably flushes even when triggered from core 1 (see the comment above).
static CORE1_L1_EVENT_COUNT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Core 1's minimal, peripheral-free platform.
struct Core1Platform {
    scheduler: &'static components::sched::cooperative::CooperativeComponentType,
    lockstep_console: &'static LockstepDriver<
        'static,
        rp2350::lockstep::Rp2350Transport,
        capsules_core::console::Console<'static>,
    >,
}

impl SyscallDriverLookup for Core1Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.lockstep_console)),
            _ => f(None),
        }
    }
}

impl KernelResources<Rp2350<'static, Rp2350DefaultPeripherals<'static>>> for Core1Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    // Must match core 0's scheduler exactly. Fine-grained Layer-2 syscall
    // lockstep needs both cores to make identical scheduling decisions --
    // preemption timing included -- round for round; a mismatch here (this
    // used to be Cooperative on core 1 vs. core 0's original RoundRobin) is
    // what caused "shadow ... has no matching leader descriptor" panics.
    // Both cores now use Cooperative (matching qemu_rv32_virt's hart 0/1,
    // which deliberately use the same scheduler for the same reason) rather
    // than giving core 1 a RoundRobin+SysTick pairing, since Cooperative
    // needs no `SchedulerTimer` at all -- simpler than keeping two SysTick
    // instances in sync.
    type Scheduler = components::sched::cooperative::CooperativeComponentType;
    type SchedulerTimer = ();
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
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Entry point for core 1, branched to directly by the bootrom after the
/// `SIO::launch_core1` handshake (not a hardware reset — `BASE_VECTORS[1]`
/// is never executed on core 1). The bootrom protocol already sets core 1's
/// initial SP and VTOR from the values core 0 passed to `launch_core1`, so
/// unlike a from-reset boot, no assembly preamble is needed here.
#[no_mangle]
pub unsafe extern "C" fn core1_entry() -> ! {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // SCRATCH DIAGNOSTIC: prove core 1 reaches this point at all.
    CORE1_STAGE.store(1, core::sync::atomic::Ordering::Relaxed);

    // Boot handshake: wait for core 0's one-time init Sync (pushed after
    // load_processes(), see main()), and echo it back. Reusing
    // `lockstep_barrier` for this (rather than hand-rolled channel calls)
    // gives the handshake the same bounded timeout / panic-on-divergence
    // behavior as every subsequent per-round barrier. Dispatch is wired to
    // `dispatch_layer1_event` defensively -- core 0 doesn't push Layer-1
    // events this early, but there's no reason to assume it can't.
    lockstep_barrier(
        &RP2350_TRANSPORT,
        SyncEntry::Sync { fingerprint: 0 },
        dispatch_layer1_event,
    );

    // SCRATCH DIAGNOSTIC
    CORE1_STAGE.store(2, core::sync::atomic::Ordering::Relaxed);

    // Fresh, independent peripheral handles -- NOT shared with core 0's.
    // `Clocks` caches configured frequencies in `Cell`s, which are neither
    // `Sync` (unsound to alias across cores) nor pre-populated by a second,
    // separately-constructed instance; sharing it would need cross-core
    // synchronization stage A doesn't require. Core 1 only ever touches
    // `.sio` and `.timer0` below, both of which are stateless MMIO wrappers
    // that don't depend on `Clocks` at all, so a fresh, unconfigured
    // `Clocks::new()` is harmless as long as core 1 never calls anything
    // that reads it (no uart/xosc/adc use here). `.init()` is deliberately
    // never called on this instance either, to avoid redundant writes to
    // shared hardware (e.g. the ticks generator) core 0 already configured.
    let clocks = static_init!(rp2350::clocks::Clocks, rp2350::clocks::Clocks::new());
    let peripherals = static_init!(
        Rp2350DefaultPeripherals,
        Rp2350DefaultPeripherals::new(clocks)
    );

    let chip = static_init!(
        Rp2350<Rp2350DefaultPeripherals>,
        Rp2350::new(peripherals, &peripherals.sio)
    );

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS_H1));
    let board_kernel = static_init!(Kernel, Kernel::new(processes.as_slice()));

    let scheduler = components::sched::cooperative::CooperativeComponent::new(processes)
        .finalize(components::cooperative_component_static!(NUM_PROCS_H1));

    // Layer-1 lockstep replay: core 1's console sits on top of a software-only
    // UART (no hardware backing) that's fed by `dispatch_layer1_event` in the
    // main loop below, rather than a real interrupt. See `Rp2350UartReplay`'s
    // doc comment.
    let uart_hooks_h1 = static_init!(Rp2350UartHooks, Rp2350UartHooks::new(&RP2350_TRANSPORT));
    let lockstep_uart_h1 = static_init!(
        LockstepUart<'static, rp2350::uart::Rp2350UartReplay, Rp2350UartHooks>,
        LockstepUart::new(&rp2350::uart::CORE1_UART_REPLAY, uart_hooks_h1)
    );
    hil::uart::Receive::set_receive_client(&rp2350::uart::CORE1_UART_REPLAY, lockstep_uart_h1);
    hil::uart::Transmit::set_transmit_client(&rp2350::uart::CORE1_UART_REPLAY, lockstep_uart_h1);

    let memory_allocation_capability_h1 =
        create_capability!(capabilities::MemoryAllocationCapability);
    let tx_buf = static_init!(
        [u8; capsules_core::console::DEFAULT_BUF_SIZE],
        [0; capsules_core::console::DEFAULT_BUF_SIZE]
    );
    let rx_buf = static_init!(
        [u8; capsules_core::console::DEFAULT_BUF_SIZE],
        [0; capsules_core::console::DEFAULT_BUF_SIZE]
    );
    let console = static_init!(
        capsules_core::console::Console<'static>,
        capsules_core::console::Console::new(
            lockstep_uart_h1,
            tx_buf,
            rx_buf,
            board_kernel.create_grant(
                capsules_core::console::DRIVER_NUM,
                &memory_allocation_capability_h1
            ),
        )
    );
    hil::uart::Receive::set_receive_client(lockstep_uart_h1, console);
    hil::uart::Transmit::set_transmit_client(lockstep_uart_h1, console);

    // Layer-2: gate every console Command syscall behind a cross-core
    // descriptor exchange. `core_id()` returns 1 at runtime here, so
    // `LockstepDriver::command` branches to the shadow path.
    let lockstep_console = static_init!(
        LockstepDriver<'static, rp2350::lockstep::Rp2350Transport, capsules_core::console::Console<'static>>,
        LockstepDriver::new(
            &RP2350_TRANSPORT,
            console,
            capsules_core::console::DRIVER_NUM,
            dispatch_layer1_event,
        )
    );

    let upcall_verifier_h1 = static_init!(
        Rp2350UpcallVerifier,
        Rp2350UpcallVerifier::new(&UPCALL_REGISTRY)
    );
    board_kernel.register_upcall_verifier(upcall_verifier_h1);

    let platform = Core1Platform { scheduler, lockstep_console };

    extern "C" {
        static _sapps: u8;
        static _eapps: u8;
        static mut _sappmem_h1: u8;
        static _eappmem_h1: u8;
    }

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem_h1),
            core::ptr::addr_of!(_eappmem_h1) as usize
                - core::ptr::addr_of!(_sappmem_h1) as usize,
        ),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|_err| {
        // No console on this core to report load errors to; core 0's own
        // load_processes() call already reports failures for the shared app
        // image, and Stage A only needs one core running it to compare
        // fingerprints against a divergence.
    });

    // SCRATCH DIAGNOSTIC
    CORE1_STAGE.store(3, core::sync::atomic::Ordering::Relaxed);

    let mut round: u32 = 0;

    // No outer per-round Sync barrier: core 0 often needs extra rounds of
    // real kernel work (interrupt servicing) that core 1, immune to those
    // peripherals, never experiences, so forcing a round-for-round
    // rendezvous here was fighting the two cores' natural pace instead of
    // verifying anything meaningful. The only cross-core synchronization
    // left is: (1) Layer 1's opportunistic, non-blocking event drain below,
    // and (2) Layer 2's per-syscall gate in `LockstepDriver::command`
    // (`libraries/lockstep/src/lib.rs`), which is where real divergence
    // detection now lives -- it spin-waits (bounded, fail-stop on timeout)
    // for the leader's descriptor rather than assuming "not here yet" means
    // "never coming."
    loop {
        round += 1;
        CORE1_ROUND.store(round, core::sync::atomic::Ordering::Relaxed);

        // Opportunistically drain and dispatch whatever's already on the
        // channel -- non-blocking, no rendezvous. UartRxReady/UartTxDone
        // replay immediately; SyscallDesc queues via store_pending_syscall
        // for LockstepDriver::command's shadow branch to pick up when this
        // core's own process reaches the matching syscall.
        while let Some(entry) = RP2350_TRANSPORT.try_pop() {
            dispatch_layer1_event(entry);
            CORE1_L1_EVENT_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }

        let _ = board_kernel.kernel_loop_operation(
            &platform,
            chip,
            None::<&kernel::ipc::IPC<{ NUM_PROCS_H1 as u8 }>>,
            true, // no_sleep: core 1 has no interrupt-driven wake configured
            &main_loop_capability,
        );
        // Drain any TX-done event that arrived on the channel before the
        // process called transmit_buffer.
        rp2350::uart::replay_pending_tx_done_for_core1();

        core::hint::spin_loop();
    }
}

fn init_clocks(
    peripherals: &Rp2350DefaultPeripherals,
    clocks: &'static rp2350::clocks::Clocks,
    resets: &'static rp2350::resets::Resets,
) {
    // // Start tick in watchdog
    // peripherals.watchdog.start_tick(12);
    //
    // Disable the Resus clock
    clocks.disable_resus();

    // Setup the external Oscillator
    peripherals.xosc.init();

    // disable ref and sys clock aux sources
    clocks.disable_sys_aux();
    clocks.disable_ref_aux();

    resets.reset(&[Peripheral::PllSys, Peripheral::PllUsb]);
    resets.unreset(&[Peripheral::PllSys, Peripheral::PllUsb], true);

    // Configure PLLs (from Pico SDK)
    //                   REF     FBDIV VCO            POSTDIV
    // PLL SYS: 12 / 1 = 12MHz * 125 = 1500MHZ / 6 / 2 = 125MHz
    // PLL USB: 12 / 1 = 12MHz * 40  = 480 MHz / 5 / 2 =  48MHz

    // It seems that the external oscillator is clocked at 12 MHz

    clocks.pll_init(PllClock::Sys, 12, 1, 1500 * 1000000, 6, 2);
    clocks.pll_init(PllClock::Usb, 12, 1, 480 * 1000000, 5, 2);

    // pico-sdk: // CLK_REF = XOSC (12MHz) / 1 = 12MHz
    clocks.configure_reference(
        ReferenceClockSource::Xosc,
        ReferenceAuxiliaryClockSource::PllUsb,
        12000000,
        12000000,
    );
    // pico-sdk: CLK SYS = PLL SYS (125MHz) / 1 = 125MHz
    clocks.configure_system(
        SystemClockSource::Auxiliary,
        SystemAuxiliaryClockSource::PllSys,
        125000000,
        125000000,
    );

    // pico-sdk: CLK USB = PLL USB (48MHz) / 1 = 48MHz
    clocks.configure_usb(UsbAuxiliaryClockSource::PllSys, 48000000, 48000000);
    // pico-sdk: CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
    clocks.configure_adc(AdcAuxiliaryClockSource::PllUsb, 48000000, 48000000);
    // pico-sdk: CLK HSTX = PLL USB (48MHz) / 1024 = 46875Hz
    clocks.configure_hstx(HstxAuxiliaryClockSource::PllSys, 48000000, 46875);
    // pico-sdk:
    // CLK PERI = clk_sys. Used as reference clock for Peripherals. No dividers so just select and enable
    // Normally choose clk_sys or clk_usb
    clocks.configure_peripheral(PeripheralAuxiliaryClockSource::System, 125000000);
}

unsafe fn get_peripherals() -> (
    &'static mut Rp2350DefaultPeripherals<'static>,
    &'static rp2350::clocks::Clocks,
    &'static rp2350::resets::Resets,
) {
    let clocks = static_init!(rp2350::clocks::Clocks, rp2350::clocks::Clocks::new());
    let resets = static_init!(rp2350::resets::Resets, rp2350::resets::Resets::new());
    let peripherals = static_init!(
        Rp2350DefaultPeripherals,
        Rp2350DefaultPeripherals::new(clocks)
    );
    (peripherals, clocks, resets)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    rp2350::init();

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Bind global variables to this thread.
    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    let (peripherals, clocks, resets) = get_peripherals();
    peripherals.init();

    resets.reset_all_except(&[
        Peripheral::IOQSpi,
        Peripheral::PadsQSpi,
        Peripheral::PllUsb,
        Peripheral::PllSys,
    ]);

    init_clocks(peripherals, clocks, resets);

    resets.unreset_all_except(&[], true);

    // Set the UART used for panic
    (*addr_of_mut!(io::WRITER)).set_uart(&peripherals.uart0);

    let gpio_tx = peripherals.pins.get_pin(RPGpio::GPIO0);
    let gpio_rx = peripherals.pins.get_pin(RPGpio::GPIO1);
    gpio_rx.set_function(GpioFunction::UART);
    gpio_tx.set_function(GpioFunction::UART);

    //// Disable IE for pads 26-29 (the Pico SDK runtime does this, not sure why)
    for pin in 26..30 {
        peripherals
            .pins
            .get_pin(RPGpio::from_usize(pin).unwrap())
            .deactivate_pads();
    }

    // Launch core 1 straight into `core1_entry`, which immediately blocks on
    // the boot handshake (see below) until this core finishes load_processes().
    {
        extern "C" {
            static _estack_h1: u8;
        }
        let sp = core::ptr::addr_of!(_estack_h1) as u32;
        let vtor = core::ptr::addr_of!(BASE_VECTORS) as u32;
        let entry = core1_entry as *const () as u32;
        peripherals.sio.launch_core1(vtor, sp, entry);
    }

    let chip = static_init!(
        Rp2350<Rp2350DefaultPeripherals>,
        Rp2350::new(peripherals, &peripherals.sio)
    );
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    let board_kernel = static_init!(Kernel, Kernel::new(processes.as_slice()));

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.timer0)
        .finalize(components::alarm_mux_component_static!(RPTimer));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(RPTimer));

    // Layer-1 lockstep replay: interpose LockstepUart between the real UART
    // and the mux so every TX/RX completion forwards to core 1 before
    // reaching the console/process-console capsules. See `core1_entry`'s
    // console wiring for the replay side.
    let uart_hooks = static_init!(Rp2350UartHooks, Rp2350UartHooks::new(&RP2350_TRANSPORT));
    let lockstep_uart = static_init!(
        LockstepUart<'static, rp2350::uart::Uart<'static>, Rp2350UartHooks>,
        LockstepUart::new(&peripherals.uart0, uart_hooks)
    );
    hil::uart::Receive::set_receive_client(&peripherals.uart0, lockstep_uart);
    hil::uart::Transmit::set_transmit_client(&peripherals.uart0, lockstep_uart);

    let uart_mux = components::console::UartMuxComponent::new(lockstep_uart, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    // Layer-2: gate every console Command syscall behind a cross-core
    // descriptor exchange. `core_id()` returns 0 at runtime here, so
    // `LockstepDriver::command` branches to the leader path (push, kick,
    // block until core 1 echoes the descriptor back, before emitting).
    let lockstep_console = static_init!(
        LockstepDriver<'static, rp2350::lockstep::Rp2350Transport, capsules_core::console::Console<'static>>,
        LockstepDriver::new(
            &RP2350_TRANSPORT,
            console,
            capsules_core::console::DRIVER_NUM,
            dispatch_layer1_event,
        )
    );

    let upcall_verifier = static_init!(
        Rp2350UpcallVerifier,
        Rp2350UpcallVerifier::new(&UPCALL_REGISTRY)
    );
    board_kernel.register_upcall_verifier(upcall_verifier);

    let gpio = GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            RPGpioPin,
            // Used for serial communication. Comment them in if you don't use serial.
            // 0 => peripherals.pins.get_pin(RPGpio::GPIO0),
            // 1 => peripherals.pins.get_pin(RPGpio::GPIO1),
            2 => peripherals.pins.get_pin(RPGpio::GPIO2),
            3 => peripherals.pins.get_pin(RPGpio::GPIO3),
            4 => peripherals.pins.get_pin(RPGpio::GPIO4),
            5 => peripherals.pins.get_pin(RPGpio::GPIO5),
            6 => peripherals.pins.get_pin(RPGpio::GPIO6),
            7 => peripherals.pins.get_pin(RPGpio::GPIO7),
            8 => peripherals.pins.get_pin(RPGpio::GPIO8),
            9 => peripherals.pins.get_pin(RPGpio::GPIO9),
            10 => peripherals.pins.get_pin(RPGpio::GPIO10),
            11 => peripherals.pins.get_pin(RPGpio::GPIO11),
            12 => peripherals.pins.get_pin(RPGpio::GPIO12),
            13 => peripherals.pins.get_pin(RPGpio::GPIO13),
            14 => peripherals.pins.get_pin(RPGpio::GPIO14),
            15 => peripherals.pins.get_pin(RPGpio::GPIO15),
            16 => peripherals.pins.get_pin(RPGpio::GPIO16),
            17 => peripherals.pins.get_pin(RPGpio::GPIO17),
            18 => peripherals.pins.get_pin(RPGpio::GPIO18),
            19 => peripherals.pins.get_pin(RPGpio::GPIO19),
            20 => peripherals.pins.get_pin(RPGpio::GPIO20),
            21 => peripherals.pins.get_pin(RPGpio::GPIO21),
            22 => peripherals.pins.get_pin(RPGpio::GPIO22),
            23 => peripherals.pins.get_pin(RPGpio::GPIO23),
            24 => peripherals.pins.get_pin(RPGpio::GPIO24),
            // LED pin
            // 25 => peripherals.pins.get_pin(RPGpio::GPIO25),
            26 => peripherals.pins.get_pin(RPGpio::GPIO26),
            27 => peripherals.pins.get_pin(RPGpio::GPIO27),
            28 => peripherals.pins.get_pin(RPGpio::GPIO28),
            29 => peripherals.pins.get_pin(RPGpio::GPIO29)
        ),
    )
    .finalize(components::gpio_component_static!(RPGpioPin<'static>));

    let led = LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, RPGpioPin<'static>>,
        LedHigh::new(peripherals.pins.get_pin(RPGpio::GPIO25))
    ));

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // PROCESS CONSOLE
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm33::support::reset),
    )
    .finalize(components::process_console_component_static!(RPTimer));
    let _ = process_console.start();

    let scheduler = components::sched::cooperative::CooperativeComponent::new(processes)
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let raspberry_pi_pico = RaspberryPiPico2 {
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        lockstep_console,
        alarm,
        gpio,
        led,
        scheduler,
    };

    kernel::debug!("Initialization complete. Enter main loop");

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
    }

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
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        kernel::debug!("Error loading processes!");
        kernel::debug!("{:?}", err);
    });

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // Boot handshake: send the one-time init Sync and wait for core 1's
    // echo. Must happen after load_processes() so core 0's own process state
    // is fully set up before lockstep iteration begins. See `core1_entry`'s
    // matching call.
    lockstep_barrier(&RP2350_TRANSPORT, SyncEntry::Sync { fingerprint: 0 }, |_| {});
    kernel::debug!("Lockstep: init sync complete");

    // Drain any interrupts/deferred calls left over from peripheral
    // initialization (UART, GPIO, alarm mux setup) to avoid a spurious
    // one-round divergence at boot.
    board_kernel.kernel_preloop_operation(&raspberry_pi_pico, chip, &main_loop_capability);

    kernel::debug!("Entering main loop.");

    // No outer per-round Sync barrier on this side either -- see the
    // matching comment in `core1_entry`. Real divergence detection now lives
    // entirely in Layer 2's per-syscall gate (`LockstepDriver::command`).
    //
    // SCRATCH DIAGNOSTIC (fail-stop verification, remove once stable):
    // report core 1's progress (written to CORE1_STAGE/CORE1_ROUND via plain
    // atomics) periodically, from core 0's own already-safe kernel::debug!
    // path.
    let mut round0: u32 = 0;

    loop {
        let _ = board_kernel.kernel_loop_operation(
            &raspberry_pi_pico,
            chip,
            Some(&raspberry_pi_pico.ipc),
            false,
            &main_loop_capability,
        );

        round0 += 1;
        if round0 <= 5 || round0 % 200 == 0 {
            kernel::debug!(
                "[core0] round0={} core1_stage={} core1_round={}",
                round0,
                CORE1_STAGE.load(core::sync::atomic::Ordering::Relaxed),
                CORE1_ROUND.load(core::sync::atomic::Ordering::Relaxed),
            );
        }
    }
}
