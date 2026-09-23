// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.
// Copyright Tock Contributors 2026.

use crate::{
    dma::{ChannelId, Dma, DmaPeripheral},
    rcc::hertz::Hertz,
};
use core::cell::Cell;
use cortexm33::dma_fence::CortexMDmaFence;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart::{self};
use kernel::platform::chip::PanicWriter;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::io_write::IoWrite;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::FieldValue;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

register_structs! {
    pub UsartRegisters {
        /// Control register 1
        (0x000 => pub cr1: ReadWrite<u32, CR1::Register>),
        /// Control register 2
        (0x004 => pub cr2: ReadWrite<u32, CR2::Register>),
        /// Control register 3
        (0x008 => pub cr3: ReadWrite<u32, CR3::Register>),
        /// Baud rate register
        (0x00C => pub brr: ReadWrite<u32, BRR::Register>),
        /// Guard time and prescaler register
        (0x010 => pub gtpr: ReadWrite<u32, GTPR::Register>),
        /// Receiver timeout register
        (0x014 => pub rtor: ReadWrite<u32, RTOR::Register>),
        /// Request register
        (0x018 => pub rqr: WriteOnly<u32, RQR::Register>),
        /// Interrupt and status register
        (0x01C => pub isr: ReadOnly<u32, ISR::Register>),
        /// Interrupt flag clear register
        (0x020 => pub icr: WriteOnly<u32, ICR::Register>),
        /// Receive data register
        (0x024 => pub rdr: ReadOnly<u32, RDR::Register>),
        /// Transmit data register
        (0x028 => pub tdr: ReadWrite<u32, TDR::Register>),
        /// Prescaler register
        (0x02C => pub presc: ReadWrite<u32, PRESC::Register>),
        (0x030 => @END),
    }
}

/// Base address for USART1 in Secure Alias mode.
pub const USART1_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x50013800 as *const UsartRegisters) };

register_bitfields![u32,
    pub CR1 [
        /// USART enable
        UE      OFFSET(0)   NUMBITS(1) [],
        /// Receiver enable
        RE      OFFSET(2)   NUMBITS(1) [],
        /// Transmitter enable
        TE      OFFSET(3)   NUMBITS(1) [],
        /// RXNE interrupt enable
        RXNEIE  OFFSET(5)   NUMBITS(1) [],
        /// Transmission complete interrupt enable
        TCIE    OFFSET(6)   NUMBITS(1) [],
        /// TXE interrupt enable
        TXEIE   OFFSET(7)   NUMBITS(1) [],
        /// Oversampling mode
        OVER8   OFFSET(15)  NUMBITS(1) []
    ],
    pub CR2 [
        /// STOP bits
        STOP    OFFSET(12)  NUMBITS(2) []
    ],
    pub CR3 [
        /// Error interrupt enable
        EIE     OFFSET(0)   NUMBITS(1) [],
        /// DMA enable transmitter
        DMAT    OFFSET(7)   NUMBITS(1) [],
        /// DMA enable receiver
        DMAR    OFFSET(6)   NUMBITS(1) []
    ],
    pub BRR [
        /// Baud rate divider
        BRR OFFSET(0) NUMBITS(16) []
    ],
    pub GTPR [
        /// Guard time value
        GT OFFSET(8) NUMBITS(8) [],
        /// Prescaler value
        PSC OFFSET(0) NUMBITS(8) []
    ],
    pub RTOR [
        /// Receiver timeout value
        RTO OFFSET(0) NUMBITS(24) [],
        /// Block length
        BLEN OFFSET(24) NUMBITS(8) []
    ],
    pub RQR [
        /// Transmit data flush request
        TXFRQ OFFSET(4) NUMBITS(1) [],
        /// Receive data flush request
        RXFRQ OFFSET(3) NUMBITS(1) [],
        /// Mute mode request
        MMRQ OFFSET(2) NUMBITS(1) [],
        /// Send break request
        SBKRQ OFFSET(1) NUMBITS(1) [],
        /// Auto baud rate request
        ABRRQ OFFSET(0) NUMBITS(1) []
    ],
    pub ISR [
        /// Transmission complete
        TC   OFFSET(6) NUMBITS(1) [],
        /// Read data register not empty
        RXNE OFFSET(5) NUMBITS(1) [],
        /// Transmit data register empty
        TXE  OFFSET(7) NUMBITS(1) [],
        /// Overrun error
        ORE  OFFSET(3) NUMBITS(1) [],
        /// Noise detected flag
        NE   OFFSET(2) NUMBITS(1) [],
        /// Framing error
        FE   OFFSET(1) NUMBITS(1) [],
        /// Parity error
        PE   OFFSET(0) NUMBITS(1) []
    ],
    pub ICR [
        /// Transmission complete clear flag
        TCCF   OFFSET(6) NUMBITS(1) [],
        /// Overrun error clear flag
        ORECF  OFFSET(3) NUMBITS(1) [],
        /// Noise detected clear flag
        NECF   OFFSET(2) NUMBITS(1) [],
        /// Framing error clear flag
        FECF   OFFSET(1) NUMBITS(1) [],
        /// Parity error clear flag
        PECF   OFFSET(0) NUMBITS(1) []
    ],
    pub RDR [
        /// Receive data value
        RDR OFFSET(0) NUMBITS(9) []
    ],
    pub TDR [
        /// Transmit data value
        TDR OFFSET(0) NUMBITS(9) []
    ],
    pub PRESC [
        /// Clock prescaler
        PRESCALER OFFSET(0) NUMBITS(4) [
            DIV1 = 0,
            DIV2 = 1,
            DIV4 = 2,
            DIV6 = 3,
            DIV8 = 4,
            DIV10 = 5,
            DIV12 = 6,
            DIV16 = 7,
            DIV32 = 8,
            DIV64 = 9,
            DIV128 = 10,
            DIV256 = 11
        ]
    ]
];

