//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![feature(lang_items)]
#![deny(missing_docs)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, debug_verbose, debug_gpio, static_init)]
extern crate kernel;
extern crate nrf52;
extern crate nrf52dk_base;
extern crate nrf5x;

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: usize = 13;
const LED2_PIN: usize = 14;
const LED3_PIN: usize = 15;
const LED4_PIN: usize = 16;

// The nRF52840DK buttons (see back of board)
const BUTTON1_PIN: usize = 11;
const BUTTON2_PIN: usize = 12;
const BUTTON3_PIN: usize = 24;
const BUTTON4_PIN: usize = 25;
const BUTTON_RST_PIN: usize = 18;

/// UART Writer
#[macro_use]
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 245760] = [0; 245760];

static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] =
    [None, None, None, None, None, None, None, None];

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52::init();

    // GPIOs
    let gpio_pins = static_init!(
        [&'static nrf5x::gpio::GPIOPin; 13],
        [
            &nrf5x::gpio::PORT[3], // Bottom right header on DK board
            &nrf5x::gpio::PORT[4],
            &nrf5x::gpio::PORT[28],
            &nrf5x::gpio::PORT[29],
            &nrf5x::gpio::PORT[30],
            &nrf5x::gpio::PORT[10], // Top right header on DK board
            &nrf5x::gpio::PORT[9],
            &nrf5x::gpio::PORT[8],
            &nrf5x::gpio::PORT[7],
            &nrf5x::gpio::PORT[6],
            &nrf5x::gpio::PORT[5],
            &nrf5x::gpio::PORT[1],
            &nrf5x::gpio::PORT[0],
        ]
    );

    // LEDs
    let led_pins = static_init!(
        [(&'static nrf5x::gpio::GPIOPin, capsules::led::ActivationMode); 4],
        [
            (
                &nrf5x::gpio::PORT[LED1_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
            (
                &nrf5x::gpio::PORT[LED2_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
            (
                &nrf5x::gpio::PORT[LED3_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
            (
                &nrf5x::gpio::PORT[LED4_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
        ]
    );

    let button_pins = static_init!(
        [(&'static nrf5x::gpio::GPIOPin, capsules::button::GpioMode); 4],
        [
            (
                &nrf5x::gpio::PORT[BUTTON1_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 13
            (
                &nrf5x::gpio::PORT[BUTTON2_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 14
            (
                &nrf5x::gpio::PORT[BUTTON3_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 15
            (
                &nrf5x::gpio::PORT[BUTTON4_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 16
        ]
    );

    nrf52dk_base::setup_board(
        BUTTON_RST_PIN,
        gpio_pins,
        LED1_PIN,
        LED2_PIN,
        LED3_PIN,
        led_pins,
        button_pins,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
}
