//! Tock kernel for the Nordic Semiconductor nRF52840 dongle.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
#[allow(unused_imports)]
use kernel::{debug, debug_gpio, debug_verbose, static_init};
use nrf52840::gpio::Pin;
use nrf52dk_base::{SpiPins, UartChannel, UartPins};

// The nRF52840 Dongle LEDs
const LED1_PIN: Pin = Pin::P0_06;
const LED2_R_PIN: Pin = Pin::P0_08;
const LED2_G_PIN: Pin = Pin::P1_09;
const LED2_B_PIN: Pin = Pin::P0_12;

// The nRF52840 Dongle button
const BUTTON_PIN: Pin = Pin::P1_06;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Pin = Pin::P0_13;
const UART_TXD: Pin = Pin::P0_15;
const UART_CTS: Pin = Pin::P0_17;
const UART_RXD: Pin = Pin::P0_20;

const SPI_MOSI: Pin = Pin::P1_01;
const SPI_MISO: Pin = Pin::P1_02;
const SPI_CLK: Pin = Pin::P1_04;

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

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    // GPIOs
    let gpio = components::gpio::GpioComponent::new(board_kernel).finalize(
        components::gpio_component_helper!(
            // left side of the USB plug
            &nrf52840::gpio::PORT[Pin::P0_13],
            &nrf52840::gpio::PORT[Pin::P0_15],
            &nrf52840::gpio::PORT[Pin::P0_17],
            &nrf52840::gpio::PORT[Pin::P0_20],
            &nrf52840::gpio::PORT[Pin::P0_22],
            &nrf52840::gpio::PORT[Pin::P0_24],
            &nrf52840::gpio::PORT[Pin::P1_00],
            &nrf52840::gpio::PORT[Pin::P0_09],
            &nrf52840::gpio::PORT[Pin::P0_10],
            // right side of the USB plug
            &nrf52840::gpio::PORT[Pin::P0_31],
            &nrf52840::gpio::PORT[Pin::P0_29],
            &nrf52840::gpio::PORT[Pin::P0_02],
            &nrf52840::gpio::PORT[Pin::P1_15],
            &nrf52840::gpio::PORT[Pin::P1_13],
            &nrf52840::gpio::PORT[Pin::P1_10],
            // Below the PCB
            &nrf52840::gpio::PORT[Pin::P0_26],
            &nrf52840::gpio::PORT[Pin::P0_04],
            &nrf52840::gpio::PORT[Pin::P0_11],
            &nrf52840::gpio::PORT[Pin::P0_14],
            &nrf52840::gpio::PORT[Pin::P1_11],
            &nrf52840::gpio::PORT[Pin::P1_07],
            &nrf52840::gpio::PORT[Pin::P1_01],
            &nrf52840::gpio::PORT[Pin::P1_04],
            &nrf52840::gpio::PORT[Pin::P1_02]
        ),
    );
    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!((
            &nrf52840::gpio::PORT[BUTTON_PIN],
            capsules::button::GpioMode::LowWhenPressed
        )),
    );

    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
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
        )
    ));
    let chip = static_init!(nrf52840::chip::Chip, nrf52840::chip::new());

    nrf52dk_base::setup_board(
        board_kernel,
        BUTTON_RST_PIN,
        &nrf52840::gpio::PORT,
        gpio,
        LED2_R_PIN,
        LED2_G_PIN,
        LED2_B_PIN,
        led,
        UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD)),
        &SpiPins::new(SPI_MOSI, SPI_MISO, SPI_CLK),
        &None,
        button,
        true,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        nrf52840::uicr::Regulator0Output::V3_0,
        false,
        chip,
    );
}
