// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Support for legacy 8250-compatible serial ports.
//!
//! This module implements support for 8250-compatible UART devices. These are somewhat common on
//! x86 platforms and provide a simple interface for diagnostics and debugging.
//!
//! This implementation is based on guidance from the following sources:
//!
//! * <https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming>
//! * <https://wiki.osdev.org/Serial_Ports>
//! * <https://docs.freebsd.org/en/articles/serial-uart/index.html>

use core::cell::Cell;
use core::fmt::{self, Write};
use core::mem::MaybeUninit;

use x86::registers::io;

use kernel::component::Component;
use kernel::debug::IoWrite;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart::{
    Configure, Error, Parameters, Parity, Receive, ReceiveClient, StopBits, Transmit,
    TransmitClient, Width,
};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;
use tock_cells::take_cell::TakeCell;
use tock_registers::{register_bitfields, LocalRegisterCopy};

/// Base I/O port address of the standard COM1 serial device.
pub const COM1_BASE: u16 = 0x03F8;

/// Base I/O port address of the standard COM2 serial device.
pub const COM2_BASE: u16 = 0x02F8;

/// Base I/O port address of the standard COM3 serial device.
pub const COM3_BASE: u16 = 0x03E8;

/// Base I/O port address of the standard COM4 serial device.
pub const COM4_BASE: u16 = 0x02E8;

/// Fixed clock frequency used to generate baud rate on 8250-compatible UART devices.
const BAUD_CLOCK: u32 = 115_200;

/// The following offsets are relative to the base I/O port address of an 8250-compatible UART
/// device. Reference: <https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming>
mod offsets {
    /// Transmit Holding Register
    pub(crate) const THR: u16 = 0;

    /// Receive Buffer Register
    pub(crate) const RBR: u16 = 0;

    /// Divisor Latch Low Register
    pub(crate) const DLL: u16 = 0;

    /// Interrupt Enable Register
    pub(crate) const IER: u16 = 1;

    /// Divisor Latch High Register
    pub(crate) const DLH: u16 = 1;

    /// Interrupt Identification Register
    pub(crate) const IIR: u16 = 2;

    /// FIFO Control Register
    pub(crate) const FCR: u16 = 2;

    /// Line Control Register
    pub(crate) const LCR: u16 = 3;

    /// Line Status Register
    pub(crate) const LSR: u16 = 5;
}

register_bitfields!(u8,
    /// Interrupt Identification Register
    IIR[
        INTERRUPT_PENDING OFFSET(0) NUMBITS(1) [],
        INTERRUPT_ID OFFSET(1) NUMBITS(3) [
            THRE = 0b001, // Transmit Holding Register Empty
            RDA = 0b010   // Received Data Available
        ]
    ],

    /// Interrupt Enable Register
    IER [
        // IER: Interrupt Enable Register
        RDA OFFSET(0) NUMBITS(1) [], // Received Data Available
        THRE OFFSET(1) NUMBITS(1) [] // Transmit Holding Register Empty
    ],

    /// Line Control Register
    LCR [
        DLAB OFFSET(7) NUMBITS(1) [], // Divisor Latch Access Bit
        PARITY OFFSET(3) NUMBITS(3) [
            None = 0,
            Odd = 1,
            Even = 3,
            Mark = 5,
            Space = 7,
        ],
        STOP_BITS OFFSET(2) NUMBITS(1) [
            One = 0,
            Two = 1
        ],
        DATA_SIZE OFFSET(0) NUMBITS(2) [
            Five = 0,
            Six = 1,
            Seven = 2,
            Eight = 3
        ]
    ],

    /// Line Status Register
    LSR [
        THRE OFFSET(5) NUMBITS(1) [] // Transmit Holding Register Empty
    ],
);

pub struct SerialPort<'a> {
    /// Base I/O port address
    base: u16,

    /// Client of transmit operations
    tx_client: OptionalCell<&'a dyn TransmitClient>,

    /// Buffer of data to transmit
    tx_buffer: TakeCell<'static, [u8]>,

    /// Number of bytes to transmit from tx_buffer
    tx_len: Cell<usize>,

    /// Index of next byte within tx_buffer to be transmitted
    tx_index: Cell<usize>,

    /// Whether the currently pending transmit has been aborted
    tx_abort: Cell<bool>,

    /// Client of receive operations
    rx_client: OptionalCell<&'a dyn ReceiveClient>,

    /// Buffer for received data
    rx_buffer: TakeCell<'static, [u8]>,

    /// Number of bytes to receive into rx_buffer
    rx_len: Cell<usize>,

    /// Index of next byte within rx_buffer to be received
    rx_index: Cell<usize>,

    /// Whether the currently pending receive has been aborted
    rx_abort: Cell<bool>,

    /// Deferred call instance
    dc: DeferredCall,
}

