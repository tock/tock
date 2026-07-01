// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![no_main]

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, debug, static_init};
use qemu_rv32_virt_chip::chip::{QemuRv32VirtChip, QemuRv32VirtDefaultPeripherals};
use rv32i::csr;
use rv32i::dma_fence::RiscvCoherentDmaFence;

pub mod io;

pub const NUM_PROCS: usize = 4;

pub type ChipHw = QemuRv32VirtChip<'static, QemuRv32VirtDefaultPeripherals<'static>>;

/// Concrete process type used on this board (matches what `load_processes` creates).
pub type ProcessHw = kernel::process::ProcessStandard<
    'static,
    ChipHw,
    kernel::process::ProcessStandardDebugFull,
>;

use qemu_rv32_virt_chip::chip::{SyncEntry, CLINT_MSIP1, LOCKSTEP_CHAN};
use qemu_rv32_virt_chip::lockstep::{DriverUpcallRules, UpcallMode, UpcallRule};

// ---------------------------------------------------------------------------
// Layer-2 upcall-verifier registry (compare-mode for console upcalls)
// ---------------------------------------------------------------------------

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

type ProcessPrinter = capsules_system::process_printer::ProcessPrinterText;

type RngDriver = components::rng::RngRandomComponentType<
    qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng<'static, 'static, RiscvCoherentDmaFence>,
>;
pub type ScreenHw = qemu_rv32_virt_chip::virtio::devices::virtio_gpu::VirtIOGPU<
    'static,
    'static,
    RiscvCoherentDmaFence,
>;

type AlarmHw = qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>;
type SchedulerTimerHw =
    components::virtual_scheduler_timer::VirtualSchedulerTimerComponentType<AlarmHw>;
type SchedulerInUse = components::sched::cooperative::CooperativeComponentType;

/// Resources for when a board panics used by io.rs.
///
/// `SingleThreadValue::bind_to_thread()` succeeds exactly once, ever, for a
/// given instance -- whichever thread (here: hart) calls it first
/// permanently owns it. Hart 0 and hart 1 each need their own panic
/// resources (so a panic on either hart gets the full CPU-state/process
/// dump, not just the bare message), so this can't be a single shared
/// instance: hart 1's bind would just fail once hart 0's `start()` already
/// bound this one. `io.rs`'s panic handler picks between this and
/// `PANIC_RESOURCES_H1` based on `mhartid`.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinter>> =
    SingleThreadValue::new();

/// Hart 1's own panic resources -- see `PANIC_RESOURCES` for why this can't
/// be the same instance.
static PANIC_RESOURCES_H1: SingleThreadValue<PanicResources<ChipHw, ProcessPrinter>> =
    SingleThreadValue::new();

