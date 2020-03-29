//! Implementation of SPI master for SiFive chips
//!
//! The instantiation of this struct should be done in the specific
//! chip crates.
//!
//! The TX and RX buffers are implemented as a FIFO. To avoid missing
//! any RX bytes, we write a maximum of tx_buffer_depth bytes at once,
//! giving the controller a chance to throw an interrupt and letting
//! us retrieve rx_buffer_depth bytes. This appears to be the same
//! strategy as the Linux kernel driver uses.
//!
//! ## Implementation status
//! - [x] set_client
//! - [x] init
//! - [x] is_busy
//! - [x] read_write_bytes
//! - [ ] write_byte
//! - [ ] read_byte
//! - [ ] read_write_byte
//! - [ ] specify_chip_select
//! - [x] set_rate
//! - [x] get_rate
//! - [x] set_clock
//! - [x] get_clock
//! - [x] set_phase
//! - [x] get_phase
//! - [x] hold_low
//! - [x] release_low
//!
//! ## Current todos
//! - Implement the IO pinmuxing
//!   https://static.dev.sifive.com/SiFive-E300-platform-reference-manual-v1.0.1.pdf p35
//! - Support chip select properly
//!   - CS registers have variable length depending on the SPI
//!     peripheral, hence this should be provided in the constructor
//! - Implement the synchronous functions
//! - Support SPI flash interface (it might be necessary to implement
//!   an entirely different struct / driver for this and use this one
//!   only as a mode select)
//! - There might still be a race condition when the RX takes too long
//!   so we have already emptied the buffer while a read is still in
//!   progress, that way we're going to miss one byte. The code should
//!   be changed so tx is always at most fifo_depth bytes ahead of
//!   rx. That should also make the fifo handling a bit easier to
//!   understand.
//!
//! - Author: Leon Schuermann <leon@is.currently.online>
//! - Date: 2020-03-29

use core::cell::Cell;
use core::cmp;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, LocalRegisterCopy, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::spi::{self, ClockPhase, ClockPolarity, SpiMasterClient};
use kernel::ReturnCode;

/// Serial peripheral interface
#[repr(C)]
pub struct SpiRegisters {
    /// serial clock divisor
    sckdiv: ReadWrite<u32, SCKDIV::Register>,
    /// serial clock mode
    sckmode: ReadWrite<u32, SCKMODE::Register>,
    _reserved0: [u32; 2],
    /// chip select id
    csid: ReadWrite<u32, CSID::Register>,
    /// chip select default
    csdef: ReadWrite<u32, CSDEF::Register>,
    /// chip select mode
    csmode: ReadWrite<u32, CSMODE::Register>,
    _reserved1: [u32; 3],
    /// delay control 0
    delay0: ReadWrite<u32, DELAY0::Register>,
    /// delay control 1
    delay1: ReadWrite<u32, DELAY1::Register>,
    _reserved2: [u32; 4],
    /// frame format
    fmt: ReadWrite<u32, FMT::Register>,
    _reserved3: [u32; 1],
    /// tx FIFO data
    txdata: ReadWrite<u32, TXDATA::Register>,
    /// rx FIFO data
    rxdata: ReadOnly<u32, RXDATA::Register>,
    /// tx FIFO watermark
    txmark: ReadWrite<u32, TXMARK::Register>,
    /// rx FIFO watermark
    rxmark: ReadWrite<u32, RXMARK::Register>,
    _reserved4: [u32; 2],
    /// SPI flash interface control
    fctrl: ReadWrite<u32>,
    /// SPI flash instruction format
    ffmt: ReadWrite<u32>,
    _reserved5: [u32; 2],
    /// SPI interrupt enable
    ie: ReadWrite<u32, IE::Register>,
    /// SPI interrupt pending
    ip: ReadWrite<u32, IP::Register>,
}

