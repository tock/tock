//! Tock kernel for the Nordic Semiconductor nRF52840 dongle.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![deny(missing_docs)]

#[allow(unused_imports)]
use kernel::{debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;
use nrf52dk_base::{SpiPins, UartPins};

// The nRF52840 Dongle LEDs
const LED1_PIN: usize = Pin::P0_06 as usize;
const LED2_R_PIN: usize = Pin::P0_08 as usize;
const LED2_G_PIN: usize = Pin::P1_09 as usize;
const LED2_B_PIN: usize = Pin::P0_12 as usize;

// The nRF52840 Dongle button
const BUTTON_PIN: usize = Pin::P1_06 as usize;
const BUTTON_RST_PIN: usize = Pin::P0_18 as usize;

const UART_RTS: usize = Pin::P0_13 as usize;
const UART_TXD: usize = Pin::P0_15 as usize;
const UART_CTS: usize = Pin::P0_17 as usize;
const UART_RXD: usize = Pin::P0_20 as usize;

const SPI_MOSI: usize = Pin::P1_01 as usize;
const SPI_MISO: usize = Pin::P1_02 as usize;
const SPI_CLK: usize = Pin::P1_04 as usize;

/// UART Writer
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 0x3C000] = [0; 0x3C000];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None, None, None, None, None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52840::init();

    // GPIOs
    let gpio_pins = static_init!(
        [&'static dyn kernel::hil::gpio::InterruptValuePin; 24],
        [
            // Left side of the USB plug
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_13 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_15 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_17 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_20 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_22 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_24 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_00 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_09 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_10 as usize]
                )
            )
            .finalize(),
            // Right side of the USB plug
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_31 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_29 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_02 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_15 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_13 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_10 as usize]
                )
            )
            .finalize(),
            // Below the PCB
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_26 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_04 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_11 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P0_14 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_11 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_07 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_01 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_04 as usize]
                )
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(
                    &nrf52840::gpio::PORT[Pin::P1_02 as usize]
                )
            )
            .finalize(),
        ]
    );

    // LEDs
    let led_pins = static_init!(
        [(
            &'static dyn kernel::hil::gpio::Pin,
            capsules::led::ActivationMode
        ); 4],
        [
            (
                &nrf52840::gpio::PORT[LED1_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
            (
                &nrf52840::gpio::PORT[LED2_R_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
            (
                &nrf52840::gpio::PORT[LED2_G_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
            (
                &nrf52840::gpio::PORT[LED2_B_PIN],
                capsules::led::ActivationMode::ActiveLow
            ),
        ]
    );

    let button_pins = static_init!(
        [(
            &'static dyn kernel::hil::gpio::InterruptValuePin,
            capsules::button::GpioMode
        ); 1],
        [(
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52840::gpio::PORT[BUTTON_PIN])
            )
            .finalize(),
            capsules::button::GpioMode::LowWhenPressed
        ),]
    );

    for &(btn, _) in button_pins.iter() {
        btn.set_floating_state(kernel::hil::gpio::FloatingState::PullUp);
    }

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    nrf52dk_base::setup_board(
        board_kernel,
        BUTTON_RST_PIN,
        &nrf52840::gpio::PORT,
        gpio_pins,
        LED2_R_PIN,
        LED2_G_PIN,
        LED2_B_PIN,
        led_pins,
        &UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD),
        &SpiPins::new(SPI_MOSI, SPI_MISO, SPI_CLK),
        &None,
        button_pins,
        true,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        nrf52840::uicr::Regulator0Output::V3_0,
        false,
    );
}
