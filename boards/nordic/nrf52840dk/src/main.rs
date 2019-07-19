//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![deny(missing_docs)]

#[allow(unused_imports)]
use kernel::{debug, debug_gpio, debug_verbose, static_init};

use nrf52dk_base::{SpiMX25R6435FPins, SpiPins, UartPins};

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

const UART_RTS: usize = 5;
const UART_TXD: usize = 6;
const UART_CTS: usize = 7;
const UART_RXD: usize = 8;

const SPI_MOSI: usize = 20;
const SPI_MISO: usize = 21;
const SPI_CLK: usize = 19;

const SPI_MX25R6435F_CHIP_SELECT: usize = 17;
const SPI_MX25R6435F_WRITE_PROTECT_PIN: usize = 22;
const SPI_MX25R6435F_HOLD_PIN: usize = 23;

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

static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None, None, None, None, None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52::init();

    // GPIOs
    let gpio_pins = static_init!(
        [&'static kernel::hil::gpio::InterruptValuePin; 13],
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
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[10])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[9])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[8])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[7])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[6])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[5])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[1])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf5x::gpio::PORT[0])
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
        &Some(SpiMX25R6435FPins::new(
            SPI_MX25R6435F_CHIP_SELECT,
            SPI_MX25R6435F_WRITE_PROTECT_PIN,
            SPI_MX25R6435F_HOLD_PIN,
        )),
        button_pins,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
}
