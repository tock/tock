use core::cell::Cell;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;

use kernel::hil;
use kernel::hil::i2c::{self, Error, I2CHwMasterClient, I2CMaster};
use kernel::ClockInterface;

use crate::ccm;

pub enum Lpi2cSpeed {
    Speed100k,
    Speed400k,
    Speed1M,
}

/// Inter-integrated Circuit
#[repr(C)]
struct Lpi2cRegisters {
    // Version ID Register
    verid: ReadOnly<u32, VERID::Register>,
    // Parameter Register
    param: ReadOnly<u32, PARAM::Register>,
    _reserved1: [u8; 8],
    // Master Control Register
    mcr: ReadWrite<u32, MCR::Register>,
    // Master Status Register
    msr: ReadWrite<u32, MSR::Register>,
    // Master Interrupt Enable Register
    mier: ReadWrite<u32, MIER::Register>,
    // Master DMA Enable Register
    mder: ReadWrite<u32, MDER::Register>,
    // Master Configuration Register 0
    mcfgr0: ReadWrite<u32, MCFGR0::Register>,
    // Master Configuration Register 1
    mcfgr1: ReadWrite<u32, MCFGR1::Register>,
    // Master Configuration Register 2
    mcfgr2: ReadWrite<u32, MCFGR2::Register>,
    // Master Configuration Register 3
    mcfgr3: ReadWrite<u32, MCFGR3::Register>,
    _reserved2: [u8; 16],
    // Master Data Match Register
    mdmr: ReadWrite<u32, MDMR::Register>,
    _reserved3: [u8; 4],
    // Master Configuration Register 0
    mccr0: ReadWrite<u32, MCCR0::Register>,
    _reserved4: [u8; 4],
    // Master Configuration Register 1
    mccr1: ReadWrite<u32, MCCR1::Register>,
    _reserved5: [u8; 4],
    // Master FIFO Control Register
    mfcr: ReadWrite<u32, MFCR::Register>,
    // Master FIFO Status Register
    mfsr: ReadOnly<u32, MFSR::Register>,
    // Master Transmit Data Register
    mtdr: WriteOnly<u32, MTDR::Register>,
    _reserved6: [u8; 12],
    // Master Receive Data Register
    mrdr: ReadOnly<u32, MRDR::Register>,
    _reserved7: [u8; 156],
    // Slave Control Register
    scr: ReadWrite<u32, SCR::Register>,
    // Slave Status Register
    ssr: ReadWrite<u32, SSR::Register>,
    // Slave Interrupt Enable Register
    sier: ReadWrite<u32, SIER::Register>,
    // Slave DMA Enable Register
    sder: ReadWrite<u32, SDER::Register>,
    _reserved8: [u8; 4],
    // Slave Configuration Register 1
    scfgr1: ReadWrite<u32, SCFGR1::Register>,
    // Slave Configuration Register 2
    scfgr2: ReadWrite<u32, SCFGR2::Register>,
    _reserved9: [u8; 20],
    // Slave Address Match Register
    samr: ReadWrite<u32, SAMR::Register>,
    _reserved10: [u8; 12],
    // Slave Status Match Register
    sasr: ReadOnly<u32, SAMR::Register>,
    // Slave Transmit ACK Register
    star: ReadWrite<u32, STAR::Register>,
    _reserved11: [u8; 8],
    // Slave Transmit Data Register
    stdr: WriteOnly<u32, STDR::Register>,
    _reserved12: [u8; 12],
    // Slave Receive Data Register
    srdr: ReadOnly<u32, SRDR::Register>,
}

