//! Board file for EK-TM4C1294XL development platform.

#![no_std]
#![no_main]
#![feature(asm, const_fn, lang_items)]
extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate tm4c129x;

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::hil;
use kernel::hil::Controller;
use kernel::Platform;

#[macro_use]
pub mod io;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static mut kernel::Process<'static>>; NUM_PROCS] =
    [None, None, None, None];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct EkTm4c1294xl {
    console: &'static capsules::console::Console<'static, tm4c129x::uart::UART>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, tm4c129x::gpt::AlarmTimer>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static, tm4c129x::gpio::GPIOPin>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static, tm4c129x::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, tm4c129x::gpio::GPIOPin>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for EkTm4c1294xl {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            _ => f(None),
        }
    }
}

/// Reset Handler
#[no_mangle]
pub unsafe fn reset_handler() {
    tm4c129x::init();

    tm4c129x::sysctl::PSYSCTLM
        .setup_system_clock(tm4c129x::sysctl::SystemClockSource::PllPioscAt120MHz);

    let console = static_init!(
        capsules::console::Console<tm4c129x::uart::UART>,
        capsules::console::Console::new(
            &tm4c129x::uart::UART0,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            kernel::Grant::create()
        )
    );
    hil::uart::UART::set_client(&tm4c129x::uart::UART0, console);
    tm4c129x::uart::UART0.specify_pins(&tm4c129x::gpio::PA[0], &tm4c129x::gpio::PA[1]);

    // Alarm
    let alarm_timer = &tm4c129x::gpt::TIMER0;
    let mux_alarm = static_init!(
        MuxAlarm<'static, tm4c129x::gpt::AlarmTimer>,
        MuxAlarm::new(alarm_timer)
    );
    alarm_timer.configure(mux_alarm);
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, tm4c129x::gpt::AlarmTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, tm4c129x::gpt::AlarmTimer>>,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, kernel::Grant::create())
    );
    virtual_alarm1.set_client(alarm);

    // LEDs
    let led_pins = static_init!(
        [(
            &'static tm4c129x::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 4],
        [
            (
                &tm4c129x::gpio::PF[0],
                capsules::led::ActivationMode::ActiveHigh
            ), // D1
            (
                &tm4c129x::gpio::PF[4],
                capsules::led::ActivationMode::ActiveHigh
            ), // D2
            (
                &tm4c129x::gpio::PN[0],
                capsules::led::ActivationMode::ActiveHigh
            ), // D3
            (
                &tm4c129x::gpio::PN[1],
                capsules::led::ActivationMode::ActiveHigh
            ), // D4
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, tm4c129x::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONs
    let button_pins = static_init!(
        [(&'static tm4c129x::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &tm4c129x::gpio::PJ[0],
                capsules::button::GpioMode::LowWhenPressed
            ), //USR_SW1
            (
                &tm4c129x::gpio::PJ[1],
                capsules::button::GpioMode::LowWhenPressed
            ), //USR_SW2
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, tm4c129x::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static tm4c129x::gpio::GPIOPin; 4],
        [
            &tm4c129x::gpio::PM[3],
            &tm4c129x::gpio::PH[2],
            &tm4c129x::gpio::PC[6],
            &tm4c129x::gpio::PC[7],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, tm4c129x::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let tm4c1294 = EkTm4c1294xl {
        console: console,
        alarm: alarm,
        gpio: gpio,
        ipc: kernel::ipc::IPC::new(),
        led: led,
        button: button,
    };

    let mut chip = tm4c129x::chip::Tm4c129x::new();

    tm4c1294.console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(tm4c1294.console), kc);

    debug!("Initialization complete. Entering main loop...\r");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }
    kernel::process::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
    kernel::main(&tm4c1294, &mut chip, &mut PROCESSES, Some(&tm4c1294.ipc));
}
