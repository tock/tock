//! IO Master Driver (I2C)

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::i2c;

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

pub struct Iom<'a> {
    registers: StaticRef<IomRegisters>,

    master_client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,

    buffer: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,
    write_index: Cell<usize>,

    read_len: Cell<usize>,
    read_index: Cell<usize>,

    smbus: Cell<bool>,
}

impl<'a> Iom<'_> {
    pub const fn new0() -> Iom<'a> {
        Iom {
            registers: IOM0_BASE,
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            smbus: Cell::new(false),
        }
    }
    pub const fn new1() -> Iom<'a> {
        Iom {
            registers: IOM1_BASE,
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            smbus: Cell::new(false),
        }
    }
    pub const fn new2() -> Iom<'a> {
        Iom {
            registers: IOM2_BASE,
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            smbus: Cell::new(false),
        }
    }
    pub const fn new3() -> Iom<'a> {
        Iom {
            registers: IOM3_BASE,
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            smbus: Cell::new(false),
        }
    }
    pub const fn new4() -> Iom<'a> {
        Iom {
            registers: IOM4_BASE,
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            smbus: Cell::new(false),
        }
    }
    pub const fn new5() -> Iom<'a> {
        Iom {
            registers: IOM5_BASE,
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            write_index: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
            smbus: Cell::new(false),
        }
    }

    fn reset_fifo(&self) {
        let regs = self.registers;

        regs.fifoctrl.modify(FIFOCTRL::FIFORSTN::CLEAR);
        regs.fifoctrl.modify(FIFOCTRL::FIFORSTN::SET);
    }

    fn write_data(&self) {
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
                    self.write_index.set(data_pushed as usize);
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
                    let d = buf[len as usize - 1] as u32;

                    regs.fifopush.set(d);
                } else if len % 4 == 2 {
                    let mut d = (buf[len as usize - 1] as u32) << 8;
                    d |= (buf[len as usize - 2] as u32) << 0;

                    regs.fifopush.set(d);
                } else if len % 4 == 3 {
                    let mut d = (buf[len as usize - 1] as u32) << 16;
                    d |= (buf[len as usize - 2] as u32) << 8;
                    d |= (buf[len as usize - 3] as u32) << 0;

                    regs.fifopush.set(d);
                }
                self.write_index.set(len as usize);
            }
        });
    }

    fn read_data(&self) {
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
                    self.read_index.set(data_popped as usize);
                    break;
                }

                let d = regs.fifopop.get().to_ne_bytes();

                buf[data_idx + 0] = d[0];
                buf[data_idx + 1] = d[1];
                buf[data_idx + 2] = d[2];
                buf[data_idx + 3] = d[3];

                data_popped = data_idx + 4;
            }

            // Get an remaining data that isn't 4 bytes long
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
                self.read_index.set(len as usize);
            }
        });
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irqs = regs.intstat.extract();

        // Clear interrrupts
        regs.intclr.set(0xFFFF_FFFF);

        if irqs.is_set(INT::CMDCMP) || irqs.is_set(INT::THR) {
            // Enable interrupts
            regs.inten.set(0xFFFF_FFFF);

            if regs.fifothr.read(FIFOTHR::FIFOWTHR) > 0 {
                let remaining = self.write_len.get() - self.write_index.get();

                if remaining > 4 {
                    regs.fifothr.write(
                        FIFOTHR::FIFORTHR.val(0) + FIFOTHR::FIFOWTHR.val(remaining as u32 / 2),
                    );
                } else {
                    regs.fifothr
                        .write(FIFOTHR::FIFORTHR.val(0) + FIFOTHR::FIFOWTHR.val(1));
                }

                self.write_data();
            } else if regs.fifothr.read(FIFOTHR::FIFORTHR) > 0 {
                let remaining = self.read_len.get() - self.read_index.get();

                if remaining > 4 {
                    regs.fifothr.write(
                        FIFOTHR::FIFORTHR.val(remaining as u32 / 2) + FIFOTHR::FIFOWTHR.val(0),
                    );
                } else {
                    regs.fifothr
                        .write(FIFOTHR::FIFORTHR.val(1) + FIFOTHR::FIFOWTHR.val(0));
                }

                self.read_data();
            }
        }

        if irqs.is_set(INT::CMDCMP) {
            if (self.read_len.get() > 0 && self.read_index.get() == self.read_len.get())
                || (self.write_len.get() > 0 && self.write_index.get() == self.write_len.get())
            {
                self.master_client.map(|client| {
                    client.command_complete(
                        self.buffer.take().unwrap(),
                        hil::i2c::Error::CommandComplete,
                    );
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

                    return;
                }
            }
        }
    }

    fn tx_rx(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
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

        self.reset_fifo();

        if write_len > 0 {
            offsetlo = data[0] as u32;
        }

        if write_len == 2 {
            regs.offsethi.set(data[1] as u32);
        } else if write_len == 3 {
            regs.offsethi.set(data[1] as u32 | ((data[2] as u32) << 8));
        }

        // Save all the data and offsets we still need to send
        self.buffer.replace(data);
        self.write_len.set(write_len as usize);
        self.read_len.set(read_len as usize);
        self.write_index.set(0);
        self.read_index.set(0);

        if write_len > 3 {
            // We can't suppord that much data, bail out now
            self.master_client.map(|client| {
                client.command_complete(self.buffer.take().unwrap(), hil::i2c::Error::NotSupported);
            });
            return;
        }

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

        self.read_data();
    }

    fn tx(&self, addr: u8, data: &'static mut [u8], len: u8) {
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

        self.reset_fifo();

        // Save all the data and offsets we still need to send
        self.buffer.replace(data);
        self.write_len.set(len as usize);
        self.read_len.set(0);
        self.write_index.set(0);

        self.write_data();

        // Clear and enable interrupts
        regs.intclr.set(0xFFFF_FFFF);
        regs.inten.set(0xFFFF_FFFF);

        // Start the transfer
        regs.cmd
            .write(CMD::TSIZE.val(len as u32) + CMD::CMD::WRITE + CMD::CONT::CLEAR);
    }

    fn rx(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
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

        self.reset_fifo();

        // Clear and enable interrupts
        regs.intclr.set(0xFFFF_FFFF);
        regs.inten.set(0xFFFF_FFFF);

        // Start the transfer
        regs.cmd
            .write(CMD::TSIZE.val(len as u32) + CMD::CMD::READ + CMD::CONT::CLEAR);

        // Save all the data and offsets we still need to send
        self.buffer.replace(buffer);
        self.read_len.set(len as usize);
        self.write_len.set(0);
        self.read_index.set(0);

        self.read_data();
    }
}

impl<'a> hil::i2c::I2CMaster for Iom<'a> {
    fn set_master_client(&self, master_client: &'a dyn i2c::I2CHwMasterClient) {
        self.master_client.set(master_client);
    }

    fn enable(&self) {
        let regs = self.registers;

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

        regs.submodctrl.write(SUBMODCTRL::SMOD1EN::CLEAR);
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        self.tx_rx(addr, data, write_len, read_len);
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        self.tx(addr, data, len);
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        self.rx(addr, buffer, len);
    }
}

impl<'a> hil::i2c::SMBusMaster for Iom<'a> {
    fn smbus_write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: u8,
        read_len: u8,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

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

        self.tx_rx(addr, data, write_len, read_len);
        Ok(())
    }

    fn smbus_write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: u8,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

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

        self.tx(addr, data, len);
        Ok(())
    }

    fn smbus_read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: u8,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let regs = self.registers;

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

        self.rx(addr, buffer, len);
        Ok(())
    }
}
