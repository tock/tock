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
use kernel::threadlocal::{ThreadLocal, ThreadLocalDyn};

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

struct Uart16550Context {
    ier: Uart16550RegshiftInt,
}

impl Uart16550Context {
    fn empty() -> Uart16550Context {
        Uart16550Context { ier: 0 }
    }
}

pub struct Uart16550State<'a> {
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
    context: Uart16550Context,
}

impl<'a> Uart16550State<'a> {
    fn empty() -> Uart16550State<'a> {
        Uart16550State {
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            context: Uart16550Context::empty(),
        }
    }
}

static UART_16550_NO_STATE: ThreadLocal<0, Option<Uart16550State>> = ThreadLocal::new([]);

static mut UART_16550_STATE: &'static dyn ThreadLocalDyn<Option<Uart16550State>> = &UART_16550_NO_STATE;

pub unsafe fn set_global_uart_state(
    uart_state: &'static dyn ThreadLocalDyn<Option<Uart16550State>>
) {
    *core::ptr::addr_of_mut!(UART_16550_STATE) = uart_state;
}

pub fn init_uart_state() {
    let closure = |state: &mut Option<Uart16550State>| {
        let _ = state.replace(Uart16550State::empty());
    };

    unsafe {
        let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(UART_16550_STATE);
        threadlocal
            .get_mut()
            .expect("Current thread does not have access to its local UART state")
            .enter_nonreentrant(closure);
    }
}

unsafe fn with_uart_state<R, F>(f: F) -> Option<R>
where
    F: FnOnce(&mut Uart16550State) -> R
{
    let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(UART_16550_STATE);
    threadlocal
        .get_mut().and_then(|c| c.enter_nonreentrant(|v| v.as_mut().map(f)))
}


pub(crate) unsafe fn with_uart_state_panic<R, F>(f: F) -> R
where
    F: FnOnce(&mut Uart16550State) -> R
{
    with_uart_state(f)
        .expect("Current thread does not have access to an initialized UART state")
}

pub struct Uart16550 {
    regs: StaticRef<Uart16550Registers>,
}

impl Uart16550 {
    pub fn new(regs: StaticRef<Uart16550Registers>) -> Uart16550 {
        // Disable all interrupts when constructing the UART
        regs.ier.set(0xF);

        regs.iir_fcr.write(FCR::Enable::CLEAR);

        Uart16550 {
            regs,
        }
    }
}

