// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Universal asynchronous receiver/transmitter (UART)
//!
//! This is the legacy, non-EasyDMA UART peripheral. It shares the same
//! peripheral instance (and MMIO address) as the UARTE peripheral
//! ([`crate::uart::Uarte`]), so a chip can use one or the other but not both
//! at the same time.
//!
//! This driver only implements a synchronous, polling transmit path, and does
//! not use interrupts or implement [`kernel::hil::uart::Transmit`] or
//! [`kernel::hil::uart::Receive`]. It exists solely to back a
//! [`kernel::platform::chip::PanicWriter`] for boards that want to print
//! panic messages over this peripheral.
//!
//! See the nRF52840 Product Specification, UART chapter, for details:
//! <https://docs.nordicsemi.com/r/bundle/ps_nrf52840/page/uart.html>

use kernel::ErrorCode;
use kernel::hil::uart;
use kernel::utilities::StaticRef;
use kernel::utilities::io_write::IoWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_bitfields};
use nrf5x::gpio::Pin;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40002000 as *const UartRegisters) };

#[repr(C)]
pub struct UartRegisters {
    _reserved1: [u32; 1],
    task_stoprx: WriteOnly<u32, Task::Register>,
    task_starttx: WriteOnly<u32, Task::Register>,
    task_stoptx: WriteOnly<u32, Task::Register>,
    _reserved2: [u32; 67],
    event_txdrdy: ReadWrite<u32, Event::Register>,
    _reserved3: [u32; 248],
    enable: ReadWrite<u32, Enable::Register>,
    _reserved4: [u32; 1],
    pselrts: ReadWrite<u32, Psel::Register>,
    pseltxd: ReadWrite<u32, Psel::Register>,
    pselcts: ReadWrite<u32, Psel::Register>,
    pselrxd: ReadWrite<u32, Psel::Register>,
    _reserved5: [u32; 1],
    txd: WriteOnly<u32, Byte::Register>,
    _reserved6: [u32; 1],
    baudrate: ReadWrite<u32, Baudrate::Register>,
    _reserved7: [u32; 17],
    config: ReadWrite<u32, Config::Register>,
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Enable UART
    Enable [
        ENABLE OFFSET(0) NUMBITS(4) [
            ON = 4,
            OFF = 0
        ]
    ],

    /// Pin select
    Psel [
        // Pin number. MSB is actually the port indicator, but since we number
        // pins sequentially the binary representation of the pin number has
        // the port bit set correctly. So, for simplicity we just treat the
        // pin number as a 6 bit field.
        PIN OFFSET(0) NUMBITS(6) [],
        // Connect/Disconnect
        CONNECT OFFSET(31) NUMBITS(1) [
            Disconnected = 1,
            Connected = 0,
        ]
    ],

    /// A single data byte sent or received over the wire
    Byte [
        VALUE OFFSET(0) NUMBITS(8)
    ],

    /// Baudrate
    Baudrate [
        BAUDRATE OFFSET(0) NUMBITS(32)
    ],

    /// Configuration of parity and flow control
    Config [
        HWFC OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(3)
    ]
];

/// UART driver
pub struct Uart {
    registers: StaticRef<UartRegisters>,
}

impl Uart {
    pub fn new(regs: StaticRef<UartRegisters>) -> Uart {
        Uart { registers: regs }
    }

    fn initialize(&self, txd: Pin, rxd: Pin, cts: Option<Pin>, rts: Option<Pin>) {
        self.disable_uart();

        // Stop any ongoing TX or RX sequences.
        self.registers.task_stoptx.write(Task::ENABLE::SET);
        self.registers.task_stoprx.write(Task::ENABLE::SET);

        // Make sure we clear the txdrdy event in case it was set from the UARTE
        // version of the peripheral.
        self.registers.event_txdrdy.write(Event::READY::CLEAR);

        self.registers.pseltxd.write(Psel::PIN.val(txd as _));
        self.registers.pselrxd.write(Psel::PIN.val(rxd as _));
        if let Some(cts_pin) = cts {
            self.registers.pselcts.write(Psel::PIN.val(cts_pin as _));
        } else {
            // If no CTS pin is provided, then we need to mark it as
            // disconnected in the register.
            self.registers.pselcts.write(Psel::CONNECT::Disconnected);
        }
        if let Some(rts_pin) = rts {
            self.registers.pselrts.write(Psel::PIN.val(rts_pin as _));
        } else {
            // If no RTS pin is provided, then we need to mark it as
            // disconnected in the register.
            self.registers.pselrts.write(Psel::CONNECT::Disconnected);
        }

        self.enable_uart();
    }

