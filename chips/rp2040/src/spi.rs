use crate::clocks;
use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::gpio::Output;
use kernel::hil::spi::SpiMaster;
use kernel::hil::spi::SpiMasterClient;
use kernel::hil::spi::{ClockPhase, ClockPolarity};
use kernel::ErrorCode;

const SPI_READ_IN_PROGRESS: u8 = 0b001;
const SPI_WRITE_IN_PROGRESS: u8 = 0b010;
const SPI_IN_PROGRESS: u8 = 0b100;
const SPI_IDLE: u8 = 0b000;

register_structs! {
    /// controls SPI port
    SpiRegisters {
        /// Control register 0, SSPCR0 on page 3-4
        (0x000 => sspcr0: ReadWrite<u32, SSPCR0::Register>),
        /// Control register 1, SSPCR1 on page 3-5
        (0x004 => sspcr1: ReadWrite<u32, SSPCR1::Register>),
        /// Data register, SSPDR on page 3-6
        (0x008 => sspdr: ReadWrite<u32, SSPDR::Register>),
        /// Status register, SSPSR on page 3-7
        (0x00C => sspsr: ReadOnly<u32, SSPSR::Register>),
        /// Clock prescale register, SSPCPSR on page 3-8
        (0x010 => sspcpsr: ReadWrite<u32, SSPCPSR::Register>),
        /// Interrupt mask set or clear register, SSPIMSC on page 3-9
        (0x014 => sspimsc: ReadWrite<u32, SSPIMSC::Register>),
        /// Raw interrupt status register, SSPRIS on page 3-10
        (0x018 => sspris: ReadOnly<u32, SSPRIS::Register>),
        /// Masked interrupt status register, SSPMIS on page 3-11
        (0x01C => sspmis: ReadOnly<u32, SSPMIS::Register>),
        /// Interrupt clear register, SSPICR on page 3-11
        (0x020 => sspicr: ReadWrite<u32, SSPICR::Register>),
        /// DMA control register, SSPDMACR on page 3-12
        (0x024 => sspdmacr: ReadWrite<u32, SSPDMACR::Register>),
        (0x028 => _reserved0),
        /// Peripheral identification registers
        (0xFE0 => sspperiphid0: ReadOnly<u32, SSPPERIPHID0::Register>),
        /// Peripheral identification registers
        (0xFE4 => sspperiphid1: ReadOnly<u32, SSPPERIPHID1::Register>),
        /// Peripheral identification registers
        (0xFE8 => sspperiphid2: ReadOnly<u32, SSPPERIPHID2::Register>),
        /// Peripheral identification registers
        (0xFEC => sspperiphid3: ReadOnly<u32, SSPPERIPHID3::Register>),
        /// PrimeCell identification registers
        (0xFF0 => ssppcellid0: ReadOnly<u32, SSPPCELLID0::Register>),
        /// PrimeCell identification registers
        (0xFF4 => ssppcellid1: ReadOnly<u32, SSPPCELLID1::Register>),
        /// PrimeCell identification registers
        (0xFF8 => ssppcellid2: ReadOnly<u32, SSPPCELLID2::Register>),
        /// PrimeCell identification registers
        (0xFFC => ssppcellid3: ReadOnly<u32, SSPPCELLID3::Register>),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    /// Control register 0
    SSPCR0 [
        /// Serial clock rate.
        SCR OFFSET(8) NUMBITS(8) [],
        /// SSPCLKOUT phase
        SPH OFFSET(7) NUMBITS(1) [],
        /// SSPCLKOUT polarity
        SPO OFFSET(6) NUMBITS(1) [],
        /// Frame format
        FRF OFFSET(4) NUMBITS(2) [
            MOTOROLA_SPI = 0b00,
            TI_SINC_SERIAL = 0b01,
            NAT_MICROWIRE = 0b10,
            RESERVED = 0b11
        ],
        /// Data Size Select
        DSS OFFSET(0) NUMBITS(4) [
            RESERVED_0 = 0b0000,
            RESERVED_1 = 0b0001,
            RESERVED_2 = 0b0010,
            DATA_4_BIT = 0b0011,
            DATA_5_BIT = 0b0100,
            DATA_6_BIT = 0b0101,
            DATA_7_BIT = 0b0110,
            DATA_8_BIT = 0b0111,
            DATA_9_BIT = 0b1000,
            DATA_10_BIT = 0b1001,
            DATA_11_BIT = 0b1010,
            DATA_12_BIT = 0b1011,
            DATA_13_BIT = 0b1100,
            DATA_14_BIT = 0b1101,
            DATA_15_BIT = 0b1110,
            DATA_16_BIT = 0b1111
        ]
    ],
    /// Control register 1
    SSPCR1 [
        /// Slave-mode output disable
        SOD OFFSET(3) NUMBITS(1) [],
        /// Master or slave mode select
        MS OFFSET(2) NUMBITS(1) [],
        /// Synchronous serial port enable
        SSE OFFSET(1) NUMBITS(1) [],
        /// Loop back mode
        LBM OFFSET(0) NUMBITS(1) []
    ],
    /// Data register
    SSPDR [
        /// Transmit/Receive FIFO: Read Receive FIFO. Write Transmit FIFO.
        DATA OFFSET(0) NUMBITS(16) []
    ],
    /// Status register
    SSPSR [
        /// PrimeCell SSP busy flag
        BSY OFFSET(4) NUMBITS(1) [],
        /// Receive FIFO full, RO
        RFF OFFSET(3) NUMBITS(1) [],
        /// Receive FIFO not empty
        RNE OFFSET(2) NUMBITS(1) [],
        /// Transmit FIFO not full
        TNF OFFSET(1) NUMBITS(1) [],
        /// Transmit FIFO empty
        TFE OFFSET(0) NUMBITS(1) []
    ],
    /// Clock prescale register
    SSPCPSR [
        /// Clock prescale divisor
        CPSDVSR OFFSET(0) NUMBITS(8) []
    ],
    /// Interrupt mask set or clear register
    SSPIMSC [
        /// Transmit FIFO interrupt mask
        TXIM OFFSET(3) NUMBITS(1) [],
        /// Receive FIFO interrupt mask
        RXIM OFFSET(2) NUMBITS(1) [],
        /// Receive timeout interrupt mask
        RTIM OFFSET(1) NUMBITS(1) [],
        /// Receive overrun interrupt mask
        RORIM OFFSET(0) NUMBITS(1) []
    ],
    /// Raw interrupt status register
    SSPRIS [
        /// Gives the raw interrupt state, prior to masking, of the SSPTXINTR interrupt
        TXRIS OFFSET(3) NUMBITS(1) [],
        /// Gives the raw interrupt state, prior to masking, of the SSPRXINTR interrupt
        RXRIS OFFSET(2) NUMBITS(1) [],
        /// Gives the raw interrupt state, prior to masking, of the SSPRTINTR interrupt
        RTRIS OFFSET(1) NUMBITS(1) [],
        /// Gives the raw interrupt state, prior to masking, of the SSPRORINTR interrupt
        RORRIS OFFSET(0) NUMBITS(1) []
    ],
    /// Masked interrupt status register
    SSPMIS [
        /// Gives the transmit FIFO masked interrupt state, after masking, of the SSPTXINTR
        TXMIS OFFSET(3) NUMBITS(1) [],
        /// Gives the receive FIFO masked interrupt state, after masking, of the SSPRXINTR i
        RXMIS OFFSET(2) NUMBITS(1) [],
        /// Gives the receive timeout masked interrupt state, after masking, of the SSPRTINT
        RTMIS OFFSET(1) NUMBITS(1) [],
        /// Gives the receive over run masked interrupt status, after masking, of the SSPROR
        RORMIS OFFSET(0) NUMBITS(1) []
    ],
    /// Interrupt clear register
    SSPICR [
        /// Clears the SSPRTINTR interrupt
        RTIC OFFSET(1) NUMBITS(1) [],
        /// Clears the SSPRORINTR interrupt
        RORIC OFFSET(0) NUMBITS(1) []
    ],
    /// DMA control register
    SSPDMACR [
        /// Transmit DMA Enable
        TXDMAE OFFSET(1) NUMBITS(1) [],
        /// Receive DMA Enable
        RXDMAE OFFSET(0) NUMBITS(1) []
    ],
    /// Peripheral identification registers
    SSPPERIPHID0 [
        /// These bits read back as 0x22
        PARTNUMBER0 OFFSET(0) NUMBITS(8) []
    ],
    /// Peripheral identification registers
    SSPPERIPHID1 [
        /// These bits read back as 0x1
        DESIGNER0 OFFSET(4) NUMBITS(4) [],
        /// These bits read back as 0x0
        PARTNUMBER1 OFFSET(0) NUMBITS(4) []
    ],
    /// Peripheral identification registers
    SSPPERIPHID2 [
        /// These bits return the peripheral revision
        REVISION OFFSET(4) NUMBITS(4) [],
        /// These bits read back as 0x4
        DESIGNER1 OFFSET(0) NUMBITS(4) []
    ],
    /// Peripheral identification registers
    SSPPERIPHID3 [
        /// These bits read back as 0x00
        CONFIGURATION OFFSET(0) NUMBITS(8) []
    ],
    /// PrimeCell identification registers
    SSPPCELLID0 [
        /// These bits read back as 0x0D
        SSPPCELLID0 OFFSET(0) NUMBITS(8) []
    ],
    /// PrimeCell identification registers
    SSPPCELLID1 [
        /// These bits read back as 0xF0
        SSPPCELLID1 OFFSET(0) NUMBITS(8) []
    ],
    /// PrimeCell identification registers
    SSPPCELLID2 [
        /// These bits read back as 0x05
        SSPPCELLID2 OFFSET(0) NUMBITS(8) []
    ],
    /// PrimeCell identification registers
    SSPPCELLID3 [
        /// These bits read back as 0xB1
        SSPPCELLID3 OFFSET(0) NUMBITS(8) []
    ]
];

const SPI0_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x4003C000 as *const SpiRegisters) };