/// USART driver implementation for the STM32U5 series.
///
/// This driver uses the GPDMA (General Purpose Direct Memory Access) controller
/// for both transmitting and receiving data, which provides high efficiency and
/// ensures that fast data bursts (such as arrow key escape sequences) are
/// captured correctly.
pub struct Usart<'a> {
    pub registers: StaticRef<UsartRegisters>,
    clock: OptionalCell<Hertz>,
    dma: OptionalCell<&'a Dma>,
    dma_channel_tx: Cell<Option<ChannelId>>,
    dma_channel_rx: Cell<Option<ChannelId>>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    tx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    rx_dma_buf: MapCell<DmaSubSliceMut<'static, u8>>,
    tx_len: Cell<usize>,
    rx_len: Cell<usize>,
    deferred_call: DeferredCall,
    tx_deferred: Cell<bool>,
    rx_deferred: Cell<bool>,
}

impl<'a> Usart<'a> {
    pub fn new(base: StaticRef<UsartRegisters>) -> Self {
        Self {
            registers: base,
            clock: OptionalCell::empty(),
            dma: OptionalCell::empty(),
            dma_channel_tx: Cell::new(None),
            dma_channel_rx: Cell::new(None),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_dma_buf: MapCell::empty(),
            rx_dma_buf: MapCell::empty(),
            tx_len: Cell::new(0),
            rx_len: Cell::new(0),
            deferred_call: DeferredCall::new(),
            tx_deferred: Cell::new(false),
            rx_deferred: Cell::new(false),
        }
    }

    pub fn set_clock(&self, clock: Hertz) {
        self.clock.set(clock);
    }

    /// The smallest baud rate divisor the hardware accepts, per RM0456 § 62.5.4
    ///
    /// `BRR` resets to zero, so a value of at least this means that a baud rate
    /// was set at some point.
    const MIN_BRR: u32 = 0x10;

    /// The kernel clock frequency the panic writer assumes when it has to set
    /// a baud rate itself
    ///
    /// This is PCLK2, which `Stm32u5xxDefaultPeripherals::init()` runs from the
    /// 16 MHz HSI.
    const PANIC_DEFAULT_CLOCK: Hertz = Hertz::mhz(16);

