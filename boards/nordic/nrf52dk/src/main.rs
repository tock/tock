//! Tock kernel for the Nordic Semiconductor nRF52 development kit (DK), a.k.a. the PCA10040. </br>
//! It is based on nRF52838 SoC (Cortex M4 core with a BLE transceiver) with many exported
//! I/O and peripherals.
//!
//! nRF52838 has only one port and uses pins 0-31!
//!
//! Furthermore, there exist another a preview development kit for nRF52840 but it is not supported
//! yet because unfortunately the pin configuration differ from nRF52-DK whereas nRF52840 uses two
//! ports where port 0 has 32 pins and port 1 has 16 pins.
//!
//! Pin Configuration
//! -------------------
//!
//! ### `GPIOs`
//! * P0.27 -> (top left header)
//! * P0.26 -> (top left header)
//! * P0.02 -> (top left header)
//! * P0.25 -> (top left header)
//! * P0.24 -> (top left header)
//! * P0.23 -> (top left header)
//! * P0.22 -> (top left header)
//! * P0.12 -> (top mid header)
//! * P0.11 -> (top mid header)
//! * P0.03 -> (bottom right header)
//! * P0.04 -> (bottom right header)
//! * P0.28 -> (bottom right header)
//! * P0.29 -> (bottom right header)
//! * P0.30 -> (bottom right header)
//! * P0.31 -> (bottom right header)
//!
//! ### `LEDs`
//! * P0.17 -> LED1
//! * P0.18 -> LED2
//! * P0.19 -> LED3
//! * P0.20 -> LED4
//!
//! ### `Buttons`
//! * P0.13 -> Button1
//! * P0.14 -> Button2
//! * P0.15 -> Button3
//! * P0.16 -> Button4
//! * P0.21 -> Reset Button
//!
//! ### `UART`
//! * P0.05 -> RTS
//! * P0.06 -> TXD
//! * P0.07 -> CTS
//! * P0.08 -> RXD
//!
//! ### `NFC`
//! * P0.09 -> NFC1
//! * P0.10 -> NFC2
//!
//! ### `LFXO`
//! * P0.01 -> XL2
//! * P0.00 -> XL1
//!
//! Author
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * July 16, 2017

#![no_std]
#![no_main]
#![deny(missing_docs)]

#[allow(unused_imports)]
use kernel::{debug, debug_gpio, debug_verbose, static_init};

use nrf52dk_base::{SpiPins, UartPins};

// The nRF52 DK LEDs (see back of board)
const LED1_PIN: usize = 17;
const LED2_PIN: usize = 18;
const LED3_PIN: usize = 19;
const LED4_PIN: usize = 20;

// The nRF52 DK buttons (see back of board)
const BUTTON1_PIN: usize = 13;
const BUTTON2_PIN: usize = 14;
const BUTTON3_PIN: usize = 15;
const BUTTON4_PIN: usize = 16;
const BUTTON_RST_PIN: usize = 21;

const UART_RTS: usize = 5;
const UART_TXD: usize = 6;
const UART_CTS: usize = 7;
const UART_RXD: usize = 8;

const SPI_MOSI: usize = 22;
const SPI_MISO: usize = 23;
const SPI_CLK: usize = 24;

/// UART Writer
#[macro_use]
pub mod io;

// FIXME: Ideally this should be replaced with Rust's builtin tests by conditional compilation
//
// Also read the instructions in `tests` how to run the tests
#[allow(dead_code)]
mod tests;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52::init();

    let gpio_pins = static_init!(
        [&'static kernel::hil::gpio::InterruptValuePin; 12],
        [
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[3])
            )
            .finalize(), // Bottom right header on DK board
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[4])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[28])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[29])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[30])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[31])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[12])
            )
            .finalize(), // Top mid header on DK board
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[11])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[27])
            )
            .finalize(), // Top left header on DK board
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[26])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[2])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[25])
            )
            .finalize(),
        ]
    );

    // LEDs
    let led_pins = static_init!(
        [(
            &'static kernel::hil::gpio::Pin,
            capsules::led::ActivationMode
        ); 4],
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
        [(
            &'static kernel::hil::gpio::InterruptValuePin,
            capsules::button::GpioMode
        ); 4],
        [
            (
                static_init!(
                    kernel::hil::gpio::InterruptValueWrapper,
                    kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[BUTTON1_PIN])
                )
                .finalize(),
                capsules::button::GpioMode::LowWhenPressed
            ), // 13
            (
                static_init!(
                    kernel::hil::gpio::InterruptValueWrapper,
                    kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[BUTTON2_PIN])
                )
                .finalize(),
                capsules::button::GpioMode::LowWhenPressed
            ), // 14
            (
                static_init!(
                    kernel::hil::gpio::InterruptValueWrapper,
                    kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[BUTTON3_PIN])
                )
                .finalize(),
                capsules::button::GpioMode::LowWhenPressed
            ), // 15
            (
                static_init!(
                    kernel::hil::gpio::InterruptValueWrapper,
                    kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[BUTTON4_PIN])
                )
                .finalize(),
                capsules::button::GpioMode::LowWhenPressed
            ), // 16
        ]
    );

    for &(btn, _) in button_pins.iter() {
        btn.set_floating_state(kernel::hil::gpio::FloatingState::PullUp);
    }

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    nrf52dk_base::setup_board(
        board_kernel,
        BUTTON_RST_PIN,
        gpio_pins,
        LED1_PIN,
        LED2_PIN,
        LED3_PIN,
        led_pins,
        &UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD),
        &SpiPins::new(SPI_MOSI, SPI_MISO, SPI_CLK),
        &None,
        button_pins,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
}
