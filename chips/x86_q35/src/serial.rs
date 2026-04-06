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
use core::fmt;
use core::mem::MaybeUninit;

use x86::registers::io::Port;

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
use tock_registers::{
    register_bitfields, registers, FakeRegister, LocalRegisterCopy, NoAccess, Read, Safe, Write,
};

registers! {
    #![buses(Port)]
    serial_registers {
        0 => rx_buffer: u8 { Read },
        #[aliased]
        0 => tx_buffer: u8 { Write },
        #[aliased]
        0 => divisor_lsb: u8 { Read, Write },

        1 => interrupt_enable: IER::Register { Read, Write },
        #[aliased]
        1 => divisor_msb: u8 { Read, Write },

        2 => interrupt_id: IIR::Register { Read },
        // FCR bitfield not defined yet, so this uses u8
        #[aliased]
        2 => fifo_control: u8 { Write },

        3 => line_control_register: LCR::Register { Read, Write },
        4 => modem_control_register: u8 { Read, Write }, // MCR bitfield not defined yet, hence u8
        5 => line_status_register: LSR::Register { Read },
        6 => modem_status_register: u8 { Read }, // MSR bitfield not defined yet, hence u8
        7 => scratch: u8 { Read, Write },
    },
}

/// Fake versuon of the 8250 UART for use in unit tests.
pub struct Fake8250 {
    tx_buffer: Cell<Option<u8>>,
    lcr: Cell<LocalRegisterCopy<u8, LCR::Register>>,
    ier: Cell<LocalRegisterCopy<u8, IER::Register>>,
}

impl Fake8250 {
    pub fn new() -> Self {
        Self {
            tx_buffer: Cell::new(None),
            lcr: Cell::new(LocalRegisterCopy::new(0)),
            ier: Cell::new(LocalRegisterCopy::new(0)),
        }
    }
}

impl Fake8250 {
    /// Simulates sending a byte (empties the transmit buffer). Returns None if the transmit buffer
    /// was empty, or Some(byte) if a byte was in the buffer.
    pub fn simulate_tx(&self) -> Option<u8> {
        self.tx_buffer.take()
    }
}

impl serial_registers::Interface for &Fake8250 {
    type rx_buffer = FakeRegister<Self, u8, Safe, NoAccess>;
    fn rx_buffer(self) -> FakeRegister<Self, u8, Safe, NoAccess> {
        FakeRegister::new(self).on_read(|_| unimplemented!())
    }

    type tx_buffer = FakeRegister<Self, u8, NoAccess, Safe>;
    fn tx_buffer(self) -> FakeRegister<Self, u8, NoAccess, Safe> {
        FakeRegister::new(self).on_write(|s, byte| {
            assert!(
                !s.lcr.get().is_set(LCR::DLAB),
                "tried to write TX buffer with DLAB bit set"
            );
            assert!(
                s.tx_buffer.replace(Some(byte.get())).is_none(),
                "TX buffer overflow"
            );
        })
    }

