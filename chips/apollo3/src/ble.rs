//! BLE driver.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;

const BLE_BASE: StaticRef<BleRegisters> =
    unsafe { StaticRef::new(0x5000_C000 as *const BleRegisters) };

register_structs! {
    pub BleRegisters {
        (0x000 => fifo: ReadWrite<u32, FIFO::Register>),
        (0x004 => _reserved0),
        (0x100 => fifoptr: ReadOnly<u32, FIFOPTR::Register>),
        (0x104 => fifothr: ReadWrite<u32, FIFOTHR::Register>),
        (0x108 => fifopop: ReadWrite<u32, FIFOPOP::Register>),
        (0x10C => fifopush: ReadWrite<u32, FIFOPUSH::Register>),
        (0x110 => fifoctrl: ReadWrite<u32, FIFOCTRL::Register>),
        (0x114 => fifoloc: ReadWrite<u32, FIFOLOC::Register>),
        (0x118 => _reserved1),
        (0x200 => clkcfg: ReadWrite<u32, CLKCFG::Register>),
        (0x204 => _reserved2),
        (0x20C => cmd: ReadWrite<u32, CMD::Register>),
        (0x210 => cmdrpt: ReadWrite<u32, CMDRPT::Register>),
        (0x214 => offsethi: ReadWrite<u32, OFFSETHI::Register>),
        (0x218 => cmdstat: ReadWrite<u32, CMDSTAT::Register>),
        (0x21C => _reserved3),
        (0x220 => inten: ReadWrite<u32, INT::Register>),
        (0x224 => intstat: ReadWrite<u32, INT::Register>),
        (0x228 => intclr: ReadWrite<u32, INT::Register>),
        (0x22C => intset: ReadWrite<u32, INT::Register>),
        (0x230 => dmatrigen: ReadWrite<u32, DMATRIGEN::Register>),
        (0x234 => dmatrigstat: ReadWrite<u32, DMATRIGSTAT::Register>),
        (0x238 => dmacfg: ReadWrite<u32, DMACFG::Register>),
        (0x23C => dmatocount: ReadWrite<u32, DMATOCOUNT::Register>),
        (0x240 => dmatargaddr: ReadWrite<u32, DMATAGADDR::Register>),
        (0x244 => dmastat: ReadWrite<u32, DMASTAT::Register>),
        (0x248 => cqcfg: ReadWrite<u32, CQCFG::Register>),
        (0x24C => cqaddr: ReadWrite<u32, CQADDR::Register>),
        (0x250 => cqstat: ReadWrite<u32, CQSTAT::Register>),
        (0x254 => cqflags: ReadWrite<u32, CQFLAGS::Register>),
        (0x258 => cqsetclear: WriteOnly<u32, CQSETCLEAR::Register>),
        (0x25C => cqpauseen: ReadWrite<u32, CQPAUSEEN::Register>),
        (0x260 => cqcuridx: ReadWrite<u32, CQCURIDX::Register>),
        (0x264 => cqendidx: ReadWrite<u32, CQENDIDX::Register>),
        (0x268 => status: ReadWrite<u32, STATUS::Register>),
        (0x26C => _reserved4),
        (0x300 => mspicfg: ReadWrite<u32, MSPICFG::Register>),
        (0x304 => blecfg: ReadWrite<u32, BLECFG::Register>),
        (0x308 => pwrcmd: ReadWrite<u32, PWRCMD::Register>),
        (0x30C => bstatus: ReadWrite<u32, BSTATUS::Register>),
        (0x310 => _reserved5),
        (0x410 => bledbg: ReadWrite<u32, BLEDBG::Register>),
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
    CLKCFG [
        IOCLKEN OFFSET(0) NUMBITS(1) [],
        FSEL OFFSET(8) NUMBITS(3) [],
        CLK32KEN OFFSET(11) NUMBITS(1) [],
        DIV3 OFFSET(12) NUMBITS(1) []
    ],
    CMD [
        CMD OFFSET(0) NUMBITS(4) [
            WRITE = 0x01,
            READ = 0x02
        ],
        OFFSETCNT OFFSET(5) NUMBITS(2) [],
        CONT OFFSET(7) NUMBITS(1) [],
        TSIZE OFFSET(8) NUMBITS(12) [],
        CMDSEL OFFSET(20) NUMBITS(2) [],
        OFFSETLO OFFSET(24) NUMBITS(8) []
    ],
    CMDRPT [
        CMDRPT OFFSET(0) NUMBITS(4) []
    ],
    OFFSETHI [
        OFFSETHI OFFSET(0) NUMBITS(16) []
    ],
    CMDSTAT [
        CCMD OFFSET(0) NUMBITS(4) [],
        CMDSTAT OFFSET(5) NUMBITS(3) [],
        CTSIZE OFFSET(8) NUMBITS(12) []
    ],
    INT [
        CMDCMP OFFSET(0) NUMBITS(1) [],
        THR OFFSET(1) NUMBITS(1) [],
        FUNDFL OFFSET(2) NUMBITS(1) [],
        FOVFL OFFSET(3) NUMBITS(1) [],
        B2MST OFFSET(4) NUMBITS(1) [],
        IACC OFFSET(5) NUMBITS(1) [],
        ICMD OFFSET(6) NUMBITS(1) [],
        BLECIRQ OFFSET(7) NUMBITS(1) [],
        BLECSSTAT OFFSET(8) NUMBITS(1) [],
        DCMP OFFSET(9) NUMBITS(1) [],
        DERR OFFSET(10) NUMBITS(1) [],
        CQPAUSED OFFSET(11) NUMBITS(1) [],
        CQUPD OFFSET(12) NUMBITS(1) [],
        CQERR OFFSET(13) NUMBITS(1) [],
        B2MSLEEP OFFSET(14) NUMBITS(1) [],
        B2MACTIVE OFFSET(15) NUMBITS(1) [],
        B2MSHUTDN OFFSET(16) NUMBITS(1) []
    ],
    DMATRIGEN [
        DCMDCMPEN OFFSET(0) NUMBITS(1) [],
        DTHREN OFFSET(1) NUMBITS(1) []
    ],
    DMATRIGSTAT [
        DCMDCMPEN OFFSET(0) NUMBITS(1) [],
        DTHREN OFFSET(1) NUMBITS(1) [],
        DTOTCMP OFFSET(2) NUMBITS(1) []
    ],
    DMACFG [
        DMAEN OFFSET(0) NUMBITS(1) [],
        DMADIR OFFSET(1) NUMBITS(1) [],
        DMAPRI OFFSET(8) NUMBITS(1) []
    ],
    DMATOCOUNT [
        TOTCOUNT OFFSET(0) NUMBITS(12) []
    ],
    DMATAGADDR [
        TARGADDR OFFSET(0) NUMBITS(20) [],
        TARGADDR28 OFFSET(28) NUMBITS(1) []
    ],
    DMASTAT [
        DMATIP OFFSET(0) NUMBITS(1) [],
        DMACPL OFFSET(1) NUMBITS(1) [],
        DMAERR OFFSET(2) NUMBITS(1) []
    ],
    CQCFG [
        CQEN OFFSET(0) NUMBITS(1) [],
        CQPRI OFFSET(1) NUMBITS(1) []
    ],
    CQADDR [
        CQADDR OFFSET(2) NUMBITS(18) [],
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
        CMDACT OFFSET(1) NUMBITS(1) [],
        ISLEST OFFSET(2) NUMBITS(1) []
    ],
    MSPICFG [
        SPOL OFFSET(0) NUMBITS(1) [],
        SPHA OFFSET(1) NUMBITS(1) [],
        FULLDUP OFFSET(2) NUMBITS(1) [],
        WTFC OFFSET(16) NUMBITS(1) [],
        RDFC OFFSET(17) NUMBITS(1) [],
        WTFCPOL OFFSET(21) NUMBITS(1) [],
        RDFCPOL OFFSET(22) NUMBITS(1) [],
        SPILSB OFFSET(23) NUMBITS(1) [],
        DINDLY OFFSET(24) NUMBITS(2) [],
        DOUTDLLY OFFSET(27) NUMBITS(2) [],
        MSPIRST OFFSET(30) NUMBITS(1) []
    ],
    BLECFG [
        PWRSMEN OFFSET(0) NUMBITS(1) [],
        BLERSTN OFFSET(1) NUMBITS(1) [],
        WAKEUPCTL OFFSET(2) NUMBITS(2) [
            ON = 0x3,
            OFF = 0x2,
            AUTO = 0x0
        ],
        DCDCFLGCTL OFFSET(4) NUMBITS(2) [],
        BLEHREQCTL OFFSET(6) NUMBITS(2) [],
        WT4ACTOFF OFFSET(8) NUMBITS(1) [],
        MCUFRCSLP OFFSET(9) NUMBITS(1) [],
        FRCCLK OFFSET(10) NUMBITS(1) [],
        STAYASLEEP OFFSET(11) NUMBITS(1) [],
        PWRISOCTL OFFSET(12) NUMBITS(2) [],
        SPIISOCTL OFFSET(14) NUMBITS(2) []
    ],
    PWRCMD [
        WAKEREQ OFFSET(0) NUMBITS(1) [],
        RESTART OFFSET(1) NUMBITS(1) []
    ],
    BSTATUS [
        B2MSTATE OFFSET(0) NUMBITS(2) [],
        SPISTATUS OFFSET(3) NUMBITS(1) [],
        DCDCREQ OFFSET(4) NUMBITS(1) [],
        DCDCFLAG OFFSET(5) NUMBITS(1) [],
        WAKEUP OFFSET(6) NUMBITS(1) [],
        BLEIRQ OFFSET(7) NUMBITS(1) [],
        PWRST OFFSET(8) NUMBITS(2) [],
        BLEHACK OFFSET(11) NUMBITS(1) [],
        BLEHREQ OFFSET(12) NUMBITS(1) []
    ],
    BLEDBG [
        DBGEN OFFSET(0) NUMBITS(1) [],
        IOCLKON OFFSET(1) NUMBITS(1) [],
        APBCLKON OFFSET(2) NUMBITS(1) [],
        DBGDATA OFFSET(3) NUMBITS(29) []
    ]
];

static mut PAYLOAD: [u8; 40] = [0x00; 40];

pub struct Ble<'a> {
    registers: StaticRef<BleRegisters>,
    rx_client: OptionalCell<&'a dyn ble_advertising::RxClient>,
    tx_client: OptionalCell<&'a dyn ble_advertising::TxClient>,

    buffer: TakeCell<'static, [u8]>,
    write_len: Cell<usize>,

    read_len: Cell<usize>,
    read_index: Cell<usize>,
}

