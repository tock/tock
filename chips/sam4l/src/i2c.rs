//! Implementation of the SAM4L TWIMS peripheral.
//!
//! The implementation, especially of repeated starts, is quite sensitive to the
//! ordering of operations (e.g. setup DMA, then set command register, then next
//! command register, then enable, then start the DMA transfer). The placement
//! of writes to interrupt enable/disable registers is also significant, but not
//! refactored in such a way that's very logical right now.
//!
//! The point is that until this changes, and this notice is taken away: IF YOU
//! CHANGE THIS DRIVER, TEST RIGOROUSLY!!!

use core::cell::Cell;
use dma::{DMAChannel, DMAClient, DMAPeripheral};
use kernel::common::cells::TakeCell;
use kernel::common::peripherals::{PeripheralManagement, PeripheralManager};
use kernel::common::regs::{FieldValue, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;
use pm;

// Listing of all registers related to the TWIM peripheral.
// Section 27.9 of the datasheet
#[repr(C)]
#[allow(dead_code)]
struct TWIMRegisters {
    cr: WriteOnly<u32, Control::Register>,
    cwgr: ReadWrite<u32, ClockWaveformGenerator::Register>,
    smbtr: ReadWrite<u32, SmbusTiming::Register>,
    cmdr: ReadWrite<u32, Command::Register>,
    ncmdr: ReadWrite<u32, Command::Register>,
    rhr: ReadOnly<u32, ReceiveHolding::Register>,
    thr: WriteOnly<u32, TransmitHolding::Register>,
    sr: ReadOnly<u32, Status::Register>,
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    scr: WriteOnly<u32, StatusClear::Register>,
    pr: ReadOnly<u32>,
    vr: ReadOnly<u32>,
    hscwgr: ReadWrite<u32>,
    srr: ReadWrite<u32, SlewRate::Register>,
    hssrr: ReadWrite<u32>,
}

// Listing of all registers related to the TWIS peripheral.
// Section 28.9 of the datasheet
#[repr(C)]
#[allow(dead_code)]
struct TWISRegisters {
    cr: ReadWrite<u32, ControlSlave::Register>,
    nbytes: ReadWrite<u32, Nbytes::Register>,
    tr: ReadWrite<u32, Timing::Register>,
    rhr: ReadOnly<u32, ReceiveHolding::Register>,
    thr: WriteOnly<u32, TransmitHolding::Register>,
    pecr: ReadOnly<u32, PacketErrorCheck::Register>,
    sr: ReadOnly<u32, StatusSlave::Register>,
    ier: WriteOnly<u32, InterruptSlave::Register>,
    idr: WriteOnly<u32, InterruptSlave::Register>,
    imr: ReadOnly<u32, InterruptSlave::Register>,
    scr: WriteOnly<u32, StatusClearSlave::Register>,
    pr: ReadOnly<u32>,
    vr: ReadOnly<u32>,
    hstr: ReadWrite<u32>,
    srr: ReadWrite<u32, SlewRateSlave::Register>,
    hssrr: ReadWrite<u32>,
}

register_bitfields![u32,
    Control [
        /// Stop the Current Transfer
        STOP 8,
        /// Software Reset
        SWRST 7,
        /// SMBus Disable
        SMDIS 5,
        /// SMBus Enable
        SMEN 4,
        /// Master Disable
        MDIS 1,
        /// Master Enable
        MEN 0
    ],

    ClockWaveformGenerator [
        /// Clock Prescaler
        EXP OFFSET(28) NUMBITS(3) [],
        /// Data Setup and Hold Cycles
        DATA OFFSET(24) NUMBITS(4) [],
        /// START and STOP Cycles
        STASTO OFFSET(16) NUMBITS(8) [],
        /// Clock High Cycles
        HIGH OFFSET(8) NUMBITS(8) [],
        /// Clock Low Cycles
        LOW OFFSET(0) NUMBITS(8) []
    ],

    SmbusTiming [
        /// SMBus Timeout Clock Prescaler
        EXP OFFSET(28) NUMBITS(4) [],
        /// Clock High Maximum Cycles
        THMAX OFFSET(16) NUMBITS(8) [],
        /// Master Clock Stretch Maximum Cycles
        TLWOM OFFSET(8) NUMBITS(8) [],
        /// Slave Clock Stretch Maximum Cycles
        TLOWS OFFSET(0) NUMBITS(8) []
    ],

    Command [
        /// HS-mode Master Code
        HSMCODE OFFSET(28) NUMBITS(3) [],
        /// HS-mode
        HS OFFSET(26) NUMBITS(1) [
            NoHSMode = 0,
            HSMode = 1
        ],
        /// ACK Last Master RX Byte
        ACKLAST OFFSET(25) NUMBITS(1) [
            NackLast = 0,
            AckLast = 1
        ],
        /// Packet Error Checking Enable
        PECEN OFFSET(24) NUMBITS(1) [
            NoPecByteVerification = 0,
            PecByteVerification = 1
        ],
        /// Number of Data Bytes in Transfer
        NBYTES OFFSET(16) NUMBITS(8) [],
        /// CMDR Valid
        VALID OFFSET(15) NUMBITS(1) [],
        /// Send STOP Condition
        STOP OFFSET(14) NUMBITS(1) [
            NoSendStop = 0,
            SendStop = 1
        ],
        /// Send START Condition
        START OFFSET(13) NUMBITS(1) [
            NoStartCondition = 0,
            StartCondition = 1
        ],
        /// Transfer is to Same Address as Previous Address
        REPSAME OFFSET(12) NUMBITS(1) [],
        /// Ten Bit Addressing Mode
        TENBIT OFFSET(11) NUMBITS(1) [
            SevenBitAddressing = 0,
            TenBitAddressing = 1
        ],
        /// Slave Address
        SADR OFFSET(1) NUMBITS(10) [],
        /// Transfer Direction
        READ OFFSET(0) NUMBITS(1) [
            Transmit = 0,
            Receive = 1
        ]
    ],

    ReceiveHolding [
        /// Received Data
        RXDATA OFFSET(0) NUMBITS(8) []
    ],

    TransmitHolding [
        /// Data to Transmit
        TXDATA OFFSET(0) NUMBITS(8) []
    ],

    Status [
        /// ACK in HS-mode Master Code Phase Received
        HSMCACK 17,
        /// Master Interface Enable
        MENB 16,
        /// Stop Request Accepted
        STOP 14,
        /// PEC Error
        PECERR 13,
        /// Timeout
        TOUT 12,
        /// Arbitration Lost
        ARBLST 10,
        /// NAK in Data Phase Received
        DNAK 9,
        /// NAK in Address Phase Received
        ANAK 8,
        /// Two-wire Bus is Free
        BUSFREE 5,
        /// Master Interface is Idle
        IDLE 4,
        /// Command Complete
        CCOMP 3,
        /// Ready for More Commands
        CRDY 2,
        /// THR Data Ready
        TXRDY 1,
        /// RHR Data Ready
        RXRDY 0
    ],

    Interrupt [
        /// ACK in HS-mode Master Code Phase Received
        HSMCACK 17,
        /// Stop Request Accepted
        STOP 14,
        /// PEC Error
        PECERR 13,
        /// Timeout
        TOUT 12,
        /// Arbitration Lost
        ARBLST 10,
        /// NAK in Data Phase Received
        DNAK 9,
        /// NAK in Address Phase Received
        ANAK 8,
        /// Two-wire Bus is Free
        BUSFREE 5,
        /// Master Interface is Idle
        IDLE 4,
        /// Command Complete
        CCOMP 3,
        /// Ready for More Commands
        CRDY 2,
        /// THR Data Ready
        TXRDY 1,
        /// RHR Data Ready
        RXRDY 0
    ],

    StatusClear [
        /// ACK in HS-mode Master Code Phase Received
        HSMCACK 17,
        /// Stop Request Accepted
        STOP 14,
        /// PEC Error
        PECERR 13,
        /// Timeout
        TOUT 12,
        /// Arbitration Lost
        ARBLST 10,
        /// NAK in Data Phase Received
        DNAK 9,
        /// NAK in Address Phase Received
        ANAK 8,
        /// Command Complete
        CCOMP 3
    ],

    SlewRate [
        /// Input Spike Filter Control
        FILTER OFFSET(28) NUMBITS(2) [
            StandardOrFast = 2,
            FastModePlus = 3
        ],
        /// Clock Slew Limit
        CLSLEW OFFSET(24) NUMBITS(2) [],
        /// Clock Drive Strength LOW
        CLDRIVEL OFFSET(16) NUMBITS(3) [],
        /// Data Slew Limit
        DASLEW OFFSET(8) NUMBITS(2) [],
        /// Data Drive Strength LOW
        DADRIVEL OFFSET(0) NUMBITS(3) []
    ]
];

register_bitfields![u32,
    ControlSlave [
        /// Ten Bit Address Match
        TENBIT OFFSET(26) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        /// Slave Address
        ADR OFFSET(16) NUMBITS(10) [],
        /// Stretch Clock on Data Byte Reception
        SODR OFFSET(15) NUMBITS(1) [],
        /// Stretch Clock on Address Match
        SOAM OFFSET(14) NUMBITS(1) [
            NoStretch = 0,
            Stretch = 1
        ],
        /// NBYTES Count Up
        CUP OFFSET(13) NUMBITS(1) [
            CountDown = 0,
            CountUp = 1
        ],
        /// Slave Receiver Data Phase ACK Value
        ACK OFFSET(12) NUMBITS(1) [
            AckLow = 0,
            AckHigh = 1
        ],
        /// Packet Error Checking Enable
        PECEN OFFSET(11) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        /// SMBus Host Header
        SMHH OFFSET(10) NUMBITS(1) [
            NoAckHostHeader = 0,
            AckHostHeader = 1
        ],
        /// SMBus Default Address
        SMDA OFFSET(9) NUMBITS(1) [
            NoAckDefaultAddress = 0,
            AckDefaultAddress = 1
        ],
        /// Software Reset
        SWRST OFFSET(7) NUMBITS(1) [],
        /// Clock Stretch Enable
        STREN OFFSET(4) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        /// General Call Address Match
        GCMATCH OFFSET(3) NUMBITS(1) [
            NoAckGeneralCallAddress = 0,
            AckGeneralCallAddress = 1
        ],
        /// Slave Address Match
        SMATCH OFFSET(2) NUMBITS(1) [
            NoAckSlaveAddress = 0,
            AckSlaveAddress = 1
        ],
        /// SMBus Mode Enable
        SMEN OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        /// Slave Enable
        SEN OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    Nbytes [
        NBYTES OFFSET(0) NUMBITS(8) []
    ],

    Timing [
        /// Clock Prescaler
        EXP OFFSET(28) NUMBITS(4) [],
        /// Data Setup Cycles
        SUDAT OFFSET(16) NUMBITS(8) [],
        /// SMBus Timeout Cycles
        TTOUT OFFSET(8) NUMBITS(8) [],
        /// SMBus Low Cycles
        TLOWS OFFSET(0) NUMBITS(8) []
    ],

    PacketErrorCheck [
        /// Calculated PEC Value
        PEC OFFSET(0) NUMBITS(8) []
    ],

    StatusSlave [
        /// Byte Transfer Finished
        BTF 23,
        /// Repeated Start Received
        REP 22,
        /// Stop Received
        STO 21,
        /// SMBus Default Address Match
        SMBDAM 20,
        /// SMBus Host Header Address Match
        SMBHHM 19,
        /// General Call Match
        GCM 17,
        /// Slave Address Match
        SAM 16,
        /// Bus Error
        BUSERR 14,
        /// SMBus PEC Error
        SMBPECERR 13,
        /// SMBus Timeout
        SMBTOUT 12,
        /// NAK Received
        NAK 8,
        /// Overrun
        ORUN 7,
        /// Underrun
        URUN 6,
        /// Transmitter Mode
        TRA 5,
        /// Transmission Complete
        TCOMP 3,
        /// Slave Enabled
        SEN 2,
        /// THR Data Ready
        TXRDY 1,
        /// RHR Data Ready
        RXRDY 0
    ],

    InterruptSlave [
        /// Byte Transfer Finished
        BTF 23,
        /// Repeated Start Received
        REP 22,
        /// Stop Received
        STO 21,
        /// SMBus Default Address Match
        SMBDAM 20,
        /// SMBus Host Header Address Match
        SMBHHM 19,
        /// General Call Match
        GCM 17,
        /// Slave Address Match
        SAM 16,
        /// Bus Error
        BUSERR 14,
        /// SMBus PEC Error
        SMBPECERR 13,
        /// SMBus Timeout
        SMBTOUT 12,
        /// NAK Received
        NAK 8,
        /// Overrun
        ORUN 7,
        /// Underrun
        URUN 6,
        /// Transmission Complete
        TCOMP 3,
        /// THR Data Ready
        TXRDY 1,
        /// RHR Data Ready
        RXRDY 0
    ],

    StatusClearSlave [
        /// Byte Transfer Finished
        BTF 23,
        /// Repeated Start Received
        REP 22,
        /// Stop Received
        STO 21,
        /// SMBus Default Address Match
        SMBDAM 20,
        /// SMBus Host Header Address Match
        SMBHHM 19,
        /// General Call Match
        GCM 17,
        /// Slave Address Match
        SAM 16,
        /// Bus Error
        BUSERR 14,
        /// SMBus PEC Error
        SMBPECERR 13,
        /// SMBus Timeout
        SMBTOUT 12,
        /// NAK Received
        NAK 8,
        /// Overrun
        ORUN 7,
        /// Underrun
        URUN 6,
        /// Transmission Complete
        TCOMP 3
    ],

    SlewRateSlave [
        /// Input Spike Filter Control
        FILTER OFFSET(28) NUMBITS(2) [],
        /// Data Slew Limit
        DASLEW OFFSET(8) NUMBITS(2) [],
        /// Data Drive Strength LOW
        DADRIVEL OFFSET(0) NUMBITS(3) []
    ]
];

// The addresses in memory (7.1 of manual) of the TWIM peripherals
const I2C_BASE_ADDRS: [StaticRef<TWIMRegisters>; 4] = unsafe {
    [
        StaticRef::new(0x40018000 as *const TWIMRegisters),
        StaticRef::new(0x4001C000 as *const TWIMRegisters),
        StaticRef::new(0x40078000 as *const TWIMRegisters),
        StaticRef::new(0x4007C000 as *const TWIMRegisters),
    ]
};

// The addresses in memory (7.1 of manual) of the TWIM peripherals
const I2C_SLAVE_BASE_ADDRS: [StaticRef<TWISRegisters>; 2] = unsafe {
    [
        StaticRef::new(0x40018400 as *const TWISRegisters),
        StaticRef::new(0x4001C400 as *const TWISRegisters),
    ]
};

// Three main I2C speeds
#[derive(Clone, Copy)]
pub enum Speed {
    Standard100k,
    Fast400k,
    FastPlus1M,
}

/// Wrapper for TWIM clock that ensures TWIS clock is off
struct TWIMClock {
    master: pm::Clock,
    slave: Option<pm::Clock>,
}
impl ClockInterface for TWIMClock {
    fn is_enabled(&self) -> bool {
        self.master.is_enabled()
    }

    fn enable(&self) {
        self.slave.map(|slave_clock| {
            if slave_clock.is_enabled() {
                panic!("I2C: Request for master clock, but slave active");
            }
        });
        self.master.enable();
    }

    fn disable(&self) {
        self.master.disable();
    }
}

/// Wrapper for TWIS clock that ensures TWIM clock is off
struct TWISClock {
    master: pm::Clock,
    slave: Option<pm::Clock>,
}
impl ClockInterface for TWISClock {
    fn is_enabled(&self) -> bool {
        let slave_clock = self.slave.expect("I2C: Use of slave with no clock");
        slave_clock.is_enabled()
    }

    fn enable(&self) {
        let slave_clock = self.slave.expect("I2C: Use of slave with no clock");
        if self.master.is_enabled() {
            panic!("I2C: Request for slave clock, but master active");
        }
        slave_clock.enable();
    }

    fn disable(&self) {
        let slave_clock = self.slave.expect("I2C: Use of slave with no clock");
        slave_clock.disable();
    }
}

/// Abstraction of the I2C hardware
pub struct I2CHw {
    master_mmio_address: StaticRef<TWIMRegisters>,
    slave_mmio_address: Option<StaticRef<TWISRegisters>>,
    master_clock: TWIMClock,
    slave_clock: TWISClock,
    dma: Cell<Option<&'static DMAChannel>>,
    dma_pids: (DMAPeripheral, DMAPeripheral),
    master_client: Cell<Option<&'static hil::i2c::I2CHwMasterClient>>,
    slave_client: Cell<Option<&'static hil::i2c::I2CHwSlaveClient>>,
    on_deck: Cell<Option<(DMAPeripheral, usize)>>,

    slave_enabled: Cell<bool>,
    my_slave_address: Cell<u8>,
    slave_read_buffer: TakeCell<'static, [u8]>,
    slave_read_buffer_len: Cell<u8>,
    slave_read_buffer_index: Cell<u8>,
    slave_write_buffer: TakeCell<'static, [u8]>,
    slave_write_buffer_len: Cell<u8>,
    slave_write_buffer_index: Cell<u8>,
}

impl PeripheralManagement<TWIMClock> for I2CHw {
    type RegisterType = TWIMRegisters;

    fn get_registers(&self) -> &TWIMRegisters {
        &*self.master_mmio_address
    }

    fn get_clock(&self) -> &TWIMClock {
        &self.master_clock
    }

    fn before_peripheral_access(&self, clock: &TWIMClock, _: &TWIMRegisters) {
        if clock.is_enabled() == false {
            clock.enable();
        }
    }

    fn after_peripheral_access(&self, clock: &TWIMClock, registers: &TWIMRegisters) {
        // If there are no interrupts active then we can disable the clock
        // for this peripheral.
        if registers.imr.get() == 0 {
            clock.disable();
        }
    }
}
type TWIMRegisterManager<'a> = PeripheralManager<'a, I2CHw, TWIMClock>;

impl PeripheralManagement<TWISClock> for I2CHw {
    type RegisterType = TWISRegisters;

    fn get_registers<'a>(&'a self) -> &'a TWISRegisters {
        &*self
            .slave_mmio_address
            .as_ref()
            .expect("Access of non-existent slave")
    }

    fn get_clock(&self) -> &TWISClock {
        &self.slave_clock
    }

    fn before_peripheral_access(&self, clock: &TWISClock, _: &TWISRegisters) {
        if clock.is_enabled() == false {
            clock.enable();
        }
    }

    fn after_peripheral_access(&self, clock: &TWISClock, registers: &TWISRegisters) {
        // If there are no interrupts active then we can disable the clock
        // for this peripheral.
        if registers.imr.get() == 0 {
            clock.disable();
        }
    }
}
type TWISRegisterManager<'a> = PeripheralManager<'a, I2CHw, TWISClock>;

