// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! PL011 UART driver for the MPS3-AN536.
//!
//! The MPS3-AN536 exposes two PL011 PrimeCell UARTs. UART0 is used as the
//! primary debug/console output.

use core::cell::Cell;
use core::fmt::Write;

use kernel::ErrorCode;
use kernel::hil::uart;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

register_structs! {
    pub ApbUartReg {
        /// UART data register.
        (0x00 => pub data: ReadWrite<u32, Data::Register>),
        /// UART state register.
        (0x04 => pub state: ReadOnly<u32, State::Register>),
        /// UART control register.
        (0x08 => pub ctrl: ReadWrite<u32, Ctrl::Register>),
        /// UART interrupt status register.
        (0x0c => pub intstatus: ReadWrite<u32, IntStatus::Register>),
        /// UART baud rate divider.
        (0x10 => pub bauddiv: ReadWrite<u32, BaudDiv::Register>),
        /// Reserved space before the peripheral identification registers.
        (0x14 => _reserved0),
        /// Peripheral identification registers.
        (0xfd0 => pub pid4: ReadOnly<u32>),
        (0xfd4 => pub pid5: ReadOnly<u32>),
        (0xfd8 => pub pid6: ReadOnly<u32>),
        (0xfdc => pub pid7: ReadOnly<u32>),
        (0xfe0 => pub pid0: ReadOnly<u32>),
        (0xfe4 => pub pid1: ReadOnly<u32>),
        (0xfe8 => pub pid2: ReadOnly<u32>),
        (0xfec => pub pid3: ReadOnly<u32>),
        /// Component identification registers.
        (0xff0 => pub cid0: ReadOnly<u32>),
        (0xff4 => pub cid1: ReadOnly<u32>),
        (0xff8 => pub cid2: ReadOnly<u32>),
        (0xffc => pub cid3: ReadOnly<u32>),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    Data [
        DATA OFFSET(0) NUMBITS(8)
    ],
    State [
        TXFULL OFFSET(0) NUMBITS(1),
        RXFULL OFFSET(1) NUMBITS(1),
        TXOVERRUN OFFSET(2) NUMBITS(1),
        RXOVERRUN OFFSET(3) NUMBITS(1)
    ],
    Ctrl [
        TX_EN OFFSET(0) NUMBITS(1),
        RX_EN OFFSET(1) NUMBITS(1),
        TX_INTEN OFFSET(2) NUMBITS(1),
        RX_INTEN OFFSET(3) NUMBITS(1),
        TXO_INTEN OFFSET(4) NUMBITS(1),
        RXO_INTEN OFFSET(5) NUMBITS(1),
        HSTEST OFFSET(6) NUMBITS(1)
    ],
    IntStatus [
        TX OFFSET(0) NUMBITS(1),
        RX OFFSET(1) NUMBITS(1),
        TXO OFFSET(2) NUMBITS(1),
        RXO OFFSET(3) NUMBITS(1)
    ],
    BaudDiv [
        BAUDDIV OFFSET(0) NUMBITS(16)
    ]
];

/// UARTE
// It should never be instanced outside this module but because a static mutable reference to it
// is exported outside this module it must be `pub`
pub struct ApbUart<'a> {
    clock: u32,
    reg: StaticRef<ApbUartReg>,

    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    tx_buffer: kernel::utilities::cells::TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,

    tx_position: Cell<usize>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    //rx_buffer: kernel::utilities::cells::TakeCell<'static, [u8]>,
}

impl<'a> ApbUart<'a> {
    /// Constructor
    // This should only be constructed once
    pub const fn new(base: u32, clock: u32) -> ApbUart<'a> {
        let reg = unsafe { StaticRef::new(base as *const ApbUartReg) };

