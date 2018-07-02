#![no_std]
#![no_main]
#![feature(lang_items, asm, panic_implementation)]

extern crate capsules;
extern crate cortexm4;

extern crate cc26x2;
extern crate cc26xx;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;

use cc26xx::aon;
use cc26xx::prcm;

#[macro_use]
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] =
    [None, None];

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0xA000] = [0; 0xA000];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc26xx::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26xx::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, cc26xx::uart::UART>,
    button: &'static capsules::button::Button<'static, cc26xx::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26xx::rtc::Rtc>,
    >,
    rng: &'static capsules::rng::SimpleRng<'static, cc26xx::trng::Trng>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x2::init();

    // Setup AON event defaults
    aon::AON_EVENT.setup();

    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26xx::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26xx::gpio::PORT[6],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26xx::gpio::PORT[7],
                capsules::led::ActivationMode::ActiveHigh
            ), // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc26xx::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONS
    let button_pins = static_init!(
        [(&'static cc26xx::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26xx::gpio::PORT[13],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
            (
                &cc26xx::gpio::PORT[14],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 1
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc26xx::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // UART
    cc26xx::uart::UART0.set_pins(3, 2);
    let console = static_init!(
        capsules::console::Console<cc26xx::uart::UART>,
        capsules::console::Console::new(
            &cc26xx::uart::UART0,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            kernel::Grant::create()
        )
    );
    kernel::hil::uart::UART::set_client(&cc26xx::uart::UART0, console);
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(console), kc);

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26xx::gpio::GPIOPin; 22],
        [
            &cc26xx::gpio::PORT[1],
            &cc26xx::gpio::PORT[5],
            &cc26xx::gpio::PORT[8],
            &cc26xx::gpio::PORT[9],
            &cc26xx::gpio::PORT[10],
            &cc26xx::gpio::PORT[11],
            &cc26xx::gpio::PORT[12],
            &cc26xx::gpio::PORT[15],
            &cc26xx::gpio::PORT[16],
            &cc26xx::gpio::PORT[17],
            &cc26xx::gpio::PORT[18],
            &cc26xx::gpio::PORT[19],
            &cc26xx::gpio::PORT[20],
            &cc26xx::gpio::PORT[21],
            &cc26xx::gpio::PORT[22],
            &cc26xx::gpio::PORT[23],
            &cc26xx::gpio::PORT[24],
            &cc26xx::gpio::PORT[25],
            &cc26xx::gpio::PORT[26],
            &cc26xx::gpio::PORT[27],
            &cc26xx::gpio::PORT[30],
            &cc26xx::gpio::PORT[31],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc26xx::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &cc26xx::rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, cc26xx::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&cc26xx::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26xx::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26xx::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, kernel::Grant::create())
    );
    virtual_alarm1.set_client(alarm);

    let rng = static_init!(
        capsules::rng::SimpleRng<'static, cc26xx::trng::Trng>,
        capsules::rng::SimpleRng::new(&cc26xx::trng::TRNG, kernel::Grant::create())
    );
    cc26xx::trng::TRNG.set_client(rng);

    let launchxl = Platform {
        console,
        gpio,
        led,
        button,
        alarm,
        rng,
    };

    let mut chip = cc26x2::chip::Cc26X2::new();

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    kernel::kernel_loop(
        &launchxl,
        &mut chip,
        &mut PROCESSES,
        Some(&kernel::ipc::IPC::new()),
    );
}
