// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! QEMU's memory mapped 16550 UART

use core::cell::Cell;

use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, Aliased, Field, InMemoryRegister, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

pub const UART0_BASE: StaticRef<Uart16550Registers> =
    unsafe { StaticRef::new(0x1000_0000 as *const Uart16550Registers) };

pub const UART_16550_BAUD_BASE: usize = 399193;

type Uart16550RegshiftInt = u8;

#[repr(C)]
pub struct Uart16550Registers {
    /// 0x00:
    /// - DLAB = 0
    ///   - Read: receiver buffer (RBR)
    ///   - Write: transmitter holding (THR)
    /// - DLAB = 1: divisor latch LSB (DLL)
    rbr_thr: Aliased<Uart16550RegshiftInt, RBR::Register, THR::Register>,

    /// 0x01:
    /// - DLAB = 0: interrupt enable (IER)
    /// - DLAB = 1: divisor latch MSB (DLM)
    ier: ReadWrite<Uart16550RegshiftInt, IER::Register>,

    /// 0x02:
    /// - Read: interrupt identification (IIR)
    /// - Write: FIFO control (FCR)
    iir_fcr: Aliased<Uart16550RegshiftInt, IIR::Register, FCR::Register>,

    /// 0x03: line control (LCR)
    lcr: ReadWrite<Uart16550RegshiftInt, LCR::Register>,

    /// 0x04: modem control (MCR)
    mcr: ReadWrite<Uart16550RegshiftInt, MCR::Register>,

    /// 0x05: line status (LSR)
    lsr: ReadOnly<Uart16550RegshiftInt, LSR::Register>,

    /// 0x06: modem status (MSR)
    msr: ReadOnly<Uart16550RegshiftInt, MSR::Register>,
}

impl Uart16550Registers {
    /// Access the DLL and DLM (divisor latch LSB and MSB) registers
    ///
    /// Setting the 7th bit of the line control register (LCR) changes
    /// the RBR/THR and IER to be the DLL and DLM register
    /// respectively.
    ///
    /// This function takes care of latching the registers and calling
    /// a closure in which this virtual register can be
    /// accessed. Prior to calling the closure, the register is
    /// latched back and the closure is instead provided with an
    /// in-memory copy of the register, which is then written back to
    /// DLL and DLM.
    ///
    /// This provides a safe way to access the DLL and DLM
    /// registers.
    fn divisor_latch_reg<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&InMemoryRegister<u16, DLR::Register>) -> R,
    {
        let dlab_field = Field::<Uart16550RegshiftInt, LCR::Register>::new(1, 7);

        // Set DLAB = 1
        self.lcr.modify(dlab_field.val(1));

        // Read the old values of rbr_thr and ier and combine them
        // into a u16
        let old_val = u16::from_be_bytes([self.ier.get(), self.rbr_thr.get()]);

        // Set DLAB = 0 prior to handing back to the caller
        self.lcr.modify(dlab_field.val(0));

        let dlr = InMemoryRegister::<u16, DLR::Register>::new(old_val);
        let ret = f(&dlr);

        // Get the bytes from the modified value
        let [new_ier, new_rbr_thr] = u16::to_be_bytes(dlr.get());

        // Set DLAB = 1
        self.lcr.modify(dlab_field.val(1));

        // Write the modified value back to the registers
        self.ier.set(new_ier as Uart16550RegshiftInt);
        self.rbr_thr.set(new_rbr_thr as Uart16550RegshiftInt);

        // Set DLAB = 0
        self.lcr.modify(dlab_field.val(0));

        ret
    }
}

