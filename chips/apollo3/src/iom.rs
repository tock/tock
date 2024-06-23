// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! IO Master Driver (I2C and SPI)

use core::cell::Cell;
use kernel::hil;
use kernel::hil::gpio::{Configure, Output};
use kernel::hil::i2c;
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiMaster, SpiMasterClient};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const IOM0_BASE: StaticRef<IomRegisters> =
    unsafe { StaticRef::new(0x5000_4000 as *const IomRegisters) };
const IOM1_BASE: StaticRef<IomRegisters> =
    unsafe { StaticRef::new(0x5000_5000 as *const IomRegisters) };
const IOM2_BASE: StaticRef<IomRegisters> =
    unsafe { StaticRef::new(0x5000_6000 as *const IomRegisters) };
const IOM3_BASE: StaticRef<IomRegisters> =
    unsafe { StaticRef::new(0x5000_7000 as *const IomRegisters) };
const IOM4_BASE: StaticRef<IomRegisters> =
    unsafe { StaticRef::new(0x5000_8000 as *const IomRegisters) };
const IOM5_BASE: StaticRef<IomRegisters> =
    unsafe { StaticRef::new(0x5000_9000 as *const IomRegisters) };

register_structs! {
    pub IomRegisters {
        (0x000 => fifo: ReadWrite<u32, FIFO::Register>),
        (0x004 => _reserved0),
        (0x100 => fifoptr: ReadWrite<u32, FIFOPTR::Register>),
        (0x104 => fifothr: ReadWrite<u32, FIFOTHR::Register>),
        (0x108 => fifopop: ReadWrite<u32, FIFOPOP::Register>),
        (0x10C => fifopush: ReadWrite<u32, FIFOPUSH::Register>),
        (0x110 => fifoctrl: ReadWrite<u32, FIFOCTRL::Register>),
        (0x114 => fifoloc: ReadWrite<u32, FIFOLOC::Register>),
        (0x118 => _reserved1),
        (0x200 => inten: ReadWrite<u32, INT::Register>),
        (0x204 => intstat: ReadWrite<u32, INT::Register>),
        (0x208 => intclr: ReadWrite<u32, INT::Register>),
        (0x20C => intset: ReadWrite<u32, INT::Register>),
        (0x210 => clkcfg: ReadWrite<u32, CLKCFG::Register>),
        (0x214 => submodctrl: ReadWrite<u32, SUBMODCTRL::Register>),
        (0x218 => cmd: ReadWrite<u32, CMD::Register>),
        (0x21C => dcx: ReadWrite<u32, DCX::Register>),
        (0x220 => offsethi: ReadWrite<u32, OFFSETHI::Register>),
        (0x224 => cmdstat: ReadOnly<u32, CMDSTAT::Register>),
        (0x228 => _reserved2),
        (0x240 => dmatrigen: ReadWrite<u32, DMATRIGEN::Register>),
        (0x244 => dmatrigstat: ReadWrite<u32, DMATRIGSTAT::Register>),
        (0x248 => _reserved3),
        (0x280 => dmacfg: ReadWrite<u32, DMACFG::Register>),
        (0x284 => _reserved4),
        (0x288 => dmatotcount: ReadWrite<u32, DMATOTCOUNT::Register>),
        (0x28C => dmatargaddr: ReadWrite<u32, DMATARGADDR::Register>),
        (0x290 => dmastat: ReadWrite<u32, DMASTAT::Register>),
        (0x294 => cqcfg: ReadWrite<u32, CQCFG::Register>),
        (0x298 => cqaddr: ReadWrite<u32, CQADDR::Register>),
        (0x29C => cqstat: ReadWrite<u32, CQSTAT::Register>),
        (0x2A0 => cqflags: ReadWrite<u32, CQFLAGS::Register>),
        (0x2A4 => cqsetclear: ReadWrite<u32, CQSETCLEAR::Register>),
        (0x2A8 => cqpauseen: ReadWrite<u32, CQPAUSEEN::Register>),
        (0x2AC => cqcuridx: ReadWrite<u32, CQCURIDX::Register>),
        (0x2B0 => cqendidx: ReadWrite<u32, CQENDIDX::Register>),
        (0x2B4 => status: ReadOnly<u32, STATUS::Register>),
        (0x2B8 => _reserved5),
        (0x300 => mspicfg: ReadWrite<u32, MSPICFG::Register>),
        (0x304 => _reserved6),
        (0x400 => mi2ccfg: ReadWrite<u32, MI2CCFG::Register>),
        (0x404 => devcfg: ReadWrite<u32, DEVCFG::Register>),
        (0x408 => _reserved7),
        (0x410 => iomdbg: ReadWrite<u32, IOMDBG::Register>),
        (0x414 => @END),
    }
}

