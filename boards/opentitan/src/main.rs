//! Board file for LowRISC OpenTitan RISC-V development platform.
//!
//! - <https://opentitan.org/>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_hmac::VirtualMuxHmac;
use earlgrey::chip::EarlGreyDefaultPeripherals;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::time::Alarm;
use kernel::Chip;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

#[allow(dead_code)]
mod aes_test;

#[allow(dead_code)]
mod multi_alarm_test;

pub mod io;
pub mod usb;

//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; 4] =
    [None, None, None, None];

static mut CHIP: Option<
    &'static earlgrey::chip::EarlGrey<
        VirtualMuxAlarm<'static, earlgrey::timer::RvTimer>,
        EarlGreyDefaultPeripherals,
    >,
> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct OpenTitan {
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
        VirtualMuxHmac<'static, lowrisc::hmac::Hmac<'static>, [u8; 32]>,
        [u8; 32],
    >,
    lldb: &'static capsules::low_level_debug::LowLevelDebug<
        'static,
        capsules::virtual_uart::UartDevice<'static>,
    >,
    i2c_master: &'static capsules::i2c_master::I2CMasterDriver<lowrisc::i2c::I2c<'static>>,
    nonvolatile_storage: &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for OpenTitan {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<Result<&dyn kernel::Driver, &dyn kernel::LegacyDriver>>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(Ok(self.led))),
            capsules::hmac::DRIVER_NUM => f(Some(Err(self.hmac))),
            capsules::gpio::DRIVER_NUM => f(Some(Err(self.gpio))),
            capsules::console::DRIVER_NUM => f(Some(Err(self.console))),
            capsules::alarm::DRIVER_NUM => f(Some(Err(self.alarm))),
            capsules::low_level_debug::DRIVER_NUM => f(Some(Err(self.lldb))),
            capsules::i2c_master::DRIVER_NUM => f(Some(Err(self.i2c_master))),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(Err(self.nonvolatile_storage)))
            }
            _ => f(None),
        }
    }
}

/// Reset Handler.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Basic setup of the platform.
    rv32i::init_memory();
    // Ibex-specific handler
    earlgrey::chip::configure_trap_handler();

    let peripherals = static_init!(
        EarlGreyDefaultPeripherals,
        EarlGreyDefaultPeripherals::new()
    );

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

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
            board_kernel.create_grant(&memory_allocation_cap)
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
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(board_kernel, uart_mux).finalize(());

    let hmac_data_buffer = static_init!([u8; 64], [0; 64]);
    let hmac_dest_buffer = static_init!([u8; 32], [0; 32]);

    let mux_hmac = components::hmac::HmacMuxComponent::new(&peripherals.hmac).finalize(
        components::hmac_mux_component_helper!(lowrisc::hmac::Hmac, [u8; 32]),
    );

    let hmac = components::hmac::HmacComponent::new(
        board_kernel,
        &mux_hmac,
        hmac_data_buffer,
        hmac_dest_buffer,
    )
    .finalize(components::hmac_component_helper!(
        lowrisc::hmac::Hmac,
        [u8; 32]
    ));

    let i2c_master = static_init!(
        capsules::i2c_master::I2CMasterDriver<lowrisc::i2c::I2c<'static>>,
        capsules::i2c_master::I2CMasterDriver::new(
            &peripherals.i2c,
            &mut capsules::i2c_master::BUF,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );

    peripherals.i2c.set_master_client(i2c_master);

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
    let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
        board_kernel,
        &peripherals.flash_ctrl,
        0x20000000,                       // Start address for userspace accessible region
        0x8000,                           // Length of userspace accessible region
        &_sstorage as *const u8 as usize, // Start address of kernel region
        &_estorage as *const u8 as usize - &_sstorage as *const u8 as usize, // Length of kernel region
    )
    .finalize(components::nv_storage_component_helper!(
        lowrisc::flash_ctrl::FlashCtrl
    ));

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
    }

    let opentitan = OpenTitan {
        gpio: gpio,
        led: led,
        console: console,
        alarm: alarm,
        hmac,
        lldb: lldb,
        i2c_master,
        nonvolatile_storage,
    };

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
        FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });
    debug!("OpenTitan initialisation complete. Entering main loop");

    let scheduler = components::sched::priority::PriorityComponent::new(board_kernel).finalize(());
    board_kernel.kernel_loop(&opentitan, chip, None, scheduler, &main_loop_cap);
}