register_bitfields![u8,
    RBR [
        Data OFFSET(0) NUMBITS(8) [],
    ],
    THR [
        Data OFFSET(0) NUMBITS(8) [],
    ],
    IER [
        ModemStatusRegisterChange OFFSET(3) NUMBITS(1) [],
        ReceiverLineStatusRegisterChange OFFSET(2) NUMBITS(1) [],
        TransmitterHoldingRegisterEmpty OFFSET(1) NUMBITS(1) [],
        ReceivedDataAvailable OFFSET(0) NUMBITS(1) [],
    ],
    IIR [
        FIFO OFFSET(6) NUMBITS(2) [
            NoFIFO = 0,
            UnusableFIFO = 2,
            FIFOEnabled = 3,
        ],
        Identification OFFSET(1) NUMBITS(3) [
            ModemStatusChange = 0,
            TransmitterHoldingRegisterEmpty = 1,
            ReceiveDataAvailable = 2,
            LineStatusChange = 3,
            CharacterTimeout = 6,
        ],
        Pending OFFSET(0) NUMBITS(1) [
            Pending = 0,
            NotPending = 1,
        ],
    ],
    FCR [
        ReceiveFIFOInterruptTriggerLevel OFFSET(6) NUMBITS(2) [
            Bytes1 = 0,
            Bytes4 = 1,
            Bytes8 = 2,
            Bytes14 = 3,
        ],
        DMAMode OFFSET(3) NUMBITS(1) [
            Mode0 = 0,
            Mode1 = 1,
        ],
        ClearTransmitFIFO OFFSET(2) NUMBITS(1) [],
        ClearReceiveFIFO OFFSET(1) NUMBITS(1) [],
        Enable OFFSET(0) NUMBITS(1) [],
    ],
    LCR [
        BreakSignal OFFSET(6) NUMBITS(1) [],
        ParityMode OFFSET(4) NUMBITS(2) [
            Odd = 0,
            Even = 1,
            High = 2,
            Low = 3,
        ],
        Parity OFFSET(3) NUMBITS(1) [],
        StopBits OFFSET(2) NUMBITS(1) [
            One = 0,
            OneHalfTwo = 1,
        ],
        DataWordLength OFFSET(0) NUMBITS(2) [
            Bits5 = 0,
            Bits6 = 1,
            Bits7 = 2,
            Bits8 = 3,
        ],
    ],
    MCR [
        LoopbackMode OFFSET(4) NUMBITS(1) [],
        AuxiliaryOutput2 OFFSET(3) NUMBITS(1) [],
        AuxiliaryOutput1 OFFSET(2) NUMBITS(1) [],
        RequestToSend OFFSET(1) NUMBITS(1) [],
        DataTerminalReady OFFSET(0) NUMBITS(1) [],
    ],
    LSR [
        ErronousDataInFIFO OFFSET(7) NUMBITS(1) [],
        THREmptyLineIdle OFFSET(6) NUMBITS(1) [],
        THREmpty OFFSET(5) NUMBITS(1) [],
        BreakSignalReceived OFFSET(4) NUMBITS(1) [],
        FramingError OFFSET(3) NUMBITS(1) [],
        ParityError OFFSET(2) NUMBITS(1) [],
        OverrunError OFFSET(1) NUMBITS(1) [],
        DataAvailable OFFSET(0) NUMBITS(1) [],
    ],
    MSR [
        CarrierDetect OFFSET(7) NUMBITS(1) [],
        RingIndicator OFFSET(6) NUMBITS(1) [],
        DataSetReady OFFSET(5) NUMBITS(1) [],
        ClearToSend OFFSET(4) NUMBITS(1) [],
        ChangeInCarrierDetect OFFSET(3) NUMBITS(1) [],
        TrailingEdgeRingIndicator OFFSET(2) NUMBITS(1) [],
        ChangeInDataSetReady OFFSET(1) NUMBITS(1) [],
        ChangeInClearToSend OFFSET(0) NUMBITS(1) [],
    ],
];

register_bitfields![u16,
    DLR [
        Divisor OFFSET(0) NUMBITS(16) [],
    ],
];

pub struct Uart16550<'a> {
    regs: StaticRef<Uart16550Registers>,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
}

impl<'a> Uart16550<'a> {
    pub fn new(regs: StaticRef<Uart16550Registers>) -> Uart16550<'a> {
        // Disable all interrupts when constructing the UART
        regs.ier.set(0xF);

        regs.iir_fcr.write(FCR::Enable::CLEAR);