register_bitfields![u32,
    FIFO [
        FIFO OFFSET(0) NUMBITS(32) []
    ],
    FIFOPTR [
        FIFO0SIZ OFFSET(0) NUMBITS(8) [],
        FIFO0REM OFFSET(8) NUMBITS(8) [],
        FIFO1SIZ OFFSET(16) NUMBITS(8) [],
        FIFO1REM OFFSET(24) NUMBITS(8) []
    ],
    FIFOTHR [
        FIFORTHR OFFSET(0) NUMBITS(6) [],
        FIFOWTHR OFFSET(8) NUMBITS(6) []
    ],
    FIFOPOP [
        FIFODOUT OFFSET(0) NUMBITS(32) []
    ],
    FIFOPUSH [
        FIFODIN OFFSET(0) NUMBITS(32) []
    ],
    FIFOCTRL [
        POPWR OFFSET(0) NUMBITS(1) [],
        FIFORSTN OFFSET(1) NUMBITS(1) []
    ],
    FIFOLOC [
        FIFOWPTR OFFSET(0) NUMBITS(4) [],
        FIFORPTR OFFSET(8) NUMBITS(4) []
    ],
    INT [
        CMDCMP OFFSET(0) NUMBITS(1) [],
        THR OFFSET(1) NUMBITS(1) [],
        FUNDFL OFFSET(2) NUMBITS(1) [],
        FOVFL OFFSET(3) NUMBITS(1) [],
        NAK OFFSET(4) NUMBITS(1) [],
        IACC OFFSET(5) NUMBITS(1) [],
        ICMD OFFSET(6) NUMBITS(1) [],
        START OFFSET(7) NUMBITS(1) [],
        STOP OFFSET(8) NUMBITS(1) [],
        ARB OFFSET(9) NUMBITS(1) [],
        DCMP OFFSET(10) NUMBITS(1) [],
        DERR OFFSET(11) NUMBITS(1) [],
        CQPAUSED OFFSET(12) NUMBITS(1) [],
        CQUPD OFFSET(13) NUMBITS(1) [],
        CQERR OFFSET(14) NUMBITS(1) []
    ],
    CLKCFG [
        IOCLKEN OFFSET(0) NUMBITS(1) [],
        FSEL OFFSET(8) NUMBITS(3) [],
        DIV3 OFFSET(11) NUMBITS(1) [],
        DIVEN OFFSET(12) NUMBITS(1) [],
        LOWPER OFFSET(16) NUMBITS(8) [],
        TOTPER OFFSET(24) NUMBITS(8) []
    ],
    SUBMODCTRL [
        SMOD0EN OFFSET(0) NUMBITS(1) [],
        SMOD0TYPE OFFSET(1) NUMBITS(3) [],
        SMOD1EN OFFSET(4) NUMBITS(1) [],
        SMOD1TYPE OFFSET(5) NUMBITS(3) []
    ],
    CMD [
        CMD OFFSET(0) NUMBITS(4) [
            WRITE = 0x1,
            READ = 0x2,
            TMW = 0x3,
            TMR = 0x4
        ],
        OFFSETCNT OFFSET(5) NUMBITS(2) [],
        CONT OFFSET(7) NUMBITS(1) [],
        TSIZE OFFSET(8) NUMBITS(12) [],
        CMDSEL OFFSET(20) NUMBITS(2) [],
        OFFSETLO OFFSET(24) NUMBITS(8) []
    ],
    DCX [
        CE0OUT OFFSET(0) NUMBITS(1) [],
        CE1OUT OFFSET(1) NUMBITS(1) [],
        CE2OUT OFFSET(2) NUMBITS(1) [],
        DCXEN OFFSET(4) NUMBITS(1) []
    ],
    OFFSETHI [
        OFFSETHI OFFSET(0) NUMBITS(16) []
    ],
    CMDSTAT [
        CCMD OFFSET(0) NUMBITS(4) [],
        CMDSTAT OFFSET(5) NUMBITS(4) [],
        CTSIZE OFFSET(8) NUMBITS(12) []
    ],
    DMATRIGEN [
        DCMDCMPEN OFFSET(0) NUMBITS(1) [],
        DTHREN OFFSET(1) NUMBITS(1) []
    ],
    DMATRIGSTAT [
        DCMDCMP OFFSET(0) NUMBITS(1) [],
        DTHR OFFSET(1) NUMBITS(1) [],
        DTOTCMP OFFSET(2) NUMBITS(1) []
    ],
    DMACFG [
        DMAEN OFFSET(0) NUMBITS(1) [],
        DMADIR OFFSET(1) NUMBITS(1) [],
        DMAPRI OFFSET(8) NUMBITS(1) [],
        DPWROFF OFFSET(9) NUMBITS(1) []
    ],
    DMATOTCOUNT [
        TOTCOUNT OFFSET(0) NUMBITS(12) []
    ],
    DMATARGADDR [
        TARGADDR OFFSET(0) NUMBITS(21) [],
        TARGADDR28 OFFSET(28) NUMBITS(1) []
    ],
    DMASTAT [
        DMATIP OFFSET(0) NUMBITS(1) [],
        DMACPL OFFSET(1) NUMBITS(1) [],
        DMAERR OFFSET(2) NUMBITS(1) []
    ],
    CQCFG [
        CQEN OFFSET(0) NUMBITS(1) [],
        CQPRI OFFSET(1) NUMBITS(1) [],
        MSPIFLGSEL OFFSET(2) NUMBITS(2) []
    ],
    CQADDR [
        CQADDR OFFSET(2) NUMBITS(19) [],
        CQADDR28 OFFSET(28) NUMBITS(1) []
    ],
    CQSTAT [
        CQTIP OFFSET(0) NUMBITS(1) [],
        CQPAUSED OFFSET(1) NUMBITS(1) [],
        CQERR OFFSET(2) NUMBITS(1) []
    ],
    CQFLAGS [
        CQFLAGS OFFSET(0) NUMBITS(16) [],
        CQIRQMASK OFFSET(16) NUMBITS(16) []
    ],
    CQSETCLEAR [
        CQFSET OFFSET(0) NUMBITS(8) [],
        CQFTGL OFFSET(8) NUMBITS(8) [],
        CQFCLR OFFSET(16) NUMBITS(8) []
    ],
    CQPAUSEEN [
        CQPEN OFFSET(0) NUMBITS(16) []
    ],
    CQCURIDX [
        CQCURIDX OFFSET(0) NUMBITS(8) []
    ],
    CQENDIDX [
        CQENDIDX OFFSET(0) NUMBITS(8) []
    ],
    STATUS [
        ERR OFFSET(0) NUMBITS(1) [],
        CMDACT OFFSET(1) NUMBITS(2) [],
        IDLESET OFFSET(2) NUMBITS(1) []
    ],
    MSPICFG [
        SPOL OFFSET(0) NUMBITS(1) [],
        SPHA OFFSET(1) NUMBITS(1) [],
        FULLDUP OFFSET(2) NUMBITS(1) [],
        WTFC OFFSET(16) NUMBITS(1) [],
        RDFC OFFSET(17) NUMBITS(1) [],
        MOSIINV OFFSET(18) NUMBITS(1) [],
        WTFCIRQ OFFSET(20) NUMBITS(1) [],
        WTFCPOL OFFSET(21) NUMBITS(1) [],
        RDFCPOL OFFSET(22) NUMBITS(1) [],
        SPILSB OFFSET(23) NUMBITS(1) [],
        DINDLY OFFSET(24) NUMBITS(3) [],
        DOUTDLY OFFSET(27) NUMBITS(3) [],
        MSPIRST OFFSET(30) NUMBITS(1) []
    ],
    MI2CCFG [
        ADDRSZ OFFSET(0) NUMBITS(1) [],
        IOMLSB OFFSET(1) NUMBITS(1) [],
        ARBEN OFFSET(2) NUMBITS(1) [],
        SDADLY OFFSET(4) NUMBITS(2) [],
        MI2CRST OFFSET(6) NUMBITS(1) [],
        SCLENDLY OFFSET(8) NUMBITS(3) [],
        SDAENDLY OFFSET(12) NUMBITS(3) [],
        SMPCNT OFFSET(16) NUMBITS(8) [],
        STRDIS OFFSET(24) NUMBITS(1) []
    ],
    DEVCFG [
        DEVADDR OFFSET(0) NUMBITS(10) []
    ],
    IOMDBG [
        DBGEN OFFSET(0) NUMBITS(1) [],
        IOCLKON OFFSET(1) NUMBITS(1) [],
        APBCLKON OFFSET(2) NUMBITS(1) [],
        DBGDATA OFFSET(3) NUMBITS(29) []
    ]
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    None,
    I2C,
    SPI,
}

