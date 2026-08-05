// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Universal asynchronous receiver/transmitter (UART)
//!
//! This is the legacy, non-EasyDMA UART peripheral. Unlike
//! [`crate::uart::Uarte`], this peripheral transfers one byte at a time and
//! generates an interrupt (`RXDRDY`/`TXDRDY`) per byte rather than per
//! buffer. It shares the same peripheral instance (and MMIO address) as the
//! UARTE peripheral, so a chip can use one or the other but not both at the
//! same time.
//!
//! See the nRF52840 Product Specification, UART chapter, for details:
//! <https://docs.nordicsemi.com/r/bundle/ps_nrf52840/page/uart.html>

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::io_write::IoWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_bitfields};
use nrf5x::gpio::Pin;
use nrf5x::pinmux;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40002000 as *const UartRegisters) };

#[repr(C)]
pub struct UartRegisters {
    task_startrx: WriteOnly<u32, Task::Register>,
    task_stoprx: WriteOnly<u32, Task::Register>,
    task_starttx: WriteOnly<u32, Task::Register>,
    task_stoptx: WriteOnly<u32, Task::Register>,
    _reserved1: [u32; 3],
    task_suspend: WriteOnly<u32, Task::Register>,
    _reserved2: [u32; 56],
    event_cts: ReadWrite<u32, Event::Register>,
    event_ncts: ReadWrite<u32, Event::Register>,
    event_rxdrdy: ReadWrite<u32, Event::Register>,
    _reserved3: [u32; 4],
    event_txdrdy: ReadWrite<u32, Event::Register>,
    _reserved4: [u32; 1],
    event_error: ReadWrite<u32, Event::Register>,
    _reserved5: [u32; 7],
    event_rxto: ReadWrite<u32, Event::Register>,
    _reserved6: [u32; 46],
    shorts: ReadWrite<u32, Shorts::Register>,
    _reserved7: [u32; 64],
    intenset: ReadWrite<u32, Interrupt::Register>,
    intenclr: ReadWrite<u32, Interrupt::Register>,
    _reserved8: [u32; 93],
    errorsrc: ReadWrite<u32, ErrorSrc::Register>,
    _reserved9: [u32; 31],
    enable: ReadWrite<u32, Enable::Register>,
    _reserved10: [u32; 1],
    pselrts: ReadWrite<u32, Psel::Register>,
    pseltxd: ReadWrite<u32, Psel::Register>,
    pselcts: ReadWrite<u32, Psel::Register>,
    pselrxd: ReadWrite<u32, Psel::Register>,
    rxd: ReadOnly<u32, Byte::Register>,
    txd: WriteOnly<u32, Byte::Register>,
    _reserved11: [u32; 1],
    baudrate: ReadWrite<u32, Baudrate::Register>,
    _reserved12: [u32; 17],
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

    /// Shortcuts
    Shorts [
        // Shortcut between CTS and STARTRX
        CTS_STARTRX OFFSET(0) NUMBITS(1),
        // Shortcut between NCTS and STOPRX
        NCTS_STOPRX OFFSET(1) NUMBITS(1)
    ],

    /// UART Interrupts
    Interrupt [
        CTS OFFSET(0) NUMBITS(1),
        NCTS OFFSET(1) NUMBITS(1),
        RXDRDY OFFSET(2) NUMBITS(1),
        TXDRDY OFFSET(3) NUMBITS(1),
        ERROR OFFSET(4) NUMBITS(1),
        RXTO OFFSET(5) NUMBITS(1)
    ],

