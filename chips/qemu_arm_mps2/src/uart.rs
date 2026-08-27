// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! ARM CMSDK APB UART, as found on the MPS2 AN385/AN386 FPGA images.
//!
//! Documented in the Cortex-M System Design Kit Technical Reference Manual
//! (ARM DDI0479C). Unlike a 16550, this device has no FIFO: it holds a
//! single byte in each direction, signaled by the TXFULL/RXFULL bits in
//! `STATE`.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::io_write::IoWrite;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields};

use crate::SYSCLK_FRQ;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_4000 as *const UartRegisters) };
pub const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_5000 as *const UartRegisters) };
pub const UART2_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_6000 as *const UartRegisters) };
pub const UART3_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_7000 as *const UartRegisters) };
pub const UART4_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_9000 as *const UartRegisters) };

#[repr(C)]
pub struct UartRegisters {
    data: ReadWrite<u32, DATA::Register>,
    state: ReadWrite<u32, STATE::Register>,
    ctrl: ReadWrite<u32, CTRL::Register>,
    intstatus: ReadWrite<u32, INTSTATUS::Register>,
    bauddiv: ReadWrite<u32, BAUDDIV::Register>,
}

register_bitfields![u32,
    DATA [
        Data OFFSET(0) NUMBITS(8) [],
    ],
    STATE [
        TxFull OFFSET(0) NUMBITS(1) [],
        RxFull OFFSET(1) NUMBITS(1) [],
        TxOverrun OFFSET(2) NUMBITS(1) [],
        RxOverrun OFFSET(3) NUMBITS(1) [],
    ],
    CTRL [
        TxEn OFFSET(0) NUMBITS(1) [],
        RxEn OFFSET(1) NUMBITS(1) [],
        TxIntEn OFFSET(2) NUMBITS(1) [],
        RxIntEn OFFSET(3) NUMBITS(1) [],
        TxOverrunIntEn OFFSET(4) NUMBITS(1) [],
        RxOverrunIntEn OFFSET(5) NUMBITS(1) [],
    ],
    INTSTATUS [
        Tx OFFSET(0) NUMBITS(1) [],
        Rx OFFSET(1) NUMBITS(1) [],
        TxOverrun OFFSET(2) NUMBITS(1) [],
        RxOverrun OFFSET(3) NUMBITS(1) [],
    ],
    BAUDDIV [
        Div OFFSET(0) NUMBITS(20) [],
    ],
];

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
}

impl<'a> Uart<'a> {
    pub const fn new(registers: StaticRef<UartRegisters>) -> Uart<'a> {
        Uart {
            registers,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
        }
    }

    /// Disable the device and clear any pending interrupt state. Safe to
    /// call at any time; used both at construction and by the panic writer.
    fn reset(&self) {
        self.registers.ctrl.set(0);
        // Both are write-one-to-clear; STATE latches the overrun conditions
        // themselves, INTSTATUS the interrupts they raised.
        self.registers
            .state
            .write(STATE::TxOverrun::SET + STATE::RxOverrun::SET);
        self.registers.intstatus.write(
            INTSTATUS::Tx::SET
                + INTSTATUS::Rx::SET
                + INTSTATUS::TxOverrun::SET
                + INTSTATUS::RxOverrun::SET,
        );
    }

    pub fn handle_interrupt(&self) {
        let intstatus = self.registers.intstatus.extract();

        if intstatus.is_set(INTSTATUS::Tx) {
            self.registers.intstatus.write(INTSTATUS::Tx::SET);
            self.transmit_continue();
        }
        if intstatus.is_set(INTSTATUS::Rx) {
            self.registers.intstatus.write(INTSTATUS::Rx::SET);
            self.receive_continue();
        }
        // Overrun conditions: clear them so the interrupt doesn't retrigger.
        // There is no HIL-level overrun reporting for this simple device.
        if intstatus.is_set(INTSTATUS::TxOverrun) {
            self.registers.state.write(STATE::TxOverrun::SET);
            self.registers.intstatus.write(INTSTATUS::TxOverrun::SET);
        }
        if intstatus.is_set(INTSTATUS::RxOverrun) {
            self.registers.state.write(STATE::RxOverrun::SET);
            self.registers.intstatus.write(INTSTATUS::RxOverrun::SET);
        }
    }

    fn transmit_continue(&self) {
        let Some(tx_data) = self.tx_buffer.take() else {
            // Spurious: no transmission in progress (e.g. panic writer used
            // the device synchronously while an async transfer was live).
            return;
        };

        let mut index = self.tx_index.get();
        if index < self.tx_len.get() && !self.registers.state.is_set(STATE::TxFull) {
            self.registers
                .data
                .write(DATA::Data.val(tx_data[index] as u32));
            index += 1;
        }

        if index < self.tx_len.get() {
            self.tx_index.set(index);
            self.tx_buffer.replace(tx_data);
        } else {
            self.registers.ctrl.modify(CTRL::TxIntEn::CLEAR);
            self.tx_client
                .map(move |client| client.transmitted_buffer(tx_data, self.tx_len.get(), Ok(())));
        }
    }

    fn receive_continue(&self) {
        let Some(rx_buffer) = self.rx_buffer.take() else {
            return;
        };

        let len = self.rx_len.get();
        let mut index = self.rx_index.get();
        if index < len && self.registers.state.is_set(STATE::RxFull) {
            rx_buffer[index] = self.registers.data.read(DATA::Data) as u8;
            index += 1;
        }

        if index == len {
            self.registers.ctrl.modify(CTRL::RxIntEn::CLEAR);
            self.rx_client.map(move |client| {
                client.received_buffer(rx_buffer, len, Ok(()), hil::uart::Error::None)
            });
        } else {
            self.rx_index.set(index);
            self.rx_buffer.replace(rx_buffer);
        }
    }
}

