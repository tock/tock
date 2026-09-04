// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Shared platform setup for the Raspberry Pi Pico 2 and boards derived from
//! it.
//!
//! It is based on RP2350SoC SoC (Cortex M33).
//!
//! This crate is both a library and a binary. The binary is the plain
//! Raspberry Pi Pico 2; the library holds everything a derived board also
//! needs, so that such a board can call [`setup`] and then add only the
//! drivers that differ. See `doc/NestedBoards.md`.

#![no_std]

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use enum_primitive::cast::FromPrimitive;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::platform::SyscallDriverLookup;
use kernel::platform::chip::Chip;
use kernel::syscall::SyscallDriver;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{Kernel, capabilities, create_capability, static_init};

use rp2350::chip::{Rp2350, Rp2350DefaultPeripherals};
use rp2350::clocks::{
    AdcAuxiliaryClockSource, HstxAuxiliaryClockSource, PeripheralAuxiliaryClockSource, PllClock,
    ReferenceAuxiliaryClockSource, ReferenceClockSource, SystemAuxiliaryClockSource,
    SystemClockSource, UsbAuxiliaryClockSource,
};
use rp2350::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2350::resets::Peripheral;
use rp2350::timer::RPTimer;
#[allow(unused)]
use rp2350::{BASE_VECTORS, xosc};

mod flash_bootloader;

// Manually setting the boot header section that contains the FCB header
//
// This section attribute is only applied when targeting bare-metal
// (`target_os = "none"`). Host builds (e.g. tests, clippy, doc) use object
// formats (Mach-O, PE, ...) that reject a bare section name like this,
// yielding errors such as: `mach-o section specifier requires a segment and
// section separated by a comma`.
#[cfg_attr(target_os = "none", link_section = ".flash_bootloader")]
#[used]
static FLASH_BOOTLOADER: [u8; 256] = flash_bootloader::FLASH_BOOTLOADER;

// This section attribute is only applied when targeting bare-metal
// (`target_os = "none"`). Host builds (e.g. tests, clippy, doc) use object
// formats (Mach-O, PE, ...) that reject a bare section name like this,
// yielding errors such as: `mach-o section specifier requires a segment and
// section separated by a comma`.
#[cfg_attr(target_os = "none", link_section = ".metadata_block")]
#[used]
static METADATA_BLOCK: [u8; 28] = flash_bootloader::METADATA_BLOCK;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

/// The chip this board runs on.
pub type ChipHw = Rp2350<'static, Rp2350DefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
pub static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

/// The GPIO driver a board built on this platform supplies.
///
/// Which pins userspace may drive is a decision for each board rather than
/// for the platform: a pin that is free on one may be wired to something on
/// another. So [`setup`] takes a closure that builds this, and each board
/// names its own pins with `components::gpio_component_helper!`.
pub type GpioDriver = capsules_core::gpio::GPIO<'static, RPGpioPin<'static>>;

/// The scheduler this board uses.
pub type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

/// Drivers every board built on this platform provides.
pub struct Platform {
    /// Inter-process communication, passed to `kernel_loop`.
    pub ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    console: &'static capsules_core::console::Console<'static>,
    /// The scheduler, for `KernelResources::scheduler`.
    pub scheduler: &'static SchedulerInUse,
    /// The scheduler timer, for `KernelResources::scheduler_timer`.
    pub systick: cortexm33::systick::SysTick,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rp2350::timer::RPTimer<'static>>,
    >,
    gpio: &'static GpioDriver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
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

// Unlike arch/* and chips/*, board crates aren't cross-compiled against
// their real target for docs (see CRATE_TARGETS in
// tools/build/build_all_docs.sh), so `doc` is what makes the host-target
// doc pass pick this real implementation over having no implementation
// at all.
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

/// Bring the chip up and instantiate the drivers every board on this platform
/// shares.
///
/// Returns the kernel, the shared [`Platform`], the peripherals, the alarm mux
/// (so a derived board can hang further virtual alarms off it) and the chip.
///
/// ### Safety
///
/// Must be called exactly once, from the main thread, before any other kernel
/// initialization. It performs `static_init!` allocations and binds
/// [`PANIC_RESOURCES`] to the calling thread.
pub unsafe fn setup(
    gpio: impl FnOnce(
        &'static Kernel,
        &'static Rp2350DefaultPeripherals<'static>,
    ) -> &'static GpioDriver,
) -> (
    &'static Kernel,
    Platform,
    &'static Rp2350DefaultPeripherals<'static>,
    &'static MuxAlarm<'static, RPTimer<'static>>,
    &'static Rp2350<'static, Rp2350DefaultPeripherals<'static>>,
) {
    ChipHw::init();

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

    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.timer0)
        .finalize(components::alarm_mux_component_static!(RPTimer));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(RPTimer));

    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    let gpio = gpio(board_kernel, peripherals);

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

    kernel::create_typed_capability!(process_console_cap, ProcessConsoleCap:
        kernel::capabilities::ProcessManagementCapability,
        kernel::capabilities::ProcessStartCapability
    );
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm33::support::reset),
        process_console_cap,
    )
    .finalize(components::process_console_component_static!(
        RPTimer,
        ProcessConsoleCap
    ));
    let _ = process_console.start();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        console,
        alarm,
        gpio,
        scheduler,
        systick: cortexm33::systick::SysTick::new_with_calibration(125_000_000),
    };

    (board_kernel, platform, peripherals, mux_alarm, chip)
}
