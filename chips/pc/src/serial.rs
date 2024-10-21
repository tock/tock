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
//! * https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming
//! * https://wiki.osdev.org/Serial_Ports
//! * https://docs.freebsd.org/en/articles/serial-uart/index.html

use core::cell::Cell;
use core::fmt::{self, Write};
use core::mem::MaybeUninit;

use x86::io;

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

/// Offset of the interrupt enable register, relative to the serial port base address.
const IER_REG_OFFSET: u16 = 1;

/// Bitmask for the "transmit hold register empty" flag of IER.
///
/// This flag, when set, causes the device to generate an interrupt when the transmit buffer is
/// empty and ready to receive another byte.
const IER_THRE_MASK: u8 = 0b0000_0010;

/// Bitmask for the "received data available" flag of IER.
///
/// This flag, when set, causes the device to generate an interrupt when the receive FIFO contains
/// data to be read.
const IER_RDA_MASK: u8 = 0b0000_0001;

/// Offset of the interrupt identification register, relative to serial port base address.
const IIR_REG_OFFSET: u16 = 2;

/// Bitmask for the interrupt pending flag of IIR.
///
/// This flag, when cleared, indicates that this serial port currently has an interrupt panding.
const IIR_REG_IP_MASK: u8 = 0b0000_0001;

/// Bitmask for the interrupt ID field of IIR.
///
/// When an interrupt is pending, this field identifies the cause interrupt cause. See the other
/// `IIR_REG_IID_*` constants for possible causes.
const IIR_REG_IID_MASK: u8 = 0b0000_1110;

/// Identifies the transmit buffer empty interrupt.
/// This interrupt clears on read.
const IIR_REG_IID_TBE: u8 = 0b0000_0010;

/// Identifies the "received data available" interrupt.
const IIR_REG_IID_RDA: u8 = 0b0000_0100;

/// Offset of the FIFO configuration register.
const FCR_OFFSET: u16 = 2;

/// Offset of the line control register, relative to serial port base address.
const LC_REG_OFFSET: u16 = 3;

/// Bitmask for DLAB field of line control register.
const LC_REG_DLAB_MASK: u8 = 0b1000_0000;

/// Offset of the line status register, relative to serial port base address.
const LS_REG_OFFSET: u16 = 5;

/// Bitmask for THRE field of line status register.
///
/// This flag, when set, indicates the transmit buffer is empty and another byte can be safely sent.
const LS_REG_THRE_MASK: u8 = 0b0010_0000;

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

impl<'a> SerialPort<'a> {
    /// Finishes out a long-running TX operation.
    fn finish_tx(&self, res: Result<(), ErrorCode>) {
        self.tx_buffer.take().map(|b| {
            self.tx_client.map(|c| {
                c.transmitted_buffer(b, self.tx_len.get(), res);
            });
        });
    }

    /// Finishes out a long-running RX operation.
    fn finish_rx(&self, res: Result<(), ErrorCode>, error: Error) {
        // Turn off RX interrupts
        unsafe {
            let mut ier = io::inb(self.base + IER_REG_OFFSET);
            ier &= !IER_RDA_MASK;
            io::outb(self.base + IER_REG_OFFSET, ier);
        }

        self.rx_buffer.take().map(|b| {
            self.rx_client
                .map(|c| c.received_buffer(b, self.rx_len.get(), res, error));
        });
    }

    /// Handler to call when a TX interrupt occurs.
    fn handle_tx_interrupt(&self) {
        if self.tx_index.get() < self.tx_len.get() {
            // Still have bytes to send
            let tx_index = self.tx_index.get();
            self.tx_buffer.map(|b| unsafe {
                io::outb(self.base, b[tx_index]);
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
                b[rx_index] = io::inb(self.base);
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
            let iir_val = unsafe { io::inb(self.base + IIR_REG_OFFSET) };

            if iir_val & IIR_REG_IP_MASK != 0 {
                // No interrupt pending for this port
                return;
            }

            match iir_val & IIR_REG_IID_MASK {
                IIR_REG_IID_TBE => self.handle_tx_interrupt(),
                IIR_REG_IID_RDA => self.handle_rx_interrupt(),
                _ => unimplemented!(),
            }
        }
    }
}

impl<'a> Configure for SerialPort<'a> {
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

        let mut lc_val = 0;

        lc_val |= match params.width {
            Width::Six => 0b0000_0001,
            Width::Seven => 0b0000_0010,
            Width::Eight => 0b0000_0011,
        };

        lc_val |= match params.stop_bits {
            StopBits::One => 0b0000_0000,
            StopBits::Two => 0b0000_0100,
        };

        lc_val |= match params.parity {
            Parity::None => 0b0000_0000,
            Parity::Odd => 0b0000_1000,
            Parity::Even => 0b0001_1000,
        };

        unsafe {
            // Program line control, and set DLAB so we can program baud divisor
            io::outb(self.base + LC_REG_OFFSET, lc_val | LC_REG_DLAB_MASK);

            // Program the divisor and clear DLAB
            let divisor_bytes = divisor.to_le_bytes();
            io::outb(self.base, divisor_bytes[0]);
            io::outb(self.base + 1, divisor_bytes[1]);
            io::outb(self.base + LC_REG_OFFSET, lc_val);

            // Disable FIFOs
            io::outb(self.base + FCR_OFFSET, 0);

            // Read IIR once to clear any pending interrupts
            let _ = io::inb(self.base + IIR_REG_OFFSET);

            // Start with all interrupts disabled
            io::outb(self.base + IER_REG_OFFSET, 0);
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
        unsafe { io::outb(self.base, tx_buffer[0]) };

        self.tx_buffer.replace(tx_buffer);
        self.tx_len.set(tx_len);
        self.tx_index.set(1);

        // Enable TX interrupts
        unsafe {
            let mut ier = io::inb(self.base + IER_REG_OFFSET);
            ier |= IER_THRE_MASK;
            io::outb(self.base + IER_REG_OFFSET, ier);
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
            let mut ier = io::inb(self.base + IER_REG_OFFSET);
            ier |= IER_RDA_MASK;
            io::outb(self.base + IER_REG_OFFSET, ier);
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

impl<'a> DeferredCallClient for SerialPort<'a> {
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
                    let line_status = io::inb(self.0 + LS_REG_OFFSET);
                    let thre_flag = line_status & LS_REG_THRE_MASK;
                    if thre_flag != 0 {
                        break;
                    }
                }

                io::outb(self.0, *b);
            }
        }

        buf.len()
    }
}