        Uart16550 {
            regs,
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
}

impl Uart16550<'_> {
    pub fn handle_interrupt(&self) {
        // Currently we can only receive a tx interrupt, however we
        // need to check the interrupt cause nonetheless as this will
        // clear the TX interrupt bit
        let iir = self.regs.iir_fcr.extract();

        // Check if the register contained a valid interrupt at all
        if !iir.matches_all(IIR::Pending::Pending) {
            // There is no active interrupt. This happens on newer QEMU
            // versions, where a transient interrupt occurs whose underlying
            // interrupt condition clears on its own, but the PLIC still holds
            // the interrupt in the asserted / pending state.
            //
            // In this case, we simply return and ignore the interrupt. It
            // should already be cleared in the PLIC.
            return;
        }

        // Check whether there is space for new data
        if iir.matches_all(IIR::Identification::TransmitterHoldingRegisterEmpty) {
            // The respective interrupt has already been cleared by
            // the extraction of IIR

            // We also check whether the tx_buffer is set, as we
            // could've generated the interrupt by transmit_sync
            if self.tx_buffer.is_some() {
                self.transmit_continue();
            }
        }
        // Check whether we've received some new data.
        else if iir.matches_all(IIR::Identification::ReceiveDataAvailable) {
            self.receive();
        }
        // We don't care about MSC interrupts, but have to ack the
        // interrupt by reading MSR
        else if iir.matches_all(IIR::Identification::ModemStatusChange) {
            let _ = self.regs.msr.get();
        }
        // We don't care about LSC interrupts, but have to ack the
        // interrupt by reading LSR
        else if iir.matches_all(IIR::Identification::LineStatusChange) {
            let _ = self.regs.lsr.get();
        }
        // We con't care about character timeout interrupts, but CAN'T
        // SUPPRESS THEM and have to ack by reading RBR
        else if iir.matches_all(IIR::Identification::CharacterTimeout) {
            let _ = self.regs.rbr_thr.get();
        }
        // All other interrupt sources are unknown, panic if we see
        // them
        else {
            panic!("UART 16550: unknown interrupt");
        }
    }

    /// Blocking transmit
    ///
    /// This function will transmit the passed slice in a blocking
    /// fashing, returning when finished.
    ///
    /// The current device configuration is used, and the device must
    /// be enabled. Otherwise, this function may block indefinitely.
    pub fn transmit_sync(&self, bytes: &[u8]) {
        // We don't want to cause excessive interrupts here, so
        // disable transmit interrupts temporarily
        let prev_ier = self.regs.ier.extract();
        if prev_ier.is_set(IER::TransmitterHoldingRegisterEmpty) {
            self.regs
                .ier
                .modify_no_read(prev_ier, IER::TransmitterHoldingRegisterEmpty::CLEAR);
        }

        for byte in bytes.iter() {
            while !self.regs.lsr.is_set(LSR::THREmpty) {}
            self.regs.rbr_thr.write(THR::Data.val(*byte));
        }

        // Restore the IER register to its original state
        self.regs.ier.set(prev_ier.get());
    }

    fn transmit_continue(&self) {
        // This should only be called as a result of a transmit
        // interrupt

        // Get the current transmission information
        let mut index = self.tx_index.get();
        let tx_data = self.tx_buffer.take().expect("UART 16550: no tx buffer");

        if index < self.tx_len.get() {
            // Still data to send
            while index < self.tx_len.get() && self.regs.lsr.is_set(LSR::THREmpty) {
                self.regs.rbr_thr.write(THR::Data.val(tx_data[index]));
                index += 1;
            }

            // Put the updated index and buffer back, wait for an
            // interrupt
            self.tx_index.set(index);
            self.tx_buffer.replace(tx_data);
        } else {
            // We are finished, disable tx interrupts
            self.regs
                .ier
                .modify(IER::TransmitterHoldingRegisterEmpty::CLEAR);

            // Callback to the client
            self.tx_client
                .map(move |client| client.transmitted_buffer(tx_data, self.tx_len.get(), Ok(())));
        }
    }

