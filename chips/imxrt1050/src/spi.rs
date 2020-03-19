use core::cell::Cell;
use core::cmp;

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::gpio::Output;
use kernel::hil::spi::{self, ClockPhase, ClockPolarity, SpiMaster, SpiMasterClient};
use kernel::{ClockInterface, ReturnCode};

use crate::dma1;
use crate::dma1::Dma1Peripheral;
use crate::gpio::PinId;
use crate::rcc;

/// Serial peripheral interface
#[repr(C)]
struct SpiRegisters {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// status register
    sr: ReadWrite<u32, SR::Register>,
    /// data register
    dr: ReadWrite<u32, DR::Register>,
    /// CRC polynomial register
    crcpr: ReadWrite<u32>,
    /// RX CRC register
    rxcrcr: ReadOnly<u32>,
    /// TX CRC register
    txcrcr: ReadOnly<u32>,
    /// I2S configuration register
    i2scfgr: ReadWrite<u32, I2SCFGR::Register>,
    /// I2S prescaler register
    i2spr: ReadWrite<u32, I2SPR::Register>,
}

register_bitfields![u32,
    CR1 [
        /// Bidirectional data mode enable
        BIDIMODE OFFSET(15) NUMBITS(1) [],
        /// Output enable in bidirectional mode
        BIDIOE OFFSET(14) NUMBITS(1) [],
        /// Hardware CRC calculation enable
        CRCEN OFFSET(13) NUMBITS(1) [],
        /// CRC transfer next
        CRCNEXT OFFSET(12) NUMBITS(1) [],
        /// Data frame format
        DFF OFFSET(11) NUMBITS(1) [],
        /// Receive only
        RXONLY OFFSET(10) NUMBITS(1) [],
        /// Software slave management
        SSM OFFSET(9) NUMBITS(1) [],
        /// Internal slave select
        SSI OFFSET(8) NUMBITS(1) [],
        /// Frame format
        LSBFIRST OFFSET(7) NUMBITS(1) [],
        /// SPI enable
        SPE OFFSET(6) NUMBITS(1) [],
        /// Baud rate control
        BR OFFSET(3) NUMBITS(3) [],
        /// Master selection
        MSTR OFFSET(2) NUMBITS(1) [],
        /// Clock polarity
        CPOL OFFSET(1) NUMBITS(1) [],
        /// Clock phase
        CPHA OFFSET(0) NUMBITS(1) []
    ],
    CR2 [
        /// Tx buffer empty interrupt enable
        TXEIE OFFSET(7) NUMBITS(1) [],
        /// RX buffer not empty interrupt enable
        RXNEIE OFFSET(6) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(5) NUMBITS(1) [],
        /// Frame format
        FRF OFFSET(4) NUMBITS(1) [],
        /// SS output enable
        SSOE OFFSET(2) NUMBITS(1) [],
        /// Tx buffer DMA enable
        TXDMAEN OFFSET(1) NUMBITS(1) [],
        /// Rx buffer DMA enable
        RXDMAEN OFFSET(0) NUMBITS(1) []
    ],
    SR [
        /// TI frame format error
        TIFRFE OFFSET(8) NUMBITS(1) [],
        /// Busy flag
        BSY OFFSET(7) NUMBITS(1) [],
        /// Overrun flag
        OVR OFFSET(6) NUMBITS(1) [],
        /// Mode fault
        MODF OFFSET(5) NUMBITS(1) [],
        /// CRC error flag
        CRCERR OFFSET(4) NUMBITS(1) [],
        /// Underrun flag
        UDR OFFSET(3) NUMBITS(1) [],
        /// Channel side
        CHSIDE OFFSET(2) NUMBITS(1) [],
        /// Transmit buffer empty
        TXE OFFSET(1) NUMBITS(1) [],
        /// Receive buffer not empty
        RXNE OFFSET(0) NUMBITS(1) []
    ],
    DR [
        /// 8-bit data register
        DR OFFSET(0) NUMBITS(8) []
    ],
    I2SCFGR [
        /// I2S mode selection
        I2SMOD OFFSET(11) NUMBITS(1) [],
        /// I2S Enable
        I2SE OFFSET(10) NUMBITS(1) [],
        /// I2S configuration mode
        I2SCFG OFFSET(8) NUMBITS(2) [],
        /// PCM frame synchronization
        PCMSYNC OFFSET(7) NUMBITS(1) [],
        /// I2S standard selection
        I2SSTD OFFSET(4) NUMBITS(2) [],
        /// Steady state clock polarity
        CKPOL OFFSET(3) NUMBITS(1) [],
        /// Data length to be transferred
        DATLEN OFFSET(1) NUMBITS(2) [],
        /// Channel length (number of bits per audio channel)
        CHLEN OFFSET(0) NUMBITS(1) []
    ],
    I2SPR [
        /// Master clock output enable
        MCKOE OFFSET(9) NUMBITS(1) [],
        /// Odd factor for the prescaler
        ODD OFFSET(8) NUMBITS(1) [],
        /// I2S Linear prescaler
        I2SDIV OFFSET(0) NUMBITS(8) []
    ]
];

