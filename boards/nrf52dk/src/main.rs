#![no_std]
#![no_main]
#![feature(lang_items,drop_types_in_const,compiler_builtins_lib)]

extern crate cortexm4;
extern crate capsules;
extern crate compiler_builtins;
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate nrf52;

use kernel::{Chip, SysTick};

// The nRF52 DK LEDs (see back of board)
const LED1_PIN: usize = 17;
const LED2_PIN: usize = 18;
const LED3_PIN: usize = 19;
const LED4_PIN: usize = 20;

#[macro_use]
mod io;


unsafe fn load_process() -> &'static mut [Option<kernel::Process<'static>>] {
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    const NUM_PROCS: usize = 1;

    // how should the kernel respond when a process faults
    const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

    #[link_section = ".app_memory"]
    static mut APP_MEMORY: [u8; 8192] = [0; 8192];

    static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None];

    let mut apps_in_flash_ptr = &_sapps as *const u8;
    let mut app_memory_ptr = APP_MEMORY.as_mut_ptr();
    let mut app_memory_size = APP_MEMORY.len();
    for i in 0..NUM_PROCS {
        let (process, flash_offset, memory_offset) = kernel::Process::create(apps_in_flash_ptr,
                                                                             app_memory_ptr,
                                                                             app_memory_size,
                                                                             FAULT_RESPONSE);

        if process.is_none() {
            break;
        }

        PROCESSES[i] = process;
        apps_in_flash_ptr = apps_in_flash_ptr.offset(flash_offset as isize);
        app_memory_ptr = app_memory_ptr.offset(memory_offset as isize);
        app_memory_size -= memory_offset;
    }

    &mut PROCESSES
}

pub struct Platform {
    console: &'static capsules::console::Console<'static, nrf52::uart::UART>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf52::gpio::GPIOPin>,
    timer: &'static capsules::timer::TimerDriver<'static, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>>,
}


impl kernel::Platform for Platform {
    #[inline(never)]
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),
            3 => f(Some(self.timer)),
            8 => f(Some(self.led)),
            _ => f(None),
        }
    }
}

// this is called once crt0.s is loaded
#[no_mangle]
pub unsafe fn reset_handler() {
    nrf52::init();

    // GPIOs
    let gpio_pins = static_init!(
        [&'static nrf52::gpio::GPIOPin; 11],
        [&nrf52::gpio::PORT[1],  // Bottom left header on DK board
        &nrf52::gpio::PORT[2],  //   |
        &nrf52::gpio::PORT[3],  //   V
        &nrf52::gpio::PORT[4],  //
        &nrf52::gpio::PORT[5],  //
        &nrf52::gpio::PORT[6],  // -----
        &nrf52::gpio::PORT[16], //
        &nrf52::gpio::PORT[15], //
        &nrf52::gpio::PORT[14], //
        &nrf52::gpio::PORT[13], //
        &nrf52::gpio::PORT[12], //
        ],
        4 * 11);

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        224/8);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // LEDs
    let led_pins = static_init!(
        [(&'static nrf52::gpio::GPIOPin, capsules::led::ActivationMode); 4],
        [(&nrf52::gpio::PORT[LED1_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf52::gpio::PORT[LED2_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf52::gpio::PORT[LED3_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf52::gpio::PORT[LED4_PIN], capsules::led::ActivationMode::ActiveLow),
        ], 256/8);

    let led = static_init!(
        capsules::led::LED<'static, nrf52::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins),
        64/8);

    let alarm = &nrf52::rtc::RTC;
    alarm.start();
    let mux_alarm = static_init!(capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc>, capsules::virtual_alarm::MuxAlarm::new(&nrf52::rtc::RTC), 16);
    alarm.set_client(mux_alarm);


    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        capsules::timer::TimerDriver<'static, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>>,
        capsules::timer::TimerDriver::new(virtual_alarm1,
                         kernel::Container::create()),
                         12);
    virtual_alarm1.set_client(timer);

    nrf52::uart::UART0.configure(nrf52::pinmux::Pinmux::new(6), /*. tx  */
                                 nrf52::pinmux::Pinmux::new(8), /* rx  */
                                 nrf52::pinmux::Pinmux::new(7), /* cts */
                                 nrf52::pinmux::Pinmux::new(5)); /*. rts */
    let console = static_init!(
        capsules::console::Console<nrf52::uart::UART>,
        capsules::console::Console::new(&nrf52::uart::UART0,
                                        115200,
                                        &mut capsules::console::WRITE_BUF,
                                        kernel::Container::create()),
                                        224/8);
    kernel::hil::uart::UART::set_client(&nrf52::uart::UART0, console);
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(
        capsules::console::App,
        capsules::console::App::default(),
        480/8);
    kernel::debug::assign_console_driver(Some(console), kc);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();

    nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}

    let platform = Platform {
        console: console,
        led: led,
        gpio: gpio,
        timer: timer,
    };

    let mut chip = nrf52::chip::NRF52::new();
    chip.systick().reset();
    chip.systick().enable(true);

    debug!("Initialization complete. Entering main loop\r");
    kernel::main(&platform,
                 &mut chip,
                 load_process(),
                 &kernel::ipc::IPC::new());

}
