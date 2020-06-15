#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(lang_items)]

extern crate capsules;
extern crate cc26x2;
extern crate cortexm4;
extern crate enum_primitive;

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};

use cc26x2::aon;
use cc26x2::prcm;
use cc26x2::pwm;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::entropy::Entropy32;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::rng::Rng;

pub mod io;

#[allow(dead_code)]
mod i2c_tests;
#[allow(dead_code)]
mod uart_echo;

// High frequency oscillator speed
pub const HFREQ: u32 = 48 * 1_000_000;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 3;
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None];

// Reference to chip for panic dumps.
static mut CHIP: Option<&'static cc26x2::chip::Cc26X2> = None;

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0x10000] = [0; 0x10000];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static>,
    button: &'static capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc<'static>>,
    >,
    rng: &'static capsules::rng::RngDriver<'static>,
    i2c_master: &'static capsules::i2c_master::I2CMasterDriver<cc26x2::i2c::I2CMaster<'static>>,
    ipc: kernel::ipc::IPC,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

mod cc1312r;
mod cc1352p;

pub struct Pinmap {
    uart0_rx: usize,
    uart0_tx: usize,
    i2c0_scl: usize,
    i2c0_sda: usize,
    red_led: usize,
    green_led: usize,
    button1: usize,
    button2: usize,
    gpio0: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    pwm0: usize,
    pwm1: usize,
}

unsafe fn configure_pins(pin: &Pinmap) {
    cc26x2::gpio::PORT[pin.uart0_rx].enable_uart0_rx();
    cc26x2::gpio::PORT[pin.uart0_tx].enable_uart0_tx();

    cc26x2::gpio::PORT[pin.i2c0_scl].enable_i2c_scl();
    cc26x2::gpio::PORT[pin.i2c0_sda].enable_i2c_sda();

    cc26x2::gpio::PORT[pin.red_led].enable_gpio();
    cc26x2::gpio::PORT[pin.green_led].enable_gpio();

    cc26x2::gpio::PORT[pin.button1].enable_gpio();
    cc26x2::gpio::PORT[pin.button2].enable_gpio();

    cc26x2::gpio::PORT[pin.gpio0].enable_gpio();

    cc26x2::gpio::PORT[pin.a7].enable_analog_input();
    cc26x2::gpio::PORT[pin.a6].enable_analog_input();
    cc26x2::gpio::PORT[pin.a5].enable_analog_input();
    cc26x2::gpio::PORT[pin.a4].enable_analog_input();
    cc26x2::gpio::PORT[pin.a3].enable_analog_input();
    cc26x2::gpio::PORT[pin.a2].enable_analog_input();
    cc26x2::gpio::PORT[pin.a1].enable_analog_input();
    cc26x2::gpio::PORT[pin.a0].enable_analog_input();

    cc26x2::gpio::PORT[pin.pwm0].enable_pwm(pwm::Timer::GPT0A);
    cc26x2::gpio::PORT[pin.pwm1].enable_pwm(pwm::Timer::GPT0B);
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

    // Power on Serial domain
    prcm::Power::enable_domain(prcm::PowerDomain::Serial);

    while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    let pinmap: &Pinmap;
    let chip_id = (cc26x2::rom::HAPI.get_chip_id)();

    if chip_id == cc1352p::CHIP_ID {
        pinmap = &cc1352p::PINMAP;
    } else {
        pinmap = &cc1312r::PINMAP;
    }

    configure_pins(pinmap);

    let chip = static_init!(cc26x2::chip::Cc26X2, cc26x2::chip::Cc26X2::new(HFREQ));
    CHIP = Some(chip);

    // LEDs
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        cc26x2::gpio::GPIOPin,
        (
            &cc26x2::gpio::PORT[pinmap.red_led],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ), // Red
        (
            &cc26x2::gpio::PORT[pinmap.green_led],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ) // Green
    ))
    .finalize(components::led_component_buf!(cc26x2::gpio::GPIOPin));

    // BUTTONS
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            cc26x2::gpio::GPIOPin,
            (
                &cc26x2::gpio::PORT[pinmap.button1],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            (
                &cc26x2::gpio::PORT[pinmap.button2],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_buf!(cc26x2::gpio::GPIOPin));

    // UART
    cc26x2::uart::UART0.initialize();
    let uart_mux = components::console::UartMuxComponent::new(
        &cc26x2::uart::UART0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    cc26x2::i2c::I2C0.initialize();

    let i2c_master = static_init!(
        capsules::i2c_master::I2CMasterDriver<cc26x2::i2c::I2CMaster<'static>>,
        capsules::i2c_master::I2CMasterDriver::new(
            &cc26x2::i2c::I2C0,
            &mut capsules::i2c_master::BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    cc26x2::i2c::I2C0.set_client(i2c_master);
    cc26x2::i2c::I2C0.enable();

    // Setup for remaining GPIO pins
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            cc26x2::gpio::GPIOPin,
            // This is the order they appear on the launchxl headers.
            // Pins 5, 8, 11, 29, 30
            0 => &cc26x2::gpio::PORT[pinmap.gpio0]
        ),
    )
    .finalize(components::gpio_component_buf!(cc26x2::gpio::GPIOPin));

    let rtc = &cc26x2::rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&cc26x2::rtc::RTC)
    );
    hil::time::Alarm::set_client(rtc, mux_alarm);

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
    hil::time::Alarm::set_client(virtual_alarm1, alarm);

    let entropy_to_random = static_init!(
        capsules::rng::Entropy32ToRandom<'static>,
        capsules::rng::Entropy32ToRandom::new(&cc26x2::trng::TRNG)
    );
    let rng = static_init!(
        capsules::rng::RngDriver<'static>,
        capsules::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    cc26x2::trng::TRNG.set_client(entropy_to_random);
    entropy_to_random.set_client(rng);

    let pwm_channels = [
        pwm::Signal::new(pwm::Timer::GPT0A),
        pwm::Signal::new(pwm::Timer::GPT0B),
        pwm::Signal::new(pwm::Timer::GPT1A),
        pwm::Signal::new(pwm::Timer::GPT1B),
        pwm::Signal::new(pwm::Timer::GPT2A),
        pwm::Signal::new(pwm::Timer::GPT2B),
        pwm::Signal::new(pwm::Timer::GPT3A),
        pwm::Signal::new(pwm::Timer::GPT3B),
    ];

    // all PWM channels are enabled
    for pwm_channel in pwm_channels.iter() {
        pwm_channel.enable();
    }

    let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

    let launchxl = Platform {
        console,
        gpio,
        led,
        button,
        alarm,
        rng,
        i2c_master,
        ipc,
    };

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }

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
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&launchxl, chip, Some(&launchxl.ipc), &main_loop_capability);
}