impl<'a> Ble<'a> {
    pub const fn new() -> Self {
        Self {
            registers: BLE_BASE,
            rx_client: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            write_len: Cell::new(0),
            read_len: Cell::new(0),
            read_index: Cell::new(0),
        }
    }

    pub fn setup_clocks(&self) {
        self.registers.clkcfg.write(CLKCFG::CLK32KEN::SET);
        self.registers.bledbg.write(BLEDBG::DBGDATA.val(1 << 14));
    }

    pub fn power_up(&self) {
        self.registers.blecfg.write(BLECFG::PWRSMEN::SET);

        while self.registers.bstatus.read(BSTATUS::PWRST) != 3 {}
    }

    pub fn ble_initialise(&self) {
        // Configure the SPI
        self.registers.mspicfg.write(
            MSPICFG::SPOL::SET
                + MSPICFG::SPHA::SET
                + MSPICFG::RDFC::CLEAR
                + MSPICFG::WTFC::CLEAR
                + MSPICFG::WTFCPOL::SET,
        );
        self.registers
            .fifothr
            .write(FIFOTHR::FIFOWTHR.val(16) + FIFOTHR::FIFORTHR.val(16));
        self.registers.fifoctrl.modify(FIFOCTRL::POPWR::SET);

        // Clock config
        self.registers
            .clkcfg
            .write(CLKCFG::FSEL.val(0x04) + CLKCFG::IOCLKEN::SET + CLKCFG::CLK32KEN::SET);

        // Disable command queue
        self.registers.cqcfg.modify(CQCFG::CQEN::CLEAR);

        // TODO: Apply the BLE patch
    }

