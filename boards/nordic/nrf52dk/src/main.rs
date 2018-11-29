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
#![feature(panic_implementation)]
#![deny(missing_docs)]
#![allow(unused)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(create_capability,debug, debug_verbose, debug_gpio, static_init)]
extern crate kernel;
extern crate cortexm4;
extern crate nrf52;
// extern crate nrf52dk_base;
extern crate nrf5x;

use capsules::virtual_uart::{UartDevice, UartMux};
use kernel::capabilities;
use kernel::hil;
use kernel::hil::uart::UART;
use nrf5x::pinmux::Pinmux;

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
// mod tests;

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

    let uicr = nrf52::uicr::Uicr::new();
    nrf52::nvmc::NVMC.erase_uicr();
    nrf52::nvmc::NVMC.configure_writeable();
    while !nrf52::nvmc::NVMC.is_ready() {}
    uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
    while !nrf52::nvmc::NVMC.is_ready() {}
    uicr.set_psel1_reset_pin(BUTTON_RST_PIN);


    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    nrf52::uart::UARTE0.initialize(
        nrf5x::pinmux::Pinmux::new(UART_TXD as u32),
        nrf5x::pinmux::Pinmux::new(UART_RXD as u32),
        nrf5x::pinmux::Pinmux::new(UART_CTS as u32),
        nrf5x::pinmux::Pinmux::new(UART_RTS as u32),
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        UartMux<'static>,
        UartMux::new(
            &nrf52::uart::UARTE0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );

    hil::uart::UART::set_client(&nrf52::uart::UARTE0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

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
    UART::set_client(console_uart, console);
    console.initialize()
}
