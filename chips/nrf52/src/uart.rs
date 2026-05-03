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
use kernel::hil::uart;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::io_write::IoWrite;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use nrf5x::gpio::Pin;
use nrf5x::pinmux;

const UARTE_MAX_BUFFER_SIZE: usize = 0xff;

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

/// Wrapper for managing MMIO for UARTE.
struct UarteRegistersManager {
    /// MMIO registers for the UARTE peripheral.
    registers: StaticRef<UarteRegisters>,
    /// Holding place for the TX DMA buffer while DMA in progress.
    tx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    /// Holding place for the RX DMA buffer while DMA in progress.
    rx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
}

impl UarteRegistersManager {
    pub fn new(regs: StaticRef<UarteRegisters>) -> Self {
        Self {
            registers: regs,
            tx_dma_buf: MapCell::empty(),
            rx_dma_buf: MapCell::empty(),
        }
    }

    /// Start a UART transmission with DMA.
    ///
    /// # Return
    ///
    /// `Ok(())` on successfully starting the DMA operation. `Err(())` if the
    /// DMA is busy and the operation could not be started.
    pub fn start_tx_dma(&self, buf: SubSliceMut<'static, u8>) -> Result<(), ()> {
        if self.tx_dma_pending() {
            return Err(());
        }

        // To create a DmaFence we must trust the implementation.
        //
        // # Safety
        //
        // The architecture-provided version is correct for the nRF52.
        let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

        // Create DmaSlice for the TX buffer. This ensures that we can soundly
        // share it with the DMA hardware.
        let tx_dma_slice = DmaSubSliceMut::new_static(buf, fence);

        // Provide the DmaSlice buffer to the hardware DMA engine.
        self.registers.txd_ptr.set(tx_dma_slice.as_mut_ptr() as u32);

        // Specify the length to transmit.
        self.registers
            .txd_maxcnt
            .write(Counter::COUNTER.val(tx_dma_slice.len() as u32));

        // Save the DmaSlice while the DMA operation executes.
        self.tx_dma_buf.replace(tx_dma_slice);

        // Start the TX DMA operation
        self.registers.task_starttx.write(Task::ENABLE::SET);

        Ok(())
    }

    pub fn finish_tx_dma(&self) -> Option<(SubSliceMut<'static, u8>, usize)> {
        // End the DMA operation so it is safe to retrieve the buffer.
        self.registers.event_endtx.write(Event::READY::CLEAR);

        self.tx_dma_buf.take().map(|dma_slice| {
            // To create a DmaFence we must trust the implementation.
            //
            // # Safety
            //
            // The architecture-provided version is correct for the nRF52.
            let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

            // # Safety
            //
            // We must ensure that the DMA hardware no longer has any access
            // to this buffer. We ensure that by setting the `event_endtx`
            // event before taking the dma slice back.
            let buf = unsafe { dma_slice.take(fence) };

            let tx_bytes = self.registers.txd_amount.get() as usize;

            (buf, tx_bytes)
        })
    }

    pub fn tx_dma_pending(&self) -> bool {
        self.tx_dma_buf.is_some()
    }

    /// Start a UART reception with DMA.
    ///
    /// # Return
    ///
    /// `Ok(())` on successfully starting the DMA operation. `Err(())` if the
    /// DMA is busy and the operation could not be started.
    pub fn start_rx_dma(&self, buf: SubSliceMut<'static, u8>) -> Result<(), ()> {
        if self.rx_dma_pending() {
            return Err(());
        }

        // To create a DmaFence we must trust the implementation.
        //
        // # Safety
        //
        // The architecture-provided version is correct for the nRF52.
        let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

        // Create DmaSlice for the RX buffer. This ensures that we can soundly
        // share it with the DMA hardware.
        let rx_dma_slice = DmaSubSliceMut::new_static(buf, fence);

        // Provide the DmaSlice buffer to the hardware DMA engine.
        self.registers.rxd_ptr.set(rx_dma_slice.as_mut_ptr() as u32);

        // Specify the length to transmit.
        self.registers
            .rxd_maxcnt
            .write(Counter::COUNTER.val(rx_dma_slice.len() as u32));

        // Save the DmaSlice while the DMA operation executes.
        self.rx_dma_buf.replace(rx_dma_slice);

        // Start the RX DMA operation
        self.registers.task_startrx.write(Task::ENABLE::SET);

        Ok(())
    }