    fn reset_fifo(&self) {
        self.registers.fifoctrl.modify(FIFOCTRL::FIFORSTN::CLEAR);
        self.registers.fifoctrl.modify(FIFOCTRL::FIFORSTN::SET);
    }

    fn send_data(&self) {
        // Set the FIFO levels
        self.registers
            .fifothr
            .write(FIFOTHR::FIFORTHR.val(16) + FIFOTHR::FIFOWTHR.val(16));

        // Disable the FIFO
        //self.registers.fifothr.write(FIFOTHR::FIFORTHR::CLEAR + FIFOTHR::FIFOWTHR::CLEAR);

        // Setup the DMA
        unsafe {
            self.registers.dmatargaddr.set(PAYLOAD.as_ptr() as u32);
        }
        self.registers.dmatocount.set(self.write_len.get() as u32);
        self.registers.dmatrigen.write(DMATRIGEN::DTHREN::SET);
        self.registers
            .dmacfg
            .write(DMACFG::DMADIR.val(1) + DMACFG::DMAPRI.val(1));

        // Setup the operation
        self.registers
            .cmd
            .modify(CMD::TSIZE.val(self.write_len.get() as u32) + CMD::CMD::WRITE);

        // Enable DMA
        self.registers.dmacfg.modify(DMACFG::DMAEN::SET);

        // Set the wake low
        self.registers.blecfg.modify(BLECFG::WAKEUPCTL::OFF);
    }