impl SerialPort<'_> {
    /// Finishes out a long-running TX operation.
    fn finish_tx(&self, res: Result<(), ErrorCode>) {
        if let Some(b) = self.tx_buffer.take() {
            self.tx_client.map(|c| {
                c.transmitted_buffer(b, self.tx_len.get(), res);
            });
        }
    }

    /// Finishes out a long-running RX operation.
    fn finish_rx(&self, res: Result<(), ErrorCode>, error: Error) {
        // Turn off RX interrupts
        unsafe {
            let ier_value = io::inb(self.base + offsets::IER);
            let mut ier: LocalRegisterCopy<u8, IER::Register> = LocalRegisterCopy::new(ier_value);
            ier.modify(IER::RDA::CLEAR);
            io::outb(self.base + offsets::IER, ier.get());
        }

        if let Some(b) = self.rx_buffer.take() {
            self.rx_client
                .map(|c| c.received_buffer(b, self.rx_len.get(), res, error));
        }
    }

    /// Handler to call when a TX interrupt occurs.
    fn handle_tx_interrupt(&self) {
        if self.tx_index.get() < self.tx_len.get() {
            // Still have bytes to send
            let tx_index = self.tx_index.get();
            self.tx_buffer.map(|b| unsafe {
                io::outb(self.base + offsets::THR, b[tx_index]);
            });
            self.tx_index.set(tx_index + 1);
        } else {
            self.finish_tx(Ok(()));
        }
    }

    /// Handler to call when an RX interrupt occurs.
    fn handle_rx_interrupt(&self) {
        if self.rx_index.get() < self.rx_len.get() {
            // Still have bytes to receive
            let rx_index = self.rx_index.get();
            self.rx_buffer.map(|b| unsafe {
                b[rx_index] = io::inb(self.base + offsets::RBR);
            });
            self.rx_index.set(rx_index + 1);
        }

        if self.rx_index.get() == self.rx_len.get() {
            self.finish_rx(Ok(()), Error::None);
        }
    }

    /// Handler to call when a serial port interrupt is received.
    pub fn handle_interrupt(&self) {
        // There may be multiple interrupts pending for this device, but IIR will only show the
        // highest-priority one. So we need to read IIR in a loop until the Interrupt Pending flag
        // becomes set (indicating that there are no more pending interrupts).
        loop {
            let iir_val = unsafe { io::inb(self.base + offsets::IIR) };

            let iir: LocalRegisterCopy<u8, IIR::Register> = LocalRegisterCopy::new(iir_val);

            if iir.is_set(IIR::INTERRUPT_PENDING) {
                // No interrupt pending for this port
                return;
            }

            if iir.matches_all(IIR::INTERRUPT_ID::THRE) {
                self.handle_tx_interrupt();
            } else if iir.matches_all(IIR::INTERRUPT_ID::RDA) {
                self.handle_rx_interrupt();
            } else {
                unimplemented!();
            }
        }
    }
}

impl Configure for SerialPort<'_> {
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        if params.baud_rate == 0 {
            return Err(ErrorCode::INVAL);
        }

        if params.hw_flow_control {
            return Err(ErrorCode::NOSUPPORT);
        }

        let divisor = BAUD_CLOCK / params.baud_rate;
        if divisor == 0 || divisor > u16::MAX.into() {
            return Err(ErrorCode::NOSUPPORT);
        }

        // Compute value for the line control register

        let mut lcr: LocalRegisterCopy<u8, LCR::Register> = LocalRegisterCopy::new(0);

        lcr.modify(match params.width {
            Width::Six => LCR::DATA_SIZE::Six,
            Width::Seven => LCR::DATA_SIZE::Seven,
            Width::Eight => LCR::DATA_SIZE::Eight,
        });

        lcr.modify(match params.stop_bits {
            StopBits::One => LCR::STOP_BITS::One,
            StopBits::Two => LCR::STOP_BITS::Two,
        });

        lcr.modify(match params.parity {
            Parity::None => LCR::PARITY::None,
            Parity::Odd => LCR::PARITY::Odd,
            Parity::Even => LCR::PARITY::Even,
        });

        lcr.modify(LCR::DLAB::SET);

        unsafe {
            // Program line control, and set DLAB so we can program baud divisor
            io::outb(self.base + offsets::LCR, lcr.get());

            // Program the divisor and clear DLAB
            lcr.modify(LCR::DLAB::CLEAR);
            let divisor_bytes = divisor.to_le_bytes();
            io::outb(self.base + offsets::DLL, divisor_bytes[0]);
            io::outb(self.base + offsets::DLH, divisor_bytes[1]);
            io::outb(self.base + offsets::LCR, lcr.get());

            // Disable FIFOs
            io::outb(self.base + offsets::FCR, 0);

            // Read IIR once to clear any pending interrupts
            let _ = io::inb(self.base + offsets::IIR);

            // Start with all interrupts disabled
            io::outb(self.base + offsets::IER, 0);
        }

        Ok(())
    }
}