const fn create_twims_clocks(
    master: pm::Clock,
    slave: Option<pm::Clock>,
) -> (TWIMClock, TWISClock) {
    (TWIMClock { master, slave }, TWISClock { master, slave })
}
pub static mut I2C0: I2CHw = I2CHw::new(
    I2C_BASE_ADDRS[0],
    Some(I2C_SLAVE_BASE_ADDRS[0]),
    create_twims_clocks(
        pm::Clock::PBA(pm::PBAClock::TWIM0),
        Some(pm::Clock::PBA(pm::PBAClock::TWIS0)),
    ),
    DMAPeripheral::TWIM0_RX,
    DMAPeripheral::TWIM0_TX,
);
pub static mut I2C1: I2CHw = I2CHw::new(
    I2C_BASE_ADDRS[1],
    Some(I2C_SLAVE_BASE_ADDRS[1]),
    create_twims_clocks(
        pm::Clock::PBA(pm::PBAClock::TWIM1),
        Some(pm::Clock::PBA(pm::PBAClock::TWIS1)),
    ),
    DMAPeripheral::TWIM1_RX,
    DMAPeripheral::TWIM1_TX,
);
pub static mut I2C2: I2CHw = I2CHw::new(
    I2C_BASE_ADDRS[2],
    None,
    create_twims_clocks(pm::Clock::PBA(pm::PBAClock::TWIM2), None),
    DMAPeripheral::TWIM2_RX,
    DMAPeripheral::TWIM2_TX,
);
pub static mut I2C3: I2CHw = I2CHw::new(
    I2C_BASE_ADDRS[3],
    None,
    create_twims_clocks(pm::Clock::PBA(pm::PBAClock::TWIM3), None),
    DMAPeripheral::TWIM3_RX,
    DMAPeripheral::TWIM3_TX,
);

