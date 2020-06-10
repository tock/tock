//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.
//!
//! Pin Configuration
//! -------------------
//!
//! ### `GPIO`
//!
//! | #  | Pin   | Ix | Header | Arduino |
//! |----|-------|----|--------|---------|
//! | 0  | P1.01 | 33 | P3 1   | D0      |
//! | 1  | P1.02 | 34 | P3 2   | D1      |
//! | 2  | P1.03 | 35 | P3 3   | D2      |
//! | 3  | P1.04 | 36 | P3 4   | D3      |
//! | 4  | P1.05 | 37 | P3 5   | D4      |
//! | 5  | P1.06 | 38 | P3 6   | D5      |
//! | 6  | P1.07 | 39 | P3 7   | D6      |
//! | 7  | P1.08 | 40 | P3 8   | D7      |
//! | 8  | P1.10 | 42 | P4 1   | D8      |
//! | 9  | P1.11 | 43 | P4 2   | D9      |
//! | 10 | P1.12 | 44 | P4 3   | D10     |
//! | 11 | P1.13 | 45 | P4 4   | D11     |
//! | 12 | P1.14 | 46 | P4 5   | D12     |
//! | 13 | P1.15 | 47 | P4 6   | D13     |
//! | 14 | P0.26 | 26 | P4 9   | D14     |
//! | 15 | P0.27 | 27 | P4 10  | D15     |
//!
//! ### `GPIO` / Analog Inputs
//!
//! | #  | Pin        | Header | Arduino |
//! |----|------------|--------|---------|
//! | 16 | P0.03 AIN1 | P2 1   | A0      |
//! | 17 | P0.04 AIN2 | P2 2   | A1      |
//! | 18 | P0.28 AIN4 | P2 3   | A2      |
//! | 19 | P0.29 AIN5 | P2 4   | A3      |
//! | 20 | P0.30 AIN6 | P2 5   | A4      |
//! | 21 | P0.31 AIN7 | P2 6   | A5      |
//! | 22 | P0.02 AIN0 | P4 8   | AVDD    |
//!
//! ### Onboard Functions
//!
//! | Pin   | Header | Function |
//! |-------|--------|----------|
//! | P0.05 | P6 3   | UART RTS |
//! | P0.06 | P6 4   | UART TXD |
//! | P0.07 | P6 5   | UART CTS |
//! | P0.08 | P6 6   | UART RXT |
//! | P0.11 | P24 1  | Button 1 |
//! | P0.12 | P24 2  | Button 2 |
//! | P0.13 | P24 3  | LED 1    |
//! | P0.14 | P24 4  | LED 2    |
//! | P0.15 | P24 5  | LED 3    |
//! | P0.16 | P24 6  | LED 4    |
//! | P0.18 | P24 8  | Reset    |
//! | P0.19 | P24 9  | SPI CLK  |
//! | P0.20 | P24 10 | SPI MOSI |
//! | P0.21 | P24 11 | SPI MISO |
//! | P0.24 | P24 14 | Button 3 |
//! | P0.25 | P24 15 | Button 4 |

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use kernel::component::Component;
#[allow(unused_imports)]
use kernel::{debug, debug_gpio, debug_verbose, static_init};
use nrf52840::gpio::Pin;
use nrf52dk_base::{LoraPins, SpiMX25R6435FPins, SpiPins, UartChannel, UartPins};

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_13;
const LED2_PIN: Pin = Pin::P0_14;
const LED3_PIN: Pin = Pin::P0_15;
const LED4_PIN: Pin = Pin::P0_16;

// The nRF52840DK buttons (see back of board)
const BUTTON1_PIN: Pin = Pin::P0_11;
const BUTTON2_PIN: Pin = Pin::P0_12;
const BUTTON3_PIN: Pin = Pin::P0_24;
const BUTTON4_PIN: Pin = Pin::P0_25;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

