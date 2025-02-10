use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::scheduler::cooperative::CooperativeSched;
use kernel::threadlocal;
use kernel::threadlocal::ConstThreadId;
use kernel::threadlocal::ThreadId;
use kernel::threadlocal::ThreadLocalAccessStatic;
use kernel::threadlocal::ThreadLocalDynInit;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::{create_capability, debug, static_init};
use kernel::{thread_local_static_init, thread_local_static_finalize, thread_local_static, thread_local_static_access};
use qemu_rv32_virt_chip::chip::{QemuRv32VirtChip, QemuRv32VirtDefaultPeripherals};
use qemu_rv32_virt_chip::plic::PLIC;
use qemu_rv32_virt_chip::plic::PLIC_BASE;
use rv32i::csr;

use kernel::utilities::registers::interfaces::Readable;
use kernel::threadlocal::DynThreadId;

use qemu_rv32_virt_chip::MAX_THREADS;

use crate::CHIP;

pub const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

use core::sync::atomic::{AtomicBool, Ordering};
pub static mut APP_THREAD_READY: AtomicBool = AtomicBool::new(false);

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct QemuRv32VirtPlatform {
    pconsole: &'static capsules_core::process_console::ProcessConsole<
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
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static CooperativeSched<'static>,
    scheduler_timer: &'static VirtualSchedulerTimer<
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >,
    virtio_rng: Option<
        &'static capsules_core::rng::RngDriver<
            'static,
            qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng<'static, 'static>,
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
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            capsules_core::rng::DRIVER_NUM => {
                if let Some(rng_driver) = self.virtio_rng {
                    f(Some(rng_driver))
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
    type CredentialsCheckingPolicy = ();
    type Scheduler = CooperativeSched<'static>;
    type SchedulerTimer = VirtualSchedulerTimer<
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
    >;
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
    fn credentials_checking_policy(&self) -> &'static Self::CredentialsCheckingPolicy {
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

pub unsafe fn spawn<const ID: usize>() {
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

    let id = ConstThreadId::<ID>::new();

    // Initialize the kernel's deferred call infrastructure for a
    // single-threaded platform configuration:
    let deferred_call_state = static_init!(
        qemu_rv32_virt_chip::QemuRv32VirtThreadLocal<kernel::deferred_call::ThreadLocalDeferredCallState>,
        qemu_rv32_virt_chip::QemuRv32VirtThreadLocal::init(kernel::deferred_call::DEFAULT_DEFERRED_CALL_STATE),
    );

    kernel::deferred_call::initialize_global_deferred_call_state(deferred_call_state);

    let channel = static_init!(
        qemu_rv32_virt_chip::channel::QemuRv32VirtChannel,
        qemu_rv32_virt_chip::channel::QemuRv32VirtChannel::new(),
    );
    kernel::deferred_call::DeferredCallClient::register(channel);


    // ---------- BASIC INITIALIZATION -----------

    // basic setup of the risc-v imac platform
    rv32i::configure_trap_handler(rv32i::PermissionMode::Machine);

    // Set up memory protection immediately after setting the trap handler, to
    // ensure that much of the board initialization routine runs with ePMP
    // protection.
    let epmp = rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP::new(
        rv32i::pmp::kernel_protection_mml_epmp::FlashRegion(
            rv32i::pmp::NAPOTRegionSpec::new(
                core::ptr::addr_of!(_sflash),
                core::ptr::addr_of!(_eflash) as usize - core::ptr::addr_of!(_sflash) as usize,
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::RAMRegion(
            rv32i::pmp::NAPOTRegionSpec::new(
                core::ptr::addr_of!(_ssram),
                core::ptr::addr_of!(_esram) as usize - core::ptr::addr_of!(_ssram) as usize,
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::MMIORegion(
            rv32i::pmp::NAPOTRegionSpec::new(
                core::ptr::null::<u8>(), // start
                0x20000000,              // size
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection_mml_epmp::KernelTextRegion(
            rv32i::pmp::TORRegionSpec::new(
                core::ptr::addr_of!(_stext),
                core::ptr::addr_of!(_etext),
            )
            .unwrap(),
        ),
    )
    .unwrap();

    // Acquire required capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board_kernel = static_init!(
        kernel::Kernel,
        kernel::Kernel::new(&PROCESSES)
    );


    // ---------- QEMU-SYSTEM-RISCV32 "virt" MACHINE PERIPHERALS ----------

    let plic = thread_local_static_finalize!(PLIC, ID);

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

    // Virtual alarm for the scheduler
    let systick_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    systick_virtual_alarm.setup();

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
    let (mut virtio_net_IDx, mut virtio_rng_IDx) = (None, None);
    for (i, virtio_device) in peripherals.virtio_mmio.iter().enumerate() {
        use qemu_rv32_virt_chip::virtio::devices::VirtIODeviceType;
        match virtio_device.query() {
            Some(VirtIODeviceType::NetworkCard) => {
                virtio_net_IDx = Some(i);
            }
            Some(VirtIODeviceType::EntropySource) => {
                virtio_rng_IDx = Some(i);
            }
            _ => (),
        }
    }

    // If there is a VirtIO EntropySource present, use the appropriate VirtIORng
    // driver and expose it to userspace though the RngDriver
    let virtio_rng_driver: Option<
        &'static capsules_core::rng::RngDriver<
            'static,
            qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng<'static, 'static>,
        >,
    > = if let Some(rng_IDx) = virtio_rng_IDx {
        use kernel::hil::rng::Rng;
        use qemu_rv32_virt_chip::virtio::devices::virtio_rng::VirtIORng;
        use qemu_rv32_virt_chip::virtio::queues::split_queue::{
            SplitVirtqueue, VirtqueueAvailableRing, VirtqueueDescriptors, VirtqueueUsedRing,
        };
        use qemu_rv32_virt_chip::virtio::queues::Virtqueue;
        use qemu_rv32_virt_chip::virtio::transports::VirtIOTransport;

        // EntropySource requires a single Virtqueue for retrieved entropy
        let descriptors = static_init!(VirtqueueDescriptors<1>, VirtqueueDescriptors::default());
        let available_ring =
            static_init!(VirtqueueAvailableRing<1>, VirtqueueAvailableRing::default(),);
        let used_ring = static_init!(VirtqueueUsedRing<1>, VirtqueueUsedRing::default(),);
        let queue = static_init!(
            SplitVirtqueue<1>,
            SplitVirtqueue::new(descriptors, available_ring, used_ring),
        );
        queue.set_transport(&peripherals.virtio_mmio[rng_IDx]);

        // VirtIO EntropySource device driver instantiation
        let rng = static_init!(VirtIORng, VirtIORng::new(queue));
        kernel::deferred_call::DeferredCallClient::register(rng);
        queue.set_client(rng);

        // Register the queues and driver with the transport, so interrupts
        // are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 1], [queue; 1]);
        peripherals.virtio_mmio[rng_IDx]
            .initialize(rng, mmio_queues)
            .unwrap();

        // Provide an internal randomness buffer
        let rng_buffer = static_init!([u8; 64], [0; 64]);
        rng.provide_buffer(rng_buffer)
            .expect("rng: provIDing initial buffer failed");

        // Userspace RNG driver over the VirtIO EntropySource
        let rng_driver = static_init!(
            capsules_core::rng::RngDriver<VirtIORng>,
            capsules_core::rng::RngDriver::new(
                rng,
                board_kernel.create_grant(capsules_core::rng::DRIVER_NUM, &memory_allocation_cap),
            ),
        );
        rng.set_client(rng_driver);

        Some(rng_driver as &'static capsules_core::rng::RngDriver<VirtIORng>)
    } else {
        // No VirtIO EntropySource discovered
        None
    };


    // If there is a VirtIO NetworkCard present, use the appropriate VirtIONet
    // driver. Currently this is not used, as work on the userspace network
    // driver and kernel network stack is in progress.
    //
    // A template dummy driver is provided to verify basic functionality of this
    // interface.
    let _virtio_net_if: Option<
        &'static qemu_rv32_virt_chip::virtio::devices::virtio_net::VirtIONet<'static>,
    > = if let Some(net_IDx) = virtio_net_IDx {
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
        tx_queue.set_transport(&peripherals.virtio_mmio[net_IDx]);

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
        rx_queue.set_transport(&peripherals.virtio_mmio[net_IDx]);

        // Incoming and outgoing packets are prefixed by a 12-byte
        // VirtIO specific header
        let tx_header_buf = static_init!([u8; 12], [0; 12]);
        let rx_header_buf = static_init!([u8; 12], [0; 12]);

        // Currently, provide a single receive buffer to write
        // incoming packets into
        let rx_buffer = static_init!([u8; 1526], [0; 1526]);

        // Instantiate the VirtIONet (NetworkCard) driver and set
        // the queues
        let virtio_net = static_init!(
            VirtIONet<'static>,
            VirtIONet::new(
                0,
                tx_queue,
                tx_header_buf,
                rx_queue,
                rx_header_buf,
                rx_buffer,
            ),
        );
        tx_queue.set_client(virtio_net);
        rx_queue.set_client(virtio_net);

        // Register the queues and driver with the transport, so
        // interrupts are routed properly
        let mmio_queues = static_init!([&'static dyn Virtqueue; 2], [rx_queue, tx_queue]);
        peripherals.virtio_mmio[net_IDx]
            .initialize(virtio_net, mmio_queues)
            .unwrap();

        // Don't forget to enable RX once when integrating this into a
        // proper Ethernet stack:
        // virtio_net.enable_rx();

        // TODO: When we have a proper Ethernet driver available for userspace,
        // return that. For now, just return a reference to the raw VirtIONet
        // driver:
        Some(virtio_net as &'static VirtIONet)
    } else {
        // No VirtIO NetworkCard discovered
        None
    };

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    let chip = static_init!(
        QemuRv32VirtChip<QemuRv32VirtDefaultPeripherals>,
        QemuRv32VirtChip::new(peripherals, hardware_timer, epmp, plic),
    );
    let chip_local = thread_local_static_finalize!(CHIP, ID);
    *chip_local = Some(chip);

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
    components::debug_writer::DebugWriterComponent::new(uart_mux)
        .finalize(components::debug_writer_component_static!());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        capsules_core::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::low_level_debug_component_static!());

    let scheduler = components::sched::cooperative::CooperativeComponent::new(&PROCESSES)
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<
            VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
        >,
        VirtualSchedulerTimer::new(systick_virtual_alarm)
    );

    let platform = QemuRv32VirtPlatform {
        pconsole,
        console,
        alarm,
        lldb,
        scheduler,
        scheduler_timer,
        virtio_rng: virtio_rng_driver,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
    };

    // Start the process console:
    let _ = platform.pconsole.start();

    debug!("QEMU RISC-V 32-bit {MAX_THREADS}-SMP \"virt\" machine core {ID}, initialization complete.");
    debug!("Entering main loop.");


    // finalize the static mut before using it
    let clic = thread_local_static_finalize!(qemu_rv32_virt_chip::clint::CLIC, ID);

    // Global initialization is done. Wake up all threads.
    (0..MAX_THREADS)
        .filter(|&id| id != ID)
        .for_each(|id| hardware_timer.set_soft_interrupt(id));

    // Send echo command to the app thread
    use qemu_rv32_virt_chip::channel::SHARED_CHANNEL_BUFFER;
    unsafe {
        SHARED_CHANNEL_BUFFER[0] = 10;
        SHARED_CHANNEL_BUFFER[1] = 5;
        SHARED_CHANNEL_BUFFER[58] = 100;
    }

    // Block until the app thread finishes initialization
    while !APP_THREAD_READY.load(Ordering::SeqCst) {}

    hardware_timer.set_soft_interrupt(1);

    // loop {
    //     hardware_timer.set_soft_interrupt(1);
    // }

    // ---------- PROCESS LOADING, SCHEDULER LOOP ----------

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
        &mut PROCESSES,
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_cap, Some(&crate::SHARED_BUFFER), Some(&qemu_rv32_virt_chip::chip::MACHINE_SOFT_FIRED_COUNT));
}