const SPI1_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x40040000 as *const SpiRegisters) };

pub struct Spi<'a> {
    registers: StaticRef<SpiRegisters>,
    clocks: OptionalCell<&'a clocks::Clocks>,
    master_client: OptionalCell<&'a dyn hil::spi::SpiMasterClient>,
    active_slave: OptionalCell<&'a crate::gpio::RPGpioPin<'a>>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_position: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_position: Cell<usize>,
    len: Cell<usize>,

    transfers: Cell<u8>,
    active_after: Cell<bool>,
}

impl<'a> Spi<'a> {
    pub const fn new_spi0() -> Self {
        Self {
            registers: SPI0_BASE,
            clocks: OptionalCell::empty(),
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

    pub const fn new_spi1() -> Self {
        Self {
            registers: SPI1_BASE,
            clocks: OptionalCell::empty(),
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

    pub fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.set(clocks);
    }

    fn enable(&self) {
        self.registers.sspcr1.modify(SSPCR1::SSE::SET);
    }

    fn disable(&self) {
        self.registers.sspcr1.modify(SSPCR1::SSE::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.sspsr.is_set(SSPSR::TFE) {
            // if transmit fifo empty is set
            if self.tx_buffer.is_some() {
                while self.registers.sspsr.is_set(SSPSR::TNF)
                    && self.tx_position.get() < self.len.get()
                {
                    self.tx_buffer.map(|buf| {
                        // debug!("position {} of {}", self.tx_position.get(), self.len.get());
                        self.registers
                            .sspdr
                            .write(SSPDR::DATA.val(buf[self.tx_position.get()].into()));
                        self.tx_position.set(self.tx_position.get() + 1);
                    });
                }
                if self.tx_position.get() >= self.len.get() {
                    self.transfers
                        .set(self.transfers.get() & !SPI_WRITE_IN_PROGRESS);
                }
            } else {
                self.registers.sspimsc.modify(SSPIMSC::TXIM::CLEAR);
            }
        }

        while self.registers.sspsr.is_set(SSPSR::RNE) {
            let byte = self.registers.sspdr.read(SSPDR::DATA) as u8;
            if self.rx_buffer.is_some() {
                if self.rx_position.get() < self.len.get() {
                    self.rx_buffer.map(|buf| {
                        buf[self.rx_position.get()] = byte;
                    });
                    self.rx_position.set(self.rx_position.get() + 1);
                } else {
                    self.transfers
                        .set(self.transfers.get() & !SPI_READ_IN_PROGRESS);
                }
            }
        }

        if self.transfers.get() == SPI_IN_PROGRESS {
            if !self.active_after.get() {
                self.active_slave.map(|p| {
                    p.set();
                });
            }
            self.master_client.map(|client| {
                self.registers.sspimsc.modify(SSPIMSC::TXIM::CLEAR);
                self.registers.sspimsc.modify(SSPIMSC::RXIM::CLEAR);
                self.disable();
                self.transfers.set(SPI_IDLE);
                self.tx_buffer
                    .take()
                    .map(|buf| client.read_write_done(buf, self.rx_buffer.take(), self.len.get()));
            });
        }
    }

    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), ErrorCode> {
        if write_buffer.is_none() && read_buffer.is_none() {
            return Err(ErrorCode::INVAL);
        }

        if self.transfers.get() == SPI_IDLE {
            self.enable();
            self.registers.sspimsc.modify(SSPIMSC::TXIM::CLEAR);
            self.registers.sspimsc.modify(SSPIMSC::RXIM::CLEAR);
            self.active_slave.map(|p| {
                p.clear();
            });

            self.transfers.set(SPI_IN_PROGRESS);

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

            read_buffer.map(|buf| {
                self.rx_buffer.replace(buf);
                self.len.set(count);
                self.rx_position.set(0);
                self.registers.sspimsc.modify(SSPIMSC::RXIM::SET);
            });

            write_buffer.map(|buf| {
                self.tx_buffer.replace(buf);
                self.len.set(count);
                self.tx_position.set(0);
                self.registers.sspimsc.modify(SSPIMSC::TXIM::SET);
            });

            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    // IdleLow  = SPO = 0
    // IdleHigh = SPO = 1
    fn set_polarity(&self, polarity: ClockPolarity) {
        self.enable();
        match polarity {
            ClockPolarity::IdleHigh => self.registers.sspcr0.modify(SSPCR0::SPO::SET),
            ClockPolarity::IdleLow => self.registers.sspcr0.modify(SSPCR0::SPO::CLEAR),
        }
        self.disable();
    }

    fn get_polarity(&self) -> ClockPolarity {
        if !self.registers.sspcr0.is_set(SSPCR0::SPO) {
            ClockPolarity::IdleLow
        } else {
            ClockPolarity::IdleHigh
        }
    }

    // SampleLeading  = SPH = 0
    // SampleTrailing = SPH = 1
    fn set_phase(&self, phase: ClockPhase) {
        self.enable();
        match phase {
            ClockPhase::SampleLeading => self.registers.sspcr0.modify(SSPCR0::SPH::CLEAR),
            ClockPhase::SampleTrailing => self.registers.sspcr0.modify(SSPCR0::SPH::SET),
        }
        self.disable();
    }

    fn get_phase(&self) -> ClockPhase {
        if !self.registers.sspcr0.is_set(SSPCR0::SPH) {
            ClockPhase::SampleLeading
        } else {
            ClockPhase::SampleTrailing
        }
    }

    fn set_active_slave(&self, slave_pin: &'a crate::gpio::RPGpioPin<'a>) {
        self.active_slave.set(slave_pin);
    }

    fn set_format(&self) {
        self.registers.sspcr0.modify(SSPCR0::DSS::DATA_8_BIT);
        self.registers.sspcr0.modify(SSPCR0::SPO::CLEAR);
        self.registers.sspcr0.modify(SSPCR0::SPH::CLEAR);
    }
}

impl<'a> SpiMaster for Spi<'a> {
    type ChipSelect = &'a crate::gpio::RPGpioPin<'a>;

    fn set_client(&self, client: &'static dyn SpiMasterClient) {
        self.master_client.set(client);
    }

    fn init(&self) {
        self.set_rate(16 * 1000 * 1000);
        // set format: 8 bit mode, SSPCLKOUT polarity and phase on 0
        self.set_format();

        // Always enable DREQ signals -- harmless if DMA is not listening
        self.registers.sspdmacr.modify(SSPDMACR::TXDMAE::SET);
        self.registers.sspdmacr.modify(SSPDMACR::RXDMAE::SET);

        // set device on master
        self.registers.sspcr1.modify(SSPCR1::MS::CLEAR);
    }

    fn is_busy(&self) -> bool {
        self.registers.sspsr.is_set(SSPSR::BSY)
    }

    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), ErrorCode> {
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }

        // debug!("{:?}", write_buffer);

        self.read_write_bytes(Some(write_buffer), read_buffer, len)
    }

    fn write_byte(&self, out_val: u8) {
        while !self.registers.sspsr.is_set(SSPSR::TFE) {}

        self.registers.sspdr.modify(SSPDR::DATA.val(out_val as u32));
    }

    fn read_byte(&self) -> u8 {
        self.read_write_byte(0)
    }

    fn read_write_byte(&self, val: u8) -> u8 {
        self.write_byte(val);

        while !self.registers.sspsr.is_set(SSPSR::RNE) {}

        self.registers.sspdr.read(SSPDR::DATA) as u8
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        self.set_active_slave(cs);
    }

    fn set_rate(&self, baudrate: u32) -> u32 {
        let freq_in = self.clocks.map_or(125_000_000, |clocks| {
            clocks.get_frequency(clocks::Clock::Peripheral)
        });
        let mut prescale = 0;
        let mut postdiv = 0;
        //a se sterge

        for p in (2..254).step_by(2) {
            if (freq_in as u64) < (((p + 2) * 256) as u64 * baudrate as u64) {
                prescale = p;
                break;
            }
        }

        for p in (2..256).rev() {
            if (freq_in / (prescale * (p - 1))) > baudrate {
                postdiv = p;
                break;
            }
        }

        self.registers
            .sspcpsr
            .modify(SSPCPSR::CPSDVSR.val(prescale));
        self.registers.sspcr0.modify(SSPCR0::SCR.val(postdiv - 1));

        freq_in / (prescale * postdiv)
    }

    fn get_rate(&self) -> u32 {
        let freq_in = self.clocks.map_or(125_000_000, |clocks| {
            clocks.get_frequency(clocks::Clock::Peripheral)
        });
        let prescale = self.registers.sspcpsr.read(SSPCPSR::CPSDVSR);
        let postdiv = self.registers.sspcr0.read(SSPCR0::SCR) + 1;
        freq_in / (prescale * postdiv)
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
}