    // Adapted from embassy-rs/embassy/embassy-stm32/src/uart/mod.rs
    fn calculate_brr(baud: u32, pclk: u32, presc: u32, mul: u32) -> u32 {
        // The calculation to be done to get the BRR is `mul * pclk / presc / baud`
        // To do this in 32-bit only we can't multiply `mul` and `pclk`
        let clock = pclk / presc;

        // The mul is applied as the last operation to prevent overflow
        let brr = clock / baud * mul;

        // The BRR calculation will be a bit off because of integer rounding.
        // Because we multiplied our inaccuracy with mul, our rounding now needs to be in proportion to mul.
        let rounding = ((clock % baud) * mul + (baud / 2)) / baud;

        brr + rounding
    }

    // Adapted from embassy-rs/embassy/embassy-stm32/src/uart/mod.rs
    //
    // Takes the registers rather than `&self` so that the panic writer, which
    // has no access to the driver instance, can reuse this calculation.
    fn find_and_set_baudrate(
        registers: &UsartRegisters,
        baudrate: u32,
        kernel_clock: Hertz,
    ) -> Result<(), BaudrateError> {
        const DIVS: [(u16, FieldValue<u32, PRESC::Register>); 12] = [
            (1, PRESC::PRESCALER::DIV1),
            (2, PRESC::PRESCALER::DIV2),
            (4, PRESC::PRESCALER::DIV4),
            (6, PRESC::PRESCALER::DIV6),
            (8, PRESC::PRESCALER::DIV8),
            (10, PRESC::PRESCALER::DIV10),
            (12, PRESC::PRESCALER::DIV12),
            (16, PRESC::PRESCALER::DIV16),
            (32, PRESC::PRESCALER::DIV32),
            (64, PRESC::PRESCALER::DIV64),
            (128, PRESC::PRESCALER::DIV128),
            (256, PRESC::PRESCALER::DIV256),
        ];

        let (mul, brr_min, brr_max) = (1, Self::MIN_BRR, 0x1_0000);

        // Find the smallest prescaler which yields a divisor in range, along
        // with the `BRR` value to write and whether it needs 8x oversampling.
        // No prescaler at all means that even the largest one leaves the
        // divisor too big, i.e. the baud rate is too low for this clock.
        let Some(found) = DIVS.iter().find_map(|&(presc, presc_fields_value)| {
            let brr = Self::calculate_brr(baudrate, kernel_clock.0, presc as u32, mul);

            if brr < brr_min {
                // Larger prescalers only make the divisor smaller, so
                // this prescaler decides the outcome either way.
                Some(if brr * 2 >= brr_min {
                    Ok((((brr << 1) & !0xF) | (brr & 0x07), presc_fields_value, true))
                } else {
                    Err(BaudrateError::TooHigh)
                })
            } else if brr < brr_max {
                Some(Ok((brr, presc_fields_value, false)))
            } else {
                None
            }
        }) else {
            return Err(BaudrateError::TooLow);
        };
        let (brr, presc, over8) = found?;

        registers.brr.write(BRR::BRR.val(brr));
        registers.presc.write(presc);
        registers.cr1.modify(if over8 {
            CR1::OVER8::SET
        } else {
            CR1::OVER8::CLEAR
        });

        Ok(())
    }

    /// Bring the USART up for the given parameters and kernel clock frequency.
    ///
    /// Shared by the [`uart::Configure`] implementation and by the panic
    /// writer, which uses it when the kernel did not configure the USART
    /// before the panic.
    ///
    /// Only the baud rate from `params` is honored; the remaining parameters
    /// are fixed at the reset configuration of 8 data bits, no parity and one
    /// stop bit.
    fn configure_registers(
        registers: &UsartRegisters,
        params: uart::Parameters,
        kernel_clock: Hertz,
    ) -> Result<(), kernel::ErrorCode> {
        // The baud rate registers can only be written while the USART is
        // disabled.
        registers.cr1.modify(CR1::UE::CLEAR);

        // Set the baud rate
        Self::find_and_set_baudrate(registers, params.baud_rate, kernel_clock)
            .map_err(|_| kernel::ErrorCode::INVAL)?;

        registers.icr.write(
            ICR::TCCF::SET + ICR::ORECF::SET + ICR::NECF::SET + ICR::FECF::SET + ICR::PECF::SET,
        );

        // Enable transmitter, receiver, and USART. This must preserve the
        // `OVER8` bit, which `find_and_set_baudrate` selected as part of the
        // baud rate divisor.
        registers.cr1.modify(
            CR1::TE::SET
                + CR1::RE::SET
                + CR1::UE::SET
                + CR1::TXEIE::CLEAR
                + CR1::TCIE::CLEAR
                + CR1::RXNEIE::CLEAR,
        );

        Ok(())
    }

