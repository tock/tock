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

const UART_RTS: Option<Pin> = Some(Pin::P0_13);
const UART_TXD: Pin = Pin::P0_15;
const UART_CTS: Option<Pin> = Some(Pin::P0_17);
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

// Static reference to chip for panic dumps
static mut CHIP: Option<&'static nrf52840::chip::Chip> = None;

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
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            // left side of the USB plug
            0 => &nrf52840::gpio::PORT[Pin::P0_13],
            1 => &nrf52840::gpio::PORT[Pin::P0_15],
            2 => &nrf52840::gpio::PORT[Pin::P0_17],
            3 => &nrf52840::gpio::PORT[Pin::P0_20],
            4 => &nrf52840::gpio::PORT[Pin::P0_22],
            5 => &nrf52840::gpio::PORT[Pin::P0_24],
            6 => &nrf52840::gpio::PORT[Pin::P1_00],
            7 => &nrf52840::gpio::PORT[Pin::P0_09],
            8 => &nrf52840::gpio::PORT[Pin::P0_10],
            // right side of the USB plug
            9 => &nrf52840::gpio::PORT[Pin::P0_31],
            10 => &nrf52840::gpio::PORT[Pin::P0_29],
            11 => &nrf52840::gpio::PORT[Pin::P0_02],
            12 => &nrf52840::gpio::PORT[Pin::P1_15],
            13 => &nrf52840::gpio::PORT[Pin::P1_13],
            14 => &nrf52840::gpio::PORT[Pin::P1_10],
            // Below the PCB
            15 => &nrf52840::gpio::PORT[Pin::P0_26],
            16 => &nrf52840::gpio::PORT[Pin::P0_04],
            17 => &nrf52840::gpio::PORT[Pin::P0_11],
            18 => &nrf52840::gpio::PORT[Pin::P0_14],
            19 => &nrf52840::gpio::PORT[Pin::P1_11],
            20 => &nrf52840::gpio::PORT[Pin::P1_07],
            21 => &nrf52840::gpio::PORT[Pin::P1_01],
            22 => &nrf52840::gpio::PORT[Pin::P1_04],
            23 => &nrf52840::gpio::PORT[Pin::P1_02]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840::gpio::PORT[BUTTON_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_buf!(nrf52840::gpio::GPIOPin));

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        nrf52840::gpio::GPIOPin,
        (
            &nrf52840::gpio::PORT[LED1_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED2_R_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED2_G_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED2_B_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    ))
    .finalize(components::led_component_buf!(nrf52840::gpio::GPIOPin));

    let chip = static_init!(nrf52840::chip::Chip, nrf52840::chip::new());
    CHIP = Some(chip);

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