register_bitfields![u32,
    VERID [
        /// Major Version Number
        MAJOR OFFSET(24) NUMBITS(8) [],
        /// Minor Version Number
        MINOR OFFSET(16) NUMBITS(8) [],
        /// Feature Identification Number
        FEATURE OFFSET(0) NUMBITS(16) []
    ],

    PARAM [
        /// Receive FIFO Size
        MRXFIFO OFFSET(8) NUMBITS(4) [],
        /// Transmit FIFO Size
        MTXFIFO OFFSET(0) NUMBITS(4) []
    ],

    MCR [
        /// Reset Receive FIFO
        RRF OFFSET(9) NUMBITS(1) [],
        /// Reset Transmit FIFO
        RTF OFFSET(8) NUMBITS(1) [],
        /// Debug Enable
        DBGEN OFFSET(3) NUMBITS(1) [],
        /// Doze Mode Enable
        DOZEN OFFSET(2) NUMBITS(1) [],
        /// Software Reset
        RST OFFSET(1) NUMBITS(1) [],
        /// Master Enable
        MEN OFFSET(0) NUMBITS(1) []
    ],

    MSR [
        /// Bus Busy Flag
        BBF OFFSET(25) NUMBITS(1) [],
        /// Master Busy Flag
        MBF OFFSET(24) NUMBITS(1) [],
        /// Data Match Flag
        DMF OFFSET(14) NUMBITS(1) [],
        /// Pin Low Timeout Flag
        PLTF OFFSET(13) NUMBITS(1) [],
        /// FIFO Error Flag
        FEF OFFSET(12) NUMBITS(1) [],
        /// Arbitration Lost Flag
        ALF OFFSET(11) NUMBITS(1) [],
        /// NACK Detect Flag
        NDF OFFSET(10) NUMBITS(1) [],
        /// STOP Detect Flag
        SDF OFFSET(9) NUMBITS(1) [],
        /// End Packet Flag
        EPF OFFSET(8) NUMBITS(1) [],
        /// Receive Data Flag
        RDF OFFSET(1) NUMBITS(1) [],
        /// Transmit Data Flag
        TDF OFFSET(0) NUMBITS(1) []
    ],

    MIER [
        /// Data Match Interrupt Enable
        DMIE OFFSET(14) NUMBITS(1) [],
        /// Pin Low Timeout Interrupt Enable
        PLTIE OFFSET(13) NUMBITS(1) [],
        /// FIFO Error Interrupt Enable
        FEIE OFFSET(12) NUMBITS(1) [],
        /// Arbitration Lost Interrupt Enable
        ALIE OFFSET(11) NUMBITS(1) [],
        /// NACK Detect Interrupt Enable
        NDIE OFFSET(10) NUMBITS(1) [],
        /// STOP Detect Interrupt Enable
        SDIE OFFSET(9) NUMBITS(1) [],
        /// End Packet Interrupt Enable
        EPIE OFFSET(8) NUMBITS(1) [],
        /// Receive Data Interrupt Enable
        RDIE OFFSET(1) NUMBITS(1) [],
        /// Transmit Data Interrupt Enable
        TDIE OFFSET(0) NUMBITS(1) []
    ],

    MDER [
        /// Receive Data DMA Enable
        RDDE OFFSET(1) NUMBITS(1) [],
        /// Transmit Data DMA Enable
        TDDE OFFSET(0) NUMBITS(1) []
    ],

    MCFGR0 [
        /// Receive Data Match Only
        RDMO OFFSET(9) NUMBITS(1) [],
        /// Circular FIFO Enable
        CIRFIFO OFFSET(8) NUMBITS(1) [],
        /// Host Request Select
        HRSEL OFFSET(2) NUMBITS(1) [],
        /// Host Request Polarity
        HRPOL OFFSET(1) NUMBITS(1) [],
        /// Host Request Enable
        HREN OFFSET(0) NUMBITS(1) []
    ],

    MCFGR1 [
        /// Pin Configuration
        PINCFG OFFSET(24) NUMBITS(3) [],
        /// Match Configuration
        MATCFG OFFSET(16) NUMBITS(3) [],
        /// Timeout Configuration
        TIMECFG OFFSET(10) NUMBITS(1) [],
        /// IGNACK
        IGNACK OFFSET(9) NUMBITS(1) [],
        /// Automatic STOP Generation
        AUTOSTOP OFFSET(8) NUMBITS(1) [],
        /// Prescaler
        PRESCALE OFFSET(0) NUMBITS(3) []
    ],

    MCFGR2 [
        /// Glitch Filter SDA
        FILTSDA OFFSET(24) NUMBITS(4) [],
        /// Glitch Filter SCL
        FILTSCL OFFSET(16) NUMBITS(4) [],
        /// Bus Idle Timeout
        BUSIDLE OFFSET(0) NUMBITS(12) []
    ],

    MCFGR3 [
        /// Pin Low Timeout
        PINLOW OFFSET(8) NUMBITS(12) []
    ],

    MDMR [
        /// Match 1 Value
        MATCH1 OFFSET(16) NUMBITS(8) [],
        /// Match 0 Value
        MATCH0 OFFSET(0) NUMBITS(8) []
    ],

    MCCR0 [
        /// Data Valid Delay
        DATAVD OFFSET(24) NUMBITS(6) [],
        /// Setup Hold Delay
        SETHOLD OFFSET(16) NUMBITS(6) [],
        /// Clock High Period
        CLKHI OFFSET(8) NUMBITS(6) [],
        /// Clock Low Period
        CLKLO OFFSET(0) NUMBITS(6) []
    ],

    MCCR1 [
        /// Data Valid Delay
        DATAVD OFFSET(24) NUMBITS(6) [],
        /// Setup Hold Delay
        SETHOLD OFFSET(16) NUMBITS(6) [],
        /// Clock High Period
        CLKHI OFFSET(8) NUMBITS(6) [],
        /// Clock Low Period
        CLKLO OFFSET(0) NUMBITS(6) []
    ],

    MFCR [
        /// Receive FIFO Watermark
        RXWATER OFFSET(16) NUMBITS(2) [],
        /// Transmit FIFO Watermark
        TXWATER OFFSET(0) NUMBITS(2) []
    ],

    MFSR [
        /// Receive FIFO Count
        RXCOUNT OFFSET(16) NUMBITS(3) [],
        /// Transmit FIFO Count
        TXCOUNT OFFSET(0) NUMBITS(3) []
    ],

    MTDR [
        /// Command Data
        CMD OFFSET(8) NUMBITS(3) [],
        /// Transmit Data
        DATA OFFSET(0) NUMBITS(8) []
    ],

    MRDR [
        /// RX Empty
        RXEMPTY OFFSET(14) NUMBITS(1) [],
        /// Receive Data
        DATA OFFSET(0) NUMBITS(8) []
    ],

    SCR [
        /// Reset Receive FIFO
        RRF OFFSET(9) NUMBITS(1) [],
        /// Reset Transmit FIFO
        RTF OFFSET(8) NUMBITS(1) [],
        /// Filter Doze Enable
        FILTDZ OFFSET(5) NUMBITS(1) [],
        /// Filter Enable
        FILTEN OFFSET(4) NUMBITS(1) [],
        /// Software Reset
        RST OFFSET(1) NUMBITS(1) [],
        /// Slave Enable
        SEN OFFSET(0) NUMBITS(1) []
    ],

    SSR [
        /// Bus Busy Flag
        BBF OFFSET(25) NUMBITS(1) [],
        /// Slave Busy Flag
        SBF OFFSET(24) NUMBITS(1) [],
        /// SMBus Alert Response Flag
        SARF OFFSET(15) NUMBITS(1) [],
        /// General Call Flag
        GCF OFFSET(14) NUMBITS(1) [],
        /// Address Match 1 Flag
        AM1F OFFSET(13) NUMBITS(1) [],
        /// Address Match 0 Flag
        AM0F OFFSET(12) NUMBITS(1) [],
        /// FIFO Error Flag
        FEF OFFSET(11) NUMBITS(1) [],
        /// Bit Error Flag
        BEF OFFSET(10) NUMBITS(1) [],
        /// STOP Detect Flag
        SDF OFFSET(9) NUMBITS(1) [],
        /// Repeated Start Flag
        RSF OFFSET(8) NUMBITS(1) [],
        /// Transmit ACK Flag
        TAF OFFSET(3) NUMBITS(1) [],
        /// Address Valid Flag
        AVF OFFSET(2) NUMBITS(1) [],
        /// Receive Data Flag
        RDF OFFSET(1) NUMBITS(1) [],
        /// Transmit Data Flag
        TDF OFFSET(0) NUMBITS(1) []
    ],

    SIER [
        /// SMBus Alert Response Interrupt Enable
        SARIE OFFSET(15) NUMBITS(1) [],
        /// General Call Interrupt Enable
        GCIE OFFSET(14) NUMBITS(1) [],
        /// Address Match 1 Interrupt Enable
        AM1F OFFSET(13) NUMBITS(1) [],
        /// Address Match 0 Interrupt Enable
        AM0IE OFFSET(12) NUMBITS(1) [],
        /// FIFO Error Interrupt Enable
        FEIE OFFSET(11) NUMBITS(1) [],
        /// Bit Error Interrupt Enable
        BEIE OFFSET(10) NUMBITS(1) [],
        /// STOP Detect Interrupt Enable
        SDIE OFFSET(9) NUMBITS(1) [],
        /// Repeated Start Interrupt Enable
        RSIE OFFSET(8) NUMBITS(1) [],
        /// Transmit ACK Interrupt Enable
        TAIE OFFSET(3) NUMBITS(1) [],
        /// Address Valid Interrupt Enable
        AVIE OFFSET(2) NUMBITS(1) [],
        /// Receive Data Interrupt Enable
        RDIE OFFSET(1) NUMBITS(1) [],
        /// Transmit Data Interrupt Enable
        TDIE OFFSET(0) NUMBITS(1) []
    ],

    SDER [
        /// Address Valid DMA Enable
        AVDE OFFSET(2) NUMBITS(1) [],
        /// Receive Data DMA Enable
        RDDE OFFSET(1) NUMBITS(1) [],
        /// Transmit Data DMA Enable
        TDDE OFFSET(0) NUMBITS(1) []
    ],

    SCFGR1 [
        /// Address Configuration
        ADDRCFG OFFSET(16) NUMBITS(3) [],
        /// High Speed Mode Enable
        HSMEN OFFSET(13) NUMBITS(1) [],
        /// Ignore NACK
        IGNACK OFFSET(12) NUMBITS(1) [],
        /// Receive Data Configuration
        RXCFG OFFSET(11) NUMBITS(1) [],
        /// Transmit Flag Configuration
        TXCFG OFFSET(10) NUMBITS(1) [],
        /// SMBus Alert Enable
        SAEN OFFSET(9) NUMBITS(1) [],
        /// General Call Enable
        GCEN OFFSET(8) NUMBITS(1) [],
        /// ACK SCL Stall
        ACKSTALL OFFSET(3) NUMBITS(1) [],
        /// TX Data SCL Stall
        TXDSTALL OFFSET(2) NUMBITS(1) [],
        /// RX SCL Stall
        RXSTALL OFFSET(1) NUMBITS(1) [],
        /// Address SCL Stall
        ADRSTALL OFFSET(0) NUMBITS(1) []
    ],

    SCFGR2 [
        /// Glitch Filter SDA
        FILTSDA OFFSET(24) NUMBITS(4) [],
        /// Glitch Filter SCL
        FILTSCL OFFSET(16) NUMBITS(4) [],
        /// Data Valid Delay
        DATAVD OFFSET(8) NUMBITS(6) [],
        /// Clock Hold Time
        CLKHOLD OFFSET(0) NUMBITS(4) []
    ],

    SAMR [
        /// Address 1 Value
        ADDR1 OFFSET(17) NUMBITS(10) [],
        /// Address 0 Value
        ADDR0 OFFSET(1) NUMBITS(10) []
    ],

    SASR [
        /// Address Not Valid
        ANV OFFSET(14) NUMBITS(1) [],
        /// Received Address
        RADDR OFFSET(0) NUMBITS(11) []
    ],

    STAR [
        /// Transmit NACK
        TXNACK OFFSET(0) NUMBITS(1) []
    ],

    STDR [
        /// Transmit Data
        TXNACK OFFSET(0) NUMBITS(8) []
    ],

    SRDR [
        /// Start Of Frame
        SOF OFFSET(15) NUMBITS(1) [],
        /// RX Empty
        RXEMPTY OFFSET(14) NUMBITS(1) [],
        /// Receive Data
        DATA OFFSET(0) NUMBITS(8) []
    ]
];