kernel::stack_size! {0x8000}

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
pub struct QemuRv32VirtPlatform {
    pub pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>,
        >,
        components::process_console::Capability,
    >,
    console: &'static capsules_core::console::Console<'static>,
    lockstep_console: &'static qemu_rv32_virt_chip::lockstep::LockstepDriver<
        'static,
        capsules_core::console::Console<'static>,
    >,
    lldb: &'static capsules_core::low_level_debug::LowLevelDebug<
        'static,
        capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >,
    pub ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static SchedulerInUse,
    scheduler_timer: &'static SchedulerTimerHw,
    rng: Option<&'static RngDriver>,
    virtio_ethernet_tap: Option<
        &'static capsules_extra::ethernet_tap::EthernetTapDriver<
            'static,
            qemu_rv32_virt_chip::virtio::devices::virtio_net::VirtIONet<
                'static,
                RiscvCoherentDmaFence,
            >,
        >,
    >,
    pub virtio_gpu_screen: Option<
        &'static capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<
            'static,
            ScreenHw,
        >,
    >,
    pub virtio_input_keyboard: Option<
        &'static qemu_rv32_virt_chip::virtio::devices::virtio_input::VirtIOInput<
            'static,
            RiscvCoherentDmaFence,
        >,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for QemuRv32VirtPlatform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.lockstep_console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            capsules_core::rng::DRIVER_NUM => {
                if let Some(rng_driver) = self.rng {
                    f(Some(rng_driver))
                } else {
                    f(None)
                }
            }
            capsules_extra::ethernet_tap::DRIVER_NUM => {
                if let Some(ethernet_tap_driver) = self.virtio_ethernet_tap {
                    f(Some(ethernet_tap_driver))
                } else {
                    f(None)
                }
            }

            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl
    KernelResources<
        qemu_rv32_virt_chip::chip::QemuRv32VirtChip<
            'static,
            QemuRv32VirtDefaultPeripherals<'static>,
        >,
    > for QemuRv32VirtPlatform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = SchedulerTimerHw;
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
        self.scheduler_timer
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
// We allocate a frame-buffer for converting Mono_8BitPage pixel data
// into an ARGB_8888 format. This can consume a large amount of stack
// space, as we allocate this buffer with `static_init!()`:
#[allow(clippy::large_stack_frames, clippy::large_stack_arrays)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    QemuRv32VirtPlatform,
    &'static qemu_rv32_virt_chip::chip::QemuRv32VirtChip<
        'static,
        QemuRv32VirtDefaultPeripherals<'static>,
    >,
    &'static kernel::process::ProcessArray<NUM_PROCS>,
) {
    // These symbols are defined in the linker script.
    extern "C" {
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
        /// End of hart 1 RAM region — used to extend hart 0's ePMP RAMRegion
        /// to cover both harts so that finish_lockstep_setup can write
        /// replica PCBs into the hart 1 RAM region.
        static _esram_h1: u8;
    }
    // ---------- BASIC INITIALIZATION -----------

    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    // Basic setup of the RISC-V IMAC platform
    rv32i::configure_trap_handler();

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Set up memory protection immediately after setting the trap handler, to
    // ensure that much of the board initialization routine runs with ePMP
    // protection.
    let epmp = rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP::new(
        rv32i::pmp::kernel_protection_mml_epmp::FlashRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_sflash),
                core::ptr::addr_of!(_eflash),
            )
            .unwrap(),
        ),
        // Cover both harts' RAM (0x80800000..0x81000000, 8 MB NAPOT) so that
        // finish_lockstep_setup() can write replica PCBs into hart 1's region.
        rv32i::pmp::kernel_protection_mml_epmp::RAMRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_ssram),
                core::ptr::addr_of!(_esram_h1),
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::MMIORegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_size(
                core::ptr::null::<u8>(), // start
                0x20000000,              // size
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::KernelTextRegion(
            rv32i::pmp::TORRegionSpec::from_start_end(
                core::ptr::addr_of!(_stext),
                core::ptr::addr_of!(_etext),
            )
            .unwrap(),
        ),
    )
    .unwrap();

    // Acquire required capabilities
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    // Create a board kernel instance

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // Create a DmaFence instance. Under QEMU, DMA peripherals are
    // cache-coherent with the main CPU, and therefore we can use the
    // `RiscvCoherentDmaFence`:
    let dma_fence = RiscvCoherentDmaFence::new();

    // ---------- QEMU-SYSTEM-RISCV32 "virt" MACHINE PERIPHERALS ----------

    let peripherals = static_init!(
        QemuRv32VirtDefaultPeripherals,
        QemuRv32VirtDefaultPeripherals::new(),
    );
    kernel::deferred_call::DeferredCallClient::register(&peripherals.uart0);

    // Create a shared UART channel for the console and for kernel
    // debug over the provided memory-mapped 16550-compatible UART.
    // LockstepUart sits between uart0 and MuxUart: its hooks fire the
    // cross-hart signals that previously lived inside Uart16550 itself.
    let lockstep_uart: &'static qemu_rv32_virt_chip::lockstep::LockstepUart<
        'static,
        qemu_rv32_virt_chip::uart::Uart16550<'static>,
        qemu_rv32_virt_chip::lockstep::QemuUartHooks,
    > = {
        use qemu_rv32_virt_chip::lockstep::{LockstepUart, QemuUartHooks};
        use qemu_rv32_virt_chip::uart::Uart16550;
        let uart_hooks: &'static QemuUartHooks =
            static_init!(QemuUartHooks, QemuUartHooks::new());
        let w: &'static LockstepUart<'static, Uart16550<'static>, QemuUartHooks> = static_init!(
            LockstepUart<'static, Uart16550<'static>, QemuUartHooks>,
            LockstepUart::new(&peripherals.uart0, uart_hooks),
        );
        // uart0 upward callbacks (TX-done, RX-done) go through the wrapper.
        hil::uart::Transmit::set_transmit_client(&peripherals.uart0, w);
        hil::uart::Receive::set_receive_client(&peripherals.uart0, w);
        w
    };
    let uart_mux = components::console::UartMuxComponent::new(lockstep_uart, 115200)
        .finalize(components::uart_mux_component_static!());

    // Use the RISC-V machine timer timesource
    let hardware_timer = static_init!(
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint,
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint::new(&qemu_rv32_virt_chip::clint::CLINT_BASE, 0)
    );

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        MuxAlarm::new(hardware_timer)
    );
    hil::time::Alarm::set_alarm_client(hardware_timer, mux_alarm);

    // Virtual alarm and driver for userspace
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        >,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    // ---------- VIRTIO PERIPHERAL DISCOVERY ----------
    //
    // This board has 8 virtio-mmio (v2 personality required!) devices
    //
    // Collect supported VirtIO peripheral indicies and initialize them if they
    // are found. If there are two instances of a supported peripheral, the one
    // on a higher-indexed VirtIO transport is used.
    let (mut virtio_gpu_idx, mut virtio_net_idx, mut virtio_rng_idx, mut virtio_input_idx) =
        (None, None, None, None);
    for (i, virtio_device) in peripherals.virtio_mmio.iter().enumerate() {
        use qemu_rv32_virt_chip::virtio::devices::VirtIODeviceType;
        match virtio_device.query() {
            Ok(VirtIODeviceType::GPUDevice) => {
                virtio_gpu_idx = Some(i);
            }
            Ok(VirtIODeviceType::NetworkCard) => {
                virtio_net_idx = Some(i);
            }
            Ok(VirtIODeviceType::EntropySource) => {
                virtio_rng_idx = Some(i);
            }
            Ok(VirtIODeviceType::InputDevice) => {
                virtio_input_idx = Some(i);
            }
            _ => (),
        }
    }

    // If there is a VirtIO EntropySource present, use the appropriate VirtIORng
    // driver and expose it to userspace though the RngDriver
    let virtio_gpu_screen: Option<
        &'static capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<
            'static,
            ScreenHw,
        >,
    > = if let Some(gpu_idx) = virtio_gpu_idx {
        use qemu_rv32_virt_chip::virtio::devices::virtio_gpu::{
            VirtIOGPU, MAX_REQ_SIZE, MAX_RESP_SIZE, PIXEL_STRIDE,
        };
        use qemu_rv32_virt_chip::virtio::queues::split_queue::{
            SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
        };
        use qemu_rv32_virt_chip::virtio::queues::Virtqueue;
        use qemu_rv32_virt_chip::virtio::transports::VirtIOTransport;

        // Video output dimensions:

        const VIDEO_WIDTH: usize = 128;
        const VIDEO_HEIGHT: usize = 128;

        // VirtIO GPU requires a single Virtqueue for sending commands. It can
        // optionally use a second VirtQueue for cursor commands, which we don't
        // use (as we don't have the concept of a cursor).
        //
        // The VirtIO GPU control queue must be able to hold two descriptors:
        // one for the request, and another for the response.
        let descriptors = static_init!(VirtqueueDescriptors<2>, VirtqueueDescriptors::default(),);
        let available_ring =
            static_init!(VirtqueueAvailableRing<2>, VirtqueueAvailableRing::default(),);
        let used_ring = static_init!(VirtqueueUsedRing<2>, VirtqueueUsedRing::default(),);
        let control_queue = static_init!(
            SplitVirtqueue<2, RiscvCoherentDmaFence>,
            SplitVirtqueue::new(descriptors, available_ring, used_ring, dma_fence),
        );
        control_queue.set_transport(&peripherals.virtio_mmio[gpu_idx]);

        // Create required buffers:
        let req_buffer = static_init!([u8; MAX_REQ_SIZE], [0; MAX_REQ_SIZE]);
        let resp_buffer = static_init!([u8; MAX_RESP_SIZE], [0; MAX_RESP_SIZE]);
        // let frame_buffer = static_init!(
        //     [u8; VIDEO_WIDTH * VIDEO_HEIGHT * PIXEL_STRIDE],
        //     [0; VIDEO_WIDTH * VIDEO_HEIGHT * PIXEL_STRIDE]
        // );

        // VirtIO GPU device driver instantiation
        let gpu = static_init!(
            VirtIOGPU<RiscvCoherentDmaFence>,
            VirtIOGPU::new(
                control_queue,
                req_buffer,
                resp_buffer,
                VIDEO_WIDTH,
                VIDEO_HEIGHT,
            )
            .unwrap()
        );
        kernel::deferred_call::DeferredCallClient::register(gpu);
        control_queue.set_client(gpu);

        // Register the queues and driver with the transport, so interrupts
        // are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 1], [control_queue; 1]);
        if peripherals.virtio_mmio[gpu_idx]
            .initialize(gpu, mmio_queues)
            .is_err()
        {
            None
        } else {
            // Convert the `ARGB_8888` pixel mode offered by this device into a
            // pixel mode that the rest of the kernel and userspace understands,
            // namely the cursed `Mono_8BitPage` mode:
            let screen_argb_8888_to_mono_8bit_page =
                components::screen_adapters::ScreenAdapterARGB8888ToMono8BitPageComponent::new(gpu)
                    .finalize(
                        components::screen_adapter_argb8888_to_mono8bitpage_component_static!(
                            ScreenHw,
                            VIDEO_WIDTH,
                            VIDEO_HEIGHT,
                            PIXEL_STRIDE
                        ),
                    );

            gpu.initialize().unwrap();

            Some(screen_argb_8888_to_mono_8bit_page)
        }
    } else {
        // No VirtIO GPU device discovered
        None
    };

    // If there is a VirtIO EntropySource present, use the appropriate VirtIORng
    // driver and expose it to userspace though the RngDriver
    let virtio_rng: Option<
        &'static qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng<RiscvCoherentDmaFence>,
    > = if let Some(rng_idx) = virtio_rng_idx {
        use qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng;
        use qemu_rv32_virt_chip::virtio::queues::split_queue::{
            SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
        };
        use qemu_rv32_virt_chip::virtio::queues::Virtqueue;
        use qemu_rv32_virt_chip::virtio::transports::VirtIOTransport;

        // EntropySource requires a single Virtqueue for retrieved entropy
        let descriptors = static_init!(VirtqueueDescriptors<1>, VirtqueueDescriptors::default(),);
        let available_ring =
            static_init!(VirtqueueAvailableRing<1>, VirtqueueAvailableRing::default(),);
        let used_ring = static_init!(VirtqueueUsedRing<1>, VirtqueueUsedRing::default(),);
        let queue = static_init!(
            SplitVirtqueue<1, RiscvCoherentDmaFence>,
            SplitVirtqueue::new(descriptors, available_ring, used_ring, dma_fence),
        );
        queue.set_transport(&peripherals.virtio_mmio[rng_idx]);

        // VirtIO EntropySource device driver instantiation
        let rng = static_init!(VirtIORng<RiscvCoherentDmaFence>, VirtIORng::new(queue));
        kernel::deferred_call::DeferredCallClient::register(rng);
        queue.set_client(rng);

        // Register the queues and driver with the transport, so interrupts
        // are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 1], [queue; 1]);
        if peripherals.virtio_mmio[rng_idx]
            .initialize(rng, mmio_queues)
            .is_err()
        {
            None
        } else {
            // Provide an internal randomness buffer
            let rng_buffer = static_init!([u8; 64], [0; 64]);
            rng.provide_buffer(rng_buffer)
                .expect("rng: providing initial buffer failed");

            Some(rng)
        }
    } else {
        // No VirtIO EntropySource discovered
        None
    };

    // If there is a VirtIO NetworkCard present, use the appropriate VirtIONet
    // driver, and expose this device through the Ethernet Tap driver
    // (forwarding raw Ethernet frames from and to userspace).
    let virtio_ethernet_tap: Option<
        &'static capsules_extra::ethernet_tap::EthernetTapDriver<
            'static,
            qemu_rv32_virt_chip::virtio::devices::virtio_net::VirtIONet<
                'static,
                RiscvCoherentDmaFence,
            >,
        >,
    > = if let Some(net_idx) = virtio_net_idx {
        use capsules_extra::ethernet_tap::EthernetTapDriver;
        use kernel::hil::ethernet::EthernetAdapterDatapath;
        use qemu_rv32_virt_chip::virtio::devices::virtio_net::VirtIONet;
        use qemu_rv32_virt_chip::virtio::queues::split_queue::{
            SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
        };
        use qemu_rv32_virt_chip::virtio::queues::Virtqueue;
        use qemu_rv32_virt_chip::virtio::transports::VirtIOTransport;

        // A VirtIO NetworkCard requires 2 Virtqueues:
        // - a TX Virtqueue with buffers for outgoing packets
        // - a RX Virtqueue where incoming packet buffers are
        //   placed and filled by the device

        // TX Virtqueue
        let tx_descriptors =
            static_init!(VirtqueueDescriptors<2>, VirtqueueDescriptors::default(),);
        let tx_available_ring =
            static_init!(VirtqueueAvailableRing<2>, VirtqueueAvailableRing::default(),);
        let tx_used_ring = static_init!(VirtqueueUsedRing<2>, VirtqueueUsedRing::default(),);
        let tx_queue = static_init!(
            SplitVirtqueue<2, RiscvCoherentDmaFence>,
            SplitVirtqueue::new(tx_descriptors, tx_available_ring, tx_used_ring, dma_fence),
        );
        tx_queue.set_transport(&peripherals.virtio_mmio[net_idx]);

        // RX Virtqueue
        let rx_descriptors =
            static_init!(VirtqueueDescriptors<2>, VirtqueueDescriptors::default(),);
        let rx_available_ring =
            static_init!(VirtqueueAvailableRing<2>, VirtqueueAvailableRing::default(),);
        let rx_used_ring = static_init!(VirtqueueUsedRing<2>, VirtqueueUsedRing::default(),);
        let rx_queue = static_init!(
            SplitVirtqueue<2, RiscvCoherentDmaFence>,
            SplitVirtqueue::new(rx_descriptors, rx_available_ring, rx_used_ring, dma_fence),
        );
        rx_queue.set_transport(&peripherals.virtio_mmio[net_idx]);

        // Incoming and outgoing packets are prefixed by a 12-byte
        // VirtIO specific header
        let tx_header_buf = static_init!([u8; 12], [0; 12]);
        let rx_header_buf = static_init!([u8; 12], [0; 12]);

        // Currently, provide a single receive buffer to write
        // incoming packets into
        let rx_buffer = static_init!([u8; 1526], [0; 1526]);

        // Instantiate the VirtIONet (NetworkCard) driver and set the queues
        let virtio_net = static_init!(
            VirtIONet<'static, RiscvCoherentDmaFence>,
            VirtIONet::new(tx_queue, tx_header_buf, rx_queue, rx_header_buf, rx_buffer),
        );
        tx_queue.set_client(virtio_net);
        rx_queue.set_client(virtio_net);

        // Register the queues and driver with the transport, so
        // interrupts are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 2], [rx_queue, tx_queue]);
        if peripherals.virtio_mmio[net_idx]
            .initialize(virtio_net, mmio_queues)
            .is_err()
        {
            None
        } else {
            // Instantiate the userspace tap network driver over this device:
            let virtio_ethernet_tap_tx_buffer = static_init!(
                [u8; capsules_extra::ethernet_tap::MAX_MTU],
                [0; capsules_extra::ethernet_tap::MAX_MTU],
            );
            let virtio_ethernet_tap = static_init!(
                EthernetTapDriver<'static, VirtIONet<'static, RiscvCoherentDmaFence>>,
                EthernetTapDriver::new(
                    virtio_net,
                    board_kernel.create_grant(
                        capsules_extra::ethernet_tap::DRIVER_NUM,
                        &memory_allocation_cap
                    ),
                    virtio_ethernet_tap_tx_buffer,
                ),
            );
            virtio_net.set_client(virtio_ethernet_tap);

            // This enables reception on the underlying device:
            virtio_ethernet_tap.initialize();

            Some(
                virtio_ethernet_tap
                    as &'static EthernetTapDriver<
                        'static,
                        VirtIONet<'static, RiscvCoherentDmaFence>,
                    >,
            )
        }
    } else {
        // No VirtIO NetworkCard discovered
        None
    };

    let virtio_input_keyboard: Option<
        &'static qemu_rv32_virt_chip::virtio::devices::virtio_input::VirtIOInput<
            RiscvCoherentDmaFence,
        >,
    > = if let Some(input_idx) = virtio_input_idx {
        use qemu_rv32_virt_chip::virtio::devices::virtio_input::VirtIOInput;
        use qemu_rv32_virt_chip::virtio::queues::split_queue::{
            SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
        };
        use qemu_rv32_virt_chip::virtio::queues::Virtqueue;
        use qemu_rv32_virt_chip::virtio::transports::VirtIOTransport;

        // Event Virtqueue
        let event_descriptors =
            static_init!(VirtqueueDescriptors<3>, VirtqueueDescriptors::default(),);
        let event_available_ring =
            static_init!(VirtqueueAvailableRing<3>, VirtqueueAvailableRing::default(),);
        let event_used_ring = static_init!(VirtqueueUsedRing<3>, VirtqueueUsedRing::default(),);
        let event_queue = static_init!(
            SplitVirtqueue<3, RiscvCoherentDmaFence>,
            SplitVirtqueue::new(event_descriptors, event_available_ring, event_used_ring, dma_fence),
        );
        event_queue.set_transport(&peripherals.virtio_mmio[input_idx]);

        // Status Virtqueue
        let status_descriptors =
            static_init!(VirtqueueDescriptors<1>, VirtqueueDescriptors::default(),);
        let status_available_ring =
            static_init!(VirtqueueAvailableRing<1>, VirtqueueAvailableRing::default(),);
        let status_used_ring = static_init!(VirtqueueUsedRing<1>, VirtqueueUsedRing::default(),);
        let status_queue = static_init!(
            SplitVirtqueue<1, RiscvCoherentDmaFence>,
            SplitVirtqueue::new(status_descriptors, status_available_ring, status_used_ring, dma_fence),
        );
        status_queue.set_transport(&peripherals.virtio_mmio[input_idx]);

        // Buffers to store events from the keyboard.
        let event_buf1 = static_init!([u8; 8], [0; 8]);
        let event_buf2 = static_init!([u8; 8], [0; 8]);
        let event_buf3 = static_init!([u8; 8], [0; 8]);
        let status_buf = static_init!([u8; 128], [0; 128]);

        // Instantiate the input driver
        let virtio_input = static_init!(
            VirtIOInput<'static, RiscvCoherentDmaFence>,
            VirtIOInput::new(event_queue, status_queue, status_buf),
        );
        event_queue.set_client(virtio_input);
        status_queue.set_client(virtio_input);

        // Register the queues and driver with the transport, so
        // interrupts are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 2], [event_queue, status_queue]);
        if peripherals.virtio_mmio[input_idx]
            .initialize(virtio_input, mmio_queues)
            .is_err()
        {
            None
        } else {
            virtio_input.provide_buffers(event_buf1, event_buf2, event_buf3);
            Some(virtio_input)
        }
    } else {
        // No Input device
        None
    };

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    let chip = static_init!(
        QemuRv32VirtChip<QemuRv32VirtDefaultPeripherals>,
        QemuRv32VirtChip::new(peripherals, hardware_timer, epmp),
    );
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Need to enable all interrupts for Tock Kernel
    chip.enable_plic_interrupts();

    // enable interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::mext::SET + csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // ---------- FINAL SYSTEM INITIALIZATION ----------

    // Create the process printer used in panic prints, etc.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    // Initialize the kernel's process console.
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint
    ));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        capsules_core::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::low_level_debug_component_static!());

    // ---------- Layer-2 console interceptor ----------

    let lockstep_console = static_init!(
        qemu_rv32_virt_chip::lockstep::LockstepDriver<
            'static,
            capsules_core::console::Console<'static>,
        >,
        qemu_rv32_virt_chip::lockstep::LockstepDriver::new(
            console,
            capsules_core::console::DRIVER_NUM,
        ),
    );

    // ---------- RNG ----------

    // Userspace RNG driver over the VirtIO EntropySource (hart 0 only).
    // LockstepRng interposer removed; re-add when hart-1 replay is ready.
    let rng_driver = virtio_rng.map(|virtio_rng| {
        use qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng;
        components::rng::RngRandomComponent::new(
            board_kernel,
            capsules_core::rng::DRIVER_NUM,
            virtio_rng,
        )
        .finalize(components::rng_random_component_static!(VirtIORng<'static, 'static, RiscvCoherentDmaFence>))
    });

    // ---------- SCHEDULER ----------

    let scheduler = components::sched::cooperative::CooperativeComponent::new(processes)
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let scheduler_timer =
        components::virtual_scheduler_timer::VirtualSchedulerTimerComponent::new(mux_alarm)
            .finalize(components::virtual_scheduler_timer_component_static!(
                AlarmHw
            ));

    let platform = QemuRv32VirtPlatform {
        pconsole,
        console,
        lockstep_console,
        alarm,
        lldb,
        scheduler,
        scheduler_timer,
        rng: rng_driver,
        virtio_ethernet_tap,
        virtio_gpu_screen,
        virtio_input_keyboard,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
    };

    debug!("QEMU RISC-V 32-bit \"virt\" machine, initialization complete.");

    // This board dynamically discovers VirtIO devices like a randomness source
    // or a network card. Print a message indicating whether or not each such
    // device and corresponding userspace driver is present:
    if virtio_gpu_screen.is_some() {
        debug!("- Found VirtIO GPUDevice, enabling video output");
    } else {
        debug!("- VirtIO GPUDevice not found, disabling video output");
    }
    if virtio_rng.is_some() {
        debug!("- Found VirtIO EntropySource device, enabling RngDriver");
    } else {
        debug!("- VirtIO EntropySource device not found, disabling RngDriver");
    }
    if virtio_ethernet_tap.is_some() {
        debug!("- Found VirtIO NetworkCard device, enabling EthernetTapDriver");
    } else {
        debug!("- VirtIO NetworkCard device not found, disabling EthernetTapDriver");
    }
    if virtio_input_keyboard.is_some() {
        debug!("- Found VirtIO Input device, enabling Input");
    } else {
        debug!("- VirtIO Input device not found, disabling Input");
    }

    // Register the Layer-2 upcall verifier before entering the kernel loop.
    let upcall_verifier = static_init!(
        qemu_rv32_virt_chip::lockstep::QemuUpcallVerifier,
        qemu_rv32_virt_chip::lockstep::QemuUpcallVerifier::new(&UPCALL_REGISTRY),
    );
    board_kernel.register_upcall_verifier(upcall_verifier);

    (board_kernel, platform, chip, processes)
}

