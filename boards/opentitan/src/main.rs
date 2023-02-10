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

use crate::hil::symmetric_encryption::AES128_BLOCK_SIZE;
use crate::otbn::OtbnComponent;
use core_capsules::virtual_aes_ccm;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core_capsules::virtual_hmac::VirtualMuxHmac;
use core_capsules::virtual_sha::VirtualMuxSha;
use earlgrey::chip::EarlGreyDefaultPeripherals;
use kernel::capabilities;
use kernel::component::Component;
use kernel::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::hil;
use kernel::hil::digest::Digest;
use kernel::hil::entropy::Entropy32;
use kernel::hil::hasher::Hasher;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::kv_system::KVSystem;
use kernel::hil::led::LedHigh;
use kernel::hil::rng::Rng;
use kernel::hil::symmetric_encryption::AES128;
use kernel::platform::mpu;
use kernel::platform::mpu::KernelMPU;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::{KernelResources, SyscallDriverLookup, TbfHeaderFilterDefaultAllow};
use kernel::scheduler::priority::PrioritySched;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::{create_capability, debug, static_init};
use lowrisc::flash_ctrl::FlashMPConfig;
use rv32i::csr;

pub mod io;
mod otbn;
#[cfg(test)]
mod tests;

const NUM_PROCS: usize = 4;

//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; 4] = [None; NUM_PROCS];

// Test access to the peripherals
#[cfg(test)]
static mut PERIPHERALS: Option<&'static EarlGreyDefaultPeripherals> = None;
// Test access to board
#[cfg(test)]
static mut BOARD: Option<&'static kernel::Kernel> = None;
// Test access to platform
#[cfg(test)]
static mut PLATFORM: Option<&'static EarlGrey> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, earlgrey::timer::RvTimer<'static>>> = None;
// Test access to TicKV
static mut TICKV: Option<
    &extra_capsules::tickv::TicKVStore<
        'static,
        core_capsules::virtual_flash::FlashUser<'static, lowrisc::flash_ctrl::FlashCtrl<'static>>,
        extra_capsules::sip_hash::SipHasher24<'static>,
    >,
> = None;
// Test access to AES CCM
static mut AES: Option<&virtual_aes_ccm::VirtualAES128CCM<'static, earlgrey::aes::Aes<'static>>> =
    None;
// Test access to SipHash
static mut SIPHASH: Option<&extra_capsules::sip_hash::SipHasher24<'static>> = None;
// Test access to RSA
static mut RSA_HARDWARE: Option<&lowrisc::rsa::OtbnRsa<'static>> = None;

// Test access to a software SHA256
#[cfg(test)]
static mut SHA256SOFT: Option<&extra_capsules::sha256::Sha256Software<'static>> = None;