pub struct Iom<'a> {
    registers: StaticRef<IomRegisters>,

    i2c_master_client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,
    spi_master_client: OptionalCell<&'a dyn SpiMasterClient>,

    buffer: TakeCell<'static, [u8]>,
    spi_read_buffer: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,
    write_index: Cell<usize>,

    read_len: Cell<usize>,
    read_index: Cell<usize>,

    op: Cell<Operation>,
    spi_phase: Cell<ClockPhase>,
    spi_cs: OptionalCell<&'a crate::gpio::GpioPin<'a>>,
    smbus: Cell<bool>,
}

impl<'a> Iom<'_> {
    pub fn new0() -> Iom<'a> {
        Iom {
            registers: IOM0_BASE,
            i2c_master_client: OptionalCell::empty(),
            spi_master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            op: Cell::new(Operation::None),
            spi_phase: Cell::new(ClockPhase::SampleLeading),
            spi_cs: OptionalCell::empty(),
            smbus: Cell::new(false),
        }
    }
    pub fn new1() -> Iom<'a> {
        Iom {
            registers: IOM1_BASE,
            i2c_master_client: OptionalCell::empty(),
            spi_master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            op: Cell::new(Operation::None),
            spi_phase: Cell::new(ClockPhase::SampleLeading),
            spi_cs: OptionalCell::empty(),
            smbus: Cell::new(false),
        }
    }
    pub fn new2() -> Iom<'a> {
        Iom {
            registers: IOM2_BASE,
            i2c_master_client: OptionalCell::empty(),
            spi_master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            op: Cell::new(Operation::None),
            spi_phase: Cell::new(ClockPhase::SampleLeading),
            spi_cs: OptionalCell::empty(),
            smbus: Cell::new(false),
        }
    }
    pub fn new3() -> Iom<'a> {
        Iom {
            registers: IOM3_BASE,
            i2c_master_client: OptionalCell::empty(),
            spi_master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            op: Cell::new(Operation::None),
            spi_phase: Cell::new(ClockPhase::SampleLeading),
            spi_cs: OptionalCell::empty(),
            smbus: Cell::new(false),
        }
    }
    pub fn new4() -> Iom<'a> {
        Iom {
            registers: IOM4_BASE,
            i2c_master_client: OptionalCell::empty(),
            spi_master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            op: Cell::new(Operation::None),
            spi_phase: Cell::new(ClockPhase::SampleLeading),
            spi_cs: OptionalCell::empty(),
            smbus: Cell::new(false),
        }
    }
    pub fn new5() -> Iom<'a> {
        Iom {
            registers: IOM5_BASE,
            i2c_master_client: OptionalCell::empty(),
            spi_master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            spi_read_buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            op: Cell::new(Operation::None),
            spi_phase: Cell::new(ClockPhase::SampleLeading),
            spi_cs: OptionalCell::empty(),
            smbus: Cell::new(false),
        }
    }

    fn i2c_reset_fifo(&self) {
        let regs = self.registers;

        // Set the value low to reset
        regs.fifoctrl.modify(FIFOCTRL::FIFORSTN::CLEAR);

        // Wait a few cycles to ensure the reset completes
        for _i in 0..30 {
            cortexm4::support::nop();
        }

        // Exit the reset state
        regs.fifoctrl.modify(FIFOCTRL::FIFORSTN::SET);
    }

    fn i2c_write_data(&self) {
        let regs = self.registers;
        let mut data_pushed = self.write_index.get();
        let len = self.write_len.get();

        if data_pushed == len {
            return;
        }

        self.buffer.map(|buf| {
            // Push some data to FIFO
            for i in (data_pushed / 4)..(len / 4) {
                let data_idx = i * 4;

                if regs.fifoptr.read(FIFOPTR::FIFO0REM) <= 4 {
                    self.write_index.set(data_pushed);
                    break;
                }

                let mut d = (buf[data_idx + 0] as u32) << 0;
                d |= (buf[data_idx + 1] as u32) << 8;
                d |= (buf[data_idx + 2] as u32) << 16;
                d |= (buf[data_idx + 3] as u32) << 24;

                regs.fifopush.set(d);

                data_pushed = data_idx + 4;
            }

            // We never filled up the FIFO
            if len < 4 || data_pushed > (len - 4) {
                // Check if we have any left over data
                if len % 4 == 1 {
                    let d = buf[len - 1] as u32;

                    regs.fifopush.set(d);
                } else if len % 4 == 2 {
                    let mut d = (buf[len - 1] as u32) << 8;
                    d |= (buf[len - 2] as u32) << 0;

                    regs.fifopush.set(d);
                } else if len % 4 == 3 {
                    let mut d = (buf[len - 1] as u32) << 16;
                    d |= (buf[len - 2] as u32) << 8;
                    d |= (buf[len - 3] as u32) << 0;

                    regs.fifopush.set(d);
                }
                self.write_index.set(len);
            }
        });
    }

    fn i2c_read_data(&self) {
        let regs = self.registers;
        let mut data_popped = self.read_index.get();
        let len = self.read_len.get();

        if data_popped == len {
            return;
        }

        self.buffer.map(|buf| {
            // Pop some data from the FIFO
            for i in (data_popped / 4)..(len / 4) {
                let data_idx = i * 4;

                if regs.fifoptr.read(FIFOPTR::FIFO1SIZ) < 4 {
                    self.read_index.set(data_popped);
                    break;
                }

                let d = regs.fifopop.get().to_ne_bytes();

                buf[data_idx + 0] = d[0];
                buf[data_idx + 1] = d[1];
                buf[data_idx + 2] = d[2];
                buf[data_idx + 3] = d[3];

                data_popped = data_idx + 4;
            }

            // Get remaining data that isn't 4 bytes long
            if len < 4 || data_popped > (len - 4) {
                // Check if we have any left over data
                if len % 4 == 1 {
                    let d = regs.fifopop.get().to_ne_bytes();

                    buf[len - 1] = d[0];
                } else if len % 4 == 2 {
                    let d = regs.fifopop.get().to_ne_bytes();

                    buf[len - 2] = d[0];
                    buf[len - 1] = d[1];
                } else if len % 4 == 3 {
                    let d = regs.fifopop.get().to_ne_bytes();

                    buf[len - 3] = d[0];
                    buf[len - 2] = d[1];
                    buf[len - 1] = d[2];
                }
                self.read_index.set(len);
            }
        });
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irqs = regs.intstat.extract();

        // Clear interrrupts
        regs.intclr.set(0xFFFF_FFFF);
        // Ensure interrupts remain enabled
        regs.inten.set(0xFFFF_FFFF);

        if irqs.is_set(INT::NAK) {
            if self.op.get() == Operation::I2C {
                // Disable interrupts
                regs.inten.set(0x00);
                self.i2c_reset_fifo();

                self.i2c_master_client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.command_complete(buffer, Err(i2c::Error::DataNak));
                    });
                });

                // Finished with SMBus
                if self.smbus.get() {
                    // Setup 400kHz
                    regs.clkcfg.write(
                        CLKCFG::TOTPER.val(0x1D)
                            + CLKCFG::LOWPER.val(0xE)
                            + CLKCFG::DIVEN.val(1)
                            + CLKCFG::DIV3.val(0)
                            + CLKCFG::FSEL.val(2)
                            + CLKCFG::IOCLKEN::SET,
                    );

                    self.smbus.set(false);
                }
            } else {
                // Disable interrupts
                regs.inten.set(0x00);

                // Clear CS
                self.spi_cs.map(|cs| cs.set());

                self.op.set(Operation::None);

                self.spi_master_client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        let read_buffer = self.spi_read_buffer.take();
                        client.read_write_done(
                            buffer,
                            read_buffer,
                            self.write_len.get(),
                            Err(ErrorCode::NOACK),
                        );
                    });
                });
            }
            return;
        }

        if self.op.get() == Operation::SPI {
            // Read the incoming data
            if let Some(buf) = self.spi_read_buffer.take() {
                while self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0 {
                    // The IOM doesn't correctly pop the data if we read it too fast
                    // there are a few erratas against the IOM when reading data
                    // from the FIFO (compared to DMA). Adding a small delay here is
                    // enough to ensure the fifoptr values update before the next
                    // iteration.
                    // See: https://ambiq.com/wp-content/uploads/2022/01/Apollo3-Blue-Errata-List.pdf
                    for _i in 0..3000 {
                        cortexm4::support::nop();
                    }

                    let d = self.registers.fifopop.get().to_ne_bytes();
                    let data_idx = self.read_index.get();

                    if let Some(b) = buf.get_mut(data_idx + 0) {
                        *b = d[0];
                        self.read_index.set(data_idx + 1);
                    }
                    if let Some(b) = buf.get_mut(data_idx + 1) {
                        *b = d[1];
                        self.read_index.set(data_idx + 2);
                    }
                    if let Some(b) = buf.get_mut(data_idx + 2) {
                        *b = d[2];
                        self.read_index.set(data_idx + 3);
                    }
                    if let Some(b) = buf.get_mut(data_idx + 3) {
                        *b = d[3];
                        self.read_index.set(data_idx + 4);
                    }
                }

                self.spi_read_buffer.replace(buf);

                if self.read_len.get() > self.read_index.get() {
                    let remaining_bytes = (self.read_len.get() - self.read_index.get()).min(32);
                    self.registers
                        .fifothr
                        .modify(FIFOTHR::FIFORTHR.val(remaining_bytes as u32));
                } else {
                    self.registers.fifothr.modify(FIFOTHR::FIFORTHR.val(0));
                }
            } else {
                while self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0 {
                    // The IOM doesn't correctly pop the data if we read it too fast
                    // there are a few erratas against the IOM when reading data
                    // from the FIFO (compared to DMA). Adding a small delay here is
                    // enough to ensure the fifoptr values update before the next
                    // iteration.
                    // See: https://ambiq.com/wp-content/uploads/2022/01/Apollo3-Blue-Errata-List.pdf
                    for _i in 0..3000 {
                        cortexm4::support::nop();
                    }

                    let _d = self.registers.fifopop.get().to_ne_bytes();
                }

                self.registers.fifothr.modify(FIFOTHR::FIFORTHR.val(0));
            }

            // Write more data out
            if self.write_index.get() < self.write_len.get() {
                if let Some(write_buffer) = self.buffer.take() {
                    let mut transfered_bytes = 0;

                    // While there is some free space in FIFO0 (writing to the SPI bus) and
                    // at least 4 bytes free in FIFO1 (reading from the SPI bus to the
                    // hardware FIFO) we write up to 24 bytes of data.
                    //
                    // The `> 4` really could be `>= 4` but > gives us a little wiggle room
                    // as the hardware does seem a little slow at updating the FIFO size
                    // registers.
                    //
                    // The 24 byte limit is along the same lines, of just making sure we
                    // don't write too much data. I don't have a good answer of why it should
                    // be 24, but that seems to work reliably from testing.
                    //
                    // There isn't a specific errata for this issue, but the official HAL
                    // uses DMA so there aren't a lot of FIFO users for large transfers like
                    // this.
                    while self.registers.fifoptr.read(FIFOPTR::FIFO0REM) > 0
                        && self.registers.fifoptr.read(FIFOPTR::FIFO1REM) > 4
                        && self.write_index.get() < self.write_len.get()
                        && transfered_bytes < 24
                    {
                        let idx = self.write_index.get();
                        let data = u32::from_le_bytes(
                            write_buffer[idx..(idx + 4)].try_into().unwrap_or([0; 4]),
                        );

                        self.registers.fifopush.set(data);
                        self.write_index.set(idx + 4);
                        transfered_bytes += 4;
                    }

                    self.buffer.replace(write_buffer);
                }

                let remaining_bytes = (self.write_len.get() - self.write_index.get()).min(32);
                self.registers
                    .fifothr
                    .modify(FIFOTHR::FIFOWTHR.val(remaining_bytes as u32));
            } else {
                self.registers.fifothr.modify(FIFOTHR::FIFOWTHR.val(0));
            }

            if (self.write_len.get() > 0
                && self.write_index.get() >= self.write_len.get()
                && self.read_len.get() > 0
                && self.read_index.get() >= self.read_len.get())
                || irqs.is_set(INT::CMDCMP)
            {
                // Disable interrupts
                regs.inten.set(0x00);

                // Clear CS
                self.spi_cs.map(|cs| cs.set());

                self.op.set(Operation::None);

                self.spi_master_client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        let read_buffer = self.spi_read_buffer.take();
                        client.read_write_done(buffer, read_buffer, self.write_len.get(), Ok(()));
                    });
                });
            }

            return;
        }

        if irqs.is_set(INT::CMDCMP) || irqs.is_set(INT::THR) {
            if self.op.get() == Operation::I2C {
                if irqs.is_set(INT::THR) {
                    if regs.fifothr.read(FIFOTHR::FIFOWTHR) > 0 {
                        let remaining = self.write_len.get() - self.write_index.get();

                        if remaining > 4 {
                            regs.fifothr.write(
                                FIFOTHR::FIFORTHR.val(0)
                                    + FIFOTHR::FIFOWTHR.val(remaining as u32 / 2),
                            );
                        } else {
                            regs.fifothr
                                .write(FIFOTHR::FIFORTHR.val(0) + FIFOTHR::FIFOWTHR.val(1));
                        }

                        self.i2c_write_data();
                    } else if regs.fifothr.read(FIFOTHR::FIFORTHR) > 0 {
                        let remaining = self.read_len.get() - self.read_index.get();

                        if remaining > 4 {
                            regs.fifothr.write(
                                FIFOTHR::FIFORTHR.val(remaining as u32 / 2)
                                    + FIFOTHR::FIFOWTHR.val(0),
                            );
                        } else {
                            regs.fifothr
                                .write(FIFOTHR::FIFORTHR.val(1) + FIFOTHR::FIFOWTHR.val(0));
                        }

                        self.i2c_read_data();
                    }
                }

                if irqs.is_set(INT::CMDCMP) || regs.intstat.is_set(INT::CMDCMP) {
                    if (self.read_len.get() > 0 && self.read_index.get() == self.read_len.get())
                        || (self.write_len.get() > 0
                            && self.write_index.get() == self.write_len.get())
                    {
                        // Disable interrupts
                        regs.inten.set(0x00);
                        self.i2c_reset_fifo();

                        self.i2c_master_client.map(|client| {
                            self.buffer.take().map(|buffer| {
                                client.command_complete(buffer, Ok(()));
                            });
                        });

                        // Finished with SMBus
                        if self.smbus.get() {
                            // Setup 400kHz
                            regs.clkcfg.write(
                                CLKCFG::TOTPER.val(0x1D)
                                    + CLKCFG::LOWPER.val(0xE)
                                    + CLKCFG::DIVEN.val(1)
                                    + CLKCFG::DIV3.val(0)
                                    + CLKCFG::FSEL.val(2)
                                    + CLKCFG::IOCLKEN::SET,
                            );

                            self.smbus.set(false);
                        }
                    }
                }
            } else {
                self.buffer.take().map(|write_buffer| {
                    let offset = self.write_index.get();
                    if self.write_len.get() > offset {
                        let burst_len = (self.write_len.get() - offset)
                            .min(self.registers.fifoptr.read(FIFOPTR::FIFO0REM) as usize);

                        // Start the transfer
                        self.registers.cmd.write(
                            CMD::TSIZE.val(burst_len as u32)
                                + CMD::CMDSEL.val(1)
                                + CMD::CONT::CLEAR
                                + CMD::CMD::WRITE
                                + CMD::OFFSETCNT.val(0_u32)
                                + CMD::OFFSETLO.val(0),
                        );

                        while self.registers.fifoptr.read(FIFOPTR::FIFO0REM) > 4
                            && self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) < 32
                            && self.write_index.get() < (((offset + burst_len) / 4) * 4)
                            && self.write_len.get() - self.write_index.get() > 4
                        {
                            let idx = self.write_index.get();
                            let data = u32::from_le_bytes(
                                write_buffer[idx..(idx + 4)].try_into().unwrap_or([0; 4]),
                            );

                            self.registers.fifopush.set(data);
                            self.write_index.set(idx + 4);
                        }

                        // Get remaining data that isn't 4 bytes long
                        if self.write_len.get() - self.write_index.get() < 4
                            && self.write_index.get() < (offset + burst_len)
                        {
                            let len = self.write_len.get() - self.write_index.get();
                            let mut buf = [0; 4];
                            // Check if we have any left over data
                            if len % 4 == 1 {
                                buf[len - 1] = write_buffer[self.write_index.get() + 0];
                            } else if len % 4 == 2 {
                                buf[len - 2] = write_buffer[self.write_index.get() + 0];
                                buf[len - 1] = write_buffer[self.write_index.get() + 1];
                            } else if len % 4 == 3 {
                                buf[len - 3] = write_buffer[self.write_index.get() + 0];
                                buf[len - 2] = write_buffer[self.write_index.get() + 1];
                                buf[len - 1] = write_buffer[self.write_index.get() + 2];
                            }
                            self.registers.fifopush.set(u32::from_le_bytes(buf));
                            self.write_index.set(self.write_index.get() + len);
                        }
                    }

                    self.buffer.replace(write_buffer);
                });
            }
        }
    }

    fn i2c_tx_rx(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let regs = self.registers;
        let mut offsetlo = 0;

        // Disable DMA as we don't support it
        regs.dmacfg.write(DMACFG::DMAEN::CLEAR);

        // Set the address
        regs.devcfg.write(DEVCFG::DEVADDR.val(addr as u32));

        // Set the DCX
        regs.dcx.set(0);

        // Set the read FIFO threashold and disable the write
        if read_len > 4 {
            regs.fifothr
                .write(FIFOTHR::FIFORTHR.val(read_len as u32 / 2) + FIFOTHR::FIFOWTHR.val(0));
        } else {
            regs.fifothr
                .write(FIFOTHR::FIFORTHR.val(1) + FIFOTHR::FIFOWTHR.val(0));
        }

        self.i2c_reset_fifo();

        if write_len > 0 {
            offsetlo = data[0] as u32;
        }

        if write_len == 2 {
            regs.offsethi.set(data[1] as u32);
        } else if write_len == 3 {
            regs.offsethi.set(data[1] as u32 | ((data[2] as u32) << 8));
        }

        if write_len > 3 {
            Err((i2c::Error::NotSupported, data))
        } else {
            // Save all the data and offsets we still need to send
            self.buffer.replace(data);
            self.write_len.set(write_len);
            self.read_len.set(read_len);
            self.write_index.set(0);
            self.read_index.set(0);
            // Clear and enable interrupts
            regs.intclr.set(0xFFFF_FFFF);
            regs.inten.set(0xFFFF_FFFF);

            // Start the transfer
            regs.cmd.write(
                CMD::TSIZE.val(read_len as u32)
                    + CMD::CMD::READ
                    + CMD::OFFSETCNT.val(write_len as u32)
                    + CMD::OFFSETLO.val(offsetlo),
            );

            Ok(())
        }
    }

    fn i2c_tx(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        // Disable DMA as we don't support it
        regs.dmacfg.write(DMACFG::DMAEN::CLEAR);

        // Set the address
        regs.devcfg.write(DEVCFG::DEVADDR.val(addr as u32));

        // Set the DCX
        regs.dcx.set(0);

        // Set the write FIFO threashold and disable the read
        if len > 4 {
            regs.fifothr
                .write(FIFOTHR::FIFORTHR.val(0) + FIFOTHR::FIFOWTHR.val(len as u32 / 2));
        } else {
            regs.fifothr
                .write(FIFOTHR::FIFORTHR.val(0) + FIFOTHR::FIFOWTHR.val(1));
        }

        self.i2c_reset_fifo();

        // Save all the data and offsets we still need to send
        self.buffer.replace(data);
        self.write_len.set(len);
        self.read_len.set(0);
        self.write_index.set(0);

        self.i2c_write_data();

        // Clear and enable interrupts
        regs.intclr.set(0xFFFF_FFFF);
        regs.inten.set(0xFFFF_FFFF);

        // Start the transfer
        regs.cmd
            .write(CMD::TSIZE.val(len as u32) + CMD::CMD::WRITE + CMD::CONT::CLEAR);
        Ok(())
    }

    fn i2c_rx(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        // Disable DMA as we don't support it
        regs.dmacfg.modify(DMACFG::DMAEN::CLEAR);

        // Set the address
        regs.devcfg.write(DEVCFG::DEVADDR.val(addr as u32));

        // Set the DCX
        regs.dcx.set(0);

        // Set the read FIFO threashold and disable the write
        if len > 4 {
            regs.fifothr
                .write(FIFOTHR::FIFORTHR.val(len as u32 / 2) + FIFOTHR::FIFOWTHR.val(0));
        } else {
            regs.fifothr
                .write(FIFOTHR::FIFORTHR.val(1) + FIFOTHR::FIFOWTHR.val(0));
        }

        self.i2c_reset_fifo();

        // Clear and enable interrupts
        regs.intclr.set(0xFFFF_FFFF);
        regs.inten.set(0xFFFF_FFFF);

        // Start the transfer
        regs.cmd
            .write(CMD::TSIZE.val(len as u32) + CMD::CMD::READ + CMD::CONT::CLEAR);

        // Save all the data and offsets we still need to send
        self.buffer.replace(buffer);
        self.read_len.set(len);
        self.write_len.set(0);
        self.read_index.set(0);

        self.i2c_read_data();

        Ok(())
    }
}