register_bitfields![u32,
    SCKDIV [
        /// Divisor for serial clock, div_width bits wide
        ///
        /// The relationship between input clock and SCK is given by f_sck = f_in / 2(div + 1)
        // TODO: This should have a flexible bit width, set per SPI controller instance
        DIV OFFSET(0) NUMBITS(12) []
    ],
    SCKMODE [
        /// Serial clock polarity
        POL OFFSET(1) NUMBITS(1) [
            IdleLow = 0,
            IdleHigh = 1
        ],
        /// Serial clock phase
        PHA OFFSET(0) NUMBITS(1) [
            SampleLeading = 0,
            SampleTrailing = 1
        ]
    ],
    CSID [
        /// Chip select id
        // TODO: Make this log_2(cs_width) bits wide. Currently cs_width = 1
        // TODO: This should be 0 bits wide for one CS
        CSID OFFSET(0) NUMBITS(1) []
    ],
    CSDEF [
        /// Chip select default value (inactive state, polarity)
        // TODO: Make this cs_width bits wide. Currently, cs_width = 1
        CSDEF OFFSET(0) NUMBITS(1) []
    ],
    CSMODE [
        /// Chip select mode
        MODE OFFSET(0) NUMBITS(2) [
            /// Assert/deassert CS at the beginning/end of each frame
            Auto = 0,
            /// Keep CS continuously asserted after the initial frame
            Hold = 2,
            /// Disable HW control of the CS pin
            Off = 3
        ]
    ],
    DELAY0 [
        /// SCK to CS delay
        SCKCS OFFSET(16) NUMBITS(8) [],
        /// CS to SCK delay
        CSSCK OFFSET(0) NUMBITS(8) []
    ],
    DELAY1 [
        /// Maximum interframe delay
        INTERXFR OFFSET(16) NUMBITS(8) [],
        /// Minimum CS inactive time
        INTERCS OFFSET(0) NUMBITS(8) []
    ],
    FMT [
        /// Number of bits per frame
        LEN OFFSET(16) NUMBITS(4) [],
        /// SPI I/O direction (reset to 1 for flash-enabled SPI controllers, otherwise 0)
        DIR OFFSET(3) NUMBITS(1) [
            RX = 0,
            TX = 1
        ],
        /// SPI endianess
        ENDIAN OFFSET(2) NUMBITS(1) [
            MSB = 0,
            LSB = 1
        ],
        /// SPI protocol
        PROTO OFFSET(0) NUMBITS(2) [
            Single = 0,
            Dual = 1,
            Quad = 2
        ]
    ],
    TXDATA [
        FULL OFFSET(31) NUMBITS(1) [],
        DATA OFFSET(0) NUMBITS(8) []
    ],
    RXDATA [
        EMPTY OFFSET(31) NUMBITS(1) [],
        DATA OFFSET(0) NUMBITS(8) []
    ],
    TXMARK [
        TXMARK OFFSET(0) NUMBITS(3) []
    ],
    RXMARK [
        RXMARK OFFSET(0) NUMBITS(3) []
    ],
    IE [
        RXWM OFFSET(1) NUMBITS(1) [],
        TXWM OFFSET(0) NUMBITS(1) []
    ],
    IP [
        RXWM OFFSET(1) NUMBITS(1) [],
        TXWM OFFSET(0) NUMBITS(1) []
    ]
];

impl From<ClockPhase> for SCKMODE::PHA::Value {
    fn from(phase: ClockPhase) -> Self {
        match phase {
            ClockPhase::SampleLeading => SCKMODE::PHA::Value::SampleLeading,
            ClockPhase::SampleTrailing => SCKMODE::PHA::Value::SampleTrailing,
        }
    }
}
impl From<SCKMODE::PHA::Value> for ClockPhase {
    fn from(phase: SCKMODE::PHA::Value) -> Self {
        match phase {
            SCKMODE::PHA::Value::SampleLeading => ClockPhase::SampleLeading,
            SCKMODE::PHA::Value::SampleTrailing => ClockPhase::SampleTrailing,
        }
    }
}

impl From<ClockPolarity> for SCKMODE::POL::Value {
    fn from(polarity: ClockPolarity) -> Self {
        match polarity {
            ClockPolarity::IdleLow => SCKMODE::POL::Value::IdleLow,
            ClockPolarity::IdleHigh => SCKMODE::POL::Value::IdleHigh,
        }
    }
}
impl From<SCKMODE::POL::Value> for ClockPolarity {
    fn from(polarity: SCKMODE::POL::Value) -> Self {
        match polarity {
            SCKMODE::POL::Value::IdleLow => ClockPolarity::IdleLow,
            SCKMODE::POL::Value::IdleHigh => ClockPolarity::IdleHigh,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SpiChipSelect {
    CS0,
}

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,

    master_client: OptionalCell<&'a dyn hil::spi::SpiMasterClient>,

    initialized: Cell<bool>,
    buffer_size: Cell<u32>,
    bus_clock_speed: u32,

    busy: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    tx_bytes: Cell<usize>,
    rx_buf: TakeCell<'static, [u8]>,
    rx_bytes: Cell<usize>,
    transfer_len: Cell<usize>,
}

impl Spi<'a> {
    pub const fn new(
        base_addr: StaticRef<SpiRegisters>,
        spi_fifo_size: u32,
        bus_clock_speed: u32,
    ) -> Spi<'a> {
        Spi {
            registers: base_addr,
            master_client: OptionalCell::empty(),
            initialized: Cell::new(false),
            buffer_size: Cell::new(spi_fifo_size),
            bus_clock_speed: bus_clock_speed,
            busy: Cell::new(false),
            tx_buf: TakeCell::empty(),
            tx_bytes: Cell::new(0),
            rx_buf: TakeCell::empty(),
            rx_bytes: Cell::new(0),
            transfer_len: Cell::new(0),
        }
    }