static mut CHIP: Option<&'static earlgrey::chip::EarlGrey<EarlGreyDefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct EarlGrey {
    led: &'static core_capsules::led::LedDriver<
        'static,
        LedHigh<'static, earlgrey::gpio::GpioPin<'static>>,
        8,
    >,
    gpio: &'static core_capsules::gpio::GPIO<'static, earlgrey::gpio::GpioPin<'static>>,
    console: &'static core_capsules::console::Console<'static>,
    alarm: &'static core_capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer<'static>>,
    >,
    hmac: &'static extra_capsules::hmac::HmacDriver<
        'static,
        VirtualMuxHmac<
            'static,
            core_capsules::virtual_digest::VirtualMuxDigest<
                'static,
                lowrisc::hmac::Hmac<'static>,
                32,
            >,
            32,
        >,
        32,
    >,
    sha: &'static extra_capsules::sha::ShaDriver<
        'static,
        VirtualMuxSha<
            'static,
            core_capsules::virtual_digest::VirtualMuxDigest<
                'static,
                lowrisc::hmac::Hmac<'static>,
                32,
            >,
            32,
        >,
        32,
    >,
    lldb: &'static core_capsules::low_level_debug::LowLevelDebug<
        'static,
        core_capsules::virtual_uart::UartDevice<'static>,
    >,
    i2c_master:
        &'static core_capsules::i2c_master::I2CMasterDriver<'static, lowrisc::i2c::I2c<'static>>,
    spi_controller: &'static core_capsules::spi_controller::Spi<
        'static,
        core_capsules::virtual_spi::VirtualSpiMasterDevice<'static, lowrisc::spi_host::SpiHost>,
    >,
    rng: &'static core_capsules::rng::RngDriver<'static>,
    aes: &'static extra_capsules::symmetric_encryption::aes::AesDriver<
        'static,
        virtual_aes_ccm::VirtualAES128CCM<'static, earlgrey::aes::Aes<'static>>,
    >,
    kv_driver: &'static extra_capsules::kv_driver::KVSystemDriver<
        'static,
        extra_capsules::tickv::TicKVStore<
            'static,
            core_capsules::virtual_flash::FlashUser<
                'static,
                lowrisc::flash_ctrl::FlashCtrl<'static>,
            >,
            extra_capsules::sip_hash::SipHasher24<'static>,
        >,
        [u8; 8],
    >,
    syscall_filter: &'static TbfHeaderFilterDefaultAllow,
    scheduler: &'static PrioritySched,
    scheduler_timer:
        &'static VirtualSchedulerTimer<VirtualMuxAlarm<'static, earlgrey::timer::RvTimer<'static>>>,
    watchdog: &'static lowrisc::aon_timer::AonTimer,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for EarlGrey {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            core_capsules::led::DRIVER_NUM => f(Some(self.led)),
            extra_capsules::hmac::DRIVER_NUM => f(Some(self.hmac)),
            extra_capsules::sha::DRIVER_NUM => f(Some(self.sha)),
            core_capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            core_capsules::console::DRIVER_NUM => f(Some(self.console)),
            core_capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            core_capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            core_capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            core_capsules::spi_controller::DRIVER_NUM => f(Some(self.spi_controller)),
            core_capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            extra_capsules::symmetric_encryption::aes::DRIVER_NUM => f(Some(self.aes)),
            extra_capsules::kv_driver::DRIVER_NUM => f(Some(self.kv_driver)),
            _ => f(None),
        }
    }
}

