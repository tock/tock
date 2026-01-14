// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Board file for qemu-system-i486 "q35" machine type

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use capsules_core::alarm;
use capsules_core::console::{self, Console};
use capsules_core::rng::RngDriver;
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use components::console::ConsoleComponent;
use components::debug_writer::DebugWriterComponent;
use core::ptr;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::debug::PanicResources;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil;
use kernel::ipc::IPC;
use kernel::platform::chip::InterruptService;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::syscall::SyscallDriver;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, static_init};
use virtio::devices::virtio_rng::VirtIORng;
use virtio::devices::VirtIODeviceType;
use virtio_pci_x86::VirtIOPCIDevice;
use x86::dma_fence::X86DmaFence;
use x86::registers::bits32::paging::{PDEntry, PTEntry, PD, PT};
use x86::registers::irq;
use x86_q35::pit::{Pit, RELOAD_1KHZ};
use x86_q35::{Pc, PcDefaultPeripherals};

mod multiboot;
use multiboot::MultibootV1Header;

mod io;

/// Multiboot V1 header, allowing this kernel to be booted directly by QEMU
///
/// When compiling for a macOS host, the `link_section` attribute is elided as
/// it yields the following error: `mach-o section specifier requires a segment
/// and section separated by a comma`.
#[cfg_attr(not(target_os = "macos"), link_section = ".multiboot")]
#[used]
static MULTIBOOT_V1_HEADER: MultibootV1Header = MultibootV1Header::new(0);

const NUM_PROCS: usize = 4;

type ChipHw = Pc<'static, PcDefaultPeripherals, VirtioDevices>;
type AlarmHw = Pit<'static, RELOAD_1KHZ>;
type SchedulerTimerHw =
    components::virtual_scheduler_timer::VirtualSchedulerTimerComponentType<AlarmHw>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new(PanicResources::new());

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

kernel::stack_size! {0x1000}

type SchedulerInUse = components::sched::cooperative::CooperativeComponentType;

// Static allocations used for page tables
//
// These are placed into custom sections so they can be properly aligned and padded in layout.ld
#[no_mangle]
#[cfg_attr(not(target_os = "macos"), link_section = ".pde")]
pub static mut PAGE_DIR: PD = [PDEntry(0); 1024];
#[no_mangle]
#[cfg_attr(not(target_os = "macos"), link_section = ".pte")]
pub static mut PAGE_TABLE: PT = [PTEntry(0); 1024];

/// Initializes a Virtio transport driver for the given PCI device.
///
/// Disables MSI/MSI-X interrupts for the device since the x86_q35 chip uses the 8259 PIC for
/// interrupt management.
///
/// On success, returns a tuple containing the interrupt line number assigned to this device as well
/// as a fully initialized Virtio PCI transport driver.
///
/// Returns `None` if the device does not report an assigned interrupt line, or if the transport
/// driver fails to initialize for some other reason. Either of these could be an indication that
/// `dev` is not a valid Virtio device.
fn init_virtio_dev(
    dev: pci_x86::Device,
    dev_type: VirtIODeviceType,
) -> Option<(u8, VirtIOPCIDevice)> {
    use pci_x86::cap::Cap;

    let int_line = dev.int_line()?;

    for cap in dev.capabilities() {
        match cap {
            Cap::Msi(cap) => {
                cap.disable();
            }
            Cap::Msix(cap) => {
                cap.disable();
            }
            _ => {}
        }
    }

    let dev = VirtIOPCIDevice::from_pci_device(dev, dev_type)?;

    Some((int_line, dev))
}

/// Provides interrupt servicing logic for Virtio devices which may or may not be present at
/// runtime.
struct VirtioDevices {
    rng: OptionalCell<(u8, &'static VirtIOPCIDevice)>,
}

impl InterruptService for VirtioDevices {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        let mut handled = false;

        self.rng.map(|(int_line, dev)| {
            if interrupt == (int_line as u32) {
                dev.handle_interrupt();
                handled = true;
            }
        });

        handled
    }
}

pub struct QemuI386Q35Platform {
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        components::process_console::Capability,
    >,
    console: &'static Console<'static>,
    lldb: &'static capsules_core::low_level_debug::LowLevelDebug<
        'static,
        capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
    >,
    ipc: IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static SchedulerInUse,
    scheduler_timer: &'static SchedulerTimerHw,
    rng: Option<&'static RngDriver<'static, VirtIORng<'static, 'static, X86DmaFence>>>,
}