        ApbUart {
            reg,
            clock,
            tx_client: OptionalCell::empty(),
            tx_buffer: kernel::utilities::cells::TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_position: Cell::new(0),
            rx_client: OptionalCell::empty(),
            //rx_buffer: kernel::utilities::cells::TakeCell::empty(),
        }
    }

    pub fn enable(&mut self, baud_rate: u32, clock: u32) -> () {
        let scaler: u32 = clock / baud_rate;

        self.reg.bauddiv.write(BaudDiv::BAUDDIV.val(scaler));
        self.reg.ctrl.write(Ctrl::TX_EN.val(1) + Ctrl::RX_EN.val(1));
    }

    fn tx_int_set(&self, on: bool) {
        let val = if on { 1 } else { 0 };
        self.reg
            .ctrl
            .modify(Ctrl::TX_INTEN.val(val) + Ctrl::TXO_INTEN.val(val));
    }

    fn _rx_int_set(&self, on: bool) {
        let val = if on { 1 } else { 0 };
        self.reg
            .ctrl
            .modify(Ctrl::RX_INTEN.val(val) + Ctrl::RXO_INTEN.val(val));
    }

    pub fn tx_int_get(&self) -> bool {
        let val = self.reg.intstatus.is_set(IntStatus::TX);
        if val {
            self.reg.intstatus.write(IntStatus::TX.val(1));
        }
        val
    }

    fn _rx_int_get(&self) -> bool {
        let val = self.reg.intstatus.is_set(IntStatus::RX);
        if val {
            self.reg.intstatus.write(IntStatus::RX.val(1));
        }
        val
    }

    /// UART interrupt handler that listens for both tx_end and rx_end events
    #[inline(never)]
    pub fn handle_tx_irq(&self) {
        if self.reg.intstatus.is_set(IntStatus::TXO) {
            sh::v!("OVF!");
        }

        self.reg.intstatus.write(IntStatus::TX.val(1));

        self.tx_buffer.map(|buf| {
            //sh::s!("#+");
            self.reg
                .data
                .write(Data::DATA.val(buf[self.tx_position.get()].into()));
            self.tx_position.set(self.tx_position.get() + 1);
        });

        if self.tx_position.get() == self.tx_len.get() {
            self.tx_client.map(|client| {
                if let Some(buf) = self.tx_buffer.take() {
                    client.transmitted_buffer(buf, self.tx_len.get(), Ok(()));
                }
            });
            self.tx_len.set(0);
            self.tx_int_set(false);
            //sh::s!("#/ ");
        }
    }

    pub fn handle_rx_irq(&self) {
        if self.rx_ready() {}
    }

    /// Check if the UART transmission is done
    pub fn tx_ready(&self) -> bool {
        !self.reg.state.is_set(State::TXFULL)
    }

    /// Check if either the rx_buffer is full or the UART has timed out
    pub fn rx_ready(&self) -> bool {
        !self.reg.state.is_set(State::RXFULL)
    }

    /// Blocking transmit of a byte slice, used for early debug output and
    /// panic handlers only.
    pub fn transmit_sync(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            while self.tx_ready() == false {}
            self.reg.data.write(Data::DATA.val(byte as u32));
        }

        while self.tx_ready() == false {}
    }

    pub fn ioprint(&mut self, s: &str) {
        self.transmit_sync(s.as_bytes());
    }
}

impl Write for ApbUart<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.ioprint(s);
        Ok(())
    }
}

impl uart::Configure for ApbUart<'_> {
    fn configure(&self, params: uart::Parameters) -> Result<(), ErrorCode> {
        if params.stop_bits != uart::StopBits::One
            || params.parity != uart::Parity::None
            || params.hw_flow_control
            || params.width != uart::Width::Eight
        {
            panic!(
                "Currently we only support uart setting of 115200bps 8N1, no hardware flow control"
            );
        }

        // TODO: Not good! this file should be generic and not rely on instance definition
        let scaler: u32 = self.clock / params.baud_rate;
        self.reg.bauddiv.write(BaudDiv::BAUDDIV.val(scaler));
        self.reg.ctrl.write(Ctrl::TX_EN.val(1) + Ctrl::RX_EN.val(1));

        Ok(())
    }
}

impl<'a> uart::Transmit<'a> for ApbUart<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.tx_len.get() == 0 {
            //sh::v!("TX<{}>", str::from_utf8(tx_data).unwrap());
            if tx_len <= tx_data.len() {
                self.tx_buffer.put(Some(tx_data));
                self.tx_len.set(tx_len);
                self.tx_int_get();
                self.tx_int_set(true);
                self.tx_buffer.map(|buf| {
                    self.reg.data.write(Data::DATA.val(buf[0].into()));
                    self.tx_position.set(1);
                });
                Ok(())
            } else {
                sh::v!("<TX SIZE>");
                Err((ErrorCode::SIZE, tx_data))
            }
        } else {
            sh::v!("<TX BUSY>");
            Err((ErrorCode::BUSY, tx_data))
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        //if self.tx_status.get() != USARTStateTX::Idle {
        //    self.disable_transmit_interrupt();
        //    self.tx_status.set(USARTStateTX::AbortRequested);
        //    self.deferred_call.set();
        Err(ErrorCode::BUSY)
        //} else {
        //    Ok(())
        //}
    }
}

impl<'a> uart::Receive<'a> for ApbUart<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        //if self.rx_status.get() == USARTStateRX::Idle {
        //    if rx_len <= rx_buffer.len() {
        //        self.rx_buffer.put(Some(rx_buffer));
        //        self.rx_position.set(0);
        //        self.rx_len.set(rx_len);
        //        self.rx_status.set(USARTStateRX::Receiving);
        //        self.enable_receive_interrupt();
        //        Ok(())
        //    } else {
        //        Err((ErrorCode::SIZE, rx_buffer))
        //    }
        //} else {
        Err((ErrorCode::BUSY, rx_buffer))
        //}
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        //if self.rx_status.get() != USARTStateRX::Idle {
        //    self.disable_receive_interrupt();
        //    self.rx_status.set(USARTStateRX::AbortRequested);
        //    self.deferred_call.set();
        Err(ErrorCode::BUSY)
        //} else {
        //    Ok(())
        //}
    }
}