    pub fn finish_rx_dma(&self) -> Option<(SubSliceMut<'static, u8>, usize)> {
        // End the DMA operation so it is safe to retrieve the buffer.
        self.registers.event_endrx.write(Event::READY::CLEAR);

        self.rx_dma_buf.take().map(|dma_slice| {
            // To create a DmaFence we must trust the implementation.
            //
            // # Safety
            //
            // The architecture-provided version is correct for the nRF52.
            let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

            // # Safety
            //
            // We must ensure that the DMA hardware no longer has any access
            // to this buffer. We ensure that by setting the `event_endrx`
            // event before taking the dma slice back.
            let buf = unsafe { dma_slice.take(fence) };

            let rx_bytes = self.registers.rxd_amount.get() as usize;

            (buf, rx_bytes)
        })
    }

    pub fn rx_dma_pending(&self) -> bool {
        self.rx_dma_buf.is_some()
    }
}

/// UARTE
// It should never be instanced outside this module but because a static mutable reference to it
// is exported outside this module it must be `pub`
pub struct Uarte<'a> {
    registers: UarteRegistersManager,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    tx_len: Cell<usize>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_len: Cell<usize>,
    rx_abort_in_progress: Cell<bool>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

impl<'a> Uarte<'a> {
    /// Constructor
    // This should only be constructed once
    pub fn new(regs: StaticRef<UarteRegisters>) -> Uarte<'a> {
        Uarte {
            registers: UarteRegistersManager::new(regs),
            tx_client: OptionalCell::empty(),
            // tx_buffer: kernel::utilities::cells::TakeCell::empty(),
            tx_len: Cell::new(0),
            // tx_remaining_bytes: Cell::new(0),
            rx_client: OptionalCell::empty(),
            rx_len: Cell::new(0),
            rx_abort_in_progress: Cell::new(false),
        }
    }

    fn initialize_inner(&self, txd: Pin, rxd: Pin, cts: Option<Pin>, rts: Option<Pin>) {
        self.disable_uart();

        // Stop any ongoing TX or RX DMA transmissions
        self.registers
            .registers
            .task_stoptx
            .write(Task::ENABLE::SET);
        self.registers
            .registers
            .task_stoprx
            .write(Task::ENABLE::SET);

        // Make sure we clear the endtx and endrx interrupts since
        // that is what we rely on to know when the DMA TX
        // finishes. Normally, we clear this interrupt as we handle
        // it, so this is not necessary. However, a bootloader (or
        // some other startup code) may have setup TX interrupts, and
        // there may be one pending. We clear it to be safe.
        self.registers
            .registers
            .event_endtx
            .write(Event::READY::CLEAR);
        self.registers
            .registers
            .event_endrx
            .write(Event::READY::CLEAR);

        self.registers
            .registers
            .pseltxd
            .write(Psel::PIN.val(txd as _));
        self.registers
            .registers
            .pselrxd
            .write(Psel::PIN.val(rxd as _));
        cts.map_or_else(
            || {
                // If no CTS pin is provided, then we need to mark it as
                // disconnected in the register.
                self.registers.registers.pselcts.write(Psel::CONNECT::SET);
            },
            |c| {
                self.registers
                    .registers
                    .pselcts
                    .write(Psel::PIN.val(c as _));
            },
        );
        rts.map_or_else(
            || {
                // If no RTS pin is provided, then we need to mark it as
                // disconnected in the register.
                self.registers.registers.pselrts.write(Psel::CONNECT::SET);
            },
            |r| {
                self.registers
                    .registers
                    .pselrts
                    .write(Psel::PIN.val(r as _));
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
        self.registers.registers.baudrate.set(divider);

        Ok(())
    }

    // Enable UART peripheral, this need to disabled for low power applications
    fn enable_uart(&self) {
        self.registers.registers.enable.write(Uart::ENABLE::ON);
    }

    #[allow(dead_code)]
    fn disable_uart(&self) {
        self.registers.registers.enable.write(Uart::ENABLE::OFF);
    }

    fn enable_rx_interrupts(&self) {
        self.registers
            .registers
            .intenset
            .write(Interrupt::ENDRX::SET);
    }

    fn enable_tx_interrupts(&self) {
        self.registers
            .registers
            .intenset
            .write(Interrupt::ENDTX::SET);
    }

    fn disable_rx_interrupts(&self) {
        self.registers
            .registers
            .intenclr
            .write(Interrupt::ENDRX::SET);
    }

    fn disable_tx_interrupts(&self) {
        self.registers
            .registers
            .intenclr
            .write(Interrupt::ENDTX::SET);
    }

    /// UART interrupt handler that listens for both tx_end and rx_end events
    #[inline(never)]
    pub fn handle_interrupt(&self) {
        if self.tx_ready() {
            self.disable_tx_interrupts();

            if let Some((mut buf, transmitted_length)) = self.registers.finish_tx_dma() {
                let active_range = buf.active_range();

                // Calculate the remaining bytes to transmit based on the length to
                // transmit, the window we just tried to transmit, and how many
                // bytes we actually did transmit.
                //
                // <-----buffer------------------->
                //          <-active range->
                // [        [              ]      ]
                let remaining_bytes = self
                    .tx_len
                    .get()
                    .saturating_sub(active_range.start)
                    .saturating_sub(transmitted_length);

                if remaining_bytes == 0 {
                    // We sent everything.
                    self.tx_client.map(|client| {
                        client.transmitted_buffer(buf.take(), self.tx_len.get(), Ok(()));
                    });
                } else {
                    // Send the next portion of the buffer.

                    // Reset back to the original slice.
                    buf.reset();
                    // Limit to just the portion of the buffer we are transmitting from.
                    buf.slice(0..self.tx_len.get());
                    // Skip what has already been transmitted.
                    buf.slice((self.tx_len.get() - remaining_bytes)..);
                    // Limit to at most the `UARTE_MAX_BUFFER_SIZE` bytes.
                    buf.slice(0..UARTE_MAX_BUFFER_SIZE);
                    // Send via DMA.
                    let _ = self.registers.start_tx_dma(buf);
                    // Re-enable interrupts.
                    self.enable_tx_interrupts();
                }
            }
        }

        if self.rx_ready() {
            self.disable_rx_interrupts();

            // Clear the ENDRX event
            self.registers
                .registers
                .event_endrx
                .write(Event::READY::CLEAR);

            if let Some((mut buf, received_length)) = self.registers.finish_rx_dma() {
                let active_range = buf.active_range();

                // Check if this ENDRX is due to an abort. If so, we want to
                // do the receive callback immediately.
                if self.rx_abort_in_progress.get() {
                    self.rx_abort_in_progress.set(false);

                    // Calculate how many bytes we actually received.
                    let received_bytes = active_range.start + received_length;

                    // Notify the client.
                    self.rx_client.map(|client| {
                        client.received_buffer(
                            buf.take(),
                            received_bytes,
                            Err(ErrorCode::CANCEL),
                            uart::Error::None,
                        );
                    });
                } else {
                    // In the normal case, we need to either pass call the callback
                    // or do another read to get more bytes.

                    // Calculate the remaining bytes to receive based on the length to
                    // receive, the window we just tried to receive into, and how many
                    // bytes we actually did receive.
                    //
                    //
                    // <-----buffer------------------->
                    //          <-active range->
                    // [        [              ]      ]
                    let remaining_bytes = self
                        .rx_len
                        .get()
                        .saturating_sub(active_range.start)
                        .saturating_sub(received_length);

                    if remaining_bytes == 0 {
                        // Signal client that the read is done
                        self.rx_client.map(|client| {
                            client.received_buffer(
                                buf.take(),
                                self.rx_len.get(),
                                Ok(()),
                                uart::Error::None,
                            );
                        });
                    } else {
                        // Receive into the next portion of the buffer.

                        // Reset back to the original slice.
                        buf.reset();
                        // Limit to just the portion of the buffer we are receiving into.
                        buf.slice(0..self.rx_len.get());
                        // Skip what has already been received.
                        buf.slice((self.rx_len.get() - remaining_bytes)..);
                        // Limit to at most the `UARTE_MAX_BUFFER_SIZE` bytes.
                        buf.slice(0..UARTE_MAX_BUFFER_SIZE);
                        // Receive via DMA.
                        let _ = self.registers.start_rx_dma(buf);
                        // Re-enable interrupts.
                        self.enable_rx_interrupts();
                    }
                }
            }
        }
    }

    /// Transmit one byte at the time and the client is responsible for polling
    /// This is used by the panic handler
    pub unsafe fn send_byte(&self, byte: u8) {
        // self.tx_remaining_bytes.set(1);
        self.registers
            .registers
            .event_endtx
            .write(Event::READY::CLEAR);
        // precaution: copy value into variable with static lifetime
        BYTE = byte;
        self.registers
            .registers
            .txd_ptr
            .set(core::ptr::addr_of!(BYTE) as u32);
        self.registers
            .registers
            .txd_maxcnt
            .write(Counter::COUNTER.val(1));
        self.registers
            .registers
            .task_starttx
            .write(Task::ENABLE::SET);
    }

    /// Check if the UART transmission is done
    pub fn tx_ready(&self) -> bool {
        self.registers.registers.event_endtx.is_set(Event::READY)
    }

    /// Check if either the rx_buffer is full or the UART has timed out
    pub fn rx_ready(&self) -> bool {
        self.registers.registers.event_endrx.is_set(Event::READY)
    }

    // Helper function used by both transmit_word and transmit_buffer
    fn setup_buffer_transmit(&self, buf: &'static mut [u8], tx_len: usize) {
        // Save the total length to transmit as we may need to send over
        // multiple iterations.
        self.tx_len.set(tx_len);

        // Create a `SubSlice` to simplify tracking which part we are sending,
        // and slice the sub slice to either the total buffer to send or what
        // fits in one send buffer.
        let mut slice_to_send = SubSliceMut::new(buf);
        slice_to_send.slice(0..core::cmp::min(tx_len, UARTE_MAX_BUFFER_SIZE));

        // Send the buffer using DMA. This is managed by the register manager to
        // ensure we are safely using DMA.
        let _ = self.registers.start_tx_dma(slice_to_send);

        // Enable interrupts so we get a interrupt when the transmission has
        // finished.
        self.enable_tx_interrupts();
    }

    fn setup_buffer_receive(&self, buf: &'static mut [u8], rx_len: usize) {
        // Save the total length to receive.
        self.rx_len.set(rx_len);

        // Create a `SubSlice` to simplify tracking which part we have received
        // into.
        let mut slice_to_receive = SubSliceMut::new(buf);
        slice_to_receive.slice(0..core::cmp::min(rx_len, UARTE_MAX_BUFFER_SIZE));

        // Use the buffer with DMA. This is managed by the register manager to
        // ensure we are safely using DMA.
        let _ = self.registers.start_rx_dma(slice_to_receive);

        // Enable interrupts so we get a interrupt when the receive has
        // finished.
        self.enable_rx_interrupts();
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
        } else if self.registers.tx_dma_pending() {
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
        if rx_len > rx_buf.len() {
            Err((ErrorCode::SIZE, rx_buf))
        } else if self.registers.rx_dma_pending() {
            Err((ErrorCode::BUSY, rx_buf))
        } else {
            self.setup_buffer_receive(rx_buf, rx_len);
            Ok(())
        }
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        // Trigger the STOPRX event to cancel the current receive call.
        if !self.registers.rx_dma_pending() {
            Ok(())
        } else {
            self.rx_abort_in_progress.set(true);
            self.registers
                .registers
                .task_stoprx
                .write(Task::ENABLE::SET);
            Err(ErrorCode::BUSY)
        }
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
    inner: Uarte<'a>,
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

impl kernel::platform::chip::PanicWriter for Uarte<'_> {
    type Config = UartPanicWriterConfig;

    unsafe fn create_panic_writer(config: Self::Config) -> impl IoWrite + core::fmt::Write {
        use uart::Configure as _;

        let inner = Uarte::new(UARTE0_BASE);
        inner.initialize(
            pinmux::Pinmux::from_pin(config.txd),
            pinmux::Pinmux::from_pin(config.rxd),
            config.cts.map(|c| unsafe { pinmux::Pinmux::from_pin(c) }),
            config.rts.map(|r| unsafe { pinmux::Pinmux::from_pin(r) }),
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