impl KernelResources<earlgrey::chip::EarlGrey<'static, EarlGreyDefaultPeripherals<'static>>>
    for EarlGrey
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = TbfHeaderFilterDefaultAllow;
    type ProcessFault = ();
    type CredentialsCheckingPolicy = ();
    type Scheduler = PrioritySched;
    type SchedulerTimer =
        VirtualSchedulerTimer<VirtualMuxAlarm<'static, earlgrey::timer::RvTimer<'static>>>;
    type WatchDog = lowrisc::aon_timer::AonTimer;
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        &self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &self.syscall_filter
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
        &self.scheduler_timer
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &self.watchdog
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static EarlGrey,
    &'static earlgrey::chip::EarlGrey<'static, EarlGreyDefaultPeripherals<'static>>,
    &'static EarlGreyDefaultPeripherals<'static>,
) {
    // Ibex-specific handler
    earlgrey::chip::configure_trap_handler();

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 6], Default::default());
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
    .finalize(components::uart_mux_component_static!());

    // LEDs
    // Start with half on and half off
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, earlgrey::gpio::GpioPin>,
        LedHigh::new(&peripherals.gpio_port[8]),
        LedHigh::new(&peripherals.gpio_port[9]),
        LedHigh::new(&peripherals.gpio_port[10]),
        LedHigh::new(&peripherals.gpio_port[11]),
        LedHigh::new(&peripherals.gpio_port[12]),
        LedHigh::new(&peripherals.gpio_port[13]),
        LedHigh::new(&peripherals.gpio_port[14]),
        LedHigh::new(&peripherals.gpio_port[15]),
    ));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        core_capsules::gpio::DRIVER_NUM,
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
    .finalize(components::gpio_component_static!(earlgrey::gpio::GpioPin));

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
    virtual_alarm_user.setup();

    let scheduler_timer_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    scheduler_timer_virtual_alarm.setup();

    let alarm = static_init!(
        core_capsules::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
        >,
        core_capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(core_capsules::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<VirtualMuxAlarm<'static, earlgrey::timer::RvTimer<'static>>>,
        VirtualSchedulerTimer::new(scheduler_timer_virtual_alarm)
    );

    let chip = static_init!(
        earlgrey::chip::EarlGrey<
            EarlGreyDefaultPeripherals,
        >,
        earlgrey::chip::EarlGrey::new(peripherals, hardware_alarm)
    );
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.enable_plic_interrupts();
    // enable interrupts globally
    csr::CSR.mie.modify(
        csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::CLEAR + csr::mie::mie::mext::SET,
    );
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        core_capsules::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux)
        .finalize(components::debug_writer_component_static!());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        core_capsules::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::low_level_debug_component_static!());

    let mux_digest = components::digest::DigestMuxComponent::new(&peripherals.hmac).finalize(
        components::digest_mux_component_static!(lowrisc::hmac::Hmac, 32),
    );

    let digest = components::digest::DigestComponent::new(&mux_digest).finalize(
        components::digest_component_static!(lowrisc::hmac::Hmac, 32,),
    );

    peripherals.hmac.set_client(digest);

    let mux_hmac = components::hmac::HmacMuxComponent::new(digest).finalize(
        components::hmac_mux_component_static!(core_capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>, 32),
    );

    let hmac = components::hmac::HmacComponent::new(
        board_kernel,
        extra_capsules::hmac::DRIVER_NUM,
        &mux_hmac,
    )
    .finalize(components::hmac_component_static!(
        core_capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>,
        32,
    ));

    digest.set_hmac_client(hmac);

    let mux_sha = components::sha::ShaMuxComponent::new(digest).finalize(
        components::sha_mux_component_static!(core_capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>, 32),
    );

    let sha = components::sha::ShaComponent::new(
        board_kernel,
        extra_capsules::sha::DRIVER_NUM,
        &mux_sha,
    )
    .finalize(components::sha_component_static!(core_capsules::virtual_digest::VirtualMuxDigest<lowrisc::hmac::Hmac, 32>, 32));

    digest.set_sha_client(sha);

    let i2c_master = static_init!(
        core_capsules::i2c_master::I2CMasterDriver<'static, lowrisc::i2c::I2c<'static>>,
        core_capsules::i2c_master::I2CMasterDriver::new(
            &peripherals.i2c0,
            &mut core_capsules::i2c_master::BUF,
            board_kernel.create_grant(
                core_capsules::i2c_master::DRIVER_NUM,
                &memory_allocation_cap
            )
        )
    );

    peripherals.i2c0.set_master_client(i2c_master);

    //SPI
    let mux_spi =
        components::spi::SpiMuxComponent::new(&peripherals.spi_host0, dynamic_deferred_caller)
            .finalize(components::spi_mux_component_static!(
                lowrisc::spi_host::SpiHost
            ));

    let spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        mux_spi,
        0,
        core_capsules::spi_controller::DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        lowrisc::spi_host::SpiHost
    ));

    peripherals.aes.initialise(
        dynamic_deferred_caller.register(&peripherals.aes).unwrap(), // Unwrap fail = dynamic deferred caller out of slots
    );

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // USB support is currently broken in the OpenTitan hardware
    // See https://github.com/lowRISC/opentitan/issues/2598 for more details
    // let usb = components::usb::UsbComponent::new(
    //     board_kernel,
    //     extra_capsules::usb::usb_user::DRIVER_NUM,
    //     &peripherals.usb,
    // )
    // .finalize(components::usb_component_static!(earlgrey::usbdev::Usb));

    // Kernel storage region, allocated with the storage_volume!
    // macro in common/utils.rs
    extern "C" {
        /// Beginning on the ROM region containing app images.
        static _sstorage: u8;
        static _estorage: u8;
    }

    // Flash setup memory protection for the ROM/Kernel
    // Only allow reads for this region, any other ops will cause an MP fault
    let mp_cfg = FlashMPConfig {
        read_en: true,
        write_en: false,
        erase_en: false,
        scramble_en: false,
        ecc_en: false,
        he_en: false,
    };

    // Allocate a flash protection region (associated cfg number: 0), for the code section.
    if let Err(e) = peripherals.flash_ctrl.mp_set_region_perms(
        &_manifest as *const u8 as usize,
        &_etext as *const u8 as usize,
        0,
        &mp_cfg,
    ) {
        debug!("Failed to set flash memory protection: {:?}", e);
    } else {
        // Lock region 0, until next system reset.
        if let Err(e) = peripherals.flash_ctrl.mp_lock_region_cfg(0) {
            debug!("Failed to lock memory protection config: {:?}", e);
        }
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

    let mux_flash = components::flash::FlashMuxComponent::new(&peripherals.flash_ctrl).finalize(
        components::flash_mux_component_static!(lowrisc::flash_ctrl::FlashCtrl),
    );

    // SipHash
    let sip_hash = static_init!(
        extra_capsules::sip_hash::SipHasher24,
        extra_capsules::sip_hash::SipHasher24::new(dynamic_deferred_caller)
    );
    sip_hash.initialise(
        dynamic_deferred_caller
            .register(sip_hash)
            .expect("dynamic deferred caller out of slots for sip_hash"),
    );
    SIPHASH = Some(sip_hash);

    // TicKV
    let tickv = components::tickv::TicKVComponent::new(
        sip_hash,
        &mux_flash,                                    // Flash controller
        lowrisc::flash_ctrl::FLASH_PAGES_PER_BANK - 1, // Region offset (End of Bank0/Use Bank1)
        // Region Size
        lowrisc::flash_ctrl::FLASH_PAGES_PER_BANK * lowrisc::flash_ctrl::PAGE_SIZE,
        flash_ctrl_read_buf, // Buffer used internally in TicKV
        page_buffer,         // Buffer used with the flash controller
    )
    .finalize(components::tickv_component_static!(
        lowrisc::flash_ctrl::FlashCtrl,
        extra_capsules::sip_hash::SipHasher24
    ));
    hil::flash::HasClient::set_client(&peripherals.flash_ctrl, mux_flash);
    sip_hash.set_client(tickv);
    TICKV = Some(tickv);

    let mux_kv = components::kv_system::KVStoreMuxComponent::new(tickv).finalize(
        components::kv_store_mux_component_static!(
            extra_capsules::tickv::TicKVStore<
                core_capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
                extra_capsules::sip_hash::SipHasher24<'static>,
            >,
            extra_capsules::tickv::TicKVKeyType,
        ),
    );

    let kv_store = components::kv_system::KVStoreComponent::new(mux_kv).finalize(
        components::kv_store_component_static!(
            extra_capsules::tickv::TicKVStore<
                core_capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
                extra_capsules::sip_hash::SipHasher24<'static>,
            >,
            extra_capsules::tickv::TicKVKeyType,
        ),
    );
    tickv.set_client(kv_store);

    let kv_driver = components::kv_system::KVDriverComponent::new(
        kv_store,
        board_kernel,
        extra_capsules::kv_driver::DRIVER_NUM,
    )
    .finalize(components::kv_driver_component_static!(
        extra_capsules::tickv::TicKVStore<
            core_capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
            extra_capsules::sip_hash::SipHasher24<'static>,
        >,
        extra_capsules::tickv::TicKVKeyType,
    ));

    let mux_otbn = crate::otbn::AccelMuxComponent::new(&peripherals.otbn)
        .finalize(otbn_mux_component_static!());

    let otbn = OtbnComponent::new(&mux_otbn).finalize(crate::otbn_component_static!());

    let otbn_rsa_internal_buf = static_init!([u8; 512], [0; 512]);

    // Use the OTBN to create an RSA engine
    if let Ok((rsa_imem_start, rsa_imem_length, rsa_dmem_start, rsa_dmem_length)) =
        crate::otbn::find_app(
            "otbn-rsa",
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            ),
        )
    {
        let rsa_hardware = static_init!(
            lowrisc::rsa::OtbnRsa<'static>,
            lowrisc::rsa::OtbnRsa::new(
                otbn,
                lowrisc::rsa::AppAddresses {
                    imem_start: rsa_imem_start,
                    imem_size: rsa_imem_length,
                    dmem_start: rsa_dmem_start,
                    dmem_size: rsa_dmem_length
                },
                otbn_rsa_internal_buf,
            )
        );
        peripherals.otbn.set_client(rsa_hardware);
        RSA_HARDWARE = Some(rsa_hardware);
    } else {
        debug!("Unable to find otbn-rsa, disabling RSA support");
    }

    // Convert hardware RNG to the Random interface.
    let entropy_to_random = static_init!(
        core_capsules::rng::Entropy32ToRandom<'static>,
        core_capsules::rng::Entropy32ToRandom::new(&peripherals.rng)
    );
    peripherals.rng.set_client(entropy_to_random);
    // Setup RNG for userspace
    let rng = static_init!(
        core_capsules::rng::RngDriver<'static>,
        core_capsules::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(core_capsules::rng::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    entropy_to_random.set_client(rng);

    const CRYPT_SIZE: usize = 7 * AES128_BLOCK_SIZE;

    let aes_source_buffer = static_init!([u8; 16], [0; 16]);
    let aes_dest_buffer = static_init!([u8; CRYPT_SIZE], [0; CRYPT_SIZE]);

    let ccm_mux = static_init!(
        virtual_aes_ccm::MuxAES128CCM<'static, earlgrey::aes::Aes<'static>>,
        virtual_aes_ccm::MuxAES128CCM::new(&peripherals.aes, dynamic_deferred_caller)
    );
    peripherals.aes.set_client(ccm_mux);
    ccm_mux.initialize_callback_handle(
        dynamic_deferred_caller.register(ccm_mux).unwrap(), // Unwrap fail = no deferred call slot available for ccm mux
    );

    let crypt_buf1 = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
    let ccm_client1 = static_init!(
        virtual_aes_ccm::VirtualAES128CCM<'static, earlgrey::aes::Aes<'static>>,
        virtual_aes_ccm::VirtualAES128CCM::new(ccm_mux, crypt_buf1)
    );
    ccm_client1.setup();
    // ccm_mux.set_client(ccm_client1);

    let aes = static_init!(
        extra_capsules::symmetric_encryption::aes::AesDriver<
            'static,
            virtual_aes_ccm::VirtualAES128CCM<'static, earlgrey::aes::Aes<'static>>,
        >,
        extra_capsules::symmetric_encryption::aes::AesDriver::new(
            ccm_client1,
            aes_source_buffer,
            aes_dest_buffer,
            board_kernel.create_grant(
                extra_capsules::symmetric_encryption::aes::DRIVER_NUM,
                &memory_allocation_cap
            )
        )
    );

    AES = Some(ccm_client1);

    #[cfg(test)]
    {
        use extra_capsules::sha256::Sha256Software;

        let sha_soft = static_init!(
            Sha256Software<'static>,
            Sha256Software::new(dynamic_deferred_caller)
        );
        sha_soft.initialize_callback_handle(dynamic_deferred_caller.register(sha_soft).unwrap());

        SHA256SOFT = Some(sha_soft);
    }

    hil::symmetric_encryption::AES128CCM::set_client(ccm_client1, aes);
    hil::symmetric_encryption::AES128::set_client(ccm_client1, aes);

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
        /// The start of the OpenTitan manifest
        static _manifest: u8;
    }

    let syscall_filter = static_init!(TbfHeaderFilterDefaultAllow, TbfHeaderFilterDefaultAllow {});
    let scheduler = components::sched::priority::PriorityComponent::new(board_kernel)
        .finalize(components::priority_component_static!());
    let watchdog = &peripherals.watchdog;

    let earlgrey = static_init!(
        EarlGrey,
        EarlGrey {
            gpio,
            led,
            console,
            alarm,
            hmac,
            sha,
            rng,
            lldb: lldb,
            i2c_master,
            spi_controller,
            aes,
            kv_driver,
            syscall_filter,
            scheduler,
            scheduler_timer,
            watchdog,
        }
    );

    let mut mpu_config = rv32i::epmp::PMPConfig::kernel_default();

    // The kernel stack, BSS and relocation data
    chip.pmp
        .allocate_kernel_region(
            &_sstack as *const u8,
            &_ezero as *const u8 as usize - &_sstack as *const u8 as usize,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        )
        .unwrap();
    // The kernel text, Manifest and vectors
    chip.pmp
        .allocate_kernel_region(
            &_manifest as *const u8,
            &_etext as *const u8 as usize - &_manifest as *const u8 as usize,
            mpu::Permissions::ReadExecuteOnly,
            &mut mpu_config,
        )
        .unwrap();
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
    // Access to the MMIO devices
    chip.pmp
        .allocate_kernel_region(
            0x4000_0000 as *const u8,
            0x900_0000,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        )
        .unwrap();

    chip.pmp.enable_kernel_mpu(&mut mpu_config);

    kernel::process::load_processes(
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

    (board_kernel, earlgrey, chip, peripherals)
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
        let (board_kernel, earlgrey, chip, _peripherals) = setup();

        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
    }
}

#[cfg(test)]
use kernel::platform::watchdog::WatchDog;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unsafe {
        let (board_kernel, earlgrey, _chip, peripherals) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&earlgrey);
        PERIPHERALS = Some(peripherals);
        MAIN_CAP = Some(&create_capability!(capabilities::MainLoopCapability));

        PLATFORM.map(|p| {
            p.watchdog().setup();
        });

        for test in tests {
            test();
        }
    }

    // Exit QEMU with a return code of 0
    crate::tests::semihost_command_exit_success()
}