impl<'a> hil::i2c::I2CMaster<'a> for Iom<'a> {
    fn set_master_client(&self, i2c_master_client: &'a dyn i2c::I2CHwMasterClient) {
        self.i2c_master_client.set(i2c_master_client);
    }

    fn enable(&self) {
        let regs = self.registers;

        self.op.set(Operation::I2C);

        // Setup the I2C
        regs.mi2ccfg.write(
            MI2CCFG::STRDIS.val(0)
                + MI2CCFG::SMPCNT.val(3)
                + MI2CCFG::SDAENDLY.val(15)
                + MI2CCFG::SCLENDLY.val(2)
                + MI2CCFG::SDADLY.val(3)
                + MI2CCFG::ARBEN::SET
                + MI2CCFG::IOMLSB::CLEAR
                + MI2CCFG::ADDRSZ::CLEAR,
        );

        // Setup 400kHz
        regs.clkcfg.write(
            CLKCFG::TOTPER.val(0x1D)
                + CLKCFG::LOWPER.val(0xE)
                + CLKCFG::DIVEN.val(1)
                + CLKCFG::DIV3.val(0)
                + CLKCFG::FSEL.val(2)
                + CLKCFG::IOCLKEN::SET,
        );

        // Enable I2C
        regs.submodctrl.write(SUBMODCTRL::SMOD1EN::SET);

        // Disable command queue
        regs.cqcfg.modify(CQCFG::CQEN::CLEAR);
    }

    fn disable(&self) {
        let regs = self.registers;

        if self.op.get() == Operation::I2C {
            regs.submodctrl.write(SUBMODCTRL::SMOD1EN::CLEAR);

            self.op.set(Operation::None);
        }
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        if self.op.get() != Operation::I2C {
            return Err((hil::i2c::Error::Busy, data));
        }
        if data.len() < write_len {
            return Err((hil::i2c::Error::Overrun, data));
        }
        self.i2c_tx_rx(addr, data, write_len, read_len)
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        if self.op.get() != Operation::I2C {
            return Err((hil::i2c::Error::Busy, data));
        }
        if data.len() < len {
            return Err((hil::i2c::Error::Overrun, data));
        }
        self.i2c_tx(addr, data, len)
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        if self.op.get() != Operation::I2C {
            return Err((hil::i2c::Error::Busy, buffer));
        }
        if buffer.len() < len {
            return Err((hil::i2c::Error::Overrun, buffer));
        }
        self.i2c_rx(addr, buffer, len)
    }
}