impl<'a> Transmit<'a> for SerialPort<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.tx_buffer.is_some() {
            return Err((ErrorCode::BUSY, tx_buffer));
        }

        if tx_len == 0 || tx_len > tx_buffer.len() {
            return Err((ErrorCode::SIZE, tx_buffer));
        }

        // Transmit the first byte
        unsafe { io::outb(self.base + offsets::THR, tx_buffer[0]) };

        self.tx_buffer.replace(tx_buffer);
        self.tx_len.set(tx_len);
        self.tx_index.set(1);

        // Enable TX interrupts
        unsafe {
            let ier_value = io::inb(self.base + offsets::IER);
            let mut ier: LocalRegisterCopy<u8, IER::Register> = LocalRegisterCopy::new(ier_value);
            ier.modify(IER::THRE::SET);
            io::outb(self.base + offsets::IER, ier.get());
        }

        Ok(())
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.tx_buffer.is_none() {
            return Ok(());
        }

        self.tx_abort.set(true);
        self.dc.set();

        Err(ErrorCode::BUSY)
    }
}

impl<'a> Receive<'a> for SerialPort<'a> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.rx_buffer.is_some() {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);

        // Enable RX interrupts
        unsafe {
            let ier_value = io::inb(self.base + offsets::IER);
            let mut ier: LocalRegisterCopy<u8, IER::Register> = LocalRegisterCopy::new(ier_value);
            ier.modify(IER::RDA::SET);
            io::outb(self.base + offsets::IER, ier.get());
        }

        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_buffer.is_none() {
            return Ok(());
        }

        self.rx_abort.set(true);
        self.dc.set();

        Err(ErrorCode::BUSY)
    }
}

impl DeferredCallClient for SerialPort<'_> {
    fn handle_deferred_call(&self) {
        if self.tx_abort.get() {
            self.finish_tx(Err(ErrorCode::CANCEL));
            self.tx_abort.set(false);
        }

        if self.rx_abort.get() {
            self.finish_rx(Err(ErrorCode::CANCEL), Error::None);
            self.rx_abort.set(false);
        }
    }

    fn register(&'static self) {
        self.dc.register(self);
    }
}

/// Component interface used to instantiate a [`SerialPort`]
pub struct SerialPortComponent {
    base: u16,
}

impl SerialPortComponent {
    /// Constructs and returns a new instance of `SerialPortComponent`.
    ///
    /// ## Safety
    ///
    /// An 8250-compatible serial port must exist at the specified address. Otherwise we could end
    /// up spamming some unknown device with I/O operations.
    ///
    /// The specified serial port must not be in use by any other instance of `SerialPort` or any
    /// other code.
    pub unsafe fn new(base: u16) -> Self {
        Self { base }
    }
}

impl Component for SerialPortComponent {
    type StaticInput = (&'static mut MaybeUninit<SerialPort<'static>>,);
    type Output = &'static SerialPort<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let serial = s.0.write(SerialPort {
            base: self.base,
            tx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            tx_abort: Cell::new(false),
            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            rx_abort: Cell::new(false),
            dc: DeferredCall::new(),
        });

        // Deferred call registration
        serial.register();

        serial
    }
}

/// Statically allocates the storage needed to finalize a [`SerialPortComponent`].
#[macro_export]
macro_rules! serial_port_component_static {
    () => {{
        (kernel::static_buf!($crate::serial::SerialPort<'static>),)
    }};
}

/// Serial port handle for blocking I/O
///
/// This struct is a lightweight version of [`SerialPort`] that can be used to perform blocking
/// serial I/O (via [`Write`] or [`IoWrite`]). It is intended for use in places where
/// interrupt-driven I/O is not possible, such as early bootstrapping or panic handling.
pub struct BlockingSerialPort(u16);

impl BlockingSerialPort {
    /// Creates and returns a new `BlockingSerialPort` instance.
    ///
    /// ## Safety
    ///
    /// An 8250-compatible serial port must exist at the specified address. Otherwise we could end
    /// up spamming some unknown device with I/O operations.
    ///
    /// For a given `base` address, there must be no other `SerialPort` or `BlockingSerialPort` in
    /// active use.
    pub unsafe fn new(base: u16) -> Self {
        Self(base)
    }
}

impl Write for BlockingSerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for BlockingSerialPort {
    fn write(&mut self, buf: &[u8]) -> usize {
        unsafe {
            for b in buf {
                // Wait for any pending transmission to complete
                loop {
                    let line_status_value = io::inb(self.0 + offsets::LSR);
                    let lsr: LocalRegisterCopy<u8, LSR::Register> =
                        LocalRegisterCopy::new(line_status_value);
                    if lsr.is_set(LSR::THRE) {
                        break;
                    }
                }

                io::outb(self.0 + offsets::THR, *b);
            }
        }

        buf.len()
    }
}