// Need to implement the `new` function on the I2C device as a constructor.
// This gets called from the device tree.
impl I2CHw {
    const fn new(
        base_addr: StaticRef<TWIMRegisters>,
        slave_base_addr: Option<StaticRef<TWISRegisters>>,
        clocks: (TWIMClock, TWISClock),
        dma_rx: DMAPeripheral,
        dma_tx: DMAPeripheral,
    ) -> I2CHw {
        I2CHw {
            master_mmio_address: base_addr,
            slave_mmio_address: slave_base_addr,
            master_clock: clocks.0,
            slave_clock: clocks.1,
            dma: Cell::new(None),
            dma_pids: (dma_rx, dma_tx),
            master_client: Cell::new(None),
            slave_client: Cell::new(None),
            on_deck: Cell::new(None),

            slave_enabled: Cell::new(false),
            my_slave_address: Cell::new(0),
            slave_read_buffer: TakeCell::empty(),
            slave_read_buffer_len: Cell::new(0),
            slave_read_buffer_index: Cell::new(0),
            slave_write_buffer: TakeCell::empty(),
            slave_write_buffer_len: Cell::new(0),
            slave_write_buffer_index: Cell::new(0),
        }
    }

    /// Set the clock prescaler and the time widths of the I2C signals
    /// in the CWGR register to make the bus run at a particular I2C speed.
    fn set_bus_speed(&self, twim: &TWIMRegisterManager) {
        // Set I2C waveform timing parameters based on ASF code
        let system_frequency = pm::get_system_frequency();
        let mut exp = 0;
        let mut f_prescaled = system_frequency / 400000 / 2;
        while (f_prescaled > 0xff) && (exp <= 0x7) {
            // Increase the prescale factor, and update our frequency
            exp += 1;
            f_prescaled /= 2;
        }

        // Check that we have a valid setting
        if exp > 0x7 {
            panic!("Cannot setup I2C waveform timing with given system clock.");
        }

        let low = f_prescaled / 2;
        let high = f_prescaled - low;
        let data = 0;
        let stasto = f_prescaled;

        twim.registers.cwgr.write(
            ClockWaveformGenerator::EXP.val(exp)
                + ClockWaveformGenerator::DATA.val(data)
                + ClockWaveformGenerator::STASTO.val(stasto)
                + ClockWaveformGenerator::HIGH.val(high)
                + ClockWaveformGenerator::LOW.val(low),
        )
    }