    // TODO: Missing initialize gpio pins. Do we even need that?

    fn enable_tx_interrupt(&self) {
        self.registers.ie.modify(IE::TXWM::SET);
    }

    fn disable_tx_interrupt(&self) {
        self.registers.ie.modify(IE::TXWM::CLEAR);
    }

    fn enable_rx_interrupt(&self) {
        self.registers.ie.modify(IE::RXWM::SET);
    }

    fn disable_rx_interrupt(&self) {
        self.registers.ie.modify(IE::RXWM::CLEAR);
    }

    fn dequeue_rx(&self, register_readout: Option<LocalRegisterCopy<u32, RXDATA::Register>>) {
        self.rx_bytes.update(|mut rx_bytes| {
            self.rx_buf.map(|buffer| {
                // An rx buffer was registered and we are actually interested in the data
                let mut rxdata: Option<_> = None;
                while {
                    if rxdata.is_some() || register_readout.is_none() {
                        rxdata = Some(self.registers.rxdata.extract());
                    } else {
                        rxdata = register_readout;
                    }

                    !rxdata.unwrap().is_set(RXDATA::EMPTY)
                } {
                    buffer[rx_bytes] = rxdata.unwrap().read(RXDATA::DATA) as u8;
                    rx_bytes += 1;
                }
            });

            rx_bytes
        });

        // RX bytes may NEVER be larger than TX bytes
        // This would indicate we read invalid data
        assert!(self.rx_bytes.get() <= self.tx_bytes.get());
    }

    fn clear_rx(&self) {
        while !self.registers.rxdata.is_set(RXDATA::EMPTY) {}
    }

    fn enqueue_tx(&self) {
        self.tx_bytes.update(|mut tx_bytes| {
            self.tx_buf.map(|buffer| {
                // Enqueue at most buffer_size bytes, to give
                // dequeue_rx a chance to dequeue _all_ received bytes
                // from the buffer
                let mut written_bytes = 0;

                while !self.registers.txdata.is_set(TXDATA::FULL)
                    && tx_bytes < self.transfer_len.get()
                    && written_bytes < self.buffer_size.get()
                {
                    // Using set is fine here, the FULL field is read
                    // only
                    self.registers.txdata.set(buffer[tx_bytes] as u32);
                    tx_bytes += 1;
                    written_bytes += 1;
                }
            });
            tx_bytes
        });
    }

    fn finish_transfer(&self) -> bool {
        // Check whether the transmit transaction part is finished
        if self.tx_bytes.get() >= self.transfer_len.get() {
            assert!(self.tx_bytes.get() == self.transfer_len.get());

            if self.rx_buf.is_some() {
                while self.rx_bytes.get() < self.transfer_len.get() {
                    // Busy wait for the remaining RX bytes
                    let mut rxdata;
                    while {
                        rxdata = self.registers.rxdata.extract();
                        rxdata.is_set(RXDATA::EMPTY)
                    } {}
                    self.dequeue_rx(Some(rxdata));
                }
            } else {
                self.clear_rx();
            }

            // Yes, we are finished
            true
        } else {
            // No, there are still some bytes left to transfer
            false
        }
    }

    pub fn handle_interrupt(&self) {
        // This is either triggered by a full receive FIFO or an empty
        // transmit FIFO.  By dequeing all available RX data in any
        // case, we are safe to transmit until the tx FIFO is full,
        // but at most buffer_size bytes, to avoid race conditions
        // with not getting the RX full interrupt fast enough.

        if self.rx_buf.is_some() {
            // First, dequeue rx bytes from the FIFO so that it is
            // cleared and we can receive a full buffer_size bytes
            // again
            self.dequeue_rx(None);
        }
        // Then enqueue as much as we can fit, at most buffer_size
        // bytes
        self.enqueue_tx();

        // Returns true, if the transaction is finished
        //
        // The transaction is finished if tx_bytes =
        // transfer_len. Even then there might still be data in the RX
        // FIFO, so dequeue until rx_bytes = transfer_len (possibly
        // busy waiting, as this function is only called upon an
        // interrupt).
        //
        // Returns true if the transfer is done.
        if self.finish_transfer() {
            // Everything has been transmitted and received,
            // disable interrupts and signal the client
            self.disable_tx_interrupt();
            self.disable_rx_interrupt();

            self.master_client.map(|client| {
                self.tx_buf.take().map(|tx_buf| {
                    self.busy.set(false);
                    client.read_write_done(tx_buf, self.rx_buf.take(), self.transfer_len.get());
                });
            });
        }
    }
}