    fn receive(&self) {
        // Receive interrupts must only be enabled when we're currently holding
        // a buffer to receive data into:
        let rx_buffer = self.rx_buffer.take().expect("UART 16550: no rx buffer");
        let len = self.rx_len.get();
        let mut index = self.rx_index.get();

        // Read in a while loop, until no more data in the FIFO
        while self.regs.lsr.is_set(LSR::DataAvailable) && index < len {
            rx_buffer[index] = self.regs.rbr_thr.get();
            index += 1;
        }

        // Check whether we've read sufficient data:
        if index == len {
            // We're done, disable interrupts and return to the client:
            self.regs.ier.modify(IER::ReceivedDataAvailable::CLEAR);

            self.rx_client.map(move |client| {
                client.received_buffer(rx_buffer, len, Ok(()), hil::uart::Error::None)
            });
        } else {
            // Store the new index and place the buffer back:
            self.rx_index.set(index);
            self.rx_buffer.replace(rx_buffer);
        }
    }
}

impl hil::uart::Configure for Uart16550<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        use hil::uart::{Parity, StopBits, Width};

        // 16550 operates at a default frequency of 115200. Dividing
        // this by the target frequency gives the divisor register
        // contents.
        let divisor: u16 = (115_200 / params.baud_rate) as u16;
        self.regs.divisor_latch_reg(|dlr| {
            dlr.set(divisor);
        });

        let mut lcr = self.regs.lcr.extract();

        match params.width {
            Width::Six => lcr.modify(LCR::DataWordLength::Bits6),
            Width::Seven => lcr.modify(LCR::DataWordLength::Bits7),
            Width::Eight => lcr.modify(LCR::DataWordLength::Bits8),
        }

        match params.stop_bits {
            StopBits::One => LCR::StopBits::One,
            // 1.5 stop bits for 5bit works, 2 stop bits for 6-8 bit
            // words. We only support 6-8 bit words, so this
            // configures 2 stop bits
            StopBits::Two => LCR::StopBits::OneHalfTwo,
        };

        match params.parity {
            Parity::None => lcr.modify(LCR::Parity.val(0b000)),
            Parity::Odd => lcr.modify(LCR::Parity.val(0b001)),
            Parity::Even => lcr.modify(LCR::Parity.val(0b011)),
        }

        match params.hw_flow_control {
            true => lcr.modify(LCR::BreakSignal::SET),
            false => lcr.modify(LCR::BreakSignal::CLEAR),
        }

        self.regs.lcr.set(lcr.get());

        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart16550<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len > tx_data.len() {
            return Err((ErrorCode::INVAL, tx_data));
        }

        if self.tx_buffer.is_some() {
            return Err((ErrorCode::BUSY, tx_data));
        }

        // Enable interrupts for the transmitter holding register
        // being empty such that we can callback to the client
        self.regs
            .ier
            .modify(IER::TransmitterHoldingRegisterEmpty::SET);

        // Start transmitting the first data word(s) already
        let mut index = 0;
        while index < tx_len && self.regs.lsr.is_set(LSR::THREmpty) {
            self.regs.rbr_thr.write(THR::Data.val(tx_data[index]));
            index += 1;
        }

        // Store the required buffer and information for the interrupt
        // handler
        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_index.set(index);

        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for Uart16550<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Ensure the provided buffer holds at least `rx_len` bytes, and
        // `rx_len` is strictly positive (otherwise we'd need to use deferred
        // calls):
        if rx_buffer.len() < rx_len && rx_len > 0 {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        // Store the receive buffer and byte count. We cannot call into the
        // generic receive routine here, as the client callback needs to be
        // called from another call stack. Hence simply enable interrupts here.
        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);

        // Enable receive interrupts:
        self.regs.ier.modify(IER::ReceivedDataAvailable::SET);

        Ok(())
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        // Currently unsupported as we'd like to avoid using deferred
        // calls. Needs to be migrated to the new UART HIL anyways.
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        // Currently unsupported.
        Err(ErrorCode::FAIL)
    }
}