    pub fn set_dma(&self, dma: &'static DMAChannel) {
        self.dma.set(Some(dma));
    }

    pub fn set_master_client(&self, client: &'static hil::i2c::I2CHwMasterClient) {
        self.master_client.set(Some(client));
    }

    pub fn set_slave_client(&self, client: &'static hil::i2c::I2CHwSlaveClient) {
        self.slave_client.set(Some(client));
    }

    pub fn handle_interrupt(&self) {
        use kernel::hil::i2c::Error;

        let old_status = {
            let twim = &TWIMRegisterManager::new(&self);

            let old_status = twim.registers.sr.extract();

            // Clear all status registers.
            twim.registers.scr.write(
                StatusClear::HSMCACK::SET
                    + StatusClear::STOP::SET
                    + StatusClear::PECERR::SET
                    + StatusClear::TOUT::SET
                    + StatusClear::ARBLST::SET
                    + StatusClear::DNAK::SET
                    + StatusClear::ANAK::SET
                    + StatusClear::CCOMP::SET,
            );

            old_status
        };

        let err = if old_status.is_set(Status::ANAK) {
            Some(Error::AddressNak)
        } else if old_status.is_set(Status::DNAK) {
            Some(Error::DataNak)
        } else if old_status.is_set(Status::ARBLST) {
            Some(Error::ArbitrationLost)
        } else if old_status.is_set(Status::CCOMP) {
            Some(Error::CommandComplete)
        } else {
            None
        };

        let on_deck = self.on_deck.get();
        self.on_deck.set(None);
        match on_deck {
            None => {
                {
                    let twim = &TWIMRegisterManager::new(&self);

                    twim.registers.cmdr.set(0);
                    twim.registers.ncmdr.set(0);
                    self.disable_interrupts(twim);

                    if err.is_some() {
                        // enable, reset, disable
                        twim.registers.cr.write(Control::MEN::SET);
                        twim.registers.cr.write(Control::SWRST::SET);
                        twim.registers.cr.write(Control::MDIS::SET);
                    }
                }

                err.map(|err| {
                    self.master_client.get().map(|client| {
                        let buf = match self.dma.get() {
                            Some(dma) => {
                                let b = dma.abort_transfer();
                                self.dma.set(Some(dma));
                                b
                            }
                            None => None,
                        };
                        buf.map(|buf| {
                            client.command_complete(buf, err);
                        });
                    });
                });
            }
            Some((dma_periph, len)) => {
                // Check to see if we are only trying to get one byte. If we
                // are, and the RXRDY bit is already set, then we already have
                // that byte in the RHR register. If we setup DMA after we
                // have the single byte we are looking for, everything breaks
                // because we will never get another byte and therefore
                // no more interrupts. So, we just read the byte we have
                // and call this I2C command complete.
                if (len == 1) && old_status.is_set(Status::TXRDY) {
                    let the_byte = {
                        let twim = &TWIMRegisterManager::new(&self);

                        twim.registers.cmdr.set(0);
                        twim.registers.ncmdr.set(0);
                        self.disable_interrupts(twim);

                        if err.is_some() {
                            // enable, reset, disable
                            twim.registers.cr.write(Control::MEN::SET);
                            twim.registers.cr.write(Control::SWRST::SET);
                            twim.registers.cr.write(Control::MDIS::SET);
                        }

                        twim.registers.rhr.read(ReceiveHolding::RXDATA) as u8
                    };

                    err.map(|err| {
                        self.master_client.get().map(|client| {
                            let buf = match self.dma.get() {
                                Some(dma) => {
                                    let b = dma.abort_transfer();
                                    self.dma.set(Some(dma));
                                    b
                                }
                                None => None,
                            };
                            buf.map(|buf| {
                                // Save the already read byte.
                                buf[0] = the_byte;
                                client.command_complete(buf, err);
                            });
                        });
                    });
                } else {
                    {
                        let twim = &TWIMRegisterManager::new(&self);
                        // Enable transaction error interrupts
                        twim.registers.ier.write(
                            Interrupt::CCOMP::SET
                                + Interrupt::ANAK::SET
                                + Interrupt::DNAK::SET
                                + Interrupt::ARBLST::SET,
                        );
                    }
                    self.dma.get().map(|dma| {
                        let buf = dma.abort_transfer().unwrap();
                        dma.prepare_transfer(dma_periph, buf, len);
                        dma.start_transfer();
                    });
                }
            }
        }
    }