const LPI2C1_BASE: StaticRef<Lpi2cRegisters> =
    unsafe { StaticRef::new(0x403F_0000 as *const Lpi2cRegisters) };

pub struct Lpi2c<'a> {
    registers: StaticRef<Lpi2cRegisters>,
    clock: Lpi2cClock,

    // I2C slave support not yet implemented
    master_client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,

    buffer: TakeCell<'static, [u8]>,
    tx_position: Cell<u8>,
    rx_position: Cell<u8>,
    tx_len: Cell<u8>,
    rx_len: Cell<u8>,

    slave_address: Cell<u8>,

    status: Cell<Lpi2cStatus>,
    // transfers: Cell<u8>
    int: Cell<usize>,
}

// Since we do not have a register for setting the number of bytes to be sent
// or a register in which we set slave address. We will need 3 additional states
// WritingAddress, WritingReadingAddress, ReadingAddress in order to check the
// status of the i2c transmission.
#[derive(Debug, Copy, Clone, PartialEq)]
enum Lpi2cStatus {
    Idle,
    Writing,
    WritingReading,
    Reading,
}

pub static mut LPI2C1: Lpi2c = Lpi2c::new(
    LPI2C1_BASE,
    Lpi2cClock(ccm::PeripheralClock::CCGR2(ccm::HCLK2::LPI2C1)),
);