impl spi::SpiMaster for Spi<'a> {
    type ChipSelect = SpiChipSelect;

    fn set_client(&self, client: &'static dyn SpiMasterClient) {
        self.master_client.set(client);
    }

    fn init(&self) {
        // disable all interrupts
        self.disable_tx_interrupt();
        self.disable_rx_interrupt();

        // set the default watermark threshold values
        self.registers.txmark.modify(TXMARK::TXMARK.val(0)); // TODO: For flash enabled SPI controllers this is 1
        self.registers.rxmark.modify(RXMARK::RXMARK.val(0));

        self.initialized.set(true);
    }

    fn is_busy(&self) -> bool {
        self.busy.get()
    }

    fn write_byte(&self, _out_byte: u8) {
        unimplemented!()
    }

    fn read_byte(&self) -> u8 {
        unimplemented!()
    }

    fn read_write_byte(&self, _val: u8) -> u8 {
        unimplemented!()
    }

    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        assert!(self.initialized.get());
        assert!(!self.busy.get());
        assert!(self.tx_buf.is_none());
        assert!(self.rx_buf.is_none());

        self.busy.set(true);

        let buffer_length = if let Some(ref read_buffer) = read_buffer {
            cmp::min(write_buffer.len(), read_buffer.len())
        } else {
            write_buffer.len()
        };

        self.tx_buf.replace(write_buffer);
        self.tx_bytes.set(0);
        self.rx_buf.put(read_buffer);
        self.rx_bytes.set(0);

        self.transfer_len.set(cmp::min(len, buffer_length));

        self.enable_tx_interrupt();
        if self.rx_buf.is_some() {
            self.enable_rx_interrupt();
        }

        // Start the transfer
        self.enqueue_tx();

        ReturnCode::SUCCESS
    }

    fn set_rate(&self, rate: u32) -> u32 {
        //           f_clk
        // f_sck = ----------
        //         2(div + 1)
        let divisor = (self.bus_clock_speed / (2 * rate)) - 1;

        self.registers.sckdiv.modify(SCKDIV::DIV.val(divisor));
        self.get_rate()
    }

    fn get_rate(&self) -> u32 {
        //           f_clk
        // f_sck = ----------
        //         2(div + 1)
        let divisor = self.registers.sckdiv.read(SCKDIV::DIV);

        self.bus_clock_speed / (2 * (divisor + 1))
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        self.registers
            .sckmode
            .modify(SCKMODE::POL.val(SCKMODE::POL::Value::from(polarity) as u32));
    }

    fn get_clock(&self) -> ClockPolarity {
        self.registers
            .sckmode
            .read_as_enum::<SCKMODE::POL::Value>(SCKMODE::POL)
            .expect("SiFive SPI: invalid sckmode polarity value")
            .into()
    }

    fn set_phase(&self, phase: ClockPhase) {
        self.registers
            .sckmode
            .modify(SCKMODE::PHA.val(SCKMODE::PHA::Value::from(phase) as u32));
    }

    fn get_phase(&self) -> ClockPhase {
        self.registers
            .sckmode
            .read_as_enum::<SCKMODE::PHA::Value>(SCKMODE::PHA)
            .expect("SiFive SPI: invalid sckmode phase value")
            .into()
    }

    fn hold_low(&self) {
        // Hold the CS pin after a transfer, so a single transaction
        // can be initiated using multiple transfer (read/write)
        // function calls
        // This should be equivalent to CSMODE = Hold
        self.registers.csmode.modify(CSMODE::MODE::Hold);
    }

    fn release_low(&self) {
        // Release the CS pin after a transfer (call to read/write)
        // This should be equivalent to CSMODE = Auto
        self.registers.csmode.modify(CSMODE::MODE::Auto);
    }

    fn specify_chip_select(&self, _cs: Self::ChipSelect) {
        // TODO: For multiple CS lines, this must actually be able to
        // do something.
        // For a single line, doing nothing works
    }
}
