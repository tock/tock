// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! UARTE driver, nRF5X-family, unsafe code

use kernel::utilities::StaticRef;
use kernel::utilities::cells::MapCell;
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_bitfields};

#[repr(C)]
pub struct UarteRegisters {
    task_startrx: WriteOnly<u32, Task::Register>,
    pub task_stoprx: WriteOnly<u32, Task::Register>,
    task_starttx: WriteOnly<u32, Task::Register>,
    pub task_stoptx: WriteOnly<u32, Task::Register>,
    _reserved1: [u32; 7],
    task_flush_rx: WriteOnly<u32, Task::Register>,
    _reserved2: [u32; 52],
    event_cts: ReadWrite<u32, Event::Register>,
    event_ncts: ReadWrite<u32, Event::Register>,
    _reserved3: [u32; 2],
    pub event_endrx: ReadWrite<u32, Event::Register>,
    _reserved4: [u32; 3],
    pub event_endtx: ReadWrite<u32, Event::Register>,
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
    pub intenset: ReadWrite<u32, Interrupt::Register>,
    pub intenclr: ReadWrite<u32, Interrupt::Register>,
    _reserved11: [u32; 93],
    errorsrc: ReadWrite<u32, ErrorSrc::Register>,
    _reserved12: [u32; 31],
    pub enable: ReadWrite<u32, Uart::Register>,
    _reserved13: [u32; 1],
    pub pselrts: ReadWrite<u32, Psel::Register>,
    pub pseltxd: ReadWrite<u32, Psel::Register>,
    pub pselcts: ReadWrite<u32, Psel::Register>,
    pub pselrxd: ReadWrite<u32, Psel::Register>,
    _reserved14: [u32; 3],
    pub baudrate: ReadWrite<u32, Baudrate::Register>,
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
    pub Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    pub Event [
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
    pub Interrupt [
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
    pub Uart [
        ENABLE OFFSET(0) NUMBITS(4) [
            ON = 8,
            OFF = 0
        ]
    ],

    /// Pin select
    pub Psel [
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
pub struct UarteRegistersManager {
    /// MMIO registers for the UARTE peripheral.
    pub registers: StaticRef<UarteRegisters>,
    /// Holding place for the TX DMA buffer while DMA in progress.
    tx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    /// Holding place for the RX DMA buffer while DMA in progress.
    rx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
}

impl UarteRegistersManager {
    /// Create a DMA-enabled register manager for UARTE.
    ///
    /// # Safety
    ///
    /// This controls DMA hardware. As such, it must be unique. This requires:
    ///
    /// - This constructor must be called at most once.
    /// - There must not be any other code that accesses the DMA buffer and
    ///   length registers.
    pub unsafe fn new(registers: StaticRef<UarteRegisters>) -> Self {
        Self {
            registers,
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
        // The architecture-provided version is correct for nRF5x chips.
        let fence = unsafe { cortexm::dma_fence::CortexMDmaFence::new() };

        // Create DmaSlice for the TX buffer. This ensures that we can soundly
        // share it with the DMA hardware.
        let tx_dma_slice = DmaSubSliceMut::new_static(buf, fence);

        // Provide the DmaSlice buffer to the hardware DMA engine.
        self.registers.txd_ptr.set(tx_dma_slice.ptr_addr() as u32);

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
            // The architecture-provided version is correct for nRF5x chips.
            let fence = unsafe { cortexm::dma_fence::CortexMDmaFence::new() };

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
        // The architecture-provided version is correct for nRF5x chips.
        let fence = unsafe { cortexm::dma_fence::CortexMDmaFence::new() };

        // Create DmaSlice for the RX buffer. This ensures that we can soundly
        // share it with the DMA hardware.
        let rx_dma_slice = DmaSubSliceMut::new_static(buf, fence);

        // Provide the DmaSlice buffer to the hardware DMA engine.
        self.registers.rxd_ptr.set(rx_dma_slice.ptr_addr() as u32);

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
            // The architecture-provided version is correct for nRF5x chips.
            let fence = unsafe { cortexm::dma_fence::CortexMDmaFence::new() };

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
