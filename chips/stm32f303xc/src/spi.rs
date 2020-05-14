use core::cell::Cell;
use core::cmp;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::gpio::Output;
use kernel::hil::spi::{self, ClockPhase, ClockPolarity, SpiMasterClient};
use kernel::{ClockInterface, ReturnCode};

use crate::gpio::PinId;
use crate::rcc;

const SPI_READ_IN_PROGRESS: u8 = 0b001;
const SPI_WRITE_IN_PROGRESS: u8 = 0b010;
const SPI_IN_PROGRESS: u8 = 0b100;
const SPI_IDLE: u8 = 0b000;

/// Serial peripheral interface
#[repr(C)]
struct SpiRegisters {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// status register
    sr: ReadWrite<u32, SR::Register>,
    // this should be _reserved: [u8; 3], but it does not work,
    // packing is correct, but writing to the data register does not work
    // leaving it commented out until an upgrade to packed data is written
    /// data register
    dr: ReadWrite<u8, DR::Register>,
    /// CRC polynomial register
    crcpr: ReadWrite<u32, CRCPR::Register>,
    /// RX CRC register
    rxcrcr: ReadOnly<u32, RXCRCR::Register>,
    /// TX CRC register
    txcrcr: ReadOnly<u32, TXCRCR::Register>,
    /// I2S configuration register
    i2scfgr: ReadWrite<u32, I2SCFGR::Register>,
    /// I2S prescaler register
    i2spr: ReadWrite<u32, I2SPR::Register>,
}

register_bitfields![u8,
    DR [
        /// 8-bit data register
        DR OFFSET(0) NUMBITS(8) []
    ]
];

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
        /// CRC length
        CRCL OFFSET(11) NUMBITS(1) [],
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
        /// Last DMA transfer for transmission
        LDMA_TX OFFSET(14) NUMBITS(1) [],
        /// Last DMA transfer for reception
        LDMA_RX OFFSET(13) NUMBITS(1) [],
        /// FIFO reception threshold
        FRXTH OFFSET(12) NUMBITS(1) [],
        /// Data size
        DS OFFSET(8) NUMBITS(4) [],
        /// Tx buffer empty interrupt enable
        TXEIE OFFSET(7) NUMBITS(1) [],
        /// RX buffer not empty interrupt enable
        RXNEIE OFFSET(6) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(5) NUMBITS(1) [],
        /// Frame format
        FRF OFFSET(4) NUMBITS(1) [],
        /// NSS pulse management
        NSS OFFSET(3) NUMBITS(1) [],
        /// SS output enable
        SSOE OFFSET(2) NUMBITS(1) [],
        /// Tx buffer DMA enable
        TXDMAEN OFFSET(1) NUMBITS(1) [],
        /// Rx buffer DMA enable
        RXDMAEN OFFSET(0) NUMBITS(1) []
    ],
    SR [
        /// FIFO transmission level
        FTLVL OFFSET(11) NUMBITS(2) [],
        /// FIFO reception level
        FRLVL OFFSET(9) NUMBITS(2) [],
        /// TI frame format error
        FRE OFFSET(8) NUMBITS(1) [],
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
    CRCPR [
        /// CRC polynomial register
        CRCPOLY OFFSET(0) NUMBITS(16) []
    ],
    RXCRCR [
        /// Rx CRC register
        RXCRC OFFSET(0) NUMBITS(16) []
    ],
    TXCRCR [
        /// Tx CRC register
        TXCRC OFFSET(0) NUMBITS(16) []
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

const SPI1_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x4001_3000 as *const SpiRegisters) };

// const SPI2_BASE: StaticRef<SpiRegisters> =
//     unsafe { StaticRef::new(0x4000_3800 as *const SpiRegisters) };

// const SPI3_BASE: StaticRef<SpiRegisters> =
//     unsafe { StaticRef::new(0x4000_3C00 as *const SpiRegisters) };

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,
    clock: SpiClock,

    // SPI slave support not yet implemented
    master_client: OptionalCell<&'a dyn hil::spi::SpiMasterClient>,

    active_slave: OptionalCell<PinId>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_position: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_position: Cell<usize>,
    len: Cell<usize>,

    transfers: Cell<u8>,

    active_after: Cell<bool>,
}

pub static mut SPI1: Spi = Spi::new(
    SPI1_BASE,
    SpiClock(rcc::PeripheralClock::APB2(rcc::PCLK2::SPI1)),
);