    // The datasheet gives a non-exhaustive list of example settings for
    // typical bauds. The register is actually just a simple clock divider,
    // as explained and with implementation from:
    // https://devzone.nordicsemi.com/f/nordic-q-a/43280/technical-question-regarding-uart-baud-rate-generator-baudrate-register-offset-0x524
    //
    // This peripheral shares the same baud rate generator hardware as the
    // UARTE peripheral.
    fn get_divider_for_baud(&self, baud_rate: u32) -> Result<u32, ErrorCode> {
        if baud_rate > 1_000_000 || baud_rate < 1200 {
            return Err(ErrorCode::INVAL);
        }

        // force 64 bit values for precision
        let system_clock = 16000000u64; // TODO: Support dynamic clock
        let scalar = 32u64;
        let target_baud: u64 = baud_rate.into();

        // n.b. bits 11-0 are ignored by hardware
        let divider64 = (((target_baud << scalar) + (system_clock >> 1)) / system_clock) + 0x800;
        let divider = (divider64 & 0xffff_f000) as u32;

        Ok(divider)
    }

    fn set_baud_rate(&self, baud_rate: u32) -> Result<(), ErrorCode> {
        let divider = self.get_divider_for_baud(baud_rate)?;
        self.registers.baudrate.set(divider);

        Ok(())
    }

    fn enable_uart(&self) {
        self.registers.enable.write(Enable::ENABLE::ON);
    }

    fn disable_uart(&self) {
        self.registers.enable.write(Enable::ENABLE::OFF);
    }

    /// Transmit one byte at a time and the caller is responsible for polling
    /// [`Uart::tx_ready`]. This is used by the panic handler.
    pub fn send_byte(&self, byte: u8) {
        self.registers.event_txdrdy.write(Event::READY::CLEAR);
        self.registers.txd.write(Byte::VALUE.val(byte as u32));
        self.registers.task_starttx.write(Task::ENABLE::SET);

        while !self.registers.event_txdrdy.is_set(Event::READY) {}
    }
}

impl uart::Configure for Uart {
    fn configure(&self, params: uart::Parameters) -> Result<(), ErrorCode> {
        // These could probably be implemented, but are currently ignored, so
        // throw an error.
        if params.stop_bits != uart::StopBits::One {
            return Err(ErrorCode::NOSUPPORT);
        }
        if params.parity != uart::Parity::None {
            return Err(ErrorCode::NOSUPPORT);
        }
        if params.hw_flow_control {
            return Err(ErrorCode::NOSUPPORT);
        }

        self.set_baud_rate(params.baud_rate)?;

        Ok(())
    }
}

/// A synchronous writer for the nRF52 useful for panics.
///
/// For boards that want to use the UART to display panic messages, this
/// provides an implementation of
/// [`PanicWriter`](kernel::platform::chip::PanicWriter) with synchronous
/// output.
///
/// This is only to be used by panic messages and is not used within the normal
/// operation of the Tock kernel.
///
/// TODO: Validate this [`UartPanicWriter`] is always sound to create.
struct UartPanicWriter {
    inner: Uart,
}

impl IoWrite for UartPanicWriter {
    fn write(&mut self, buf: &[u8]) -> usize {
        for &c in buf {
            self.inner.send_byte(c);
        }
        buf.len()
    }
}

impl core::fmt::Write for UartPanicWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

/// Configuration for the synchronous UART panic writer.
///
/// This captures everything needed to setup the UART for panic display, even
/// if the normal kernel had initialized it differently.
pub struct UartPanicWriterConfig {
    pub params: uart::Parameters,
    pub txd: Pin,
    pub rxd: Pin,
    pub cts: Option<Pin>,
    pub rts: Option<Pin>,
}

impl kernel::platform::chip::PanicWriter for Uart {
    type Config = UartPanicWriterConfig;

    fn create_panic_writer(
        config: Self::Config,
        _panic: &core::panic::PanicInfo,
    ) -> impl IoWrite + core::fmt::Write {
        use uart::Configure as _;

        let inner = Uart::new(UART0_BASE);
        inner.initialize(config.txd, config.rxd, config.cts, config.rts);
        let _ = inner.configure(config.params);
        UartPanicWriter { inner }
    }
}
