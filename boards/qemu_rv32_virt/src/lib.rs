// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![no_main]

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::process::ProcessArray;
use kernel::scheduler::cooperative::CooperativeSched;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::{create_capability, debug, static_init};
use qemu_rv32_virt_chip::chip::{QemuRv32VirtChip, QemuRv32VirtDefaultPeripherals};
use rv32i::csr;

pub mod io;

pub const NUM_PROCS: usize = 4;

/// Static variables used by io.rs.
static mut PROCESSES: Option<&'static ProcessArray<NUM_PROCS>> = None;

// Reference to the chip for panic dumps.
static mut CHIP: Option<&'static QemuRv32VirtChip<QemuRv32VirtDefaultPeripherals>> = None;

// Reference to the process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

pub type ChipHw = QemuRv32VirtChip<'static, QemuRv32VirtDefaultPeripherals<'static>>;
type RngDriver = components::rng::RngRandomComponentType<
    qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng<'static, 'static>,
>;
pub type ScreenHw = qemu_rv32_virt_chip::virtio::devices::virtio_gpu::VirtIOGPU<'static, 'static>;

type AlarmHw = qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>;
type SchedulerTimerHw =
    components::virtual_scheduler_timer::VirtualSchedulerTimerComponentType<AlarmHw>;

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
    lldb: &'static capsules_core::low_level_debug::LowLevelDebug<
        'static,
        capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >,
    pub ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static CooperativeSched<'static>,
    scheduler_timer: &'static SchedulerTimerHw,
    rng: Option<&'static RngDriver>,
    virtio_ethernet_tap: Option<
        &'static capsules_extra::ethernet_tap::EthernetTapDriver<
            'static,
            qemu_rv32_virt_chip::virtio::devices::virtio_net::VirtIONet<'static>,
        >,
    >,
    pub virtio_gpu_screen: Option<
        &'static capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<
            'static,
            ScreenHw,
        >,
    >,
    pub virtio_input_keyboard:
        Option<&'static qemu_rv32_virt_chip::virtio::devices::virtio_input::VirtIOInput<'static>>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for QemuRv32VirtPlatform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
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
    type Scheduler = CooperativeSched<'static>;
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
    }

    // ---------- BASIC INITIALIZATION -----------

    // Basic setup of the RISC-V IMAC platform
    rv32i::configure_trap_handler();

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
        rv32i::pmp::kernel_protection_mml_epmp::RAMRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_ssram),
                core::ptr::addr_of!(_esram),
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
    PROCESSES = Some(processes);

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // ---------- QEMU-SYSTEM-RISCV32 "virt" MACHINE PERIPHERALS ----------

    let peripherals = static_init!(
        QemuRv32VirtDefaultPeripherals,
        QemuRv32VirtDefaultPeripherals::new(),
    );

    // Create a shared UART channel for the console and for kernel
    // debug over the provided memory-mapped 16550-compatible
    // UART.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    // Use the RISC-V machine timer timesource
    let hardware_timer = static_init!(
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint,
        qemu_rv32_virt_chip::chip::QemuRv32VirtClint::new(&qemu_rv32_virt_chip::clint::CLINT_BASE)
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
            SplitVirtqueue<2>,
            SplitVirtqueue::new(descriptors, available_ring, used_ring),
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
            VirtIOGPU,
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
        peripherals.virtio_mmio[gpu_idx]
            .initialize(gpu, mmio_queues)
            .unwrap();

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
    } else {
        // No VirtIO GPU device discovered
        None
    };

    // If there is a VirtIO EntropySource present, use the appropriate VirtIORng
    // driver and expose it to userspace though the RngDriver
    let virtio_rng: Option<&'static qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng> =
        if let Some(rng_idx) = virtio_rng_idx {
            use qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng;
            use qemu_rv32_virt_chip::virtio::queues::split_queue::{
                SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
            };
            use qemu_rv32_virt_chip::virtio::queues::Virtqueue;
            use qemu_rv32_virt_chip::virtio::transports::VirtIOTransport;

            // EntropySource requires a single Virtqueue for retrieved entropy
            let descriptors =
                static_init!(VirtqueueDescriptors<1>, VirtqueueDescriptors::default(),);
            let available_ring =
                static_init!(VirtqueueAvailableRing<1>, VirtqueueAvailableRing::default(),);
            let used_ring = static_init!(VirtqueueUsedRing<1>, VirtqueueUsedRing::default(),);
            let queue = static_init!(
                SplitVirtqueue<1>,
                SplitVirtqueue::new(descriptors, available_ring, used_ring),
            );
            queue.set_transport(&peripherals.virtio_mmio[rng_idx]);

            // VirtIO EntropySource device driver instantiation
            let rng = static_init!(VirtIORng, VirtIORng::new(queue));
            kernel::deferred_call::DeferredCallClient::register(rng);
            queue.set_client(rng);

            // Register the queues and driver with the transport, so interrupts
            // are routed properly
            let mmio_queues = static_init!([&'static dyn Virtqueue; 1], [queue; 1]);
            peripherals.virtio_mmio[rng_idx]
                .initialize(rng, mmio_queues)
                .unwrap();

            // Provide an internal randomness buffer
            let rng_buffer = static_init!([u8; 64], [0; 64]);
            rng.provide_buffer(rng_buffer)
                .expect("rng: providing initial buffer failed");

            Some(rng)
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
            qemu_rv32_virt_chip::virtio::devices::virtio_net::VirtIONet<'static>,
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
            SplitVirtqueue<2>,
            SplitVirtqueue::new(tx_descriptors, tx_available_ring, tx_used_ring),
        );
        tx_queue.set_transport(&peripherals.virtio_mmio[net_idx]);

        // RX Virtqueue
        let rx_descriptors =
            static_init!(VirtqueueDescriptors<2>, VirtqueueDescriptors::default(),);
        let rx_available_ring =
            static_init!(VirtqueueAvailableRing<2>, VirtqueueAvailableRing::default(),);
        let rx_used_ring = static_init!(VirtqueueUsedRing<2>, VirtqueueUsedRing::default(),);
        let rx_queue = static_init!(
            SplitVirtqueue<2>,
            SplitVirtqueue::new(rx_descriptors, rx_available_ring, rx_used_ring),
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
            VirtIONet<'static>,
            VirtIONet::new(tx_queue, tx_header_buf, rx_queue, rx_header_buf, rx_buffer),
        );
        tx_queue.set_client(virtio_net);
        rx_queue.set_client(virtio_net);

        // Register the queues and driver with the transport, so
        // interrupts are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 2], [rx_queue, tx_queue]);
        peripherals.virtio_mmio[net_idx]
            .initialize(virtio_net, mmio_queues)
            .unwrap();

        // Instantiate the userspace tap network driver over this device:
        let virtio_ethernet_tap_tx_buffer = static_init!(
            [u8; capsules_extra::ethernet_tap::MAX_MTU],
            [0; capsules_extra::ethernet_tap::MAX_MTU],
        );
        let virtio_ethernet_tap = static_init!(
            EthernetTapDriver<'static, VirtIONet<'static>>,
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

        Some(virtio_ethernet_tap as &'static EthernetTapDriver<'static, VirtIONet<'static>>)
    } else {
        // No VirtIO NetworkCard discovered
        None
    };

    let virtio_input_keyboard: Option<
        &'static qemu_rv32_virt_chip::virtio::devices::virtio_input::VirtIOInput,
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
            SplitVirtqueue<3>,
            SplitVirtqueue::new(event_descriptors, event_available_ring, event_used_ring),
        );
        event_queue.set_transport(&peripherals.virtio_mmio[input_idx]);

        // Status Virtqueue
        let status_descriptors =
            static_init!(VirtqueueDescriptors<1>, VirtqueueDescriptors::default(),);
        let status_available_ring =
            static_init!(VirtqueueAvailableRing<1>, VirtqueueAvailableRing::default(),);
        let status_used_ring = static_init!(VirtqueueUsedRing<1>, VirtqueueUsedRing::default(),);
        let status_queue = static_init!(
            SplitVirtqueue<1>,
            SplitVirtqueue::new(status_descriptors, status_available_ring, status_used_ring),
        );
        status_queue.set_transport(&peripherals.virtio_mmio[input_idx]);

        // Buffers to store events from the keyboard.
        let event_buf1 = static_init!([u8; 8], [0; 8]);
        let event_buf2 = static_init!([u8; 8], [0; 8]);
        let event_buf3 = static_init!([u8; 8], [0; 8]);
        let status_buf = static_init!([u8; 128], [0; 128]);

        // Instantiate the input driver
        let virtio_input = static_init!(
            VirtIOInput<'static>,
            VirtIOInput::new(event_queue, status_queue, status_buf),
        );
        event_queue.set_client(virtio_input);
        status_queue.set_client(virtio_input);

        // Register the queues and driver with the transport, so
        // interrupts are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 2], [event_queue, status_queue]);
        peripherals.virtio_mmio[input_idx]
            .initialize(virtio_input, mmio_queues)
            .unwrap();

        virtio_input.provide_buffers(event_buf1, event_buf2, event_buf3);

        Some(virtio_input)
    } else {
        // No Input device
        None
    };

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    let chip = static_init!(
        QemuRv32VirtChip<QemuRv32VirtDefaultPeripherals>,
        QemuRv32VirtChip::new(peripherals, hardware_timer, epmp),
    );
    CHIP = Some(chip);

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
    PROCESS_PRINTER = Some(process_printer);

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

    // ---------- RNG ----------

    // Userspace RNG driver over the VirtIO EntropySource
    let rng_driver = virtio_rng.map(|rng| {
        components::rng::RngRandomComponent::new(board_kernel, capsules_core::rng::DRIVER_NUM, rng)
            .finalize(components::rng_random_component_static!(
                qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng
            ))
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

    (board_kernel, platform, chip)
}