impl<'a> hil::i2c::SMBusMaster<'a> for Iom<'a> {
    fn smbus_write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        if self.op.get() != Operation::I2C {
            return Err((hil::i2c::Error::Busy, data));
        }

        // Setup 100kHz
        regs.clkcfg.write(
            CLKCFG::TOTPER.val(0x77)
                + CLKCFG::LOWPER.val(0x3B)
                + CLKCFG::DIVEN.val(1)
                + CLKCFG::DIV3.val(0)
                + CLKCFG::FSEL.val(2)
                + CLKCFG::IOCLKEN::SET,
        );

        self.smbus.set(true);

        self.i2c_tx_rx(addr, data, write_len, read_len)
    }

    fn smbus_write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        if self.op.get() != Operation::I2C {
            return Err((hil::i2c::Error::Busy, data));
        }

        // Setup 100kHz
        regs.clkcfg.write(
            CLKCFG::TOTPER.val(0x77)
                + CLKCFG::LOWPER.val(0x3B)
                + CLKCFG::DIVEN.val(1)
                + CLKCFG::DIV3.val(0)
                + CLKCFG::FSEL.val(2)
                + CLKCFG::IOCLKEN::SET,
        );

        self.smbus.set(true);

        self.i2c_tx(addr, data, len)
    }

    fn smbus_read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

        if self.op.get() != Operation::I2C {
            return Err((hil::i2c::Error::Busy, buffer));
        }

        // Setup 100kHz
        regs.clkcfg.write(
            CLKCFG::TOTPER.val(0x77)
                + CLKCFG::LOWPER.val(0x3B)
                + CLKCFG::DIVEN.val(1)
                + CLKCFG::DIV3.val(0)
                + CLKCFG::FSEL.val(2)
                + CLKCFG::IOCLKEN::SET,
        );

        self.smbus.set(true);

        self.i2c_rx(addr, buffer, len)
    }
}

