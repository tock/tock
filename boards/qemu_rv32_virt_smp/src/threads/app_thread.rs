use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
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
use kernel::threadlocal::ThreadLocalDyn;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::{create_capability, debug, static_init};
use kernel::{thread_local_static_init, thread_local_static_finalize, thread_local_static, thread_local_static_access};
use qemu_rv32_virt_chip::chip::QemuRv32VirtChip;
use qemu_rv32_virt_chip::plic::PLIC;
use qemu_rv32_virt_chip::plic::PLIC_BASE;
use rv32i::csr;

use kernel::utilities::registers::interfaces::Readable;
use kernel::threadlocal::DynThreadId;
use kernel::platform::chip::InterruptService;

use qemu_rv32_virt_chip::MAX_THREADS;

use virtio::transports::mmio::VirtIOMMIODevice;
use qemu_rv32_virt_chip::{virtio_mmio, interrupts};

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


// Peripherals supported by this thread
pub struct QemuRv32VirtPeripherals {
    pub virtio_mmio: [VirtIOMMIODevice; 8],
}

impl QemuRv32VirtPeripherals {
    pub fn new() -> Self {
        Self {
            virtio_mmio: [
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_0_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_1_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_2_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_3_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_4_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_5_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_6_BASE),
                VirtIOMMIODevice::new(virtio_mmio::VIRTIO_MMIO_7_BASE),
            ],
        }
    }
}

impl InterruptService for QemuRv32VirtPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
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

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct QemuRv32VirtPlatform {
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
    shared_channel: &'static qemu_rv32_virt_chip::channel::QemuRv32VirtChannel<'static>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for QemuRv32VirtPlatform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
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
            QemuRv32VirtPeripherals,
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
    type SharedChannel = qemu_rv32_virt_chip::channel::QemuRv32VirtChannel<'static>;

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
    fn shared_channel(&self) -> &Self::SharedChannel {
        self.shared_channel
    }
}

pub unsafe fn spawn<const ID: usize>(channel: &'static mut qemu_rv32_virt_chip::channel::QemuRv32VirtChannel) {
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
        kernel::Kernel::new(&*core::ptr::addr_of!(PROCESSES))
    );


    // ---------- QEMU-SYSTEM-RISCV32 "virt" MACHINE PERIPHERALS ----------

    let plic = thread_local_static_finalize!(PLIC, ID);

    // init channel receiver
    // let channel = static_init!(
    //     qemu_rv32_virt_chip::channel::QemuRv32VirtChannel,
    //     qemu_rv32_virt_chip::channel::QemuRv32VirtChannel::new(&mut *core::ptr::addr_of_mut!(qemu_rv32_virt_chip::channel::CHANNEL_BUFFER)),
    // );
    kernel::deferred_call::DeferredCallClient::register(channel);

    // Open an empty uart portal
    use qemu_rv32_virt_chip::portal::{QemuRv32VirtPortal, PORTALS};
    use qemu_rv32_virt_chip::portal_cell::QemuRv32VirtPortalCell;
    use qemu_rv32_virt_chip::uart::{Uart16550, UART0_BASE};
    use qemu_rv32_virt_chip::chip::COUNTER;

    let uart_portal = static_init!(
        QemuRv32VirtPortalCell<Uart16550>,
        QemuRv32VirtPortalCell::empty(QemuRv32VirtPortal::Uart16550(core::ptr::null()).id())
    );

    let counter_portal = static_init!(
        QemuRv32VirtPortalCell<core::sync::atomic::AtomicUsize>,
        QemuRv32VirtPortalCell::empty(QemuRv32VirtPortal::Counter(core::ptr::null()).id())
    );

    (&*core::ptr::addr_of!(PORTALS))
        .get_mut()
        .expect("App thread doesn't not have access to its local portals")
        .enter_nonreentrant(|ps| {
            ps[uart_portal.get_id()] = QemuRv32VirtPortal::Uart16550(uart_portal as *mut _ as *const _);
            ps[counter_portal.get_id()] = QemuRv32VirtPortal::Counter(counter_portal as *mut _ as *const _);
        });

    // Initialize peripherals
    let peripherals = static_init!(
        QemuRv32VirtPeripherals,
        QemuRv32VirtPeripherals::new(),
    );

    // Create a shared UART channel for the console and for kernel
    // debug over the provided memory-mapped 16550-compatible
    // UART.
    // let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
    //     .finalize(components::uart_mux_component_static!());

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


    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    let chip = static_init!(
        QemuRv32VirtChip<QemuRv32VirtPeripherals>,
        QemuRv32VirtChip::new(peripherals, hardware_timer, epmp, plic),
    );
    let chip_local = thread_local_static_finalize!(CHIP, ID);
    *chip_local = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    // TODO: Enable a specific set of external interrupts for this kernel instance
    // chip.enable_plic_interrupts();

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

    // components::debug_writer::DebugWriterComponent::new(uart_mux)
    //     .finalize(components::debug_writer_component_static!());

    let scheduler = components::sched::cooperative::CooperativeComponent::new(&*core::ptr::addr_of!(PROCESSES))
        .finalize(components::cooperative_component_static!(NUM_PROCS));

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<
            VirtualMuxAlarm<'static, qemu_rv32_virt_chip::chip::QemuRv32VirtClint<'static>>,
        >,
        VirtualSchedulerTimer::new(systick_virtual_alarm)
    );

    let platform = QemuRv32VirtPlatform {
        alarm,
        scheduler,
        scheduler_timer,
        virtio_rng: virtio_rng_driver,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
        shared_channel: channel,
    };

    // finalize the static mut before using it
    let clic = thread_local_static_finalize!(qemu_rv32_virt_chip::clint::CLIC, ID);


    use core::sync::atomic::Ordering;
    crate::threads::main_thread::APP_THREAD_READY.store(true, Ordering::SeqCst);

    // use kernel::smp::portal_cell::Portalable;
    // uart_portal.conjure(&platform, chip);

    // panic!("Panic at core {}", ID);

    // debug!("QEMU RISC-V 32-bit \"virt\" machine core {ID}, initialization complete.");
    // debug!("Entering main loop.");

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
        &mut *core::ptr::addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_cap,
                             Some(&*core::ptr::addr_of!(qemu_rv32_virt_chip::chip::COUNTER)),
                             Some(&|| {
                                 use kernel::smp::portal_cell::Portalable;
                                 counter_portal.enter(|c| {
                                     c.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
                                 }).unwrap_or_else(|| {
                                     counter_portal.conjure(&platform, chip);
                                 });

                                     unsafe {
                                         crate::DEBUG_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
                                     }
                             }));
}