const SPI3_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x40003C00 as *const SpiRegisters) };

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,
    clock: SpiClock,

    // SPI slave support not yet implemented
    master_client: OptionalCell<&'a dyn hil::spi::SpiMasterClient>,

    tx_dma: OptionalCell<&'a dma1::Stream<'a>>,
    tx_dma_pid: Dma1Peripheral,
    rx_dma: OptionalCell<&'a dma1::Stream<'a>>,
    rx_dma_pid: Dma1Peripheral,

    dma_len: Cell<usize>,
    transfers_in_progress: Cell<u8>,

    active_slave: OptionalCell<PinId>,
}

// for use by `set_dma`
pub struct TxDMA<'a>(pub &'a dma1::Stream<'a>);
pub struct RxDMA<'a>(pub &'a dma1::Stream<'a>);

pub static mut SPI3: Spi = Spi::new(
    SPI3_BASE,
    SpiClock(rcc::PeripheralClock::APB1(rcc::PCLK1::SPI3)),
    Dma1Peripheral::SPI3_TX,
    Dma1Peripheral::SPI3_RX,
);

impl Spi<'a> {
    const fn new(
        base_addr: StaticRef<SpiRegisters>,
        clock: SpiClock,
        tx_dma_pid: Dma1Peripheral,
        rx_dma_pid: Dma1Peripheral,
    ) -> Spi<'a> {
        Spi {
            registers: base_addr,
            clock,

            master_client: OptionalCell::empty(),

            tx_dma: OptionalCell::empty(),
            tx_dma_pid: tx_dma_pid,
            rx_dma: OptionalCell::empty(),
            rx_dma_pid: rx_dma_pid,

            dma_len: Cell::new(0),
            transfers_in_progress: Cell::new(0),

            active_slave: OptionalCell::empty(),
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn set_dma(&self, tx_dma: TxDMA<'a>, rx_dma: RxDMA<'a>) {
        self.tx_dma.set(tx_dma.0);
        self.rx_dma.set(rx_dma.0);
    }

    pub fn handle_interrupt(&self) {
        // Used only during debugging. Since we use DMA, we do not enable SPI
        // interrupts during normal operations
    }

    // for use by dma1
    pub fn get_address_dr(&self) -> u32 {
        &self.registers.dr as *const ReadWrite<u32, DR::Register> as u32
    }

    fn set_active_slave(&self, slave_pin: PinId) {
        self.active_slave.set(slave_pin);
    }

    fn set_cr<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        self.registers.cr1.modify(CR1::SPE::CLEAR);
        f();
        self.registers.cr1.modify(CR1::SPE::SET);
    }

    // IdleLow  = CPOL = 0
    // IdleHigh = CPOL = 1
    fn set_polarity(&self, polarity: ClockPolarity) {
        self.set_cr(|| match polarity {
            ClockPolarity::IdleLow => self.registers.cr1.modify(CR1::CPOL::CLEAR),
            ClockPolarity::IdleHigh => self.registers.cr1.modify(CR1::CPOL::SET),
        });
    }

    fn get_polarity(&self) -> ClockPolarity {
        if !self.registers.cr1.is_set(CR1::CPOL) {
            ClockPolarity::IdleLow
        } else {
            ClockPolarity::IdleHigh
        }
    }

    // SampleLeading  = CPHA = 0
    // SampleTrailing = CPHA = 1
    fn set_phase(&self, phase: ClockPhase) {
        self.set_cr(|| match phase {
            ClockPhase::SampleLeading => self.registers.cr1.modify(CR1::CPHA::CLEAR),
            ClockPhase::SampleTrailing => self.registers.cr1.modify(CR1::CPHA::SET),
        });
    }

    fn get_phase(&self) -> ClockPhase {
        if !self.registers.cr1.is_set(CR1::CPHA) {
            ClockPhase::SampleLeading
        } else {
            ClockPhase::SampleTrailing
        }
    }

    fn enable_tx(&self) {
        self.registers.cr2.modify(CR2::TXDMAEN::SET);
    }

    fn disable_tx(&self) {
        self.registers.cr2.modify(CR2::TXDMAEN::CLEAR);
    }

    fn enable_rx(&self) {
        self.registers.cr2.modify(CR2::RXDMAEN::SET);
    }

    fn disable_rx(&self) {
        self.registers.cr2.modify(CR2::RXDMAEN::CLEAR);
    }

    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        if write_buffer.is_none() && read_buffer.is_none() {
            return ReturnCode::EINVAL;
        }

        self.hold_low();

        let mut count: usize = len;
        write_buffer
            .as_ref()
            .map(|buf| count = cmp::min(count, buf.len()));
        read_buffer
            .as_ref()
            .map(|buf| count = cmp::min(count, buf.len()));

        self.dma_len.set(count);

        self.transfers_in_progress.set(0);

        read_buffer.map(|rx_buffer| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.rx_dma.map(move |dma| {
                dma.do_transfer(rx_buffer, count);
            });
            self.enable_rx();
        });

        write_buffer.map(|tx_buffer| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.tx_dma.map(move |dma| {
                dma.do_transfer(tx_buffer, count);
            });
            self.enable_tx();
        });

        ReturnCode::SUCCESS
    }
}