impl Uart16550 {
    pub fn handle_interrupt(&self) {
        // Currently we can only receive a tx interrupt, however we
        // need to check the interrupt cause nonetheless as this will
        // clear the TX interrupt bit
        let iir = self.regs.iir_fcr.extract();

        // Check if the register contained a valid interrupt at all
        if !iir.matches_all(IIR::Pending::Pending) {
            panic!("UART 16550: interrupt without interrupt");
        }

        // Check whether there is space for new data
        if iir.matches_all(IIR::Identification::TransmitterHoldingRegisterEmpty) {
            // The respective interrupt has already been cleared by
            // the extraction of IIR

            // We also check whether the tx_buffer is set, as we
            // could've generated the interrupt by transmit_sync
            let closure = |state: &mut Uart16550State| {
                if state.tx_buffer.is_some() {
                    self.transmit_continue();
                }
            };
            unsafe { with_uart_state_panic(closure); }
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

    pub fn try_transmit_continue(&self) -> bool {
        let closure = |state: &mut Uart16550State| {
            if state.tx_buffer.is_some() {
                self.transmit_continue();
                true
            } else {
                false
            }
        };
        unsafe { with_uart_state_panic(closure) }
    }

    fn transmit_continue(&self) {
        // This should only be called as a result of a transmit
        // interrupt
        let closure = |state: &mut Uart16550State| {
            // Get the current transmission information
            let mut index = state.tx_index.get();
            let tx_data = state.tx_buffer.take().expect("UART 16550: no tx buffer");

            if index < state.tx_len.get() {
                // Still data to send
                while index < state.tx_len.get() && self.regs.lsr.is_set(LSR::THREmpty) {
                    self.regs.rbr_thr.write(THR::Data.val(tx_data[index]));
                    index += 1;
                }

                // Put the updated index and buffer back, wait for an
                // interrupt
                state.tx_index.set(index);
                state.tx_buffer.replace(tx_data);
            } else {
                // We are finished, disable tx interrupts
                self.regs
                    .ier
                    .modify(IER::TransmitterHoldingRegisterEmpty::CLEAR);

                // Callback to the client
                state.tx_client
                    .map(|client| client.transmitted_buffer(tx_data, state.tx_len.get(), Ok(())));
            }
        };

        // TODO: doc safety!
        unsafe { with_uart_state_panic(closure); }
    }

    fn receive(&self) {
        // Receive interrupts must only be enabled when we're currently holding
        // a buffer to receive data into:

        let closure = |state: &mut Uart16550State| {
            let rx_buffer = state.rx_buffer.take().expect("UART 16550: no rx buffer");
            let len = state.rx_len.get();
            let mut index = state.rx_index.get();

            // Read in a while loop, until no more data in the FIFO
            while self.regs.lsr.is_set(LSR::DataAvailable) && index < len {
                rx_buffer[index] = self.regs.rbr_thr.get();
                index += 1;
            }

            // Check whether we've read sufficient data:
            if index == len {
                // We're done, disable interrupts and return to the client:
                self.regs.ier.modify(IER::ReceivedDataAvailable::CLEAR);

                state.rx_client.map(move |client| {
                    client.received_buffer(rx_buffer, len, Ok(()), hil::uart::Error::None)
                });
            } else {
                // Store the new index and place the buffer back:
                state.rx_index.set(index);
                state.rx_buffer.replace(rx_buffer);
            }
        };

        // TODO: safety
        unsafe { with_uart_state_panic(closure); }
    }

    // Called before traveling to the other side of a portal
    pub fn save_context(&self) {
        let closure = |state: &mut Uart16550State| {
            state.context.ier = self.regs.ier.get();
        };
        unsafe { with_uart_state_panic(closure); }
    }

    // Called after passing through the portal
    pub fn restore_context(&self) {
        let closure = |state: &mut Uart16550State| {
            self.regs.ier.set(state.context.ier);
        };
        unsafe { with_uart_state_panic(closure); }
    }

    pub fn set_transmit_client(client: &'static dyn hil::uart::TransmitClient) {
        let closure = |state: &mut Uart16550State| {
            state.tx_client.set(client)
        };
        unsafe { with_uart_state_panic(closure); }
    }

    pub fn set_receive_client(client: &'static dyn hil::uart::ReceiveClient) {
        let closure = |state: &mut Uart16550State| {
            state.rx_client.set(client)
        };
        unsafe { with_uart_state_panic(closure); }
    }
}

impl hil::uart::Configure for Uart16550 {
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
        };

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
        };

        match params.hw_flow_control {
            true => lcr.modify(LCR::BreakSignal::SET),
            false => lcr.modify(LCR::BreakSignal::CLEAR),
        };

        self.regs.lcr.set(lcr.get());

        Ok(())
    }
}

impl hil::uart::Transmit<'static> for Uart16550 {
    fn set_transmit_client(&self, client: &'static dyn hil::uart::TransmitClient) {
        unimplemented!("Use Uart16550::set_transmit_client instead.");
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let closure = |state: &mut Uart16550State| {
            if tx_len > tx_data.len() {
                return Err((ErrorCode::INVAL, tx_data));
            }

            if state.tx_buffer.is_some() {
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
            state.tx_buffer.replace(tx_data);
            state.tx_len.set(tx_len);
            state.tx_index.set(index);

            Ok(())
        };

        unsafe { with_uart_state_panic(closure) }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl hil::uart::Receive<'static> for Uart16550 {
    fn set_receive_client(&self, client: &'static dyn hil::uart::ReceiveClient) {
        unimplemented!("Use Uart16550::set_receive_client instead.");
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


        let closure = |state: &mut Uart16550State| {
            // Store the receive buffer and byte count. We cannot call into the
            // generic receive routine here, as the client callback needs to be
            // called from another call stack. Hence simply enable interrupts here.
            unsafe {
                // Shorten rx_buffer's lifetime to placate FnMut's complaint. This
                // is safe because the uart state is static.
                // TODO: fix it with a better interface
                state.rx_buffer.replace(&mut *(rx_buffer as *mut _));
            }
            state.rx_len.set(rx_len);
            state.rx_index.set(0);
        };

        unsafe { with_uart_state_panic(closure); }

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