    /// UART Errors
    ErrorSrc [
        OVERRUN OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(1),
        FRAMING OFFSET(2) NUMBITS(1),
        BREAK OFFSET(3) NUMBITS(1)
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
        PIN OFFSET(0) NUMBITS(6),
        // Connect/Disconnect
        CONNECT OFFSET(31) NUMBITS(1)
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

#[derive(Copy, Clone, PartialEq)]
enum UARTStateTX {
    Idle,
    Transmitting,
    AbortRequested,
}

#[derive(Copy, Clone, PartialEq)]
enum UARTStateRX {
    Idle,
    Receiving,
    AbortRequested,
}

/// UART
// It should never be instanced outside this module but because a static mutable reference to it
// is exported outside this module it must be `pub`
pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,

    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_position: Cell<usize>,
    tx_status: Cell<UARTStateTX>,

    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_position: Cell<usize>,
    rx_status: Cell<UARTStateRX>,

    deferred_call: DeferredCall,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

impl<'a> Uart<'a> {
    /// Constructor
    // This should only be constructed once
    pub fn new(regs: StaticRef<UartRegisters>) -> Uart<'a> {
        Uart {
            registers: regs,

            tx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_position: Cell::new(0),
            tx_status: Cell::new(UARTStateTX::Idle),

            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_position: Cell::new(0),
            rx_status: Cell::new(UARTStateRX::Idle),

            deferred_call: DeferredCall::new(),
        }
    }

    fn initialize_inner(&self, txd: Pin, rxd: Pin, cts: Option<Pin>, rts: Option<Pin>) {
        self.disable_uart();

        // Stop any ongoing TX or RX sequences.
        self.registers.task_stoptx.write(Task::ENABLE::SET);
        self.registers.task_stoprx.write(Task::ENABLE::SET);

        // Make sure we clear the txdrdy, rxdrdy and error events. Normally we
        // clear these as we handle them, so this is not necessary. However, a
        // bootloader (or some other startup code) may have setup the UART,
        // and there may be a stale event pending. We clear it to be safe.
        self.registers.event_txdrdy.write(Event::READY::CLEAR);
        self.registers.event_rxdrdy.write(Event::READY::CLEAR);
        self.registers.event_error.write(Event::READY::CLEAR);

        self.registers.pseltxd.write(Psel::PIN.val(txd as _));
        self.registers.pselrxd.write(Psel::PIN.val(rxd as _));
        cts.map_or_else(
            || {
                // If no CTS pin is provided, then we need to mark it as
                // disconnected in the register.
                self.registers.pselcts.write(Psel::CONNECT::SET);
            },
            |c| {
                self.registers.pselcts.write(Psel::PIN.val(c as _));
            },
        );
        rts.map_or_else(
            || {
                // If no RTS pin is provided, then we need to mark it as
                // disconnected in the register.
                self.registers.pselrts.write(Psel::CONNECT::SET);
            },
            |r| {
                self.registers.pselrts.write(Psel::PIN.val(r as _));
            },
        );

        self.enable_uart();
    }

    /// Configure which pins the UART should use for txd, rxd, cts and rts
    pub fn initialize(
        &self,
        txd: pinmux::Pinmux,
        rxd: pinmux::Pinmux,
        cts: Option<pinmux::Pinmux>,
        rts: Option<pinmux::Pinmux>,
    ) {
        self.initialize_inner(
            txd.into(),
            rxd.into(),
            cts.map(Into::into),
            rts.map(Into::into),
        )
    }

    // The datasheet gives a non-exhaustive list of example settings for
    // typical bauds. The register is actually just a simple clock divider,
    // as explained and with implementation from:
    // https://devzone.nordicsemi.com/f/nordic-q-a/43280/technical-question-regarding-uart-baud-rate-generator-baudrate-register-offset-0x524
    //
    // This peripheral shares the same baud rate generator hardware as the
    // UARTE peripheral.
    fn get_divider_for_baud(baud_rate: u32) -> Result<u32, ErrorCode> {
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
        let divider = Self::get_divider_for_baud(baud_rate)?;
        self.registers.baudrate.set(divider);

        Ok(())
    }

    // Enable UART peripheral, this needs to be disabled for low power applications
    fn enable_uart(&self) {
        self.registers.enable.write(Enable::ENABLE::ON);
    }

    fn disable_uart(&self) {
        self.registers.enable.write(Enable::ENABLE::OFF);
    }

    fn enable_tx_interrupt(&self) {
        self.registers.intenset.write(Interrupt::TXDRDY::SET);
    }

    fn disable_tx_interrupt(&self) {
        self.registers.intenclr.write(Interrupt::TXDRDY::SET);
    }

    fn enable_rx_interrupt(&self) {
        self.registers
            .intenset
            .write(Interrupt::RXDRDY::SET + Interrupt::ERROR::SET);
    }

    fn disable_rx_interrupt(&self) {
        self.registers
            .intenclr
            .write(Interrupt::RXDRDY::SET + Interrupt::ERROR::SET);
    }

    fn tx_progress(&self) {
        // Write the next byte and advance our position. The TXDRDY event
        // that brought us here indicates the previous byte finished
        // transmitting, so the peripheral is ready for another.
        self.tx_buffer.map(|buf| {
            self.registers
                .txd
                .write(Byte::VALUE.val(buf[self.tx_position.get()] as u32));
        });
        self.tx_position.set(self.tx_position.get() + 1);
    }

    fn rx_progress(&self) {
        let byte = self.registers.rxd.read(Byte::VALUE) as u8;
        self.rx_buffer.map(|buf| {
            buf[self.rx_position.get()] = byte;
        });
        self.rx_position.set(self.rx_position.get() + 1);
    }

    /// UART interrupt handler that listens for RX, TX and error events
    #[inline(never)]
    pub fn handle_interrupt(&self) {
        if self.registers.event_txdrdy.is_set(Event::READY)
            && self.tx_status.get() == UARTStateTX::Transmitting
        {
            self.registers.event_txdrdy.write(Event::READY::CLEAR);

            if self.tx_position.get() < self.tx_len.get() {
                // There is more to send.
                self.tx_progress();
            } else {
                // We already wrote every byte in the buffer, and this
                // TXDRDY event tells us the last one finished transmitting.
                self.disable_tx_interrupt();
                self.registers.task_stoptx.write(Task::ENABLE::SET);
                self.tx_status.set(UARTStateTX::Idle);

                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|buf| {
                        client.transmitted_buffer(buf, self.tx_len.get(), Ok(()));
                    });
                });
            }
        }

        if self.registers.event_error.is_set(Event::READY) {
            self.registers.event_error.write(Event::READY::CLEAR);

            // Read which error(s) occurred and clear them (write-1-to-clear).
            let errorsrc = self.registers.errorsrc.extract();
            self.registers.errorsrc.set(errorsrc.get());

            if self.rx_status.get() == UARTStateRX::Receiving {
                self.disable_rx_interrupt();
                self.registers.task_stoprx.write(Task::ENABLE::SET);
                self.rx_status.set(UARTStateRX::Idle);

                let error = if errorsrc.is_set(ErrorSrc::OVERRUN) {
                    uart::Error::OverrunError
                } else if errorsrc.is_set(ErrorSrc::PARITY) {
                    uart::Error::ParityError
                } else if errorsrc.is_set(ErrorSrc::FRAMING) {
                    uart::Error::FramingError
                } else if errorsrc.is_set(ErrorSrc::BREAK) {
                    uart::Error::BreakError
                } else {
                    uart::Error::None
                };

                let rx_position = self.rx_position.get();
                self.rx_client.map(|client| {
                    self.rx_buffer.take().map(|buf| {
                        client.received_buffer(buf, rx_position, Err(ErrorCode::FAIL), error);
                    });
                });
            }
        }

        if self.registers.event_rxdrdy.is_set(Event::READY)
            && self.rx_status.get() == UARTStateRX::Receiving
        {
            self.registers.event_rxdrdy.write(Event::READY::CLEAR);

            self.rx_progress();

            if self.rx_position.get() == self.rx_len.get() {
                // Reception done.
                self.disable_rx_interrupt();
                self.registers.task_stoprx.write(Task::ENABLE::SET);
                self.rx_status.set(UARTStateRX::Idle);

                self.rx_client.map(|client| {
                    self.rx_buffer.take().map(|buf| {
                        client.received_buffer(buf, self.rx_len.get(), Ok(()), uart::Error::None);
                    });
                });
            }
        }
    }

    /// Transmit one byte at the time and the client is responsible for polling
    /// This is used by the panic handler
    pub unsafe fn send_byte(&self, byte: u8) {
        self.registers.event_txdrdy.write(Event::READY::CLEAR);
        self.registers.task_starttx.write(Task::ENABLE::SET);
        self.registers.txd.write(Byte::VALUE.val(byte as u32));
    }

    /// Check if the UART transmission is done
    pub fn tx_ready(&self) -> bool {
        self.registers.event_txdrdy.is_set(Event::READY)
    }
}