impl Lpi2c<'_> {
    const fn new(base_addr: StaticRef<Lpi2cRegisters>, clock: Lpi2cClock) -> Self {
        Self {
            registers: base_addr,
            clock,

            master_client: OptionalCell::empty(),

            slave_address: Cell::new(0),

            buffer: TakeCell::empty(),
            tx_position: Cell::new(0),
            rx_position: Cell::new(0),

            tx_len: Cell::new(0),
            rx_len: Cell::new(0),
            status: Cell::new(Lpi2cStatus::Idle),

            int: Cell::new(0),
        }
    }

    pub fn set_speed(&self, speed: Lpi2cSpeed, _system_clock_in_mhz: usize) {
        self.disable();
        match speed {
            Lpi2cSpeed::Speed100k => {
                let prescaler = 4;
                self.registers.mccr0.modify(MCCR0::CLKHI.val(12));
                self.registers.mccr0.modify(MCCR0::CLKLO.val(24));
                self.registers.mccr0.modify(MCCR0::SETHOLD.val(12));
                self.registers.mccr0.modify(MCCR0::DATAVD.val(6));
                self.registers
                    .mcfgr1
                    .modify(MCFGR1::PRESCALE.val(prescaler as u32));
            }
            Lpi2cSpeed::Speed400k => {
                let prescaler = 1;
                self.registers.mccr0.modify(MCCR0::CLKHI.val(12));
                self.registers.mccr0.modify(MCCR0::CLKLO.val(24));
                self.registers.mccr0.modify(MCCR0::SETHOLD.val(12));
                self.registers.mccr0.modify(MCCR0::DATAVD.val(6));
                self.registers
                    .mcfgr1
                    .modify(MCFGR1::PRESCALE.val(prescaler as u32));
            }
            Lpi2cSpeed::Speed1M => {
                panic!("i2c speed 1MHz not implemented");
            }
        }
        self.enable();
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

    pub fn send_byte(&self) -> bool {
        if self.buffer.is_some() && self.tx_position.get() < self.tx_len.get() {
            self.buffer.map(|buf| {
                let byte = buf[self.tx_position.get() as usize];
                self.registers.mtdr.write(MTDR::DATA.val(byte as u32));
                self.tx_position.set(self.tx_position.get() + 1);
            });
            true
        } else {
            false
            // panic!("i2c error, attempting to transmit more bytes than available in the buffer");
        }
    }

    pub fn read_byte(&self) -> bool {
        let byte = self.registers.mrdr.read(MRDR::DATA) as u8;
        if self.buffer.is_some() && self.rx_position.get() < self.rx_len.get() {
            self.buffer.map(|buf| {
                buf[self.rx_position.get() as usize] = byte;
                self.rx_position.set(self.rx_position.get() + 1);
            });
            true
        } else {
            false
        }
    }

    pub fn handle_event(&self) {
        self.int.set(self.int.get() + 1);
        if self.int.get() > 20 {
            panic!("exceded 100");
        }

        if self.registers.msr.is_set(MSR::FEF) {
            debug!("FIFO Error");
            self.registers.msr.modify(MSR::FEF::SET);
        }

        // Transmit data is requested
        if self.registers.msr.is_set(MSR::TDF) {
            // send the next byte
            if self.tx_position.get() < self.tx_len.get() {
                self.send_byte();
            } else {
                self.registers.mtdr.write(MTDR::CMD.val(0b010));
            }
        }

        // Receive data is ready
        while self.registers.msr.is_set(MSR::RDF) {
            // read the next byte
            if self.rx_position.get() < self.rx_len.get() {
                self.read_byte();
            } else {
                self.registers.mtdr.write(MTDR::CMD.val(0b010));
            }
        }

        // End packet flag set
        if self.registers.msr.is_set(MSR::EPF) {
            match self.status.get() {
                // if it is in the Address state, we set the status
                // accordingly and send the next byte
                Lpi2cStatus::Writing | Lpi2cStatus::WritingReading => {
                    // if there are more bytes to be sent, send next byte
                    if self.tx_position.get() < self.tx_len.get() {
                        self.stop();
                        self.master_client.map(|client| {
                            self.buffer
                                .take()
                                .map(|buf| client.command_complete(buf, Error::DataNak))
                        });
                    } else {
                        if self.status.get() == Lpi2cStatus::Writing {
                            self.registers.mtdr.write(MTDR::CMD.val(0b010));
                            self.stop();
                            self.master_client.map(|client| {
                                self.buffer
                                    .take()
                                    .map(|buf| client.command_complete(buf, Error::CommandComplete))
                            });
                        } else {
                            self.status.set(Lpi2cStatus::Reading);
                            self.start_read();
                        }
                    }
                }
                Lpi2cStatus::Reading => {
                    // if there are more bytes to be read, read next byte
                    if self.rx_position.get() == self.rx_len.get() {
                        self.stop();
                        self.master_client.map(|client| {
                            self.buffer
                                .take()
                                .map(|buf| client.command_complete(buf, Error::CommandComplete))
                        });
                    } else {
                        self.stop();
                        self.master_client.map(|client| {
                            self.buffer
                                .take()
                                .map(|buf| client.command_complete(buf, Error::DataNak))
                        });
                    }
                }
                _ => panic!("i2c should not arrive here"),
            }
        }

        // abort transfer due to NACK
        if self.registers.msr.is_set(MSR::NDF) {
            self.registers.msr.modify(MSR::NDF::SET);
            self.registers.mtdr.write(MTDR::CMD.val(0b010));
            self.stop();
            let err = Error::DataNak;
            self.master_client.map(|client| {
                self.buffer
                    .take()
                    .map(|buf| client.command_complete(buf, err))
            });
        }
    }

    pub fn handle_error(&self) {}

    fn reset(&self) {
        self.disable();
        self.enable();
    }

    fn start_write(&self) {
        self.tx_position.set(0);
        self.registers.mier.modify(MIER::EPIE::CLEAR);
        self.registers
            .mtdr
            .write(MTDR::CMD.val(0b100) + MTDR::DATA.val((self.slave_address.get() << 1) as u32));

        self.registers.mcfgr1.modify(MCFGR1::PINCFG::CLEAR);

        self.registers.mier.modify(
            MIER::TDIE::SET + MIER::NDIE::SET + MIER::RDIE::SET + MIER::EPIE::SET + MIER::FEIE::SET,
        );
    }

    fn stop(&self) {
        self.tx_position.set(0);
        self.tx_len.set(0);
        self.rx_position.set(0);
        self.rx_len.set(0);
        self.status.set(Lpi2cStatus::Idle);
        if self.registers.msr.is_set(MSR::FEF) {
            self.registers.msr.modify(MSR::FEF::SET);
        }
        self.registers.mier.modify(
            MIER::TDIE::CLEAR
                + MIER::NDIE::CLEAR
                + MIER::EPIE::CLEAR
                + MIER::SDIE::CLEAR
                + MIER::RDIE::CLEAR,
        );
    }

    fn start_read(&self) {
        self.rx_position.set(0);

        // setting slave address
        self.registers.mier.modify(MIER::EPIE::CLEAR);
        self.registers
            .mtdr
            .write(MTDR::CMD.val(100) + MTDR::DATA.val((self.slave_address.get() << 1 + 1) as u32));

        self.registers.mcfgr1.modify(MCFGR1::PINCFG::CLEAR);
        self.registers
            .mier
            .modify(MIER::NDIE::SET + MIER::EPIE::SET + MIER::RDIE::SET);
    }
}

