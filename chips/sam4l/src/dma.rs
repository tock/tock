//! Implementation of the PDCA DMA peripheral.

use crate::pm;
use core::cell::Cell;
use core::{cmp, intrinsics};
use kernel::common::cells::VolatileCell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

/// Memory registers for a DMA channel. Section 16.6.1 of the datasheet.
#[repr(C)]
#[allow(dead_code)]
struct DMARegisters {
    mar: ReadWrite<u32, MemoryAddress::Register>,
    psr: VolatileCell<DMAPeripheral>,
    _psr_padding: [u8; 3],
    tcr: ReadWrite<u32, TransferCounter::Register>,
    marr: ReadWrite<u32, MemoryAddressReload::Register>,
    tcrr: ReadWrite<u32, TransferCounter::Register>,
    cr: WriteOnly<u32, Control::Register>,
    mr: ReadWrite<u32, Mode::Register>,
    sr: ReadOnly<u32, Status::Register>,
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
}

register_bitfields![u32,
    MemoryAddress [
        MADDR OFFSET(0) NUMBITS(32) []
    ],

    MemoryAddressReload [
        MARV OFFSET(0) NUMBITS(32) []
    ],

    TransferCounter [
        /// Transfer Counter Value
        TCV OFFSET(0) NUMBITS(16) []
    ],

    Control [
        /// Transfer Error Clear
        ECLR 8,
        /// Transfer Disable
        TDIS 1,
        /// Transfer Enable
        TEN 0
    ],

    Mode [
        /// Ring Buffer
        RING OFFSET(3) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        /// Event Trigger
        ETRIG OFFSET(2) NUMBITS(1) [
            StartOnRequest = 0,
            StartOnEvent = 1
        ],
        /// Size of Transfer
        SIZE OFFSET(0) NUMBITS(2) [
            Byte = 0,
            Halfword = 1,
            Word = 2
        ]
    ],

    Status [
        /// Transfer Enabled
        TEN 0
    ],

    Interrupt [
        /// Transfer Error
        TERR 2,
        /// Transfer Complete
        TRC 1,
        /// Reload Counter Zero
        RCZ 0
    ]
];

/// The PDCA's base addresses in memory (Section 7.1 of manual).
const DMA_BASE_ADDR: usize = 0x400A2000;

/// The number of bytes between each memory mapped DMA Channel (Section 16.6.1).
const DMA_CHANNEL_SIZE: usize = 0x40;

/// Shared counter that Keeps track of how many DMA channels are currently
/// active.
static mut NUM_ENABLED: usize = 0;

/// The DMA channel number. Each channel transfers data between memory and a
/// particular peripheral function (e.g., SPI read or SPI write, but not both
/// simultaneously). There are 16 available channels (Section 16.7).
#[derive(Copy, Clone)]
pub enum DMAChannelNum {
    // Relies on the fact that assigns values 0-15 to each constructor in order
    DMAChannel00 = 0,
    DMAChannel01 = 1,
    DMAChannel02 = 2,
    DMAChannel03 = 3,
    DMAChannel04 = 4,
    DMAChannel05 = 5,
    DMAChannel06 = 6,
    DMAChannel07 = 7,
    DMAChannel08 = 8,
    DMAChannel09 = 9,
    DMAChannel10 = 10,
    DMAChannel11 = 11,
    DMAChannel12 = 12,
    DMAChannel13 = 13,
    DMAChannel14 = 14,
    DMAChannel15 = 15,
}

/// The peripheral function a channel is assigned to (Section 16.7). `*_RX`
/// means transfer data from peripheral to memory, `*_TX` means transfer data
/// from memory to peripheral.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DMAPeripheral {
    USART0_RX = 0,
    USART1_RX = 1,
    USART2_RX = 2,
    USART3_RX = 3,
    SPI_RX = 4,
    TWIM0_RX = 5,
    TWIM1_RX = 6,
    TWIM2_RX = 7,
    TWIM3_RX = 8,
    TWIS0_RX = 9,
    TWIS1_RX = 10,
    ADCIFE_RX = 11,
    CATB_RX = 12,
    IISC_CH0_RX = 14,
    IISC_CH1_RX = 15,
    PARC_RX = 16,
    AESA_RX = 17,
    USART0_TX = 18,
    USART1_TX = 19,
    USART2_TX = 20,
    USART3_TX = 21,
    SPI_TX = 22,
    TWIM0_TX = 23,
    TWIM1_TX = 24,
    TWIM2_TX = 25,
    TWIM3_TX = 26,
    TWIS0_TX = 27,
    TWIS1_TX = 28,
    ADCIFE_TX = 29,
    CATB_TX = 30,
    ABDACB_SDR0_TX = 31,
    ABDACB_SDR1_TX = 32,
    IISC_CH0_TX = 33,
    IISC_CH1_TX = 34,
    DACC_TX = 35,
    AESA_TX = 36,
    LCDCA_ACMDR_TX = 37,
    LCDCA_ABMDR_TX = 38,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum DMAWidth {
    Width8Bit = 0,
    Width16Bit = 1,
    Width32Bit = 2,
}

