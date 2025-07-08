// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Universal asynchronous receiver/transmitter with EasyDMA (UARTE)
//!
//! Author
//! -------------------
//!
//! * Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: March 10 2018

use core::cell::Cell;
use core::cmp::min;
use kernel::hil::uart;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use nrf5x::pinmux;

const UARTE_MAX_BUFFER_SIZE: u32 = 0xff;

static mut BYTE: u8 = 0;

pub const UARTE0_BASE: StaticRef<UarteRegisters> =
    unsafe { StaticRef::new(0x40002000 as *const UarteRegisters) };

#[repr(C)]
pub struct UarteRegisters {
    task_startrx: WriteOnly<u32, Task::Register>,
    task_stoprx: WriteOnly<u32, Task::Register>,
    task_starttx: WriteOnly<u32, Task::Register>,
    task_stoptx: WriteOnly<u32, Task::Register>,
    _reserved1: [u32; 7],
    task_flush_rx: WriteOnly<u32, Task::Register>,
    _reserved2: [u32; 52],
    event_cts: ReadWrite<u32, Event::Register>,
    event_ncts: ReadWrite<u32, Event::Register>,
    _reserved3: [u32; 2],
    event_endrx: ReadWrite<u32, Event::Register>,
    _reserved4: [u32; 3],
    event_endtx: ReadWrite<u32, Event::Register>,
    event_error: ReadWrite<u32, Event::Register>,
    _reserved6: [u32; 7],
    event_rxto: ReadWrite<u32, Event::Register>,
    _reserved7: [u32; 1],
    event_rxstarted: ReadWrite<u32, Event::Register>,
    event_txstarted: ReadWrite<u32, Event::Register>,
    _reserved8: [u32; 1],
    event_txstopped: ReadWrite<u32, Event::Register>,
    _reserved9: [u32; 41],
    shorts: ReadWrite<u32, Shorts::Register>,
    _reserved10: [u32; 64],
    intenset: ReadWrite<u32, Interrupt::Register>,
    intenclr: ReadWrite<u32, Interrupt::Register>,
    _reserved11: [u32; 93],
    errorsrc: ReadWrite<u32, ErrorSrc::Register>,
    _reserved12: [u32; 31],
    enable: ReadWrite<u32, Uart::Register>,
    _reserved13: [u32; 1],
    pselrts: ReadWrite<u32, Psel::Register>,
    pseltxd: ReadWrite<u32, Psel::Register>,
    pselcts: ReadWrite<u32, Psel::Register>,
    pselrxd: ReadWrite<u32, Psel::Register>,
    _reserved14: [u32; 3],
    baudrate: ReadWrite<u32, Baudrate::Register>,
    _reserved15: [u32; 3],
    rxd_ptr: ReadWrite<u32, Pointer::Register>,
    rxd_maxcnt: ReadWrite<u32, Counter::Register>,
    rxd_amount: ReadOnly<u32, Counter::Register>,
    _reserved16: [u32; 1],
    txd_ptr: ReadWrite<u32, Pointer::Register>,
    txd_maxcnt: ReadWrite<u32, Counter::Register>,
    txd_amount: ReadOnly<u32, Counter::Register>,
    _reserved17: [u32; 7],
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
        // Shortcut between ENDRX and STARTRX
        ENDRX_STARTRX OFFSET(5) NUMBITS(1),
        // Shortcut between ENDRX and STOPRX
        ENDRX_STOPRX OFFSET(6) NUMBITS(1)
    ],

    /// UART Interrupts
    Interrupt [
        CTS OFFSET(0) NUMBITS(1),
        NCTS OFFSET(1) NUMBITS(1),
        ENDRX OFFSET(4) NUMBITS(1),
        ENDTX OFFSET(8) NUMBITS(1),
        ERROR OFFSET(9) NUMBITS(1),
        RXTO OFFSET(17) NUMBITS(1),
        RXSTARTED OFFSET(19) NUMBITS(1),
        TXSTARTED OFFSET(20) NUMBITS(1),
        TXSTOPPED OFFSET(22) NUMBITS(1)
    ],

    /// UART Errors
    ErrorSrc [
        OVERRUN OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(1),
        FRAMING OFFSET(2) NUMBITS(1),
        BREAK OFFSET(3) NUMBITS(1)
    ],

    /// Enable UART
    Uart [
        ENABLE OFFSET(0) NUMBITS(4) [
            ON = 8,
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

    /// Baudrate
    Baudrate [
        BAUDRAUTE OFFSET(0) NUMBITS(32)
    ],

    /// DMA pointer
    Pointer [
        POINTER OFFSET(0) NUMBITS(32)
    ],

    /// Counter value
    Counter [
        COUNTER OFFSET(0) NUMBITS(8)
    ],

    /// Configuration of parity and flow control
    Config [
        HWFC OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(3)
    ]
];

/// UARTE
// It should never be instanced outside this module but because a static mutable reference to it
// is exported outside this module it must be `pub`
pub struct Uarte<'a> {
    registers: StaticRef<UarteRegisters>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    tx_buffer: kernel::utilities::cells::TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_remaining_bytes: Cell<usize>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_buffer: kernel::utilities::cells::TakeCell<'static, [u8]>,
    rx_remaining_bytes: Cell<usize>,
    rx_abort_in_progress: Cell<bool>,
    offset: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

impl<'a> Uarte<'a> {
    /// Constructor
    // This should only be constructed once
    pub const fn new(regs: StaticRef<UarteRegisters>) -> Uarte<'a> {
        Uarte {
            registers: regs,
            tx_client: OptionalCell::empty(),
            tx_buffer: kernel::utilities::cells::TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_remaining_bytes: Cell::new(0),
            rx_client: OptionalCell::empty(),
            rx_buffer: kernel::utilities::cells::TakeCell::empty(),
            rx_remaining_bytes: Cell::new(0),
            rx_abort_in_progress: Cell::new(false),
            offset: Cell::new(0),
        }
    }

    /// Configure which pins the UART should use for txd, rxd, cts and rts
    pub fn initialize(
        &self,
        txd: pinmux::Pinmux,
        rxd: pinmux::Pinmux,
        cts: Option<pinmux::Pinmux>,
        rts: Option<pinmux::Pinmux>,
    ) {
        self.registers.pseltxd.write(Psel::PIN.val(txd.into()));
        self.registers.pselrxd.write(Psel::PIN.val(rxd.into()));
        cts.map_or_else(
            || {
                // If no CTS pin is provided, then we need to mark it as
                // disconnected in the register.
                self.registers.pselcts.write(Psel::CONNECT::SET);
            },
            |c| {
                self.registers.pselcts.write(Psel::PIN.val(c.into()));
            },
        );
        rts.map_or_else(
            || {
                // If no RTS pin is provided, then we need to mark it as
                // disconnected in the register.
                self.registers.pselrts.write(Psel::CONNECT::SET);
            },
            |r| {
                self.registers.pselrts.write(Psel::PIN.val(r.into()));
            },
        );

        // Make sure we clear the endtx interrupt since that is what we rely on
        // to know when the DMA TX finishes. Normally, we clear this interrupt
        // as we handle it, so this is not necessary. However, a bootloader (or
        // some other startup code) may have setup TX interrupts, and there may
        // be one pending. We clear it to be safe.
        self.registers.event_endtx.write(Event::READY::CLEAR);

        self.enable_uart();
    }

    // The datasheet gives a non-exhaustive list of example settings for
    // typical bauds. The register is actually just a simple clock divider,
    // as explained and with implementation from:
    // https://devzone.nordicsemi.com/f/nordic-q-a/43280/technical-question-regarding-uart-baud-rate-generator-baudrate-register-offset-0x524
    //
    // Technically only RX is limited to 1MBaud, can TX up to 8MBaud:
    // https://devzone.nordicsemi.com/f/nordic-q-a/84204/framing-error-and-noisy-data-when-using-uarte-at-high-baud-rate
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

    // Enable UART peripheral, this need to disabled for low power applications
    fn enable_uart(&self) {
        self.registers.enable.write(Uart::ENABLE::ON);
    }

    #[allow(dead_code)]
    fn disable_uart(&self) {
        self.registers.enable.write(Uart::ENABLE::OFF);
    }

    fn enable_rx_interrupts(&self) {
        self.registers.intenset.write(Interrupt::ENDRX::SET);
    }

    fn enable_tx_interrupts(&self) {
        self.registers.intenset.write(Interrupt::ENDTX::SET);
    }

    fn disable_rx_interrupts(&self) {
        self.registers.intenclr.write(Interrupt::ENDRX::SET);
    }

    fn disable_tx_interrupts(&self) {
        self.registers.intenclr.write(Interrupt::ENDTX::SET);
    }

    /// UART interrupt handler that listens for both tx_end and rx_end events
    #[inline(never)]
    pub fn handle_interrupt(&self) {
        if self.tx_ready() {
            self.disable_tx_interrupts();
            self.registers.event_endtx.write(Event::READY::CLEAR);
            let tx_bytes = self.registers.txd_amount.get() as usize;

            let rem = match self.tx_remaining_bytes.get().checked_sub(tx_bytes) {
                None => return,
                Some(r) => r,
            };

            // All bytes have been transmitted
            if rem == 0 {
                // Signal client write done
                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|tx_buffer| {
                        client.transmitted_buffer(tx_buffer, self.tx_len.get(), Ok(()));
                    });
                });
            } else {
                // Not all bytes have been transmitted then update offset and continue transmitting
                self.offset.set(self.offset.get() + tx_bytes);
                self.tx_remaining_bytes.set(rem);
                self.set_tx_dma_pointer_to_buffer();
                self.registers
                    .txd_maxcnt
                    .write(Counter::COUNTER.val(min(rem as u32, UARTE_MAX_BUFFER_SIZE)));
                self.registers.task_starttx.write(Task::ENABLE::SET);
                self.enable_tx_interrupts();
            }
        }

        if self.rx_ready() {
            self.disable_rx_interrupts();

            // Clear the ENDRX event
            self.registers.event_endrx.write(Event::READY::CLEAR);

            // Get the number of bytes in the buffer that was received this time
            let rx_bytes = self.registers.rxd_amount.get() as usize;

            // Check if this ENDRX is due to an abort. If so, we want to
            // do the receive callback immediately.
            if self.rx_abort_in_progress.get() {
                self.rx_abort_in_progress.set(false);
                self.rx_client.map(|client| {
                    self.rx_buffer.take().map(|rx_buffer| {
                        client.received_buffer(
                            rx_buffer,
                            self.offset.get() + rx_bytes,
                            Err(ErrorCode::CANCEL),
                            uart::Error::None,
                        );
                    });
                });
            } else {
                // In the normal case, we need to either pass call the callback
                // or do another read to get more bytes.

                // Update how many bytes we still need to receive and
                // where we are storing in the buffer.
                self.rx_remaining_bytes
                    .set(self.rx_remaining_bytes.get().saturating_sub(rx_bytes));
                self.offset.set(self.offset.get() + rx_bytes);

                let rem = self.rx_remaining_bytes.get();
                if rem == 0 {
                    // Signal client that the read is done
                    self.rx_client.map(|client| {
                        self.rx_buffer.take().map(|rx_buffer| {
                            client.received_buffer(
                                rx_buffer,
                                self.offset.get(),
                                Ok(()),
                                uart::Error::None,
                            );
                        });
                    });
                } else {
                    // Setup how much we can read. We already made sure that
                    // this will fit in the buffer.
                    let to_read = core::cmp::min(rem, 255);
                    self.registers
                        .rxd_maxcnt
                        .write(Counter::COUNTER.val(to_read as u32));

                    // Actually do the receive.
                    self.set_rx_dma_pointer_to_buffer();
                    self.registers.task_startrx.write(Task::ENABLE::SET);
                    self.enable_rx_interrupts();
                }
            }
        }
    }

    /// Transmit one byte at the time and the client is responsible for polling
    /// This is used by the panic handler
    pub unsafe fn send_byte(&self, byte: u8) {
        self.tx_remaining_bytes.set(1);
        self.registers.event_endtx.write(Event::READY::CLEAR);
        // precaution: copy value into variable with static lifetime
        BYTE = byte;
        self.registers.txd_ptr.set(core::ptr::addr_of!(BYTE) as u32);
        self.registers.txd_maxcnt.write(Counter::COUNTER.val(1));
        self.registers.task_starttx.write(Task::ENABLE::SET);
    }

    /// Check if the UART transmission is done
    pub fn tx_ready(&self) -> bool {
        self.registers.event_endtx.is_set(Event::READY)
    }

    /// Check if either the rx_buffer is full or the UART has timed out
    pub fn rx_ready(&self) -> bool {
        self.registers.event_endrx.is_set(Event::READY)
    }

    fn set_tx_dma_pointer_to_buffer(&self) {
        self.tx_buffer.map(|tx_buffer| {
            self.registers
                .txd_ptr
                .set(tx_buffer[self.offset.get()..].as_ptr() as u32);
        });
    }

    fn set_rx_dma_pointer_to_buffer(&self) {
        self.rx_buffer.map(|rx_buffer| {
            self.registers
                .rxd_ptr
                .set(rx_buffer[self.offset.get()..].as_ptr() as u32);
        });
    }

    // Helper function used by both transmit_word and transmit_buffer
    fn setup_buffer_transmit(&self, buf: &'static mut [u8], tx_len: usize) {
        self.tx_remaining_bytes.set(tx_len);
        self.tx_len.set(tx_len);
        self.offset.set(0);
        self.tx_buffer.replace(buf);
        self.set_tx_dma_pointer_to_buffer();

        self.registers
            .txd_maxcnt
            .write(Counter::COUNTER.val(min(tx_len as u32, UARTE_MAX_BUFFER_SIZE)));
        self.registers.task_starttx.write(Task::ENABLE::SET);

        self.enable_tx_interrupts();
    }
}

