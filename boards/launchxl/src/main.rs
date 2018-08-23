#![no_std]
#![no_main]
#![feature(lang_items, asm, panic_implementation)]

extern crate capsules;
extern crate cc26x2;
extern crate cortexm4;
extern crate fixedvec;

#[allow(unused_imports)]
#[macro_use(create_capability, debug, debug_gpio, static_init)]
extern crate kernel;

use capsules::virtual_uart::{UartDevice, UartMux};
use cc26x2::aon;
use cc26x2::prcm;
use kernel::capabilities;
use kernel::hil;

#[macro_use]
pub mod io;
#[allow(dead_code)]
mod i2c_tests;
pub mod rfcore_const;
pub mod rfcore_driver;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0xA000] = [0; 0xA000];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, UartDevice<'static>>,
    button: &'static capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
    >,
    rng: &'static capsules::rng::SimpleRng<'static, cc26x2::trng::Trng>,
    radio: &'static rfcore_driver::Radio,
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
            cc26x2::rfc::DRIVER_NUM => f(Some(self.radio)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x2::init();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Setup AON event defaults
    aon::AON.setup();

    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}

    prcm::Power::enable_domain(prcm::PowerDomain::Serial);

    while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26x2::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26x2::gpio::PORT[6],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26x2::gpio::PORT[7],
                capsules::led::ActivationMode::ActiveHigh
            ), // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONS
    let button_pins = static_init!(
        [(&'static cc26x2::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26x2::gpio::PORT[13],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
            (
                &cc26x2::gpio::PORT[14],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 1
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // UART

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        UartMux<'static>,
        UartMux::new(
            &cc26x2::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    hil::uart::UART::set_client(&cc26x2::uart::UART0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

    cc26x2::uart::UART0.initialize_and_set_pins(3, 2);

    let console = static_init!(
        capsules::console::Console<UartDevice>,
        capsules::console::Console::new(
            console_uart,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::uart::UART::set_client(console_uart, console);
    console.initialize();

    // Create virtual device for kernel debug.
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart,
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::UART::set_client(debugger_uart, debugger);

    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    // TODO(alevy): Enable I2C, but it's not used anywhere yet. We need a system
    // call driver
    cc26x2::i2c::I2C0.initialize_and_set_pins(5, 4);

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26x2::gpio::GPIOPin; 21],
        [
            &cc26x2::gpio::PORT[1],
            &cc26x2::gpio::PORT[8],
            &cc26x2::gpio::PORT[9],
            &cc26x2::gpio::PORT[10],
            &cc26x2::gpio::PORT[11],
            &cc26x2::gpio::PORT[12],
            &cc26x2::gpio::PORT[15],
            &cc26x2::gpio::PORT[16],
            &cc26x2::gpio::PORT[17],
            &cc26x2::gpio::PORT[18],
            &cc26x2::gpio::PORT[19],
            &cc26x2::gpio::PORT[20],
            &cc26x2::gpio::PORT[21],
            &cc26x2::gpio::PORT[22],
            &cc26x2::gpio::PORT[23],
            &cc26x2::gpio::PORT[24],
            &cc26x2::gpio::PORT[25],
            &cc26x2::gpio::PORT[26],
            &cc26x2::gpio::PORT[27],
            &cc26x2::gpio::PORT[30],
            &cc26x2::gpio::PORT[31],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &cc26x2::rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&cc26x2::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm1,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    virtual_alarm1.set_client(alarm);

    let rng = static_init!(
        capsules::rng::SimpleRng<'static, cc26x2::trng::Trng>,
        capsules::rng::SimpleRng::new(
            &cc26x2::trng::TRNG,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    cc26x2::trng::TRNG.set_client(rng);

    let radio = static_init!(
        rfcore_driver::Radio,
        rfcore_driver::Radio::new(&cc26x2::rfc::RFC)
    );

    radio.power_up();

    let launchxl = Platform {
        console,
        gpio,
        led,
        button,
        alarm,
        rng,
        radio,
    };

    let mut chip = cc26x2::chip::Cc26X2::new();

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    let ipc = &kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

    kernel::procs::load_processes(
        board_kernel,
        &cortexm4::syscall::SysCall::new(),
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    board_kernel.kernel_loop(&launchxl, &mut chip, Some(&ipc), &main_loop_capability);
}