    pub fn handle_interrupt(&self) {
        let irqs = self.registers.intstat.extract();

        // Disable and clear interrupts
        self.disable_interrupts();

        if irqs.is_set(INT::BLECSSTAT) || irqs.is_set(INT::B2MST) {
            // Enable interrupts
            self.enable_interrupts();

            if self.registers.bstatus.is_set(BSTATUS::BLEIRQ) {
                panic!("Read requested while trying to write");
            }

            if !self.registers.bstatus.is_set(BSTATUS::SPISTATUS) {
                panic!("SPI not ready");
            }

            // If we have data, send it
            if self.buffer.is_some() {
                // Send the data
                self.send_data();
            }
        }

        if irqs.is_set(INT::DCMP) {
            // Disable and clear DMA
            self.registers.dmacfg.set(0x00000000);

            // Disable the wake controller
            self.registers.blecfg.modify(BLECFG::WAKEUPCTL::OFF);

            // Reset FIFOs
            self.reset_fifo();

            if self.buffer.is_some() {
                self.tx_client.map(|client| {
                    client.transmit_event(self.buffer.take().unwrap(), kernel::ReturnCode::SUCCESS);
                });
            }

            self.enable_interrupts();
        }

        if irqs.is_set(INT::BLECIRQ) {
            self.rx_client.map(|client| {
                self.registers
                    .cmd
                    .modify(CMD::TSIZE.val(0) + CMD::CMD::READ);

                unsafe {
                    let mut i = 0;

                    while self.registers.fifoptr.read(FIFOPTR::FIFO1SIZ) > 0 && i < 40 {
                        let temp = self.registers.fifopop.get().to_ne_bytes();

                        PAYLOAD[i + 0] = temp[0];
                        PAYLOAD[i + 1] = temp[1];
                        PAYLOAD[i + 2] = temp[2];
                        PAYLOAD[i + 3] = temp[3];

                        i = i + 4;
                    }

                    client.receive_event(&mut PAYLOAD, 10, kernel::ReturnCode::SUCCESS);
                }
            });
        }
    }

    pub fn enable_interrupts(&self) {
        self.registers.inten.set(0x18381);
    }

    pub fn disable_interrupts(&self) {
        self.registers.intclr.set(0xFFFF_FFFF);
        self.registers.inten.set(0x00);
    }

    fn replace_radio_buffer(&self, buf: &'static mut [u8]) -> &'static mut [u8] {
        // set payload
        for (i, c) in buf.as_ref().iter().enumerate() {
            unsafe {
                PAYLOAD[i] = *c;
            }
        }
        buf
    }
}

impl<'a> ble_advertising::BleAdvertisementDriver<'a> for Ble<'a> {
    fn transmit_advertisement(&self, buf: &'static mut [u8], len: usize, _channel: RadioChannel) {
        let res = self.replace_radio_buffer(buf);

        // Setup all of the buffers
        self.buffer.replace(res);
        self.write_len.set(len as usize);
        self.read_len.set(0);
        self.read_index.set(0);

        // Enable interrupts
        self.enable_interrupts();

        // Wakeup BLE
        self.registers.blecfg.modify(BLECFG::WAKEUPCTL::ON);

        // See if we can send the data
        if self.registers.bstatus.is_set(BSTATUS::SPISTATUS) {
            self.send_data();
        }
    }

    fn receive_advertisement(&self, _channel: RadioChannel) {
        unimplemented!();
    }

    fn set_receive_client(&self, client: &'a dyn ble_advertising::RxClient) {
        self.rx_client.set(client);
    }

    fn set_transmit_client(&self, client: &'a dyn ble_advertising::TxClient) {
        self.tx_client.set(client);
    }
}

impl ble_advertising::BleConfig for Ble<'_> {
    fn set_tx_power(&self, _tx_power: u8) -> kernel::ReturnCode {
        kernel::ReturnCode::SUCCESS
    }
}