impl spi::SpiMaster for Spi<'a> {
    type ChipSelect = PinId;

    fn set_client(&self, client: &'static dyn SpiMasterClient) {
        self.master_client.set(client);
    }

    fn init(&self) {
        // enable error interrupt (used only for debugging)
        // self.registers.cr2.modify(CR2::ERRIE::SET);

        self.registers.cr1.modify(
            // 2 line unidirectional mode
            CR1::BIDIMODE::CLEAR +
            // Select as master
            CR1::MSTR::SET +
            // Software slave management
            CR1::SSM::SET +
            CR1::SSI::SET +
            // 8 bit data frame format
            CR1::DFF::CLEAR +
            // Enable
            CR1::SPE::SET,
        );
    }

    fn is_busy(&self) -> bool {
        self.registers.sr.is_set(SR::BSY)
    }

    fn write_byte(&self, out_byte: u8) {
        // loop till TXE (Transmit Buffer Empty) becomes 1
        while !self.registers.sr.is_set(SR::TXE) {}

        self.registers.dr.modify(DR::DR.val(out_byte as u32));
    }

    fn read_byte(&self) -> u8 {
        self.read_write_byte(0)
    }

    fn read_write_byte(&self, val: u8) -> u8 {
        self.write_byte(val);

        // loop till RXNE becomes 1
        while !self.registers.sr.is_set(SR::RXNE) {}

        self.registers.dr.read(DR::DR) as u8
    }

    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        // If busy, don't start
        if self.is_busy() {
            return ReturnCode::EBUSY;
        }

        self.read_write_bytes(Some(write_buffer), read_buffer, len)
    }

    /// We *only* support 1Mhz. If `rate` is set to any value other than
    /// `1_000_000`, then this function panics
    fn set_rate(&self, rate: u32) -> u32 {
        if rate != 1_000_000 {
            panic!("rate must be 1_000_000");
        }

        self.set_cr(|| {
            // HSI is 16Mhz and Fpclk is also 16Mhz. 0b011 is Fpclk / 16
            self.registers.cr1.modify(CR1::BR.val(0b011));
        });

        1_000_000
    }

    /// We *only* support 1Mhz. If we need to return any other value other than
    /// `1_000_000`, then this function panics
    fn get_rate(&self) -> u32 {
        if self.registers.cr1.read(CR1::BR) != 0b011 {
            panic!("rate not set to 1_000_000");
        }

        1_000_000
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        self.set_polarity(polarity);
    }

    fn get_clock(&self) -> ClockPolarity {
        self.get_polarity()
    }

    fn set_phase(&self, phase: ClockPhase) {
        self.set_phase(phase);
    }

    fn get_phase(&self) -> ClockPhase {
        self.get_phase()
    }

    fn hold_low(&self) {
        self.active_slave.map(|p| {
            p.get_pin().as_ref().map(|pin| {
                pin.clear();
            });
        });
    }

    fn release_low(&self) {
        self.active_slave.map(|p| {
            p.get_pin().as_ref().map(|pin| {
                pin.set();
            });
        });
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        self.set_active_slave(cs);
    }
}

impl dma1::StreamClient for Spi<'a> {
    fn transfer_done(&self, pid: dma1::Dma1Peripheral) {
        if pid == self.tx_dma_pid {
            self.disable_tx();
        }

        if pid == self.rx_dma_pid {
            self.disable_rx();
        }

        self.transfers_in_progress
            .set(self.transfers_in_progress.get() - 1);

        if self.transfers_in_progress.get() == 0 {
            self.release_low();

            let tx_buffer = self.tx_dma.and_then(|tx_dma| tx_dma.return_buffer());
            let rx_buffer = self.rx_dma.and_then(|rx_dma| rx_dma.return_buffer());

            let length = self.dma_len.get();
            self.dma_len.set(0);

            self.master_client.map(|client| {
                tx_buffer.map(|t| {
                    client.read_write_done(t, rx_buffer, length);
                });
            });
        }
    }
}

struct SpiClock(rcc::PeripheralClock);

impl ClockInterface for SpiClock {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