    fn setup_transfer(
        &self,
        twim: &TWIMRegisterManager,
        chip: u8,
        flags: FieldValue<u32, Command::Register>,
        direction: FieldValue<u32, Command::Register>,
        len: u8,
    ) {
        // disable before configuring
        twim.registers.cr.write(Control::MDIS::SET);

        // Configure the command register with the settings for this transfer.
        twim.registers.cmdr.write(
            Command::SADR.val(chip as u32)
                + flags
                + Command::VALID::SET
                + Command::NBYTES.val(len as u32)
                + direction,
        );
        twim.registers.ncmdr.set(0);

        // Enable transaction error interrupts
        twim.registers.ier.write(
            Interrupt::CCOMP::SET
                + Interrupt::ANAK::SET
                + Interrupt::DNAK::SET
                + Interrupt::ARBLST::SET,
        );
    }

    fn setup_nextfer(
        &self,
        twim: &TWIMRegisterManager,
        chip: u8,
        flags: FieldValue<u32, Command::Register>,
        direction: FieldValue<u32, Command::Register>,
        len: u8,
    ) {
        // disable before configuring
        twim.registers.cr.write(Control::MDIS::SET);

        twim.registers.ncmdr.write(
            Command::SADR.val(chip as u32)
                + flags
                + Command::VALID::SET
                + Command::NBYTES.val(len as u32)
                + direction,
        );

        // Enable
        twim.registers.cr.write(Control::MEN::SET);
    }