// ---------------------------------------------------------------------------
// Lockstep replica setup — called from main() after load_processes()
// ---------------------------------------------------------------------------

/// Signal hart 1 it's safe to proceed, and synchronize before either hart
/// enters its kernel loop.
///
/// Must be called after `load_processes()` so hart 0's own process state is
/// fully set up before lockstep iteration begins. Hart 1 does not receive a
/// copy of hart 0's processes here: since libtock-c has no PIC support for
/// RISC-V, an app's compiled memory image embeds absolute addresses for
/// whatever RAM_START it was linked against, so it can't simply be
/// byte-copied to hart 1's different base address (see layout.ld). Instead
/// hart 1 loads its own, independently-built-and-linked copy of each app via
/// its own `load_processes()` call in `start_secondary()` -- this function
/// only provides the synchronization handshake.
#[inline(never)]
pub unsafe fn finish_lockstep_setup() {
    // Signal hart 1 it's safe to touch shared bss (including LOCKSTEP_CHAN
    // itself) now that hart 0 has zeroed it. CLINT MSIP[1] = CLINT_BASE + 4.
    core::ptr::write_volatile(CLINT_MSIP1, 1);

    // Synchronize with hart 1 through LOCKSTEP_CHAN itself rather than a
    // separate atomic + volatile flag. push()'s internal Release fence
    // (before advancing `tail`) pairs with pop()'s Acquire load of `tail`,
    // so hart 1 actually receiving this message -- not merely observing
    // some other "go" signal -- is what proves hart 0's shared bss writes up
    // to this point are visible to it.
    while !LOCKSTEP_CHAN.a_send(SyncEntry::Sync { fingerprint: 0 }) {
        core::hint::spin_loop();
    }
    let _ack = LOCKSTEP_CHAN.a_spin_recv();
    debug!("Lockstep: init sync complete");
}