impl hil::uart::Configure for Uart<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        // The hardware is fixed at 8 data bits, no parity, one stop bit, no
        // flow control; it cannot represent anything else.
        if params.width != hil::uart::Width::Eight
            || params.parity != hil::uart::Parity::None
            || params.stop_bits != hil::uart::StopBits::One
            || params.hw_flow_control
        {
            return Err(ErrorCode::NOSUPPORT);
        }

        if params.baud_rate == 0 {
            return Err(ErrorCode::INVAL);
        }
        // The divisor must fit BAUDDIV's 20-bit field, and the CMSDK UART TRM
        // (ARM DDI0479C) documents 16 as the minimum legal value.
        const BAUDDIV_MIN: u32 = 16;
        const BAUDDIV_MAX: u32 = (1 << 20) - 1;
        let bauddiv = SYSCLK_FRQ / params.baud_rate;
        if !(BAUDDIV_MIN..=BAUDDIV_MAX).contains(&bauddiv) {
            return Err(ErrorCode::INVAL);
        }
        self.registers.bauddiv.write(BAUDDIV::Div.val(bauddiv));
        self.registers
            .ctrl
            .modify(CTRL::TxEn::SET + CTRL::RxEn::SET);

        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 || tx_len > tx_data.len() {
            return Err((ErrorCode::SIZE, tx_data));
        }
        if self.tx_buffer.is_some() {
            return Err((ErrorCode::BUSY, tx_data));
        }

        self.registers.ctrl.modify(CTRL::TxIntEn::SET);

        let mut index = 0;
        if !self.registers.state.is_set(STATE::TxFull) {
            self.registers.data.write(DATA::Data.val(tx_data[0] as u32));
            index = 1;
        }

        self.tx_len.set(tx_len);
        self.tx_index.set(index);
        self.tx_buffer.replace(tx_data);

        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }
        if self.rx_buffer.is_some() {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);
        self.registers.ctrl.modify(CTRL::RxIntEn::SET);

        Ok(())
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

/// A synchronous, polling writer for panic messages.
///
/// This bypasses all interrupt-driven state above and is only ever used
/// from the panic handler.
pub struct UartPanicWriter<'a> {
    inner: Uart<'a>,
}

impl UartPanicWriter<'_> {
    fn transmit_sync(&self, bytes: &[u8]) {
        self.inner.registers.ctrl.modify(CTRL::TxIntEn::CLEAR);
        for byte in bytes {
            while self.inner.registers.state.is_set(STATE::TxFull) {}
            self.inner
                .registers
                .data
                .write(DATA::Data.val(*byte as u32));
        }
        while self.inner.registers.state.is_set(STATE::TxFull) {}
    }
}

impl IoWrite for UartPanicWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.transmit_sync(buf);
        buf.len()
    }
}

impl core::fmt::Write for UartPanicWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

pub struct UartPanicWriterConfig {
    pub base: StaticRef<UartRegisters>,
    pub params: hil::uart::Parameters,
}

impl kernel::platform::chip::PanicWriter for UartPanicWriter<'_> {
    type Config = UartPanicWriterConfig;

    unsafe fn create_panic_writer(config: Self::Config) -> impl IoWrite + core::fmt::Write {
        use hil::uart::Configure as _;

        let inner = Uart::new(config.base);
        inner.reset();
        let _ = inner.configure(config.params);
        UartPanicWriter { inner }
    }
}