    fn master_enable(&self, twim: &TWIMRegisterManager) {
        // Enable to begin transfer
        twim.registers.cr.write(Control::MEN::SET);
    }

    fn write(
        &self,
        chip: u8,
        flags: FieldValue<u32, Command::Register>,
        data: &'static mut [u8],
        len: u8,
    ) {
        let twim = &TWIMRegisterManager::new(&self);
        self.dma.get().map(move |dma| {
            dma.enable();
            dma.prepare_transfer(self.dma_pids.1, data, len as usize);
            self.setup_transfer(twim, chip, flags, Command::READ::Transmit, len);
            self.master_enable(twim);
            dma.start_transfer();
        });
    }

    fn read(
        &self,
        chip: u8,
        flags: FieldValue<u32, Command::Register>,
        data: &'static mut [u8],
        len: u8,
    ) {
        let twim = &TWIMRegisterManager::new(&self);
        self.dma.get().map(move |dma| {
            dma.enable();
            dma.prepare_transfer(self.dma_pids.0, data, len as usize);
            self.setup_transfer(twim, chip, flags, Command::READ::Receive, len);
            self.master_enable(twim);
            dma.start_transfer();
        });
    }

    fn write_read(&self, chip: u8, data: &'static mut [u8], split: u8, read_len: u8) {
        let twim = &TWIMRegisterManager::new(&self);
        self.dma.get().map(move |dma| {
            dma.enable();
            dma.prepare_transfer(self.dma_pids.1, data, split as usize);
            self.setup_transfer(
                twim,
                chip,
                Command::START::StartCondition,
                Command::READ::Transmit,
                split,
            );
            self.setup_nextfer(
                twim,
                chip,
                Command::START::StartCondition + Command::STOP::SendStop,
                Command::READ::Receive,
                read_len,
            );
            self.on_deck.set(Some((self.dma_pids.0, read_len as usize)));
            dma.start_transfer();
        });
    }

    fn disable_interrupts(&self, twim: &TWIMRegisterManager) {
        twim.registers.idr.set(!0);
    }