impl<'a> SpiMaster<'a> for Iom<'a> {
    type ChipSelect = &'a crate::gpio::GpioPin<'a>;

    fn init(&self) -> Result<(), ErrorCode> {
        self.op.set(Operation::SPI);

        self.registers.mspicfg.write(
            MSPICFG::FULLDUP::SET
                + MSPICFG::WTFC::CLEAR
                + MSPICFG::RDFC::CLEAR
                + MSPICFG::MOSIINV::CLEAR
                + MSPICFG::WTFCIRQ::CLEAR
                + MSPICFG::WTFCPOL::CLEAR
                + MSPICFG::RDFCPOL::CLEAR
                + MSPICFG::SPILSB::CLEAR
                + MSPICFG::DINDLY::CLEAR
                + MSPICFG::DOUTDLY::CLEAR
                + MSPICFG::MSPIRST::CLEAR,
        );

        // Enable SPI
        self.registers
            .submodctrl
            .write(SUBMODCTRL::SMOD1EN::CLEAR + SUBMODCTRL::SMOD0EN::SET);

        self.registers.dmatrigen.write(DMATRIGEN::DTHREN::SET);

        Ok(())
    }

    fn set_client(&self, client: &'a dyn SpiMasterClient) {
        self.spi_master_client.set(client);
    }

    fn is_busy(&self) -> bool {
        self.op.get() != Operation::None
    }

    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8], Option<&'static mut [u8]>)> {
        let write_len = write_buffer.len().min(len);
        let read_len = if let Some(ref buffer) = read_buffer {
            buffer.len().min(len)
        } else {
            0
        };

        // Disable DMA as we don't support it
        self.registers.dmacfg.write(DMACFG::DMAEN::CLEAR);

        // Set the DCX
        self.registers.dcx.set(0);

        self.write_index.set(0);
        self.read_index.set(0);

        // Clear interrupts
        self.registers.intclr.set(0xFFFF_FFFF);

        // Trigger CS
        self.spi_cs.map(|cs| cs.clear());

        // Start the transfer
        self.registers.cmd.write(
            CMD::TSIZE.val(write_len as u32)
                + CMD::CMDSEL.val(1)
                + CMD::CONT::CLEAR
                + CMD::CMD::WRITE
                + CMD::OFFSETCNT.val(0_u32)
                + CMD::OFFSETLO.val(0),
        );

        if let Some(buf) = read_buffer {
            self.spi_read_buffer.replace(buf);
        }

        while self.registers.cmdstat.read(CMDSTAT::CMDSTAT) == 0x02 {}

        let mut transfered_bytes = 0;

        // While there is some free space in FIFO0 (writing to the SPI bus) and
        // at least 4 bytes free in FIFO1 (reading from the SPI bus to the
        // hardware FIFO) we write up to 24 bytes of data.
        //
        // The `> 4` really could be `>= 4` but > gives us a little wiggle room
        // as the hardware does seem a little slow at updating the FIFO size
        // registers.
        //
        // The 24 byte limit is along the same lines, of just making sure we
        // don't write too much data. I don't have a good answer of why it should
        // be 24, but that seems to work reliably from testing.
        //
        // There isn't a specific errata for this issue, but the official HAL
        // uses DMA so there aren't a lot of FIFO users for large transfers like
        // this.
        while self.registers.fifoptr.read(FIFOPTR::FIFO0REM) > 0
            && self.registers.fifoptr.read(FIFOPTR::FIFO1REM) > 4
            && self.write_index.get() < write_len
            && transfered_bytes < 24
        {
            let idx = self.write_index.get();
            let data =
                u32::from_le_bytes(write_buffer[idx..(idx + 4)].try_into().unwrap_or([0; 4]));

            self.registers.fifopush.set(data);
            self.write_index.set(idx + 4);
            transfered_bytes += 4;

            if let Some(buf) = self.spi_read_buffer.take() {
                if self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0
                    && self.read_index.get() < read_len
                {
                    let d = self.registers.fifopop.get().to_ne_bytes();

                    let data_idx = self.read_index.get();

                    buf[data_idx + 0] = d[0];
                    buf[data_idx + 1] = d[1];
                    buf[data_idx + 2] = d[2];
                    buf[data_idx + 3] = d[3];

                    self.read_index.set(data_idx + 4);
                }

                self.spi_read_buffer.replace(buf);
            } else {
                if self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0 {
                    let _d = self.registers.fifopop.get();
                }
            }
        }

        // Save all the data and offsets we still need to send
        self.buffer.replace(write_buffer);
        self.write_len.set(write_len);
        self.read_len.set(read_len);
        self.op.set(Operation::SPI);

        if read_len > self.read_index.get() {
            let remaining_bytes = (read_len - self.read_index.get()).min(32);
            self.registers
                .fifothr
                .modify(FIFOTHR::FIFORTHR.val(remaining_bytes as u32));
        } else {
            self.registers.fifothr.modify(FIFOTHR::FIFORTHR.val(0));
        }

        if write_len > self.write_index.get() {
            let remaining_bytes = (self.write_len.get() - self.write_index.get()).min(32);

            self.registers
                .fifothr
                .modify(FIFOTHR::FIFOWTHR.val(remaining_bytes as u32));
        } else {
            self.registers.fifothr.modify(FIFOTHR::FIFOWTHR.val(0));
        }

        // Enable interrupts
        self.registers.inten.set(0xFFFF_FFFF);

        Ok(())
    }

    fn write_byte(&self, val: u8) -> Result<(), ErrorCode> {
        let burst_len = 1;

        // Disable DMA as we don't support it
        self.registers.dmacfg.write(DMACFG::DMAEN::CLEAR);

        // Set the DCX
        self.registers.dcx.set(0);

        // Clear interrupts
        self.registers.intclr.set(0xFFFF_FFFF);

        // Trigger CS
        self.spi_cs.map(|cs| cs.clear());

        // Start the transfer
        self.registers.cmd.write(
            CMD::TSIZE.val(burst_len as u32)
                + CMD::CMDSEL.val(1)
                + CMD::CONT::CLEAR
                + CMD::CMD::WRITE
                + CMD::OFFSETCNT.val(0_u32)
                + CMD::OFFSETLO.val(0),
        );

        self.registers.fifopush.set(val as u32);

        self.spi_cs.map(|cs| cs.set());

        Ok(())
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        let burst_len = 1;

        // Disable DMA as we don't support it
        self.registers.dmacfg.write(DMACFG::DMAEN::CLEAR);

        // Set the DCX
        self.registers.dcx.set(0);

        // Clear interrupts
        self.registers.intclr.set(0xFFFF_FFFF);

        // Trigger CS
        self.spi_cs.map(|cs| cs.clear());

        // Start the transfer
        self.registers.cmd.write(
            CMD::TSIZE.val(burst_len as u32)
                + CMD::CMDSEL.val(1)
                + CMD::CONT::CLEAR
                + CMD::CMD::READ
                + CMD::OFFSETCNT.val(0_u32)
                + CMD::OFFSETLO.val(0),
        );

        if self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0 {
            let d = self.registers.fifopop.get().to_ne_bytes();

            self.spi_cs.map(|cs| cs.set());
            return Ok(d[0]);
        }

        self.spi_cs.map(|cs| cs.set());

        Err(ErrorCode::FAIL)
    }

    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        let burst_len = 1;

        // Disable DMA as we don't support it
        self.registers.dmacfg.write(DMACFG::DMAEN::CLEAR);

        // Set the DCX
        self.registers.dcx.set(0);

        // Clear interrupts
        self.registers.intclr.set(0xFFFF_FFFF);

        // Trigger CS
        self.spi_cs.map(|cs| cs.clear());

        // Start the transfer
        self.registers.cmd.write(
            CMD::TSIZE.val(burst_len as u32)
                + CMD::CMDSEL.val(1)
                + CMD::CONT::CLEAR
                + CMD::CMD::WRITE
                + CMD::OFFSETCNT.val(0_u32)
                + CMD::OFFSETLO.val(0),
        );

        self.registers.fifopush.set(val as u32);

        if self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0 {
            let d = self.registers.fifopop.get().to_ne_bytes();

            self.spi_cs.map(|cs| cs.set());
            return Ok(d[0]);
        }

        self.spi_cs.map(|cs| cs.set());

        Err(ErrorCode::FAIL)
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        cs.make_output();
        cs.set();
        self.spi_cs.set(cs);

        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        if self.op.get() != Operation::SPI && self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        let div: u32 = 48000000 / rate; // TODO: Change to `48000000_u32.div_ceil(rate)` when api out of nightly
        let n = div.trailing_zeros().max(6);

        let div3 = u32::from(
            (rate < (48000000 / 16384))
                || ((rate >= (48000000 / 3)) && (rate <= ((48000000 / 2) - 1))),
        );
        let denom = (1 << n) * (1 + (div3 * 2));
        let tot_per = if div % denom > 0 {
            (div / denom) + 1
        } else {
            div / denom
        };
        let v1 = 31 - tot_per.leading_zeros();
        let fsel = if v1 > 7 { v1 + n - 6 } else { n + 1 };

        if fsel > 7 {
            return Err(ErrorCode::NOSUPPORT);
        }

        let diven = u32::from((rate >= (48000000 / 4)) || ((1 << (fsel - 1)) == div));
        let low_per = if self.spi_phase.get() == ClockPhase::SampleLeading {
            (tot_per - 1) / 2
        } else {
            (tot_per - 2) / 2
        };

        self.registers.clkcfg.write(
            CLKCFG::TOTPER.val(tot_per - 1)
                + CLKCFG::LOWPER.val(low_per)
                + CLKCFG::DIVEN.val(diven)
                + CLKCFG::DIV3.val(div3)
                + CLKCFG::FSEL.val(fsel)
                + CLKCFG::IOCLKEN::SET,
        );

        Ok(self.get_rate())
    }

    fn get_rate(&self) -> u32 {
        let fsel = self.registers.clkcfg.read(CLKCFG::FSEL);
        let div3 = self.registers.clkcfg.read(CLKCFG::DIV3);
        let diven = self.registers.clkcfg.read(CLKCFG::DIVEN);
        let tot_per = self.registers.clkcfg.read(CLKCFG::TOTPER) + 1;

        let denom_final = (1 << (fsel - 1)) * (1 + div3 * 2) * (1 + diven * (tot_per));

        if ((48000000) % denom_final) > (denom_final / 2) {
            (48000000 / denom_final) + 1
        } else {
            48000000 / denom_final
        }
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        if self.op.get() != Operation::SPI && self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        if polarity == ClockPolarity::IdleLow {
            self.registers.mspicfg.modify(MSPICFG::SPOL::CLEAR);
        } else {
            self.registers.mspicfg.modify(MSPICFG::SPOL::SET);
        }

        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        if self.registers.mspicfg.is_set(MSPICFG::SPOL) {
            ClockPolarity::IdleHigh
        } else {
            ClockPolarity::IdleLow
        }
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        if self.op.get() != Operation::SPI && self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        let low_per = if self.spi_phase.get() == ClockPhase::SampleLeading {
            (self.registers.clkcfg.read(CLKCFG::LOWPER) * 2) + 1
        } else {
            (self.registers.clkcfg.read(CLKCFG::LOWPER) * 2) + 2
        };

        if phase == ClockPhase::SampleLeading {
            self.registers
                .clkcfg
                .modify(CLKCFG::LOWPER.val((low_per - 1) / 2));
        } else {
            self.registers
                .clkcfg
                .modify(CLKCFG::LOWPER.val((low_per - 2) / 2));
        }

        self.spi_phase.set(phase);

        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        self.spi_phase.get()
    }

    fn hold_low(&self) {
        self.spi_cs.map(|cs| cs.clear());
    }

    fn release_low(&self) {
        self.spi_cs.map(|cs| cs.set());
    }
}
