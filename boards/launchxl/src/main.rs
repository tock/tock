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
//mod components;
//use components::radio::RadioComponent;
use capsules::virtual_uart::{UartDevice, UartMux};
use cc26x2::aon;
use cc26x2::prcm;
use cc26x2::radio;
// use cc26x2::rtc;
use kernel::capabilities;
use kernel::hil;

#[macro_use]
pub mod io;
#[allow(dead_code)]
mod i2c_tests;
mod radio_tests;
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
    radio: &'static capsules::virtual_rfcore::VirtualRadioDriver<'static, cc26x2::radio::rfcore_driver::Radio>,
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
            capsules::virtual_rfcore::DRIVER_NUM => f(Some(self.radio)),
            _ => f(None),
        }
    }
}

static mut HELIUM_BUF: [u8; 128] = [0x00; 128];

/// Booster pack standard pinout
///
/// 1  -> 3v3
/// 2  -> DIO23 (analog)
/// 3  -> DIO3  (UARTRX)
/// 4  -> DIO2  (UARTTX)
/// 5  -> DIO22 (GPIO)
/// 6  -> DIO24 (analog)
/// 7  -> DIO10 (SPI CLK)
/// 8  -> DIO21 (GPIO)
/// 9  -> DIO4  (I2CSCL)
/// 10 -> DIO5  (I2CSDA)
///
/// 11 -> DIO15 (GPIO)
/// 12 -> DIO14 (SPI CS - other)
/// 13 -> DIO13 (SPI CS - display)
/// 14 -> DIO8  (SPI MISO)
/// 15 -> DIO9  (SPI MOSI)
/// 16 -> LPRST
/// 17 -> unused
/// 18 -> DIO11 (SPI CS - RF)
/// 19 -> DIO12 (PWM)
/// 20 -> GND
///
/// 21 -> 5v
/// 22 -> GND
/// 23 -> DIO25 (analog)
/// 24 -> DIO26 (analog)
/// 25 -> DIO17 (analog)
/// 26 -> DIO28 (analog)
/// 27 -> DIO29 (analog)
/// 28 -> DIO30 (analog)
/// 29 -> DIO0  (GPIO)
/// 30 -> DIO1  (GPIO)
///
/// 31 -> DIO17
/// 32 -> DIO16
/// 33 -> TMS
/// 34 -> TCK
/// 35 -> BPRST
/// 36 -> DIO18 (PWM)
/// 37 -> DIO19 (PWM)
/// 38 -> DIO20 (PWM)
/// 39 -> DIO6  (PWM)
/// 40 -> DIO7  (PWM)
///
unsafe fn configure_pins() {
    cc26x2::gpio::PORT[0].enable_gpio();
    cc26x2::gpio::PORT[1].enable_gpio();

    cc26x2::gpio::PORT[2].enable_uart_rx();
    cc26x2::gpio::PORT[3].enable_uart_tx();

    cc26x2::gpio::PORT[4].enable_i2c_scl();
    cc26x2::gpio::PORT[5].enable_i2c_sda();

    cc26x2::gpio::PORT[6].enable_gpio(); // Red LED
    cc26x2::gpio::PORT[7].enable_gpio(); // Green LED

    // SPI MISO cc26x2::gpio::PORT[8]
    // SPI MOSI cc26x2::gpio::PORT[9]
    // SPI CLK  cc26x2::gpio::PORT[10]
    // SPI CS   cc26x2::gpio::PORT[11]

    // PWM      cc26x2::gpio::PORT[12]

    cc26x2::gpio::PORT[13].enable_gpio();
    cc26x2::gpio::PORT[14].enable_gpio();

    cc26x2::gpio::PORT[15].enable_gpio();

    // unused   cc26x2::gpio::PORT[16]
    // unused   cc26x2::gpio::PORT[17]

    // PWM      cc26x2::gpio::PORT[18]
    // PWM      cc26x2::gpio::PORT[19]
    // PWM      cc26x2::gpio::PORT[20]

    cc26x2::gpio::PORT[21].enable_gpio();
    cc26x2::gpio::PORT[22].enable_gpio();

    // analog   cc26x2::gpio::PORT[23]
    // analog   cc26x2::gpio::PORT[24]
    // analog   cc26x2::gpio::PORT[25]
    // analog   cc26x2::gpio::PORT[26]
    // analog   cc26x2::gpio::PORT[27]
    // analog   cc26x2::gpio::PORT[28]
    // analog   cc26x2::gpio::PORT[29]
    // analog   cc26x2::gpio::PORT[30]
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

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    configure_pins();

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
            ), // Button 1
            (
                &cc26x2::gpio::PORT[14],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
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

    cc26x2::uart::UART0.initialize();

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
    cc26x2::i2c::I2C0.initialize();

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26x2::gpio::GPIOPin; 5],
        [
            // This is the order they appear on the launchxl headers.
            // Pins 5, 8, 11, 29, 30
            &cc26x2::gpio::PORT[22],
            &cc26x2::gpio::PORT[21],
            &cc26x2::gpio::PORT[15],
            &cc26x2::gpio::PORT[0],
            &cc26x2::gpio::PORT[1],
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
    
    let virtual_alarm2 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm2.set_client(alarm);

    let rng = static_init!(
        capsules::rng::SimpleRng<'static, cc26x2::trng::Trng>,
        capsules::rng::SimpleRng::new(
            &cc26x2::trng::TRNG,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    
    radio::RFC.set_client(&radio::RADIO);

    let virtual_radio = static_init!(
        capsules::virtual_rfcore::VirtualRadioDriver<'static, cc26x2::radio::rfcore_driver::Radio>,
        capsules::virtual_rfcore::VirtualRadioDriver::new(
            &cc26x2::radio::RADIO,
            board_kernel.create_grant(&memory_allocation_capability),
            &mut HELIUM_BUF
        )
    );

    kernel::hil::radio_client::RadioDriver::set_transmit_client(&radio::RADIO, virtual_radio);
    kernel::hil::radio_client::RadioDriver::set_receive_client(&radio::RADIO, virtual_radio, &mut HELIUM_BUF);

    let rfc = &cc26x2::radio::RADIO;
    rfc.test_power_up();

    let launchxl = Platform {
        console,
        gpio,
        led,
        button,
        alarm,
        rng,
        radio: virtual_radio,
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
