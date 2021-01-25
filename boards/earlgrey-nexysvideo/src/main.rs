//! Board file for LowRISC OpenTitan RISC-V development platform.
//!
//! - <https://opentitan.org/>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_hmac::VirtualMuxHmac;
use capsules::virtual_sha::VirtualMuxSha;
use earlgrey::chip::EarlGreyDefaultPeripherals;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::common::registers::interfaces::ReadWriteable;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::digest::Digest;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::time::Alarm;
use kernel::mpu::KernelMPU;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use kernel::{mpu, Chip};
use rv32i::csr;

#[cfg(test)]
mod tests;

mod otbn;
#[allow(dead_code)]
mod tickv_test;

pub mod io;
pub mod usb;

const NUM_PROCS: usize = 4;
const NUM_UPCALLS_IPC: usize = NUM_PROCS + 1;

//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::Process>; 4] = [None; NUM_PROCS];

// Test access to the peripherals
#[cfg(test)]
static mut PERIPHERALS: Option<&'static EarlGreyDefaultPeripherals> = None;
// Test access to scheduler
#[cfg(test)]
static mut SCHEDULER: Option<&kernel::PrioritySched> = None;
// Test access to board
#[cfg(test)]
static mut BOARD: Option<&'static kernel::Kernel> = None;
// Test access to platform
#[cfg(test)]
static mut PLATFORM: Option<&'static EarlGreyNexysVideo> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, earlgrey::timer::RvTimer<'static>>> = None;

static mut CHIP: Option<
    &'static earlgrey::chip::EarlGrey<
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
        EarlGreyDefaultPeripherals,
    >,
> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::PanicFaultPolicy = kernel::procs::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct EarlGreyNexysVideo {
    led: &'static capsules::led::LedDriver<
        'static,
        LedHigh<'static, earlgrey::gpio::GpioPin<'static>>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static, earlgrey::gpio::GpioPin<'static>>,
    console: &'static capsules::console::Console<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer<'static>>,
    >,
    hmac: &'static capsules::hmac::HmacDriver<
        'static,
        VirtualMuxHmac<
            'static,
            capsules::virtual_digest::VirtualMuxDigest<'static, lowrisc::hmac::Hmac<'static>, 32>,
            32,
        >,
        32,
    >,
    sha: &'static capsules::sha::ShaDriver<
        'static,
        VirtualMuxSha<
            'static,
            capsules::virtual_digest::VirtualMuxDigest<'static, lowrisc::hmac::Hmac<'static>, 32>,
            32,
        >,
        32,
    >,
    lldb: &'static capsules::low_level_debug::LowLevelDebug<
        'static,
        capsules::virtual_uart::UartDevice<'static>,
    >,
    i2c_master: &'static capsules::i2c_master::I2CMasterDriver<'static, lowrisc::i2c::I2c<'static>>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for EarlGreyNexysVideo {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::hmac::DRIVER_NUM => f(Some(self.hmac)),
            capsules::sha::DRIVER_NUM => f(Some(self.sha)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            _ => f(None),
        }
    }
}

unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static EarlGreyNexysVideo,
    &'static earlgrey::chip::EarlGrey<
        'static,
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer<'static>>,
        EarlGreyDefaultPeripherals<'static>,
    >,
    &'static EarlGreyDefaultPeripherals<'static>,
) {
    // Ibex-specific handler
    earlgrey::chip::configure_trap_handler();

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 3], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let peripherals = static_init!(
        EarlGreyDefaultPeripherals,
        EarlGreyDefaultPeripherals::new(dynamic_deferred_caller)
    );

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&peripherals.gpio_port[7]), // First LED
        None,
        None,
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &peripherals.uart0,
        earlgrey::uart::UART0_BAUDRATE,
        dynamic_deferred_caller,
    )
    .finalize(());

    // LEDs
    // Start with half on and half off
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        LedHigh<'static, earlgrey::gpio::GpioPin>,
        LedHigh::new(&peripherals.gpio_port[8]),
        LedHigh::new(&peripherals.gpio_port[9]),
        LedHigh::new(&peripherals.gpio_port[10]),
        LedHigh::new(&peripherals.gpio_port[11]),
        LedHigh::new(&peripherals.gpio_port[12]),
        LedHigh::new(&peripherals.gpio_port[13]),
        LedHigh::new(&peripherals.gpio_port[14]),
        LedHigh::new(&peripherals.gpio_port[15]),
    ))
    .finalize(components::led_component_buf!(
        LedHigh<'static, earlgrey::gpio::GpioPin>
    ));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            earlgrey::gpio::GpioPin,
            0 => &peripherals.gpio_port[0],
            1 => &peripherals.gpio_port[1],
            2 => &peripherals.gpio_port[2],
            3 => &peripherals.gpio_port[3],
            4 => &peripherals.gpio_port[4],
            5 => &peripherals.gpio_port[5],
            6 => &peripherals.gpio_port[6],
            7 => &peripherals.gpio_port[15]
        ),
    )
    .finalize(components::gpio_component_buf!(earlgrey::gpio::GpioPin));

    let hardware_alarm = static_init!(earlgrey::timer::RvTimer, earlgrey::timer::RvTimer::new());
    hardware_alarm.setup();

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, earlgrey::timer::RvTimer>,
        MuxAlarm::new(hardware_alarm)
    );
    hil::time::Alarm::set_alarm_client(hardware_alarm, mux_alarm);

    ALARM = Some(mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let scheduler_timer_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>>,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let chip = static_init!(
        earlgrey::chip::EarlGrey<
            VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
            EarlGreyDefaultPeripherals,
        >,
        earlgrey::chip::EarlGrey::new(scheduler_timer_virtual_alarm, peripherals, hardware_alarm)
    );
    scheduler_timer_virtual_alarm.set_alarm_client(chip.scheduler_timer());
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.enable_plic_interrupts();
    // enable interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET + csr::mie::mie::mext::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        capsules::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(());

    let mux_digest = components::digest::DigestMuxComponent::new(&peripherals.hmac).finalize(
        components::digest_mux_component_helper!(lowrisc::hmac::Hmac, 32),
    );

    let digest_key_buffer = static_init!([u8; 32], [0; 32]);

    let digest = components::digest::DigestComponent::new(&mux_digest, digest_key_buffer).finalize(
        components::digest_component_helper!(lowrisc::hmac::Hmac, 32,),
    );

    peripherals.hmac.set_client(digest);

    let hmac_key_buffer = static_init!([u8; 32], [0; 32]);
    let hmac_data_buffer = static_init!([u8; 64], [0; 64]);
    let hmac_dest_buffer = static_init!([u8; 32], [0; 32]);

    let mux_hmac = components::hmac::HmacMuxComponent::new(digest).finalize(
        components::hmac_mux_component_helper!(capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>, 32),
    );

    let hmac = components::hmac::HmacComponent::new(
        board_kernel,
        capsules::hmac::DRIVER_NUM,
        &mux_hmac,
        hmac_key_buffer,
        hmac_data_buffer,
        hmac_dest_buffer,
    )
    .finalize(components::hmac_component_helper!(
        capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>,
        32,
    ));

    digest.set_hmac_client(hmac);

    let sha_data_buffer = static_init!([u8; 64], [0; 64]);
    let sha_dest_buffer = static_init!([u8; 32], [0; 32]);

    let mux_sha = components::sha::ShaMuxComponent::new(digest).finalize(
        components::sha_mux_component_helper!(capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>, 32),
    );

    let sha = components::sha::ShaComponent::new(
        board_kernel,
        capsules::sha::DRIVER_NUM,
        &mux_sha,
        sha_data_buffer,
        sha_dest_buffer,
    )
    .finalize(components::sha_component_helper!(capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>, 32));

    digest.set_sha_client(sha);

    let i2c_master = static_init!(
        capsules::i2c_master::I2CMasterDriver<'static, lowrisc::i2c::I2c<'static>>,
        capsules::i2c_master::I2CMasterDriver::new(
            &peripherals.i2c0,
            &mut capsules::i2c_master::BUF,
            board_kernel.create_grant(capsules::i2c_master::DRIVER_NUM, &memory_allocation_cap)
        )
    );

    peripherals.i2c0.set_master_client(i2c_master);

    peripherals.aes.initialise(
        dynamic_deferred_caller
            .register(&peripherals.aes)
            .expect("dynamic deferred caller out of slots"),
    );

    // USB support is currently broken in the OpenTitan hardware
    // See https://github.com/lowRISC/opentitan/issues/2598 for more details
    // let usb = usb::UsbComponent::new(board_kernel).finalize(());

    // Kernel storage region, allocated with the storage_volume!
    // macro in common/utils.rs
    extern "C" {
        /// Beginning on the ROM region containing app images.
        static _sstorage: u8;
        static _estorage: u8;
    }

    // Flash
    let flash_ctrl_read_buf = static_init!(
        [u8; lowrisc::flash_ctrl::PAGE_SIZE],
        [0; lowrisc::flash_ctrl::PAGE_SIZE]
    );
    let page_buffer = static_init!(
        lowrisc::flash_ctrl::LowRiscPage,
        lowrisc::flash_ctrl::LowRiscPage::default()
    );

    let mux_flash = components::tickv::FlashMuxComponent::new(&peripherals.flash_ctrl).finalize(
        components::flash_user_component_helper!(lowrisc::flash_ctrl::FlashCtrl),
    );

    // TicKV
    let _tickv = components::tickv::TicKVComponent::new(
        &mux_flash,                                  // Flash controller
        0x20040000 / lowrisc::flash_ctrl::PAGE_SIZE, // Region offset (size / page_size)
        0x40000,                                     // Region size
        flash_ctrl_read_buf,                         // Buffer used internally in TicKV
        page_buffer,                                 // Buffer used with the flash controller
    )
    .finalize(components::tickv_component_helper!(
        lowrisc::flash_ctrl::FlashCtrl
    ));
    hil::flash::HasClient::set_client(&peripherals.flash_ctrl, mux_flash);

    let _mux_otbn = crate::otbn::AccelMuxComponent::new(&peripherals.otbn)
        .finalize(otbn_mux_component_helper!(1024));

    peripherals.otbn.initialise(
        dynamic_deferred_caller
            .register(&peripherals.otbn)
            .expect("dynamic deferred caller out of slots"),
    );

    /// These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
        /// The start of the kernel stack (Included only for kernel PMP)
        static _sstack: u8;
        /// The end of the kernel stack (Included only for kernel PMP)
        static _estack: u8;
        /// The start of the kernel text (Included only for kernel PMP)
        static _stext: u8;
        /// The end of the kernel text (Included only for kernel PMP)
        static _etext: u8;
        /// The start of the kernel relocation region
        /// (Included only for kernel PMP)
        static _srelocate: u8;
        /// The end of the kernel relocation region
        /// (Included only for kernel PMP)
        static _erelocate: u8;
        /// The start of the kernel BSS (Included only for kernel PMP)
        static _szero: u8;
        /// The end of the kernel BSS (Included only for kernel PMP)
        static _ezero: u8;
    }

    let earlgrey_nexysvideo = static_init!(
        EarlGreyNexysVideo,
        EarlGreyNexysVideo {
            gpio: gpio,
            led: led,
            console: console,
            alarm: alarm,
            hmac,
            sha,
            lldb: lldb,
            i2c_master,
        }
    );

    let mut mpu_config = rv32i::epmp::PMPConfig::default();
    // The kernel stack
    chip.pmp.allocate_kernel_region(
        &_sstack as *const u8,
        &_estack as *const u8 as usize - &_sstack as *const u8 as usize,
        mpu::Permissions::ReadWriteOnly,
        &mut mpu_config,
    );
    // The kernel text
    chip.pmp.allocate_kernel_region(
        &_stext as *const u8,
        &_etext as *const u8 as usize - &_stext as *const u8 as usize,
        mpu::Permissions::ReadExecuteOnly,
        &mut mpu_config,
    );
    // The kernel relocate data
    chip.pmp.allocate_kernel_region(
        &_srelocate as *const u8,
        &_erelocate as *const u8 as usize - &_srelocate as *const u8 as usize,
        mpu::Permissions::ReadWriteOnly,
        &mut mpu_config,
    );
    // The kernel BSS
    chip.pmp.allocate_kernel_region(
        &_szero as *const u8,
        &_ezero as *const u8 as usize - &_szero as *const u8 as usize,
        mpu::Permissions::ReadWriteOnly,
        &mut mpu_config,
    );
    // The app locations
    chip.pmp.allocate_kernel_region(
        &_sapps as *const u8,
        &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        mpu::Permissions::ReadWriteOnly,
        &mut mpu_config,
    );
    // The app memory locations
    chip.pmp.allocate_kernel_region(
        &_sappmem as *const u8,
        &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        mpu::Permissions::ReadWriteOnly,
        &mut mpu_config,
    );

    chip.pmp.enable_kernel_mpu(&mut mpu_config);

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });
    debug!("OpenTitan initialisation complete. Entering main loop");

    (board_kernel, earlgrey_nexysvideo, chip, peripherals)
}

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        let (board_kernel, earlgrey_nexysvideo, chip, _peripherals) = setup();

        let scheduler =
            components::sched::priority::PriorityComponent::new(board_kernel).finalize(());
        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(
            earlgrey_nexysvideo,
            chip,
            None::<&kernel::ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
            None,
            scheduler,
            &main_loop_cap,
        );
    }
}

#[cfg(test)]
use kernel::watchdog::WatchDog;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unsafe {
        let (board_kernel, earlgrey_nexysvideo, chip, peripherals) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&earlgrey_nexysvideo);
        PERIPHERALS = Some(peripherals);
        SCHEDULER =
            Some(components::sched::priority::PriorityComponent::new(board_kernel).finalize(()));
        MAIN_CAP = Some(&create_capability!(capabilities::MainLoopCapability));

        chip.watchdog().setup();

        for test in tests {
            test();
        }
    }

    // Exit QEMU with a return code of 0
    crate::tests::semihost_command_exit_success()
}