// ---------------------------------------------------------------------------
// Hart 1 minimal platform — no peripherals connected
// ---------------------------------------------------------------------------

pub struct Hart1Platform {
    pub scheduler: &'static SchedulerInUse,
    pub scheduler_timer: &'static SchedulerTimerHw,
    pub console: &'static capsules_core::console::Console<'static>,
    pub alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >,
    lockstep_console: &'static qemu_rv32_virt_chip::lockstep::LockstepDriver<
        'static,
        capsules_core::console::Console<'static>,
    >,
}

impl kernel::platform::SyscallDriverLookup for Hart1Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.lockstep_console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

impl
    kernel::platform::KernelResources<
        qemu_rv32_virt_chip::chip::QemuRv32VirtChip<
            'static,
            QemuRv32VirtDefaultPeripherals<'static>,
        >,
    > for Hart1Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = SchedulerTimerHw;
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
        self.scheduler_timer
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Minimal initialization for hart 1: CPU-local state only, no peripherals.
///
/// Reads the hart-1 process array pointer written by `finish_lockstep_setup()`
/// (called on hart 0 before the MSIP signal) and constructs a minimal Tock
/// kernel loop.  The caller must spin-wait on MSIP[1] before invoking this
/// function (see hart 1's `main()`).
#[inline(never)]
pub unsafe fn start_secondary() -> (
    &'static kernel::Kernel,
    Hart1Platform,
    &'static qemu_rv32_virt_chip::chip::QemuRv32VirtChip<
        'static,
        QemuRv32VirtDefaultPeripherals<'static>,
    >,
) {
    extern "C" {
        static _stext: u8;
        static _etext: u8;
        static _sflash: u8;
        static _eflash_h1: u8;
        static _ssram: u8;
        static _esram_h1: u8;
        static _sapps_h1: u8;
        static _eapps_h1: u8;
        static mut _sappmem_h1: u8;
        static _eappmem_h1: u8;
    }

    let _ = PANIC_RESOURCES_H1
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    rv32i::configure_trap_handler();

    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Memory protection covering both harts' RAM (0x80800000..0x81000000, 8 MB NAPOT).
    // Hart 1 needs access to hart 0's .bss for large shared statics.
    let epmp = rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP::new(
        // Extends past hart 0's _eflash (0x80200000) to cover hart 1's own
        // prog_h1 app flash region too -- see _eflash_h1's definition in
        // layout.ld for why this needs NAPOT rounding up to 4 MB.
        rv32i::pmp::kernel_protection_mml_epmp::FlashRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_sflash),
                core::ptr::addr_of!(_eflash_h1),
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::RAMRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_ssram),    // 0x80800000 — covers both harts' RAM
                core::ptr::addr_of!(_esram_h1), // 0x81000000
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::MMIORegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_size(
                core::ptr::null::<u8>(),
                0x20000000,
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::KernelTextRegion(
            rv32i::pmp::TORRegionSpec::from_start_end(
                core::ptr::addr_of!(_stext),
                core::ptr::addr_of!(_etext),
            )
            .unwrap(),
        ),
    )
    .unwrap();

    // Synchronize with hart 0 through LOCKSTEP_CHAN. b_spin_recv()'s
    // internal Acquire load of `tail` pairs with finish_lockstep_setup()'s
    // push()-internal Release fence, so actually receiving this message --
    // not just being woken up -- is what guarantees hart 0's shared bss
    // writes up to that point are visible here. See the comment at the
    // matching a_send() call. Hart 1 no longer receives a process array
    // pointer this way: it loads its own, independently-built copy of each
    // app below (see layout.ld for why a byte-copy of hart 0's processes
    // can't work for non-PIC RISC-V apps).
    debug!("[H1] start_secondary: before b_spin_recv");
    let init_entry = LOCKSTEP_CHAN.b_spin_recv();
    while !LOCKSTEP_CHAN.b_send(init_entry) {
        core::hint::spin_loop();
    }
    debug!("[H1] start_secondary: init sync done");

    let processes = static_init!(
        kernel::process::ProcessArray<NUM_PROCS>,
        kernel::process::ProcessArray::new()
    );
    PANIC_RESOURCES_H1.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // Per-hart timer: each hart has its own mtimecmp register in the CLINT.
    let hardware_timer = static_init!(
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint,
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint::new(
            &qemu_rv32_virt_chip::clint::CLINT_BASE, 1
        )
    );
    let mux_alarm = static_init!(
        MuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        MuxAlarm::new(hardware_timer)
    );
    hil::time::Alarm::set_alarm_client(hardware_timer, mux_alarm);

    // QemuRv32VirtChip needs a peripherals struct even though hart 1 won't use them.
    let peripherals = static_init!(
        QemuRv32VirtDefaultPeripherals,
        QemuRv32VirtDefaultPeripherals::new(),
    );
    kernel::deferred_call::DeferredCallClient::register(&peripherals.uart0);

    let chip = static_init!(
        QemuRv32VirtChip<QemuRv32VirtDefaultPeripherals>,
        QemuRv32VirtChip::new(peripherals, hardware_timer, epmp),
    );
    PANIC_RESOURCES_H1.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Process printer used in panic prints, mirroring hart 0's own.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES_H1.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    // Userspace-facing alarm, independently replicated: Hart 1 has its own
    // CLINT mtimecmp (unlike the UART, no replay from Hart 0 is needed). A
    // replica process issuing the same set_alarm syscall as its Hart 0
    // counterpart will independently compute and arm the same deadline off
    // the same shared mtime, so both harts fire in lockstep.
    //
    // Must happen before load_processes() below: creating a grant after
    // the first process is created panics (Kernel::create_grant ->
    // "Grants finalized. Cannot create a new grant.", since
    // load_processes() is what finalizes the grant count).
    let memory_alloc_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        >,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_alloc_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    // Wire Hart 1's Console to the hardware-free replay stub.
    // Hart 0 owns the physical UART; Hart 1 receives data via MSIP replay.
    // A LockstepUart wrapper sits between HART1_UART_BUF and Console so that
    // the same hooks infrastructure is present on both harts (hooks are no-ops
    // on hart 1, but the wrapper structure mirrors the hart-0 chain).
    let memory_alloc_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let console = {
        use capsules_core::console::{Console, DEFAULT_BUF_SIZE};
        use qemu_rv32_virt_chip::lockstep::{LockstepUart, QemuUartHooks};
        use qemu_rv32_virt_chip::uart::{VirtualUartBuffer, HART1_UART_BUF};
        let tx_buf = static_init!([u8; DEFAULT_BUF_SIZE], [0; DEFAULT_BUF_SIZE]);
        let rx_buf = static_init!([u8; DEFAULT_BUF_SIZE], [0; DEFAULT_BUF_SIZE]);
        let uart_hooks: &'static QemuUartHooks =
            static_init!(QemuUartHooks, QemuUartHooks::new());
        let lockstep_uart: &'static LockstepUart<'static, VirtualUartBuffer, QemuUartHooks> =
            static_init!(
                LockstepUart<'static, VirtualUartBuffer, QemuUartHooks>,
                LockstepUart::new(&HART1_UART_BUF, uart_hooks),
            );
        hil::uart::Receive::set_receive_client(&HART1_UART_BUF, lockstep_uart);
        hil::uart::Transmit::set_transmit_client(&HART1_UART_BUF, lockstep_uart);
        let console: &'static Console<'static> = static_init!(
            Console<'static>,
            Console::new(
                lockstep_uart,
                tx_buf,
                rx_buf,
                board_kernel.create_grant(capsules_core::console::DRIVER_NUM, &memory_alloc_cap),
            )
        );
        hil::uart::Receive::set_receive_client(lockstep_uart, console);
        hil::uart::Transmit::set_transmit_client(lockstep_uart, console);
        console
    };

    // Layer-2 console interceptor for hart 1. Reads mhartid at construction
    // so LockstepDriver::command branches to the hart-1 compare path.
    let lockstep_console = static_init!(
        qemu_rv32_virt_chip::lockstep::LockstepDriver<
            'static,
            capsules_core::console::Console<'static>,
        >,
        qemu_rv32_virt_chip::lockstep::LockstepDriver::new(
            console,
            capsules_core::console::DRIVER_NUM,
        ),
    );

    // Load hart 1's own, independently-linked copy of each app from its own
    // flash region, the same way main() loads hart 0's. Built with hart 0's
    // chip already, so unlike the old create_replica() approach there's no
    // need to patch up the chip reference afterwards.
    const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
        capsules_system::process_policies::PanicFaultPolicy {};
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let load_result = kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps_h1),
            core::ptr::addr_of!(_eapps_h1) as usize - core::ptr::addr_of!(_sapps_h1) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem_h1),
            core::ptr::addr_of!(_eappmem_h1) as usize - core::ptr::addr_of!(_sappmem_h1) as usize,
        ),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    );

    if let Err(err) = load_result {
        debug!("Hart 1: error loading processes!");
        debug!("{:?}", err);
    }

    let scheduler = components::sched::cooperative::CooperativeComponent::new(processes)
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let scheduler_timer =
        components::virtual_scheduler_timer::VirtualSchedulerTimerComponent::new(mux_alarm)
            .finalize(components::virtual_scheduler_timer_component_static!(AlarmHw));

    // Disarm hart 1's mtimecmp before enabling interrupts.
    //
    // Both `QemuRv32VirtClint` instances share the same ClintRegisters struct
    // offset (0x4000), so hart 1's driver inadvertently writes to hart 0's
    // mtimecmp. Until the CLINT driver gains per-hart support we simply keep
    // hart 1's mtimecmp pinned to max and suppress the mtimer interrupt in MIE.
    const CLINT_MTIMECMP1_LO: *mut u32 = 0x0200_4008 as *mut u32;
    const CLINT_MTIMECMP1_HI: *mut u32 = 0x0200_400C as *mut u32;
    core::ptr::write_volatile(CLINT_MTIMECMP1_HI, 0xFFFF_FFFF);
    core::ptr::write_volatile(CLINT_MTIMECMP1_LO, 0xFFFF_FFFF);

    // Enable machine software and timer interrupts. The CLINT driver now uses
    // per-hart mtimecmp offsets so hart 1's timer no longer clobbers hart 0.
    csr::CSR.mie.modify(csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    let platform = Hart1Platform {
        scheduler,
        scheduler_timer,
        console,
        alarm,
        lockstep_console,
    };

    // Register the Layer-2 upcall verifier for hart 1 before entering the
    // kernel loop. The QemuUpcallVerifier reads mhartid at construction so
    // the hart-0 and hart-1 instances branch correctly inside on_upcall.
    let upcall_verifier = static_init!(
        qemu_rv32_virt_chip::lockstep::QemuUpcallVerifier,
        qemu_rv32_virt_chip::lockstep::QemuUpcallVerifier::new(&UPCALL_REGISTRY),
    );
    board_kernel.register_upcall_verifier(upcall_verifier);

    (board_kernel, platform, chip)
}
