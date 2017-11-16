#![no_std]
#![no_main]
#![feature(asm,const_fn,lang_items,compiler_builtins_lib)]

extern crate cortexm3;
extern crate capsules;
extern crate compiler_builtins;
#[allow(unused_imports)]
#[macro_use(debug,static_init)]
extern crate kernel;
extern crate stm32;
extern crate stm32f1;

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::Platform;
use kernel::hil;
use kernel::hil::Controller;

#[macro_use]
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None, None, None, None];


/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NucleoF103 {
    console: &'static capsules::console::Console<'static, stm32::usart::USART>,
    alarm: &'static capsules::alarm::AlarmDriver<'static,
                                                 VirtualMuxAlarm<'static,
                                                                 stm32::timer::AlarmTimer>>,
    button: &'static capsules::button::Button<'static, stm32::gpio::GPIOPin>,
    gpio: &'static capsules::gpio::GPIO<'static, stm32::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, stm32::gpio::GPIOPin>,
    ipc: kernel::ipc::IPC,
}


/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for NucleoF103 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {

        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f1::init();

    stm32::rcc::RCC.setup_system_clock(stm32::rcc::SystemClockSource::PllInternalOscillatorAt64MHz);

    let mut chip = stm32f1::chip::STM32F1::new();

    let console = static_init!(
        capsules::console::Console<stm32::usart::USART>,
        capsules::console::Console::new(&stm32::usart::USART2,
                     115200,
                     &mut capsules::console::WRITE_BUF,
                     kernel::Grant::create()));
    hil::uart::UART::set_client(&stm32::usart::USART2, console);
    stm32::usart::USART2.specify_pins(&stm32::gpio::PA[3], &stm32::gpio::PA[2]);

    // Alarm
    let alarm_timer = &stm32::timer::TIMER2;
    let mux_alarm = static_init!(
        MuxAlarm<'static, stm32::timer::AlarmTimer>,
        MuxAlarm::new(alarm_timer));
    alarm_timer.configure(mux_alarm);
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, stm32::timer::AlarmTimer>,
        VirtualMuxAlarm::new(mux_alarm));
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, stm32::timer::AlarmTimer>>,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, kernel::Grant::create()));
    virtual_alarm1.set_client(alarm);

    // LEDs
    let led_pins = static_init!(
        [(&'static stm32::gpio::GPIOPin, capsules::led::ActivationMode); 1],
        [(&stm32::gpio::PA[5], capsules::led::ActivationMode::ActiveHigh)]);
    let led = static_init!(
        capsules::led::LED<'static, stm32::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins));

    // Buttons
    let button_pins = static_init!(
        [(&'static stm32::gpio::GPIOPin, capsules::button::GpioMode); 1],
        [(&stm32::gpio::PC[13], capsules::button::GpioMode::LowWhenPressed)]);
    let button = static_init!(
        capsules::button::Button<'static, stm32::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create()));
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static stm32::gpio::GPIOPin; 19],[
        // &stm32::gpio::PA[3], // D0 (RX)
        // &stm32::gpio::PA[2], // D1 (TX)
        &stm32::gpio::PA[10], // D2
        &stm32::gpio::PB[3],  // D3
        &stm32::gpio::PB[5],  // D4
        &stm32::gpio::PB[4],  // D5
        &stm32::gpio::PB[10], // D6
        &stm32::gpio::PA[8],  // D7
        &stm32::gpio::PA[9],  // D8
        &stm32::gpio::PC[7],  // D9
        &stm32::gpio::PB[6],  // D10
        &stm32::gpio::PA[7],  // D11
        &stm32::gpio::PA[6],  // D12
        // &stm32::gpio::PA[5], // D13 (LED)
        &stm32::gpio::PB[9],  // D14
        &stm32::gpio::PB[8],  // D15
        &stm32::gpio::PA[0],  // A0
        &stm32::gpio::PA[1],  // A1
        &stm32::gpio::PA[4],  // A2
        &stm32::gpio::PB[0],  // A3
        &stm32::gpio::PC[1],  // A4
        &stm32::gpio::PC[0],  // A5
        ]);
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, stm32::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins));
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let nucleo = NucleoF103 {
        console: console,
        alarm: alarm,
        button: button,
        gpio: gpio,
        led: led,
        ipc: kernel::ipc::IPC::new(),
    };

    nucleo.console.initialize();
    // Attach the kernel debug interface to this console
    let kc = static_init!(
        capsules::console::App,
        capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(nucleo.console), kc);

    debug!("Initialization complete. Entering main loop ...");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }
    kernel::process::load_processes(&_sapps as *const u8,
                                    &mut APP_MEMORY,
                                    &mut PROCESSES,
                                    FAULT_RESPONSE);
    kernel::main(&nucleo, &mut chip, &mut PROCESSES, &nucleo.ipc);
}