impl i2c::I2CMaster for Lpi2c<'_> {
    fn set_master_client(&self, master_client: &'static dyn I2CHwMasterClient) {
        self.master_client.replace(master_client);
    }
    fn enable(&self) {
        self.registers.mcr.modify(MCR::RRF::SET);
        self.registers.mcr.modify(MCR::RTF::SET);
        self.registers.mcr.modify(MCR::MEN::SET);
    }
    fn disable(&self) {
        self.registers.mcr.modify(MCR::MEN::CLEAR);
    }
    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        if self.status.get() == Lpi2cStatus::Idle {
            self.reset();
            self.status.set(Lpi2cStatus::WritingReading);
            self.slave_address.set(addr);
            self.buffer.replace(data);
            self.tx_len.set(write_len);
            self.rx_len.set(read_len);
            self.registers.mcfgr1.modify(MCFGR1::AUTOSTOP::CLEAR);
            self.start_write();
        }
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        if self.status.get() == Lpi2cStatus::Idle {
            self.reset();
            self.status.set(Lpi2cStatus::Writing);
            self.slave_address.set(addr);
            self.buffer.replace(data);
            self.tx_len.set(len);
            self.registers.mcfgr1.modify(MCFGR1::AUTOSTOP::CLEAR);
            self.start_write();
        }
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        if self.status.get() == Lpi2cStatus::Idle {
            self.reset();
            self.status.set(Lpi2cStatus::Reading);
            self.slave_address.set(addr);
            self.buffer.replace(buffer);
            self.rx_len.set(len);
            self.registers.mcfgr1.modify(MCFGR1::AUTOSTOP::CLEAR);
            self.start_read();
        }
    }
}

struct Lpi2cClock(ccm::PeripheralClock);

impl ClockInterface for Lpi2cClock {
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