    /// Handle possible interrupt for TWIS module.
    pub fn handle_slave_interrupt(&self) {
        if self.slave_mmio_address.is_some() {
            let twis = &TWISRegisterManager::new(&self);

            // Get current status from the hardware.
            let status = twis.registers.sr.extract();
            let imr = twis.registers.imr.extract();
            // This will still be a "status" register, just with all of the
            // status bits corresponding to disabled interrupts cleared.
            let interrupts = status.bitand(imr.get());

            // Check for errors.
            if interrupts.matches_any(
                StatusSlave::BUSERR::SET
                    + StatusSlave::SMBPECERR::SET
                    + StatusSlave::SMBTOUT::SET
                    + StatusSlave::ORUN::SET
                    + StatusSlave::URUN::SET,
            ) {
                // From the datasheet: If a bus error (misplaced START or STOP)
                // condition is detected, the SR.BUSERR bit is set and the TWIS
                // waits for a new START condition.
                if interrupts.is_set(StatusSlave::BUSERR) {
                    // Restart and wait for the next start byte
                    twis.registers.scr.set(status.get());
                    return;
                }

                panic!("ERR 0x{:x}", interrupts.get());
            }

            // Check if we got the address match interrupt
            if interrupts.is_set(StatusSlave::SAM) {
                twis.registers.nbytes.write(Nbytes::NBYTES.val(0));

                // Did we get a read or a write?
                if status.is_set(StatusSlave::TRA) {
                    // This means the slave is in transmit mode, AKA we got a
                    // read.

                    // Clear the byte transfer done if set (copied from ASF)
                    twis.registers.scr.write(StatusClearSlave::BTF::SET);

                    // Setup interrupts that we now care about
                    twis.registers
                        .ier
                        .write(InterruptSlave::TCOMP::SET + InterruptSlave::BTF::SET);
                    twis.registers.ier.write(
                        InterruptSlave::BUSERR::SET
                            + InterruptSlave::SMBPECERR::SET
                            + InterruptSlave::SMBTOUT::SET
                            + InterruptSlave::ORUN::SET
                            + InterruptSlave::URUN::SET,
                    );

                    if self.slave_read_buffer.is_some() {
                        // Have buffer to send, start reading
                        self.slave_read_buffer_index.set(0);
                        let len = self.slave_read_buffer_len.get();

                        if len >= 1 {
                            self.slave_read_buffer.map(|buffer| {
                                twis.registers
                                    .thr
                                    .write(TransmitHolding::TXDATA.val(buffer[0] as u32));
                            });
                            self.slave_read_buffer_index.set(1);
                        } else {
                            // Send dummy byte
                            twis.registers.thr.write(TransmitHolding::TXDATA.val(0x2e));
                        }

                        // Make it happen by clearing status.
                        twis.registers.scr.set(status.get());
                    } else {
                        // Call to upper layers asking for a buffer to send
                        self.slave_client.get().map(|client| {
                            client.read_expected();
                        });
                    }
                } else {
                    // Slave is in receive mode, AKA we got a write.

                    // Get transmission complete and rxready interrupts.
                    twis.registers
                        .ier
                        .write(InterruptSlave::TCOMP::SET + InterruptSlave::RXRDY::SET);

                    // Set index to 0
                    self.slave_write_buffer_index.set(0);

                    if self.slave_write_buffer.is_some() {
                        // Clear to continue with existing buffer.
                        twis.registers.scr.set(status.get());
                    } else {
                        // Call to upper layers asking for a buffer to
                        // read into.
                        self.slave_client.get().map(|client| {
                            client.write_expected();
                        });
                    }
                }
            } else {
                // Did not get address match interrupt.

                if interrupts.is_set(StatusSlave::TCOMP) {
                    // Transmission complete

                    let nbytes = twis.registers.nbytes.get();

                    twis.registers.idr.set(!0);
                    twis.registers.ier.write(InterruptSlave::SAM::SET);
                    twis.registers.scr.set(status.get());

                    if status.is_set(StatusSlave::TRA) {
                        // read
                        self.slave_client.get().map(|client| {
                            self.slave_read_buffer.take().map(|buffer| {
                                client.command_complete(
                                    buffer,
                                    nbytes as u8,
                                    hil::i2c::SlaveTransmissionType::Read,
                                );
                            });
                        });
                    } else {
                        // write

                        let len = self.slave_write_buffer_len.get();
                        let idx = self.slave_write_buffer_index.get();

                        if len > idx {
                            self.slave_write_buffer.map(|buffer| {
                                buffer[idx as usize] =
                                    twis.registers.rhr.read(ReceiveHolding::RXDATA) as u8;
                            });
                            self.slave_write_buffer_index.set(idx + 1);
                        } else {
                            // Just drop on floor
                            twis.registers.rhr.get();
                        }

                        self.slave_client.get().map(|client| {
                            self.slave_write_buffer.take().map(|buffer| {
                                client.command_complete(
                                    buffer,
                                    nbytes as u8,
                                    hil::i2c::SlaveTransmissionType::Write,
                                );
                            });
                        });
                    }
                } else if interrupts.is_set(StatusSlave::BTF) {
                    // Byte transfer finished. Send the next byte from the
                    // buffer.

                    if self.slave_read_buffer.is_some() {
                        // Have buffer to send, start reading
                        let len = self.slave_read_buffer_len.get();
                        let idx = self.slave_read_buffer_index.get();

                        if len > idx {
                            self.slave_read_buffer.map(|buffer| {
                                twis.registers.thr.write(
                                    TransmitHolding::TXDATA.val(buffer[idx as usize] as u32),
                                );
                            });
                            self.slave_read_buffer_index.set(idx + 1);
                        } else {
                            // Send dummy byte
                            twis.registers.thr.write(TransmitHolding::TXDATA.val(0xdf));
                        }
                    } else {
                        // Send a default byte
                        twis.registers.thr.write(TransmitHolding::TXDATA.val(0xdc));
                    }

                    // Make it happen by clearing status.
                    twis.registers.scr.set(status.get());
                } else if interrupts.is_set(StatusSlave::RXRDY) {
                    // Receive byte ready.

                    if self.slave_write_buffer.is_some() {
                        // Check that the BTF byte is set at the beginning of
                        // the transfer. Sometimes a spurious RX ready interrupt
                        // happens at the beginning (right after the address
                        // byte) that we need to ignore, and checking the BTF
                        // bit fixes that. However, sometimes in the middle of a
                        // transfer we get an RXREADY interrupt where the BTF
                        // bit is NOT set. I don't know why.
                        if status.is_set(StatusSlave::BTF)
                            || self.slave_write_buffer_index.get() > 0
                        {
                            // Have buffer to read into
                            let len = self.slave_write_buffer_len.get();
                            let idx = self.slave_write_buffer_index.get();

                            if len > idx {
                                self.slave_write_buffer.map(|buffer| {
                                    buffer[idx as usize] =
                                        twis.registers.rhr.read(ReceiveHolding::RXDATA) as u8;
                                });
                                self.slave_write_buffer_index.set(idx + 1);
                            } else {
                                // Just drop on floor
                                twis.registers.rhr.get();
                            }
                        } else {
                            // Just drop on floor
                            twis.registers.rhr.get();
                        }
                    } else {
                        // Just drop on floor
                        twis.registers.rhr.get();
                    }

                    twis.registers.scr.set(status.get());
                }
            }
        }
    }

    /// Receive the bytes the I2C master is writing to us.
    fn slave_write_receive(&self, buffer: &'static mut [u8], len: u8) {
        self.slave_write_buffer.replace(buffer);
        self.slave_write_buffer_len.set(len);

        if self.slave_enabled.get() {
            if self.slave_mmio_address.is_some() {
                let twis = &TWISRegisterManager::new(&self);

                let status = twis.registers.sr.extract();
                let imr = twis.registers.imr.extract();
                let interrupts = status.bitand(imr.get());

                // Address match status bit still set, so we need to tell the TWIS
                // to continue.
                if interrupts.is_set(StatusSlave::SAM) && !status.is_set(StatusSlave::TRA) {
                    twis.registers.scr.set(status.get());
                }
            }
        }
    }