    /// Associates a DMA controller and channels with the USART driver.
    pub fn set_dma(
        usart: &'static Self,
        dma: &'a Dma,
        tx_channel: ChannelId,
        rx_channel: ChannelId,
    ) {
        usart.dma.set(dma);
        usart.dma_channel_tx.set(Some(tx_channel));
        usart.dma_channel_rx.set(Some(rx_channel));
        dma.set_client(tx_channel, usart);
        dma.set_client(rx_channel, usart);
    }

    /// Hardware interrupt handler for the USART.
    ///
    /// This handles non-DMA events like errors. DMA-specific completions
    /// are handled in `handle_dma_interrupt`.
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        let isr = regs.isr.extract();

        // Clear any error flags (Parity, Framing, Noise, Overrun).
        if isr.any_matching_bits_set(ISR::PE::SET + ISR::FE::SET + ISR::NE::SET + ISR::ORE::SET) {
            regs.icr
                .write(ICR::PECF::SET + ICR::FECF::SET + ICR::NECF::SET + ICR::ORECF::SET);
        }
    }

    /// Handles completion interrupts from the GPDMA controller.
    ///
    /// This is called when a DMA transfer for either Transmit or Receive
    /// has finished.
    pub fn handle_dma_interrupt(&self, is_tx: bool) {
        if is_tx {
            self.dma.map(|dma| {
                if let Some(ch) = self.dma_channel_tx.get() {
                    dma.clear_interrupt(ch);
                }
            });
            self.registers.cr3.modify(CR3::DMAT::CLEAR);
            self.tx_deferred.set(false);
            if let Some(dma_slice) = self.tx_dma_buf.take() {
                let fence = unsafe { CortexMDmaFence::new() };
                let mut subslice = unsafe { dma_slice.take(fence) };
                subslice.reset();
                let buf = subslice.take();
                let len = self.tx_len.get();
                self.tx_client.map(move |client| {
                    client.transmitted_buffer(buf, len, Ok(()));
                });
            }
        } else {
            self.dma.map(|dma| {
                if let Some(ch) = self.dma_channel_rx.get() {
                    dma.clear_interrupt(ch);
                }
            });
            self.registers.cr3.modify(CR3::DMAR::CLEAR);
            self.rx_deferred.set(false);
            if let Some(dma_slice) = self.rx_dma_buf.take() {
                let fence = unsafe { CortexMDmaFence::new() };
                let mut subslice = unsafe { dma_slice.take(fence) };
                subslice.reset();
                let buf = subslice.take();
                let len = self.rx_len.get();
                self.rx_client.map(move |client| {
                    client.received_buffer(buf, len, Ok(()), uart::Error::None);
                });
            }
        }
    }

    /// Synchronous (Blocking) send.
    /// ONLY for use in the Panic handler when DMA is unavailable.
    pub fn transmit_byte(&self, byte: u8) {
        let regs = &*self.registers;

        // 1. Disable DMA transmitter temporarily if it was enabled
        // so that manual writes to TDR are processed correctly.
        let dmat_enabled = regs.cr3.is_set(CR3::DMAT);
        if dmat_enabled {
            regs.cr3.modify(CR3::DMAT::CLEAR);
        }

        // 2. Wait until TXE (Transmit Data Register Empty) is set
        while !regs.isr.is_set(ISR::TXE) {}
        regs.tdr.write(TDR::TDR.val(byte as u32));

        // 3. Wait for Transmission Complete before potentially re-enabling DMA
        while !regs.isr.is_set(ISR::TC) {}

        // 4. Restore DMA state
        if dmat_enabled {
            regs.cr3.modify(CR3::DMAT::SET);
        }
    }
}

enum BaudrateError {
    TooHigh,
    TooLow,
}