impl DeferredCallClient for Uart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self)
    }

    fn handle_deferred_call(&self) {
        if self.tx_status.get() == UARTStateTX::AbortRequested {
            self.tx_status.set(UARTStateTX::Idle);
            let tx_position = self.tx_position.get();
            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, tx_position, Err(ErrorCode::CANCEL));
                });
            });
        }

        if self.rx_status.get() == UARTStateRX::AbortRequested {
            self.rx_status.set(UARTStateRX::Idle);
            let rx_position = self.rx_position.get();
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|buf| {
                    client.received_buffer(
                        buf,
                        rx_position,
                        Err(ErrorCode::CANCEL),
                        uart::Error::Aborted,
                    );
                });
            });
        }
    }
}

impl uart::Configure for Uart<'_> {
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

impl<'a> uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
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
        if self.tx_status.get() != UARTStateTX::Idle {
            return Err((ErrorCode::BUSY, tx_data));
        }

        self.tx_status.set(UARTStateTX::Transmitting);
        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_position.set(0);

        self.registers.event_txdrdy.write(Event::READY::CLEAR);

        // Start the transmit sequence and send the first byte. Subsequent
        // bytes are sent from the TXDRDY interrupt handler.
        self.registers.task_starttx.write(Task::ENABLE::SET);
        self.tx_progress();

        self.enable_tx_interrupt();

        Ok(())
    }

    fn transmit_word(&self, _data: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.tx_status.get() != UARTStateTX::Transmitting {
            return Ok(());
        }

        self.disable_tx_interrupt();
        self.registers.task_stoptx.write(Task::ENABLE::SET);
        self.tx_status.set(UARTStateTX::AbortRequested);

        self.deferred_call.set();

        Err(ErrorCode::BUSY)
    }
}