impl Spi<'a> {
    const fn new(base_addr: StaticRef<SpiRegisters>, clock: SpiClock) -> Spi<'a> {
        Spi {
            registers: base_addr,
            clock,

            master_client: OptionalCell::empty(),
            active_slave: OptionalCell::empty(),

            tx_buffer: TakeCell::empty(),
            tx_position: Cell::new(0),

            rx_buffer: TakeCell::empty(),
            rx_position: Cell::new(0),

            len: Cell::new(0),

            transfers: Cell::new(SPI_IDLE),

            active_after: Cell::new(false),
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

    pub fn handle_interrupt(&self) {
        if self.registers.sr.is_set(SR::TXE) {
            if self.tx_buffer.is_some() && self.tx_position.get() < self.len.get() {
                self.tx_buffer.map(|buf| {
                    self.registers
                        .dr
                        .write(DR::DR.val(buf[self.tx_position.get()]));
                    self.tx_position.set(self.tx_position.get() + 1);
                });
            } else {
                self.registers.cr2.modify(CR2::TXEIE::CLEAR);
                self.transfers
                    .set(self.transfers.get() & !SPI_WRITE_IN_PROGRESS);
            }
        }

        if self.registers.sr.is_set(SR::RXNE) {
            while self.registers.sr.read(SR::FRLVL) > 0 {
                let byte = self.registers.dr.read(DR::DR) as u8;
                if self.rx_buffer.is_some() && self.rx_position.get() < self.len.get() {
                    self.rx_buffer.map(|buf| {
                        buf[self.rx_position.get()] = byte;
                    });
                }
                self.rx_position.set(self.rx_position.get() + 1);
            }

            if self.rx_position.get() >= self.len.get() {
                self.transfers
                    .set(self.transfers.get() & !SPI_READ_IN_PROGRESS);
            }
        }

        if self.transfers.get() == SPI_IN_PROGRESS {
            // we release the line and put the SPI in IDLE as the client might
            // initiate another SPI transfer right away
            if !self.active_after.get() {
                self.active_slave.map(|p| {
                    p.get_pin().as_ref().map(|pin| {
                        pin.set();
                    });
                });
            }
            self.transfers.set(SPI_IDLE);
            self.master_client.map(|client| {
                self.tx_buffer
                    .take()
                    .map(|buf| client.read_write_done(buf, self.rx_buffer.take(), self.len.get()))
            });
            self.transfers.set(SPI_IDLE);
        }
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

    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        if write_buffer.is_none() && read_buffer.is_none() {
            return ReturnCode::EINVAL;
        }

        if self.transfers.get() == 0 {
            self.registers.cr2.modify(CR2::RXNEIE::CLEAR);
            self.active_slave.map(|p| {
                p.get_pin().as_ref().map(|pin| {
                    pin.clear();
                });
            });

            self.transfers.set(self.transfers.get() | SPI_IN_PROGRESS);

            let mut count: usize = len;
            write_buffer
                .as_ref()
                .map(|buf| count = cmp::min(count, buf.len()));
            read_buffer
                .as_ref()
                .map(|buf| count = cmp::min(count, buf.len()));

            if write_buffer.is_some() {
                self.transfers
                    .set(self.transfers.get() | SPI_WRITE_IN_PROGRESS);
            }

            if read_buffer.is_some() {
                self.transfers
                    .set(self.transfers.get() | SPI_READ_IN_PROGRESS);
            }

            self.rx_position.set(0);

            read_buffer.map(|buf| {
                self.rx_buffer.replace(buf);
                self.len.set(count);
            });

            self.registers.cr2.modify(CR2::RXNEIE::SET);

            write_buffer.map(|buf| {
                self.tx_buffer.replace(buf);
                self.len.set(count);
                self.tx_position.set(0);
                self.registers.cr2.modify(CR2::TXEIE::SET);
            });

            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
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

        self.registers.cr2.modify(
            // Set 8 bit mode
            CR2::DS.val (0b0111)+
            // Set FIFO level at 1/4
            CR2::FRXTH::SET,
        );

        self.registers.cr1.modify(
            // 2 line unidirectional mode
            CR1::BIDIMODE::CLEAR +
            // Select as master
            CR1::MSTR::SET +
            // Software slave management
            CR1::SSM::SET +
            CR1::SSI::SET +
            // Enable
            CR1::SPE::SET,
        );
    }

    fn is_busy(&self) -> bool {
        self.registers.sr.is_set(SR::BSY)
    }

    fn write_byte(&self, out_byte: u8) {
        // debug! ("spi write byte {}", out_byte);
        // loop till TXE (Transmit Buffer Empty) becomes 1
        while !self.registers.sr.is_set(SR::TXE) {}

        self.registers.dr.modify(DR::DR.val(out_byte));
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
        // debug! ("stm32f3 spi set rate");
        if rate != 1_000_000 {
            panic!("rate must be 1_000_000");
        }

        self.set_cr(|| {
            // HSI is 8Mhz and Fpclk is also 8Mhz. 0b010 is Fpclk / 8
            self.registers.cr1.modify(CR1::BR.val(0b010));
        });

        1_000_000
    }

    /// We *only* support 1Mhz. If we need to return any other value other than
    /// `1_000_000`, then this function panics
    fn get_rate(&self) -> u32 {
        if self.registers.cr1.read(CR1::BR) != 0b010 {
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
        self.active_after.set(true);
    }

    fn release_low(&self) {
        self.active_after.set(false);
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        self.set_active_slave(cs);
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