pub static mut DMA_CHANNELS: [DMAChannel; 16] = [
    DMAChannel::new(DMAChannelNum::DMAChannel00),
    DMAChannel::new(DMAChannelNum::DMAChannel01),
    DMAChannel::new(DMAChannelNum::DMAChannel02),
    DMAChannel::new(DMAChannelNum::DMAChannel03),
    DMAChannel::new(DMAChannelNum::DMAChannel04),
    DMAChannel::new(DMAChannelNum::DMAChannel05),
    DMAChannel::new(DMAChannelNum::DMAChannel06),
    DMAChannel::new(DMAChannelNum::DMAChannel07),
    DMAChannel::new(DMAChannelNum::DMAChannel08),
    DMAChannel::new(DMAChannelNum::DMAChannel09),
    DMAChannel::new(DMAChannelNum::DMAChannel10),
    DMAChannel::new(DMAChannelNum::DMAChannel11),
    DMAChannel::new(DMAChannelNum::DMAChannel12),
    DMAChannel::new(DMAChannelNum::DMAChannel13),
    DMAChannel::new(DMAChannelNum::DMAChannel14),
    DMAChannel::new(DMAChannelNum::DMAChannel15),
];

pub struct DMAChannel {
    registers: StaticRef<DMARegisters>,
    client: OptionalCell<&'static dyn DMAClient>,
    width: Cell<DMAWidth>,
    enabled: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
}

pub trait DMAClient {
    fn transfer_done(&self, pid: DMAPeripheral);
}

impl DMAChannel {
    const fn new(channel: DMAChannelNum) -> DMAChannel {
        DMAChannel {
            registers: unsafe {
                StaticRef::new(
                    (DMA_BASE_ADDR + (channel as usize) * DMA_CHANNEL_SIZE) as *const DMARegisters,
                )
            },
            client: OptionalCell::empty(),
            width: Cell::new(DMAWidth::Width8Bit),
            enabled: Cell::new(false),
            buffer: TakeCell::empty(),
        }
    }

    pub fn initialize(&self, client: &'static mut dyn DMAClient, width: DMAWidth) {
        self.client.set(client);
        self.width.set(width);
    }

    pub fn enable(&self) {
        pm::enable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
        pm::enable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));

        if !self.enabled.get() {
            unsafe {
                let num_enabled = intrinsics::atomic_xadd(&mut NUM_ENABLED, 1);
                if num_enabled == 1 {
                    pm::enable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
                    pm::enable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
                }
            }
            let registers: &DMARegisters = &*self.registers;
            // Disable all interrupts
            registers
                .idr
                .write(Interrupt::TERR::SET + Interrupt::TRC::SET + Interrupt::RCZ::SET);

            self.enabled.set(true);
        }
    }

    pub fn disable(&self) {
        if self.enabled.get() {
            unsafe {
                let num_enabled = intrinsics::atomic_xsub(&mut NUM_ENABLED, 1);
                if num_enabled == 1 {
                    pm::disable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
                    pm::disable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
                }
            }
            let registers: &DMARegisters = &*self.registers;
            registers.cr.write(Control::TDIS::SET);
            self.enabled.set(false);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn handle_interrupt(&mut self) {
        let registers: &DMARegisters = &*self.registers;
        registers
            .idr
            .write(Interrupt::TERR::SET + Interrupt::TRC::SET + Interrupt::RCZ::SET);
        let channel = registers.psr.get();

        self.client.map(|client| {
            client.transfer_done(channel);
        });
    }

    pub fn start_transfer(&self) {
        let registers: &DMARegisters = &*self.registers;
        registers.cr.write(Control::TEN::SET);
    }

    pub fn prepare_transfer(&self, pid: DMAPeripheral, buf: &'static mut [u8], mut len: usize) {
        // TODO(alevy): take care of zero length case

        let registers: &DMARegisters = &*self.registers;

        let maxlen = buf.len()
            / match self.width.get() {
                DMAWidth::Width8Bit /*  DMA is acting on bytes     */ => 1,
                DMAWidth::Width16Bit /* DMA is acting on halfwords */ => 2,
                DMAWidth::Width32Bit /* DMA is acting on words     */ => 4,
            };
        len = cmp::min(len, maxlen);
        registers.mr.write(Mode::SIZE.val(self.width.get() as u32));

        registers.psr.set(pid);
        registers
            .marr
            .write(MemoryAddressReload::MARV.val(&buf[0] as *const u8 as u32));
        registers.tcrr.write(TransferCounter::TCV.val(len as u32));

        registers.ier.write(Interrupt::TRC::SET);

        // Store the buffer reference in the TakeCell so it can be returned to
        // the caller in `handle_interrupt`
        self.buffer.replace(buf);
    }

    pub fn do_transfer(&self, pid: DMAPeripheral, buf: &'static mut [u8], len: usize) {
        self.prepare_transfer(pid, buf, len);
        self.start_transfer();
    }

    /// Aborts any current transactions and returns the buffer used in the
    /// transaction.
    pub fn abort_transfer(&self) -> Option<&'static mut [u8]> {
        let registers: &DMARegisters = &*self.registers;
        registers
            .idr
            .write(Interrupt::TERR::SET + Interrupt::TRC::SET + Interrupt::RCZ::SET);

        // Reset counter
        registers.tcr.write(TransferCounter::TCV.val(0));

        self.buffer.take()
    }

    pub fn transfer_counter(&self) -> usize {
        let registers: &DMARegisters = &*self.registers;
        registers.tcr.read(TransferCounter::TCV) as usize
    }
}