impl<'a> uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buf: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len > rx_buf.len() {
            return Err((ErrorCode::SIZE, rx_buf));
        }
        if self.rx_status.get() != UARTStateRX::Idle {
            return Err((ErrorCode::BUSY, rx_buf));
        }

        self.rx_status.set(UARTStateRX::Receiving);
        self.rx_buffer.replace(rx_buf);
        self.rx_len.set(rx_len);
        self.rx_position.set(0);

        self.registers.event_rxdrdy.write(Event::READY::CLEAR);
        self.registers.task_startrx.write(Task::ENABLE::SET);

        self.enable_rx_interrupt();

        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_status.get() != UARTStateRX::Receiving {
            return Ok(());
        }

        self.disable_rx_interrupt();
        self.registers.task_stoprx.write(Task::ENABLE::SET);
        self.rx_status.set(UARTStateRX::AbortRequested);

        self.deferred_call.set();

        Err(ErrorCode::BUSY)
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
struct UartPanicWriter<'a> {
    inner: Uart<'a>,
}

impl IoWrite for UartPanicWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> usize {
        for &c in buf {
            unsafe {
                self.inner.send_byte(c);
            }
            while !self.inner.tx_ready() {}
        }
        buf.len()
    }
}

impl core::fmt::Write for UartPanicWriter<'_> {
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

