//! Board file for LowRISC OpenTitan RISC-V development platform.
//!
//! - <https://opentitan.org/>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_hmac::VirtualMuxHmac;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

#[allow(dead_code)]
mod aes_test;

pub mod io;
pub mod usb;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; 4] =
    [None, None, None, None];

static mut CHIP: Option<&'static ibex::chip::Ibex> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 16384] = [0; 16384];

// Force the emission of the `.apps` segment in the kernel elf image
// NOTE: This will cause the kernel to overwrite any existing apps when flashed!
#[used]
#[link_section = ".app.hack"]
static APP_HACK: u8 = 0;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct OpenTitan {
    led: &'static capsules::led::LED<'static, ibex::gpio::GpioPin>,
    gpio: &'static capsules::gpio::GPIO<'static, ibex::gpio::GpioPin>,
    console: &'static capsules::console::Console<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, ibex::timer::RvTimer<'static>>,
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
    usb: &'static capsules::usb::usb_user::UsbSyscallDriver<
        'static,
        capsules::usb::usbc_client::Client<'static, lowrisc::usbdev::Usb<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for OpenTitan {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::hmac::DRIVER_NUM => f(Some(self.hmac)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            capsules::usb::usb_user::DRIVER_NUM => f(Some(self.usb)),
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
    ibex::chip::configure_trap_handler();

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
        Some(&ibex::gpio::PORT[7]), // First LED
        None,
        None,
    );

    let chip = static_init!(ibex::chip::Ibex, ibex::chip::Ibex::new());
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.enable_plic_interrupts();
    // enable interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET + csr::mie::mie::mext::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &ibex::uart::UART0,
        230400,
        dynamic_deferred_caller,
    )
    .finalize(());

    // LEDs
    // Start with half on and half off
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        ibex::gpio::GpioPin,
        (
            &ibex::gpio::PORT[8],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[9],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[10],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[11],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[12],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[13],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[14],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &ibex::gpio::PORT[15],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ))
    .finalize(components::led_component_buf!(ibex::gpio::GpioPin));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            ibex::gpio::GpioPin,
            0 => &ibex::gpio::PORT[0],
            1 => &ibex::gpio::PORT[1],
            2 => &ibex::gpio::PORT[2],
            3 => &ibex::gpio::PORT[3],
            4 => &ibex::gpio::PORT[4],
            5 => &ibex::gpio::PORT[5],
            6 => &ibex::gpio::PORT[6],
            7 => &ibex::gpio::PORT[15]
        ),
    )
    .finalize(components::gpio_component_buf!(ibex::gpio::GpioPin));

    let alarm = &ibex::timer::TIMER;
    alarm.setup();

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, ibex::timer::RvTimer>,
        MuxAlarm::new(alarm)
    );
    hil::time::Alarm::set_client(&ibex::timer::TIMER, mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, ibex::timer::RvTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, ibex::timer::RvTimer>>,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_client(virtual_alarm_user, alarm);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(board_kernel, uart_mux).finalize(());

    let hmac_data_buffer = static_init!([u8; 64], [0; 64]);
    let hmac_dest_buffer = static_init!([u8; 32], [0; 32]);

    let mux_hmac = components::hmac::HmacMuxComponent::new(&ibex::hmac::HMAC).finalize(
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

    let usb = usb::UsbComponent::new(board_kernel).finalize(());

    debug!("OpenTitan initialisation complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }

    let opentitan = OpenTitan {
        gpio: gpio,
        led: led,
        console: console,
        alarm: alarm,
        hmac,
        lldb: lldb,
        usb,
    };

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&opentitan, chip, None, &main_loop_cap);
}