// FIX: nRF52 SPI seems to work only when respective pins are P0_29,P0_30,P0_26
const SPI_MOSI: Pin = Pin::P0_20;
const SPI_MISO: Pin = Pin::P0_21;
const SPI_CLK: Pin = Pin::P0_19;

const SPI_MX25R6435F_CHIP_SELECT: Pin = Pin::P0_17;
const SPI_MX25R6435F_WRITE_PROTECT_PIN: Pin = Pin::P0_22;
const SPI_MX25R6435F_HOLD_PIN: Pin = Pin::P0_23;

// FIX: better assignments
const LORA_CHIP_SELECT: Pin = Pin::P1_01;
const LORA_RESET: Pin = Pin::P1_02;
const LORA_INT: Pin = Pin::P1_03;

/// Debug Writer
pub mod io;

// Whether to use UART debugging or Segger RTT (USB) debugging.
// - Set to false to use UART.
// - Set to true to use Segger RTT over USB.
const USB_DEBUGGING: bool = false;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut APP_MEMORY: [u8; 0x2C000] = [0; 0x2C000];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None, None, None, None, None];

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

    let uart_channel = if USB_DEBUGGING {
        // Initialize Segger RTT as early as possible so that any panic beyond this point can use the
        // RTT memory object.
        let mut rtt_memory_refs =
            components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());

        // XXX: This is inherently unsafe as it aliases the mutable reference to rtt_memory. This
        // aliases reference is only used inside a panic handler, which should be OK, but maybe we
        // should use a const reference to rtt_memory and leverage interior mutability instead.
        self::io::set_rtt_memory(&mut *rtt_memory_refs.get_rtt_memory_ptr());

        UartChannel::Rtt(rtt_memory_refs)
    } else {
        UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD))
    };

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            0 => &nrf52840::gpio::PORT[Pin::P1_01],
            1 => &nrf52840::gpio::PORT[Pin::P1_02],
            2 => &nrf52840::gpio::PORT[Pin::P1_03],
            3 => &nrf52840::gpio::PORT[Pin::P1_04],
            4 => &nrf52840::gpio::PORT[Pin::P1_05],
            5 => &nrf52840::gpio::PORT[Pin::P1_06],
            6 => &nrf52840::gpio::PORT[Pin::P1_07],
            7 => &nrf52840::gpio::PORT[Pin::P1_08],
            8 => &nrf52840::gpio::PORT[Pin::P1_10],
            9 => &nrf52840::gpio::PORT[Pin::P1_11],
            10 => &nrf52840::gpio::PORT[Pin::P1_12],
            11 => &nrf52840::gpio::PORT[Pin::P1_13],
            12 => &nrf52840::gpio::PORT[Pin::P1_14],
            13 => &nrf52840::gpio::PORT[Pin::P1_15],
            14 => &nrf52840::gpio::PORT[Pin::P0_26],
            15 => &nrf52840::gpio::PORT[Pin::P0_27]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840::gpio::PORT[BUTTON1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //13
            (
                &nrf52840::gpio::PORT[BUTTON2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //14
            (
                &nrf52840::gpio::PORT[BUTTON3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //15
            (
                &nrf52840::gpio::PORT[BUTTON4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ) //16
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
            &nrf52840::gpio::PORT[LED2_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED3_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED4_PIN],
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
        LED1_PIN,
        LED2_PIN,
        LED3_PIN,
        led,
        uart_channel,
        &SpiPins::new(SPI_MOSI, SPI_MISO, SPI_CLK),
        &Some(SpiMX25R6435FPins::new(
            SPI_MX25R6435F_CHIP_SELECT,
            SPI_MX25R6435F_WRITE_PROTECT_PIN,
            SPI_MX25R6435F_HOLD_PIN,
        )),
        button,
        true,
        &Some(LoraPins::new(LORA_CHIP_SELECT, LORA_RESET, LORA_INT)),
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        nrf52840::uicr::Regulator0Output::DEFAULT,
        false,
        chip,
    );
}