impl DeferredCallClient for Usart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if self.tx_deferred.get() {
            self.tx_deferred.set(false);
            self.tx_client.map(move |client| {
                let dma_slice = self.tx_dma_buf.take().unwrap();
                let fence = unsafe { CortexMDmaFence::new() };
                let mut subslice = unsafe { dma_slice.take(fence) };
                subslice.reset();
                let buf = subslice.take();
                client.transmitted_buffer(buf, 0, Err(kernel::ErrorCode::CANCEL));
            });
        }
        if self.rx_deferred.get() {
            self.rx_deferred.set(false);
            self.rx_client.map(move |client| {
                let dma_slice = self.rx_dma_buf.take().unwrap();
                let fence = unsafe { CortexMDmaFence::new() };
                let mut subslice = unsafe { dma_slice.take(fence) };
                subslice.reset();
                let buf = subslice.take();
                client.received_buffer(
                    buf,
                    0,
                    Err(kernel::ErrorCode::CANCEL),
                    uart::Error::Aborted,
                );
            });
        }
    }
}

impl crate::dma::DmaClient for Usart<'_> {
    fn transfer_done(&self, channel: ChannelId) {
        if let Some(tx_ch) = self.dma_channel_tx.get() {
            if channel == tx_ch {
                self.handle_dma_interrupt(true);
                return;
            }
        }
        if let Some(rx_ch) = self.dma_channel_rx.get() {
            if channel == rx_ch {
                self.handle_dma_interrupt(false);
            }
        }
    }
}

impl<'a> uart::Transmit<'a> for Usart<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        if self.tx_dma_buf.is_some() {
            return Err((kernel::ErrorCode::BUSY, tx_buffer));
        }

        let Some(dma) = self.dma.get() else {
            return Err((kernel::ErrorCode::OFF, tx_buffer));
        };

        // Move buffer into SubSlice
        let mut subslice = SubSliceMut::new(tx_buffer);
        subslice.slice(0..tx_len);

        // Hardware fence
        let fence = unsafe { CortexMDmaFence::new() };
        // Convert subslice into DmaSlice
        let dma_slice = DmaSubSliceMut::new_static(subslice, fence);

        // Extract the physical pointer and length for MMIO
        let ptr = dma_slice.as_mut_ptr() as u32;
        let len = dma_slice.len() as u32;

        // Save DmaSlice in the struct
        self.tx_dma_buf.replace(dma_slice);
        self.tx_len.set(tx_len);

        // Trigger USART
        if let Some(ch) = self.dma_channel_tx.get() {
            dma.setup(ch, DmaPeripheral::Usart1Tx, ptr, len);
            self.registers.cr3.modify(CR3::DMAT::SET);
            Ok(())
        } else {
            self.tx_dma_buf
                .take()
                .map(|s| {
                    let f = unsafe { CortexMDmaFence::new() };
                    let mut buf = unsafe { s.take(f) };
                    buf.reset();
                    Err((kernel::ErrorCode::RESERVE, buf.take()))
                })
                .unwrap()
        }
    }

    fn transmit_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cr3.modify(CR3::DMAT::CLEAR);
        if self.tx_dma_buf.is_some() {
            self.tx_deferred.set(true);
            self.deferred_call.set();
            Err(kernel::ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}

impl uart::Configure for Usart<'_> {
    fn configure(&self, params: uart::Parameters) -> Result<(), kernel::ErrorCode> {
        // Get the clock frequency that feeds this USART
        // It must have been provided with `set_clock` at this point
        let Some(clock_frequency) = self.clock.get() else {
            return Err(kernel::ErrorCode::FAIL);
        };

        Self::configure_registers(&self.registers, params, clock_frequency)
    }
}