impl SyscallDriverLookup for QemuI386Q35Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            console::DRIVER_NUM => f(Some(self.console)),
            alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            capsules_core::rng::DRIVER_NUM => {
                if let Some(rng) = self.rng {
                    f(Some(rng))
                } else {
                    f(None)
                }
            }
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl<C: kernel::platform::chip::Chip> KernelResources<C> for QemuI386Q35Platform {
    type SyscallDriverLookup = Self;
    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }

    type SyscallFilter = ();
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }

    type ProcessFault = ();
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }

    type Scheduler = SchedulerInUse;
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }

    type SchedulerTimer = SchedulerTimerHw;
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.scheduler_timer
    }

    type WatchDog = ();
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    type ContextSwitchCallback = ();
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}
// `allow(unsupported_calling_conventions)`: cdecl is not valid when testing
// this code on an x86_64 machine. This avoids a warning until a more permanent
// fix is decided. See: https://github.com/tock/tock/pull/4662
#[allow(unsupported_calling_conventions)]
#[no_mangle]
unsafe extern "cdecl" fn main() {
    // ---------- BASIC INITIALIZATION -----------

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Basic setup of the i486 platform
    // Allocate statics for default peripherals and build them via the chip helper
    let default_peripherals = unsafe {
        static_init!(
            PcDefaultPeripherals,
            PcDefaultPeripherals::new(
                (
                    (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
                    (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
                    (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
                    (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
                    kernel::static_buf!(x86_q35::vga_uart_driver::VgaText<'static>),
                ),
                &mut *ptr::addr_of_mut!(PAGE_DIR),
            )
        )
    };
    default_peripherals.setup_circular_deps();
    let virtio_devs = static_init!(
        VirtioDevices,
        VirtioDevices {
            rng: OptionalCell::empty(),
        }
    );
    let chip: &'static Pc<PcDefaultPeripherals, VirtioDevices> = unsafe {
        static_init!(
            Pc<PcDefaultPeripherals, VirtioDevices>,
            Pc::new(
                &*default_peripherals,
                &mut *ptr::addr_of_mut!(PAGE_DIR),
                &mut *ptr::addr_of_mut!(PAGE_TABLE),
                virtio_devs,
            ),
        )
    };
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Acquire required capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // We use the default x86 implementation of `DmaFence`:
    let dma_fence = X86DmaFence::new();

    // ---------- QEMU-SYSTEM-I386 "Q35" MACHINE PERIPHERALS ----------

    // Create a shared UART channel for the console and for kernel
    // debug over the provided 8250-compatible UART.
    let uart_mux = components::console::UartMuxComponent::new(chip.com1, 115_200)
        .finalize(components::uart_mux_component_static!());

    // Alternative for VGA
    let vga_uart_mux = components::console::UartMuxComponent::new(chip.vga, 115_200)
        .finalize(components::uart_mux_component_static!());

    // Debug output: default to the VGA mux is
    // active.  If you prefer to keep debug on the serial port even with VGA
    // enabled, comment the line below and uncomment the next one.

    // Debug output uses VGA when available, otherwise COM1
    let debug_uart_device = vga_uart_mux;

    // let debug_uart_device  = com1_uart_mux;

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        MuxAlarm::new(chip.pit),
    );
    hil::time::Alarm::set_alarm_client(chip.pit, mux_alarm);

    // Virtual alarm and driver for userspace
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        >,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    // ---------- VIRTIO PERIPHERAL DISCOVERY ----------
    //
    // On x86, PCI is used to discover and communicate with Virtio devices.
    //
    // Enumerate the PCI bus to find supported Virtio devices. If there are two instances of a
    // supported peripheral, we use the first one we encounter.
    let mut virtio_rng_dev = None;
    for dev in pci_x86::iter() {
        use virtio::devices::VirtIODeviceType;
        use virtio_pci_x86::{DEVICE_ID_BASE, VENDOR_ID};

        // Only consider Virtio devices
        if dev.vendor_id() != VENDOR_ID {
            continue;
        }
        let dev_id = dev.device_id();
        if dev_id < DEVICE_ID_BASE {
            continue;
        }

        // Decode device type
        let dev_id = (dev_id - DEVICE_ID_BASE) as u32;
        let Some(dev_type) = VirtIODeviceType::from_device_id(dev_id) else {
            continue;
        };

        if dev_type == VirtIODeviceType::EntropySource {
            // Only consider first entropy source found
            if virtio_rng_dev.is_some() {
                continue;
            }

            virtio_rng_dev = Some(dev);
        }
    }

    // If there is a VirtIO EntropySource present, use the appropriate VirtIORng
    // driver and expose it to userspace though the RngDriver
    let virtio_rng: Option<&'static VirtIORng<X86DmaFence>> = if let Some(rng_dev) = virtio_rng_dev
    {
        use virtio::queues::split_queue::{
            SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
        };
        use virtio::queues::Virtqueue;
        use virtio::transports::VirtIOTransport;

        // Initialize PCI transport driver
        let (int_line, transport) = init_virtio_dev(rng_dev, VirtIODeviceType::EntropySource)
            .expect("virtio pci init failed");
        let transport = static_init!(VirtIOPCIDevice, transport);

        // EntropySource requires a single Virtqueue for retrieved entropy
        let descriptors = static_init!(VirtqueueDescriptors<1>, VirtqueueDescriptors::default(),);
        let available_ring =
            static_init!(VirtqueueAvailableRing<1>, VirtqueueAvailableRing::default(),);
        let used_ring = static_init!(VirtqueueUsedRing<1>, VirtqueueUsedRing::default(),);
        let queue = static_init!(
            SplitVirtqueue<1, X86DmaFence>,
            SplitVirtqueue::new(descriptors, available_ring, used_ring, dma_fence),
        );
        queue.set_transport(transport);

        // VirtIO EntropySource device driver instantiation
        let rng = static_init!(VirtIORng<X86DmaFence>, VirtIORng::new(queue));
        DeferredCallClient::register(rng);
        queue.set_client(rng);

        // Register the queues and driver with the transport, so interrupts
        // are routed properly
        let queues = static_init!([&'static dyn Virtqueue; 1], [queue; 1]);
        transport.initialize(rng, queues).unwrap();

        // Provide an internal randomness buffer
        let rng_buffer = static_init!([u8; 64], [0; 64]);
        rng.provide_buffer(rng_buffer)
            .expect("rng: providing initial buffer failed");

        // Device is successfully initialized, register it with the VirtioDevices struct so that
        // interrupts are routed properly
        virtio_devs.rng.set((int_line, transport));

        Some(rng)
    } else {
        None
    };

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    // PIT interrupts need to be started manually
    chip.pit.start();

    // Enable interrupts after all drivers are initialized
    irq::enable();

    // ---------- FINAL SYSTEM INITIALIZATION ----------

    // Create the process printer used in panic prints, etc.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    // ProcessConsole stays on COM1 because we have no keyboard input yet.
    // As soon as keyboard support will be added, the process console
    // may be used with the VGA and keyboard.
    //
    // let console_uart_device = vga_uart_mux;

    // For now the ProcessConsole (interactive shell) is wired to COM1 so the user can
    // type commands over the serial port.  Once keyboard input is implemented
    // we can switch `console_uart_device` to `vga_uart_mux`.
    let console_uart_device = uart_mux;

    // Initialize the kernel's process console.
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        console_uart_device,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(
        Pit<'static, RELOAD_1KHZ>
    ));

    // Setup the console.
    let console = ConsoleComponent::new(board_kernel, console::DRIVER_NUM, console_uart_device)
        .finalize(components::console_component_static!());

    // Create the debugger object that handles calls to `debug!()`.
    DebugWriterComponent::new::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
        debug_uart_device,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        capsules_core::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::low_level_debug_component_static!());

    // ---------- RNG ----------

    // Userspace RNG driver over the VirtIO EntropySource
    let rng_driver = virtio_rng.map(|rng| {
        components::rng::RngRandomComponent::new(board_kernel, capsules_core::rng::DRIVER_NUM, rng)
            .finalize(components::rng_random_component_static!(
                VirtIORng<X86DmaFence>
            ))
    });

    let scheduler = components::sched::cooperative::CooperativeComponent::new(processes)
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let scheduler_timer =
        components::virtual_scheduler_timer::VirtualSchedulerTimerComponent::new(mux_alarm)
            .finalize(components::virtual_scheduler_timer_component_static!(
                AlarmHw
            ));

    let platform = QemuI386Q35Platform {
        pconsole,
        console,
        alarm,
        lldb,
        scheduler,
        scheduler_timer,
        rng: rng_driver,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
    };

    // Start the process console:
    let _ = platform.pconsole.start();

    debug!("QEMU i486 \"Q35\" machine, initialization complete.");
    debug!("Entering main loop.");

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

    // ---------- PROCESS LOADING, SCHEDULER LOOP ----------

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            ptr::addr_of!(_sapps),
            ptr::addr_of!(_eapps) as usize - ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            ptr::addr_of_mut!(_sappmem),
            ptr::addr_of!(_eappmem) as usize - ptr::addr_of!(_sappmem) as usize,
        ),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_cap);
}