    type divisor_lsb = FakeRegister<Self, u8, Safe, Safe>;
    fn divisor_lsb(self) -> FakeRegister<Self, u8, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|_| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }

    type interrupt_enable = FakeRegister<Self, IER::Register, Safe, Safe>;
    fn interrupt_enable(self) -> FakeRegister<Self, IER::Register, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|s| {
                assert!(
                    !s.lcr.get().is_set(LCR::DLAB),
                    "tried to read IER with DLAB bit set"
                );
                s.ier.get()
            })
            .on_write(|s, v| {
                assert!(
                    !s.lcr.get().is_set(LCR::DLAB),
                    "tried to write IER with DLAB bit set"
                );
                s.ier.set(v)
            })
    }

    type divisor_msb = FakeRegister<Self, u8, Safe, Safe>;
    fn divisor_msb(self) -> FakeRegister<Self, u8, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|_| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }

    type interrupt_id = FakeRegister<Self, IIR::Register, Safe, NoAccess>;
    fn interrupt_id(self) -> FakeRegister<Self, IIR::Register, Safe, NoAccess> {
        FakeRegister::new(self).on_read(|_| unimplemented!())
    }

    type fifo_control = FakeRegister<Self, u8, NoAccess, Safe>;
    fn fifo_control(self) -> FakeRegister<Self, u8, NoAccess, Safe> {
        FakeRegister::new(self).on_write(|_, _| unimplemented!())
    }

    type line_control_register = FakeRegister<Self, LCR::Register, Safe, Safe>;
    fn line_control_register(self) -> FakeRegister<Self, LCR::Register, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|_| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }

    type modem_control_register = FakeRegister<Self, u8, Safe, Safe>;
    fn modem_control_register(self) -> FakeRegister<Self, u8, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|_| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }

    type line_status_register = FakeRegister<Self, LSR::Register, Safe, NoAccess>;
    fn line_status_register(self) -> FakeRegister<Self, LSR::Register, Safe, NoAccess> {
        FakeRegister::new(self).on_read(|_| unimplemented!())
    }

    type modem_status_register = FakeRegister<Self, u8, Safe, NoAccess>;
    fn modem_status_register(self) -> FakeRegister<Self, u8, Safe, NoAccess> {
        FakeRegister::new(self).on_read(|_| unimplemented!())
    }

    type scratch = FakeRegister<Self, u8, Safe, Safe>;
    fn scratch(self) -> FakeRegister<Self, u8, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|_| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use kernel::deferred_call::initialize_deferred_call_state;
    use kernel::platform::chip::ThreadIdProvider;
    use std::boxed::Box;
    use std::mem::forget;
    use std::ptr;
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
    use std::thread_local;

    enum StdThreadId {}
    unsafe impl ThreadIdProvider for StdThreadId {
        fn running_thread_id() -> usize {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            thread_local![static THREAD_NUM: usize = COUNTER.fetch_update(Relaxed, Relaxed, |previous| previous.checked_add(1)).expect("too many threads")];
            THREAD_NUM.with(|&n| n)
        }
    }

    struct LeakedBox<T: ?Sized>(*mut T);
    impl<T: ?Sized> LeakedBox<T> {
        pub fn new(value: Box<T>) -> (Self, &'static mut T) {
            let value = Box::leak(value);
            (Self(ptr::from_mut(value)), value)
        }
        pub fn done(self, value: &'static mut T) {
            assert!(ptr::eq(self.0, ptr::from_mut(value)));
            let _ = unsafe { Box::from_raw(self.0) };
            forget(self)
        }
    }
    impl<T: ?Sized> Drop for LeakedBox<T> {
        fn drop(&mut self) {
            panic!("memory leak: LeakedBox dropped without calling done");
        }
    }

    struct Client(Cell<Option<(&'static mut [u8], usize, Result<(), ErrorCode>)>>);
    impl TransmitClient for Client {
        fn transmitted_buffer(
            &self,
            tx_buffer: &'static mut [u8],
            tx_len: usize,
            rval: Result<(), ErrorCode>,
        ) {
            self.0.set(Some((tx_buffer, tx_len, rval)));
        }
    }

    #[test]
    fn serial_port() {
        initialize_deferred_call_state::<StdThreadId>();
        let fake = Fake8250::new();
        let driver = SerialPort::new(&fake);
        let client = Client(Cell::new(None));
        driver.set_transmit_client(&client);
        let (leaked, buffer) = LeakedBox::new(Box::new(*b"hi") as _);
        driver.transmit_buffer(buffer, 2).unwrap();
        fake.simulate_tx().unwrap();
        driver.handle_tx_interrupt();
        fake.simulate_tx().unwrap();
        driver.handle_tx_interrupt();
        assert!(fake.simulate_tx().is_none());
        leaked.done(client.0.take().unwrap().0);
    }
}

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

pub type RealSerialPort<'a> = SerialPort<'a, serial_registers::Real<Port>>;

pub struct SerialPort<'a, R: serial_registers::Interface> {
    registers: R,

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

impl<R: serial_registers::Interface> SerialPort<'_, R> {
    fn new(registers: R) -> Self {
        Self {
            registers,
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
        }
    }

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
        let ier_value = self.registers.interrupt_enable().get();
        let mut ier: LocalRegisterCopy<u8, IER::Register> = LocalRegisterCopy::new(ier_value);
        ier.modify(IER::RDA::CLEAR);
        self.registers.interrupt_enable().set(ier.get());

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
            self.tx_buffer.map(|b| {
                self.registers.tx_buffer().set(b[tx_index]);
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
            self.rx_buffer.map(|b| {
                b[rx_index] = self.registers.rx_buffer().get();
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
            let iir_val = self.registers.interrupt_id().get();

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

impl<R: serial_registers::Interface> Configure for SerialPort<'_, R> {
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

        // Program line control, and set DLAB so we can program baud divisor
        self.registers.line_control_register().set(lcr.get());

        // Program the divisor and clear DLAB
        lcr.modify(LCR::DLAB::CLEAR);
        let divisor_bytes = divisor.to_le_bytes();
        self.registers.divisor_lsb().set(divisor_bytes[0]);
        self.registers.divisor_msb().set(divisor_bytes[1]);
        self.registers.line_control_register().set(lcr.get());

        // Disable FIFOs
        self.registers.fifo_control().set(0);

        // Read IIR once to clear any pending interrupts
        self.registers.interrupt_id().get();

        // Start with all interrupts disabled
        self.registers.interrupt_enable().set(0);

        Ok(())
    }
}

impl<'a, R: serial_registers::Interface> Transmit<'a> for SerialPort<'a, R> {
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
        self.registers.tx_buffer().set(tx_buffer[0]);

        self.tx_buffer.replace(tx_buffer);
        self.tx_len.set(tx_len);
        self.tx_index.set(1);

        // Enable TX interrupts
        let ier_value = self.registers.interrupt_enable().get();
        let mut ier: LocalRegisterCopy<u8, IER::Register> = LocalRegisterCopy::new(ier_value);
        ier.modify(IER::THRE::SET);
        self.registers.interrupt_enable().set(ier.get());

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

impl<'a, R: serial_registers::Interface> Receive<'a> for SerialPort<'a, R> {
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
        let ier_value = self.registers.interrupt_enable().get();
        let mut ier: LocalRegisterCopy<u8, IER::Register> = LocalRegisterCopy::new(ier_value);
        ier.modify(IER::RDA::SET);
        self.registers.interrupt_enable().set(ier.get());

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

impl<R: serial_registers::Interface> DeferredCallClient for SerialPort<'_, R> {
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
    type StaticInput =
        (&'static mut MaybeUninit<SerialPort<'static, serial_registers::Real<Port>>>,);
    type Output = &'static SerialPort<'static, serial_registers::Real<Port>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let serial = s.0.write(SerialPort::new(unsafe {
            serial_registers::Real::new(Port::new(self.base))
        }));

        // Deferred call registration
        serial.register();

        serial
    }
}

/// Serial port handle for blocking I/O
///
/// This struct is a lightweight version of [`SerialPort`] that can be used to perform blocking
/// serial I/O (via [`Write`] or [`IoWrite`]). It is intended for use in places where
/// interrupt-driven I/O is not possible, such as early bootstrapping or panic handling.
pub struct BlockingSerialPort<R: serial_registers::Interface>(R);

impl BlockingSerialPort<serial_registers::Real<Port>> {
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
        Self(unsafe { serial_registers::Real::new(Port::new(base)) })
    }
}

impl<R: serial_registers::Interface> fmt::Write for BlockingSerialPort<R> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl<R: serial_registers::Interface> IoWrite for BlockingSerialPort<R> {
    fn write(&mut self, buf: &[u8]) -> usize {
        for b in buf {
            // Wait for any pending transmission to complete
            loop {
                let line_status_value = self.0.line_status_register().get();
                let lsr: LocalRegisterCopy<u8, LSR::Register> =
                    LocalRegisterCopy::new(line_status_value);
                if lsr.is_set(LSR::THRE) {
                    break;
                }
            }

            self.0.tx_buffer().set(*b);
        }

        buf.len()
    }
}