impl<'a> uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        if self.rx_dma_buf.is_some() {
            return Err((kernel::ErrorCode::BUSY, rx_buffer));
        }

        let Some(dma) = self.dma.get() else {
            return Err((kernel::ErrorCode::OFF, rx_buffer));
        };

        let mut subslice = SubSliceMut::new(rx_buffer);
        subslice.slice(0..rx_len);
        let fence = unsafe { CortexMDmaFence::new() };
        let dma_slice = DmaSubSliceMut::new_static(subslice, fence);

        let ptr = dma_slice.as_mut_ptr() as u32;
        let len = dma_slice.len() as u32;

        self.rx_dma_buf.replace(dma_slice);
        self.rx_len.set(rx_len);

        if let Some(ch) = self.dma_channel_rx.get() {
            dma.setup(ch, DmaPeripheral::Usart1Rx, ptr, len);
            self.registers.cr3.modify(CR3::DMAR::SET);
            Ok(())
        } else {
            self.rx_dma_buf
                .take()
                .map(|s| {
                    let f = unsafe { CortexMDmaFence::new() };
                    let mut buf = unsafe { s.take(f) };
                    buf.reset();
                    Err((kernel::ErrorCode::RESERVE, buf.take()))
                })
                .unwrap()
        }
    }

    fn receive_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cr3.modify(CR3::DMAR::CLEAR);
        if self.rx_dma_buf.is_some() {
            self.rx_deferred.set(true);
            self.deferred_call.set();
            Err(kernel::ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }

    fn receive_word(&self) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}

struct UsartPanicWriter {
    /// If the panic writer cannot set a reasonable baud rate, or the USART is
    /// not enabled, this is `None` and all writes are ignored. Polling the
    /// status flags a disabled USART would spin forever.
    registers: Option<StaticRef<UsartRegisters>>,
}

impl IoWrite for UsartPanicWriter {
    fn write(&mut self, buf: &[u8]) -> usize {
        let Some(regs) = self.registers else {
            return 0;
        };

        for &byte in buf {
            while !regs.isr.is_set(ISR::TXE) {}
            regs.tdr.write(TDR::TDR.val(byte as u32));
            while !regs.isr.is_set(ISR::TC) {}
        }
        buf.len()
    }
}

impl core::fmt::Write for UsartPanicWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

/// Configuration for the synchronous USART panic writer.
///
/// A USART which the kernel already configured is used as is. Otherwise, this
/// captures everything needed to set it up for panic display.
pub struct UsartPanicWriterConfig {
    pub registers: StaticRef<UsartRegisters>,
    /// The parameters to fall back to when the kernel did not set a baud rate
    ///
    /// A USART which the kernel already configured is used as is, so that the
    /// panic output keeps the settings of the console it interrupts.
    pub params: uart::Parameters,
}

impl PanicWriter for Usart<'_> {
    type Config = UsartPanicWriterConfig;

    fn create_panic_writer(
        config: Self::Config,
        _panic: &core::panic::PanicInfo,
    ) -> impl IoWrite + core::fmt::Write {
        let registers = config.registers;

        // A transfer may have been in flight when the panic occurred. Stop the
        // peripheral from issuing further DMA requests, so that the bytes this
        // writer polls out are not interleaved with DMA-driven ones.
        registers.cr3.modify(CR3::DMAT::CLEAR + CR3::DMAR::CLEAR);

        if registers.brr.read(BRR::BRR) >= Self::MIN_BRR {
            // The kernel already set a baud rate, so the USART is configured.
            // Keep that configuration and only make sure the transmitter is on
            // and that no interrupts fire while the bytes are polled out.
            registers.cr1.modify(
                CR1::TE::SET
                    + CR1::UE::SET
                    + CR1::TXEIE::CLEAR
                    + CR1::TCIE::CLEAR
                    + CR1::RXNEIE::CLEAR,
            );
        } else {
            // No baud rate was set yet, so configure the USART from `params`
            // and the default kernel clock. Unlike other chips, this does not
            // go through `uart::Configure` on a fresh `Usart`, because
            // constructing one would claim another deferred call slot.
            //
            // The result is not needed: the check below reads the outcome back
            // from the hardware, which also catches an unclocked USART that
            // ignored the writes even though they "succeeded".
            let _ = Self::configure_registers(&registers, config.params, Self::PANIC_DEFAULT_CLOCK);
        }

        // If no reasonable baud rate could be set, or the USART is not enabled
        // (e.g. it is not clocked and its registers read as zero), ignore all
        // writes rather than spin forever polling its status flags.
        let transmit_available =
            registers.brr.read(BRR::BRR) >= Self::MIN_BRR && registers.cr1.is_set(CR1::UE);

        UsartPanicWriter {
            registers: if transmit_available {
                Some(registers)
            } else {
                None
            },
        }
    }
}