impl<'a> uart::Transmit<'a> for Uarte<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 || tx_len > tx_data.len() {
            Err((ErrorCode::SIZE, tx_data))
        } else if self.tx_buffer.is_some() {
            Err((ErrorCode::BUSY, tx_data))
        } else {
            self.setup_buffer_transmit(tx_data, tx_len);
            Ok(())
        }
    }

    fn transmit_word(&self, _data: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl uart::Configure for Uarte<'_> {
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

impl<'a> uart::Receive<'a> for Uarte<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buf: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.rx_buffer.is_some() {
            return Err((ErrorCode::BUSY, rx_buf));
        }
        // truncate rx_len if necessary
        let truncated_length = core::cmp::min(rx_len, rx_buf.len());

        self.rx_remaining_bytes.set(truncated_length);
        self.offset.set(0);
        self.rx_buffer.replace(rx_buf);
        self.set_rx_dma_pointer_to_buffer();

        let truncated_uart_max_length = core::cmp::min(truncated_length, 255);

        self.registers
            .rxd_maxcnt
            .write(Counter::COUNTER.val(truncated_uart_max_length as u32));
        self.registers.task_stoprx.write(Task::ENABLE::SET);
        self.registers.task_startrx.write(Task::ENABLE::SET);

        self.enable_rx_interrupts();
        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        // Trigger the STOPRX event to cancel the current receive call.
        if self.rx_buffer.is_none() {
            Ok(())
        } else {
            self.rx_abort_in_progress.set(true);
            self.registers.task_stoprx.write(Task::ENABLE::SET);
            Err(ErrorCode::BUSY)
        }
    }
}