impl kernel::platform::chip::PanicWriter for Uart<'_> {
    type Config = UartPanicWriterConfig;

    fn create_panic_writer(
        config: Self::Config,
        _panic: &core::panic::PanicInfo,
    ) -> impl IoWrite + core::fmt::Write {
        use uart::Configure as _;

        let inner = Uart::new(UART0_BASE);
        inner.initialize(
            pinmux::Pinmux::new(config.txd),
            pinmux::Pinmux::new(config.rxd),
            config.cts.map(pinmux::Pinmux::new),
            config.rts.map(pinmux::Pinmux::new),
        );
        let _ = inner.configure(config.params);
        UartPanicWriter { inner }
    }
}

#[cfg(test)]
mod tests {
    use kernel::ErrorCode;

    #[test]
    fn baud_rate_divider_calculation() {
        let get_divider_for_baud = super::Uart::get_divider_for_baud;
        assert_eq!(get_divider_for_baud(0), Err(ErrorCode::INVAL));
        assert_eq!(get_divider_for_baud(4_000_000), Err(ErrorCode::INVAL));

        // The constants below are the list from the Nordic technical documents.
        //
        // n.b., some datasheet constants do not match formula constants,
        // so we skip those, see nordic forum thread for details:
        // https://devzone.nordicsemi.com/f/nordic-q-a/84204/framing-error-and-noisy-data-when-using-uarte-at-high-baud-rate
        //
        // This is a *datasheet bug*, i.e., for a target baud of 115200, the
        // datasheet divisor yields 115108 (-0.079% err) where direct
        // computation of the divider yields 115203 (+0.002% err). Both work in
        // practice, but the error here is an annoying and uncharacteristic
        // Nordic quirk.
        assert_eq!(get_divider_for_baud(1200), Ok(0x0004F000));
        assert_eq!(get_divider_for_baud(2400), Ok(0x0009D000));
        assert_eq!(get_divider_for_baud(4800), Ok(0x0013B000));
        assert_eq!(get_divider_for_baud(9600), Ok(0x00275000));
        //assert_eq!(get_divider_for_baud(14400), Ok(0x003AF000));
        assert_eq!(get_divider_for_baud(19200), Ok(0x004EA000));
        //assert_eq!(get_divider_for_baud(28800), Ok(0x0075C000));
        //assert_eq!(get_divider_for_baud(38400), Ok(0x009D0000));
        //assert_eq!(get_divider_for_baud(57600), Ok(0x00EB0000));
        assert_eq!(get_divider_for_baud(76800), Ok(0x013A9000));
        //assert_eq!(get_divider_for_baud(115200), Ok(0x01D60000));
        //assert_eq!(get_divider_for_baud(230400), Ok(0x03B00000));
        assert_eq!(get_divider_for_baud(250000), Ok(0x04000000));
        //assert_eq!(get_divider_for_baud(460800), Ok(0x07400000));
        //assert_eq!(get_divider_for_baud(921600), Ok(0x0F000000));
        assert_eq!(get_divider_for_baud(1000000), Ok(0x10000000));
        //
        // For completeness of testing, we do verify that the calculation works
        // as-expected to generate the empirically correct divisors.  (i.e.,
        // these are not the datasheet constants, but are the correct divisors
        // for the desired bauds):
        assert_eq!(get_divider_for_baud(14400), Ok(0x003B0000));
        assert_eq!(get_divider_for_baud(28800), Ok(0x0075F000));
        assert_eq!(get_divider_for_baud(38400), Ok(0x009D5000));
        assert_eq!(get_divider_for_baud(57600), Ok(0x00EBF000));
        assert_eq!(get_divider_for_baud(115200), Ok(0x01D7E000));
        assert_eq!(get_divider_for_baud(230400), Ok(0x03AFB000));
        assert_eq!(get_divider_for_baud(460800), Ok(0x075F7000));
        assert_eq!(get_divider_for_baud(921600), Ok(0x0EBEE000));
    }
}