    /// Prepare a buffer for the I2C master to read from after a read call.
    fn slave_read_send(&self, buffer: &'static mut [u8], len: u8) {
        self.slave_read_buffer.replace(buffer);
        self.slave_read_buffer_len.set(len);
        self.slave_read_buffer_index.set(0);

        if self.slave_enabled.get() {
            if self.slave_mmio_address.is_some() {
                let twis = &TWISRegisterManager::new(&self);

                // Check to see if we should send the first byte.
                let status = twis.registers.sr.extract();
                let imr = twis.registers.imr.extract();
                let interrupts = status.bitand(imr.get());

                // Address match status bit still set. We got this function
                // call in response to an incoming read. Send the first
                // byte.
                if interrupts.is_set(StatusSlave::SAM) && status.is_set(StatusSlave::TRA) {
                    twis.registers.scr.write(StatusClearSlave::BTF::SET);

                    let len = self.slave_read_buffer_len.get();

                    if len >= 1 {
                        self.slave_read_buffer.map(|buffer| {
                            twis.registers
                                .thr
                                .write(TransmitHolding::TXDATA.val(buffer[0] as u32));
                        });
                        self.slave_read_buffer_index.set(1);
                    } else {
                        // Send dummy byte
                        twis.registers.thr.write(TransmitHolding::TXDATA.val(0x75));
                    }

                    // Make it happen by clearing status.
                    twis.registers.scr.set(status.get());
                }
            }
        }
    }

    fn slave_disable_interrupts(&self, twis: &TWISRegisterManager) {
        twis.registers.idr.set(!0);
    }

    fn slave_set_address(&self, address: u8) {
        self.my_slave_address.set(address);
    }

    fn slave_listen(&self) {
        if self.slave_mmio_address.is_some() {
            let twis = &TWISRegisterManager::new(&self);

            // Enable and configure
            let control = ControlSlave::ADR.val((self.my_slave_address.get() as u32) & 0x7F)
                + ControlSlave::SOAM::Stretch
                + ControlSlave::CUP::CountUp
                + ControlSlave::STREN::Enable
                + ControlSlave::SMATCH::AckSlaveAddress;
            twis.registers.cr.write(control);

            // Set this separately because that makes the HW happy.
            twis.registers.cr.write(control + ControlSlave::SEN::Enable);
        }
    }
}

impl DMAClient for I2CHw {
    fn transfer_done(&self, _pid: DMAPeripheral) {}
}

impl hil::i2c::I2CMaster for I2CHw {
    /// This enables the entire I2C peripheral
    fn enable(&self) {
        //disable the i2c slave peripheral
        hil::i2c::I2CSlave::disable(self);

        let twim = &TWIMRegisterManager::new(&self);

        // enable, reset, disable
        twim.registers.cr.write(Control::MEN::SET);
        twim.registers.cr.write(Control::SWRST::SET);
        twim.registers.cr.write(Control::MDIS::SET);

        // Init the bus speed
        self.set_bus_speed(twim);

        // slew
        twim.registers.srr.write(
            SlewRate::FILTER::StandardOrFast
                + SlewRate::CLDRIVEL.val(7)
                + SlewRate::DADRIVEL.val(7),
        );

        // clear interrupts
        twim.registers.scr.set(!0);
    }

    /// This disables the entire I2C peripheral
    fn disable(&self) {
        let twim = &TWIMRegisterManager::new(&self);
        twim.registers.cr.write(Control::MDIS::SET);
        self.disable_interrupts(twim);
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        I2CHw::write(
            self,
            addr,
            Command::START::StartCondition + Command::STOP::SendStop,
            data,
            len,
        );
    }

    fn read(&self, addr: u8, data: &'static mut [u8], len: u8) {
        I2CHw::read(
            self,
            addr,
            Command::START::StartCondition + Command::STOP::SendStop,
            data,
            len,
        );
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        I2CHw::write_read(self, addr, data, write_len, read_len)
    }
}

impl hil::i2c::I2CSlave for I2CHw {
    fn enable(&self) {
        if self.slave_mmio_address.is_some() {
            let twis = &TWISRegisterManager::new(&self);

            // enable, reset, disable
            twis.registers.cr.write(ControlSlave::SEN::SET);
            twis.registers.cr.write(ControlSlave::SWRST::SET);
            twis.registers.cr.set(0);

            // slew
            twis.registers
                .srr
                .write(SlewRateSlave::FILTER.val(0x2) + SlewRateSlave::DADRIVEL.val(7));

            // clear interrupts
            twis.registers.scr.set(!0);

            // We want to interrupt only on slave address match so we can
            // wait for a message from a master and then decide what to do
            // based on read/write.
            twis.registers.ier.write(InterruptSlave::SAM::SET);

            // Also setup all of the error interrupts.
            twis.registers.ier.write(
                InterruptSlave::BUSERR::SET
                    + InterruptSlave::SMBPECERR::SET
                    + InterruptSlave::SMBTOUT::SET
                    + InterruptSlave::ORUN::SET
                    + InterruptSlave::URUN::SET,
            );
        }

        self.slave_enabled.set(true);
    }

    /// This disables the entire I2C peripheral
    fn disable(&self) {
        self.slave_enabled.set(false);

        if self.slave_mmio_address.is_some() {
            let twis = &TWISRegisterManager::new(&self);
            twis.registers.cr.set(0);
            self.slave_disable_interrupts(twis);
        }
    }

    fn set_address(&self, addr: u8) {
        self.slave_set_address(addr);
    }

    fn write_receive(&self, data: &'static mut [u8], max_len: u8) {
        self.slave_write_receive(data, max_len);
    }

    fn read_send(&self, data: &'static mut [u8], max_len: u8) {
        self.slave_read_send(data, max_len);
    }

    fn listen(&self) {
        self.slave_listen();
    }
}

impl hil::i2c::I2CMasterSlave for I2CHw {}
