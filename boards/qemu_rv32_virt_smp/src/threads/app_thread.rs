use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::scheduler::cooperative::CooperativeSched;
use kernel::threadlocal::ConstThreadId;
use kernel::threadlocal::ThreadLocalDyn;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::collections::atomic_ring_buffer::AtomicRingBuffer;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::platform::chip::InterruptService;
use kernel::{create_capability, debug, static_init};

use kernel::threadlocal::DynThreadId;

use qemu_rv32_virt_chip::chip::QemuRv32VirtChip;
use qemu_rv32_virt_chip::uart::Uart16550;
use qemu_rv32_virt_chip::interrupts;

use rv32i::csr;

use crate::CHIP;
use crate::PortalInstanceKey;

const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an empty list
// at least.
pub static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the process printer for panic dumps.
pub static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};


// Peripherals supported by this thread
pub struct QemuRv32VirtPeripherals;

impl<'a> QemuRv32VirtPeripherals {
    pub fn new() -> Self {
        Self
    }
}

impl InterruptService for QemuRv32VirtPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
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
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for QemuRv32VirtPlatform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
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

pub unsafe fn spawn<const ID: usize>(
    channel: &'static AtomicRingBuffer<qemu_rv32_virt_chip::portal::QemuRv32VirtVoyagerReference>,
) {
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
    rv32i::configure_trap_handler();

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

    // Initialize the platform-level interrupt controler
    qemu_rv32_virt_chip::plic::init_plic();

    // Initialize the core-local interrupt controler
    qemu_rv32_virt_chip::clint::init_clic();


    // Initialize the hardware portal
    let hw_portal = static_init!(
        qemu_rv32_virt_chip::portal::QemuRv32VirtPortal,
        qemu_rv32_virt_chip::portal::QemuRv32VirtPortal::new(
            channel,
            unsafe { DynThreadId::new(0) },
            static_init!(
                qemu_rv32_virt_chip::portal::QemuRv32VirtVoyager,
                qemu_rv32_virt_chip::portal::QemuRv32VirtVoyager::Empty,
            )
        )
    );
    qemu_rv32_virt_chip::portal::init_portal_panic(hw_portal);

    // ----- Creating a mux portal and deivce -----
    use capsules_core::portals::mux_demux::{MuxPortal, MuxPortalDevice, MuxDevice, MuxTraveler};

    let mux_portal = static_init!(
        MuxPortal,
        MuxPortal::new(hw_portal),
    );
    kernel::deferred_call::DeferredCallClient::register(mux_portal);
    hil::portal::Portal::set_portal_client(hw_portal, mux_portal);

    let mux_portal_device = static_init!(
        MuxPortalDevice,
        MuxPortalDevice::new(
            mux_portal,
            static_init!(
                MuxTraveler,
                MuxTraveler::Uart(0, kernel::utilities::cells::TakeCell::empty()),
            ),
            PortalInstanceKey::AppKernelUart as usize,
        ),
    );
    mux_portal_device.setup();

    // ---------- End of setting a mux portal and device --------

    // Initialize the uart portal client and connect it to the hardware portal
    use capsules_core::portals::teleportable_uart::{UartPortalClient, UartTraveler};
    let uart_portal_client = static_init!(
        UartPortalClient,
        UartPortalClient::new(
            static_init!(
                UartTraveler,
                UartTraveler::empty(),
            ),
            mux_portal_device,
        )
    );
    hil::portal::Portal::set_portal_client(mux_portal_device, uart_portal_client);


    // Initialize peripherals
    let peripherals = static_init!(
        QemuRv32VirtPeripherals,
        QemuRv32VirtPeripherals::new(),
    );

    // Create a shared UART channel for the console and for kernel
    // debug over the provided memory-mapped 16550-compatible
    // UART.
    let uart_mux = components::console::UartMuxComponent::new(uart_portal_client, 115200)
        .finalize(components::uart_mux_component_static!());

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux)
        .finalize(components::debug_writer_component_static!());

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

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    // Escape nonreentrant to get a reference of the local plic
    // TODO: impl the interrupt trait for ThreadLocal<Plic> to avoid this behavior.
    let plic = qemu_rv32_virt_chip::plic::with_plic_panic(|plic| &*(plic as *mut _));

    let chip = static_init!(
        QemuRv32VirtChip<QemuRv32VirtPeripherals>,
        QemuRv32VirtChip::new(peripherals, hardware_timer, epmp, plic),
    );

    let threadlocal_chip = *core::ptr::addr_of_mut!(CHIP);
    threadlocal_chip
        .get_mut()
        .map(|clocal| clocal.enter_nonreentrant(|c| c.replace(chip)))
        .expect("This thread cannot access thread-local chip");

    // Need to enable all interrupts for Tock Kernel
    // TODO: Enable a specific set of external interrupts for this kernel instance
    chip.disable_plic_interrupts();

    // Enable interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::mext::SET + csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);


    // ---------- FINAL SYSTEM INITIALIZATION ----------

    // Create the process printer used in panic prints, etc.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

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
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
    };

    // Initialization done. Inform the main kernel thread.
    crate::threads::main_thread::APP_THREAD_READY
        .store(true, core::sync::atomic::Ordering::SeqCst);

    debug!("QEMU RISC-V 32-bit \"virt\" machine core {ID}, initialization complete.");
    debug!("Entering application kernel loop.");

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

    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.ipc),
        &main_loop_cap,
        None
        // Some(&|| {
        //     debug!("debug message from app core");
        // })
    );
}