#[cfg(test)]
mod tests {
    use kernel::ErrorCode;

    #[test]
    fn baud_rate_divider_calculation() {
        let u = super::Uarte::new(super::UARTE0_BASE);
        assert_eq!(u.get_divider_for_baud(0), Err(ErrorCode::INVAL));
        assert_eq!(u.get_divider_for_baud(4_000_000), Err(ErrorCode::INVAL));

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
        assert_eq!(u.get_divider_for_baud(1200), Ok(0x0004F000));
        assert_eq!(u.get_divider_for_baud(2400), Ok(0x0009D000));
        assert_eq!(u.get_divider_for_baud(4800), Ok(0x0013B000));
        assert_eq!(u.get_divider_for_baud(9600), Ok(0x00275000));
        //assert_eq!(u.get_divider_for_baud(14400), Ok(0x003AF000));
        assert_eq!(u.get_divider_for_baud(19200), Ok(0x004EA000));
        //assert_eq!(u.get_divider_for_baud(28800), Ok(0x0075C000));
        //assert_eq!(u.get_divider_for_baud(38400), Ok(0x009D0000));
        //assert_eq!(u.get_divider_for_baud(57600), Ok(0x00EB0000));
        assert_eq!(u.get_divider_for_baud(76800), Ok(0x013A9000));
        //assert_eq!(u.get_divider_for_baud(115200), Ok(0x01D60000));
        //assert_eq!(u.get_divider_for_baud(230400), Ok(0x03B00000));
        assert_eq!(u.get_divider_for_baud(250000), Ok(0x04000000));
        //assert_eq!(u.get_divider_for_baud(460800), Ok(0x07400000));
        //assert_eq!(u.get_divider_for_baud(921600), Ok(0x0F000000));
        assert_eq!(u.get_divider_for_baud(1000000), Ok(0x10000000));
        //
        // For completeness of testing, we do verify that the calculation works
        // as-expected to generate the empirically correct divisors.  (i.e.,
        // these are not the datasheet constants, but are the correct divisors
        // for the desired bauds):
        assert_eq!(u.get_divider_for_baud(14400), Ok(0x003B0000));
        assert_eq!(u.get_divider_for_baud(28800), Ok(0x0075F000));
        assert_eq!(u.get_divider_for_baud(38400), Ok(0x009D5000));
        assert_eq!(u.get_divider_for_baud(57600), Ok(0x00EBF000));
        assert_eq!(u.get_divider_for_baud(115200), Ok(0x01D7E000));
        assert_eq!(u.get_divider_for_baud(230400), Ok(0x03AFB000));
        assert_eq!(u.get_divider_for_baud(460800), Ok(0x075F7000));
        assert_eq!(u.get_divider_for_baud(921600), Ok(0x0EBEE000));
    }
}
