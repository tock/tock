// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use crate::pio;
use core::cell::Cell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, FieldValue, LocalRegisterCopy, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
        pub ChannelRegisters {
                (0x000 => read_addr: ReadWrite<u32, READ_ADDR::Register>),
                (0x004 => write_addr: ReadWrite<u32, WRITE_ADDR::Register>),
                (0x008 => trans_count: ReadWrite<u32, TRANS_COUNT::Register>),
                (0x00C => ctrl_trig: ReadWrite<u32, CTRL_TRIG::Register>),
                (0x010 => _reserved0),
                (0x040 => @END),
        },
        pub DmaRegisters {
                (0x000 => channels: [ChannelRegisters; 12]),
                (0x300 => _reserved),

                (0x400 => intr: ReadWrite<u32, INTR::Register>),
                (0x404 => inte0: ReadWrite<u32, INTE::Register>),
                (0x408 => intf0: ReadWrite<u32, INTF::Register>),
                (0x40C => ints0: ReadWrite<u32, INTS::Register>),
                (0x410 => _reserved1),
                (0x414 => inte1: ReadWrite<u32, INTE::Register>),
                (0x418 => intf1: ReadWrite<u32, INTF::Register>),
                (0x41C => ints1: ReadWrite<u32, INTS::Register>),
                (0x420 => timer0: ReadWrite<u32, TIMER0::Register>),
                (0x424 => timer1: ReadWrite<u32, TIMER1::Register>),
                (0x428 => timer2: ReadWrite<u32, TIMER2::Register>),
                (0x42C => timer3: ReadWrite<u32, TIMER3::Register>),
                (0x430 => multi_chan_trigger: ReadWrite<u32>),
                (0x434 => sniff_ctrl: ReadWrite<u32, SNIFF_CTRL::Register>),
                (0x438 => sniff_data: ReadWrite<u32>),
                (0x43C => _reserved13),

                (0x440 => fifo_levels: ReadWrite<u32, FIFO_LEVELS::Register>),
                (0x444 => chan_abort: ReadWrite<u32>),
                (0x448 => n_channels: ReadWrite<u32>),
                (0x44C => _reserved14),

                (0x800 => ch0_dbg_ctdreq: ReadWrite<u32>),
                (0x804 => ch0_dbg_tcr: ReadWrite<u32>),
                (0x808 => _reserved15),

                (0x840 => ch1_dbg_ctdreq: ReadWrite<u32>),
                (0x844 => ch1_dbg_tcr: ReadWrite<u32>),
                (0x848 => _reserved16),

                (0x880 => ch2_dbg_ctdreq: ReadWrite<u32>),
                (0x884 => ch2_dbg_tcr: ReadWrite<u32>),
                (0x888 => _reserved17),

                (0x8C0 => ch3_dbg_ctdreq: ReadWrite<u32>),
                (0x8C4 => ch3_dbg_tcr: ReadWrite<u32>),
                (0x8C8 => _reserved18),

                (0x900 => ch4_dbg_ctdreq: ReadWrite<u32>),
                (0x904 => ch4_dbg_tcr: ReadWrite<u32>),
                (0x908 => _reserved19),

                (0x940 => ch5_dbg_ctdreq: ReadWrite<u32>),
                (0x944 => ch5_dbg_tcr: ReadWrite<u32>),
                (0x948 => _reserved20),

                (0x980 => ch6_dbg_ctdreq: ReadWrite<u32>),
                (0x984 => ch6_dbg_tcr: ReadWrite<u32>),
                (0x988 => _reserved21),

                (0x9C0 => ch7_dbg_ctdreq: ReadWrite<u32>),
                (0x9C4 => ch7_dbg_tcr: ReadWrite<u32>),
                (0x9C8 => _reserved22),

                (0xA00 => ch8_dbg_ctdreq: ReadWrite<u32>),
                (0xA04 => ch8_dbg_tcr: ReadWrite<u32>),
                (0xA08 => _reserved23),

                (0xA40 => ch9_dbg_ctdreq: ReadWrite<u32>),
                (0xA44 => ch9_dbg_tcr: ReadWrite<u32>),
                (0xA48 => _reserved24),

                (0xA80 => ch10_dbg_ctdreq: ReadWrite<u32>),
                (0xA84 => ch10_dbg_tcr: ReadWrite<u32>),
                (0xA88 => _reserved25),

                (0xAC0 => ch11_dbg_ctdreq: ReadWrite<u32>),
                (0xAC4 => ch11_dbg_tcr: ReadWrite<u32>),
                (0xAC8 => @END),
    }
}
register_bitfields![u32,
    READ_ADDR [
        READ_ADDR OFFSET(0) NUMBITS(32) []
    ],
    WRITE_ADDR [
        WRITE_ADDR OFFSET(0) NUMBITS(32) []
    ],
    TRANS_COUNT [
        TRANS_COUNT OFFSET(0) NUMBITS(32) []
    ],
    CTRL_TRIG [
        AHB_ERROR OFFSET(31) NUMBITS(1) [],
        READ_ERROR OFFSET(30) NUMBITS(1) [],
        WRITE_ERROR OFFSET(29) NUMBITS(1) [],
        BUSY OFFSET(24) NUMBITS(1) [],
        SNIFF_EN OFFSET(23) NUMBITS(1) [],
        BSWAP OFFSET(22) NUMBITS(1) [],
        IRQ_QUIET OFFSET(21) NUMBITS(1) [],
        TREQ_SEL OFFSET(15) NUMBITS(6) [
            /// Select PIO0's TX FIFO 0 as TREQ
            SelectPIO0STXFIFO0AsTREQ = 0,
            /// Select PIO0's TX FIFO 1 as TREQ
            SelectPIO0STXFIFO1AsTREQ = 1,
            /// Select PIO0's TX FIFO 2 as TREQ
            SelectPIO0STXFIFO2AsTREQ = 2,
            /// Select PIO0's TX FIFO 3 as TREQ
            SelectPIO0STXFIFO3AsTREQ = 3,
            /// Select PIO0's RX FIFO 0 as TREQ
            SelectPIO0SRXFIFO0AsTREQ = 4,
            /// Select PIO0's RX FIFO 1 as TREQ
            SelectPIO0SRXFIFO1AsTREQ = 5,
            /// Select PIO0's RX FIFO 2 as TREQ
            SelectPIO0SRXFIFO2AsTREQ = 6,
            /// Select PIO0's RX FIFO 3 as TREQ
            SelectPIO0SRXFIFO3AsTREQ = 7,
            /// Select PIO1's TX FIFO 0 as TREQ
            SelectPIO1STXFIFO0AsTREQ = 8,
            /// Select PIO1's TX FIFO 1 as TREQ
            SelectPIO1STXFIFO1AsTREQ = 9,
            /// Select PIO1's TX FIFO 2 as TREQ
            SelectPIO1STXFIFO2AsTREQ = 10,
            /// Select PIO1's TX FIFO 3 as TREQ
            SelectPIO1STXFIFO3AsTREQ = 11,
            /// Select PIO1's RX FIFO 0 as TREQ
            SelectPIO1SRXFIFO0AsTREQ = 12,
            /// Select PIO1's RX FIFO 1 as TREQ
            SelectPIO1SRXFIFO1AsTREQ = 13,
            /// Select PIO1's RX FIFO 2 as TREQ
            SelectPIO1SRXFIFO2AsTREQ = 14,
            /// Select PIO1's RX FIFO 3 as TREQ
            SelectPIO1SRXFIFO3AsTREQ = 15,
            /// Select SPI0's TX FIFO as TREQ
            SelectSPI0STXFIFOAsTREQ = 16,
            /// Select SPI0's RX FIFO as TREQ
            SelectSPI0SRXFIFOAsTREQ = 17,
            /// Select SPI1's TX FIFO as TREQ
            SelectSPI1STXFIFOAsTREQ = 18,
            /// Select SPI1's RX FIFO as TREQ
            SelectSPI1SRXFIFOAsTREQ = 19,
            /// Select UART0's TX FIFO as TREQ
            SelectUART0STXFIFOAsTREQ = 20,
            /// Select UART0's RX FIFO as TREQ
            SelectUART0SRXFIFOAsTREQ = 21,
            /// Select UART1's TX FIFO as TREQ
            SelectUART1STXFIFOAsTREQ = 22,
            /// Select UART1's RX FIFO as TREQ
            SelectUART1SRXFIFOAsTREQ = 23,
            /// Select PWM Counter 0's Wrap Value as TREQ
            SelectPWMCounter0SWrapValueAsTREQ = 24,
            /// Select PWM Counter 1's Wrap Value as TREQ
            SelectPWMCounter1SWrapValueAsTREQ = 25,
            /// Select PWM Counter 2's Wrap Value as TREQ
            SelectPWMCounter2SWrapValueAsTREQ = 26,
            /// Select PWM Counter 3's Wrap Value as TREQ
            SelectPWMCounter3SWrapValueAsTREQ = 27,
            /// Select PWM Counter 4's Wrap Value as TREQ
            SelectPWMCounter4SWrapValueAsTREQ = 28,
            /// Select PWM Counter 5's Wrap Value as TREQ
            SelectPWMCounter5SWrapValueAsTREQ = 29,
            /// Select PWM Counter 6's Wrap Value as TREQ
            SelectPWMCounter6SWrapValueAsTREQ = 30,
            /// Select PWM Counter 7's Wrap Value as TREQ
            SelectPWMCounter7SWrapValueAsTREQ = 31,
            /// Select I2C0's TX FIFO as TREQ
            SelectI2C0STXFIFOAsTREQ = 32,
            /// Select I2C0's RX FIFO as TREQ
            SelectI2C0SRXFIFOAsTREQ = 33,
            /// Select I2C1's TX FIFO as TREQ
            SelectI2C1STXFIFOAsTREQ = 34,
            /// Select I2C1's RX FIFO as TREQ
            SelectI2C1SRXFIFOAsTREQ = 35,
            /// Select the ADC as TREQ
            SelectTheADCAsTREQ = 36,
            /// Select the XIP Streaming FIFO as TREQ
            SelectTheXIPStreamingFIFOAsTREQ = 37,
            /// Select the XIP SSI TX FIFO as TREQ
            SelectTheXIPSSITXFIFOAsTREQ = 38,
            /// Select the XIP SSI RX FIFO as TREQ
            SelectTheXIPSSIRXFIFOAsTREQ = 39,
            /// Select Timer 0 as TREQ
            SelectTimer0AsTREQ = 59,
            /// Select Timer 1 as TREQ
            SelectTimer1AsTREQ = 60,
            /// Select Timer 2 as TREQ (Optional)
            SelectTimer2AsTREQOptional = 61,
            /// Select Timer 3 as TREQ (Optional)
            SelectTimer3AsTREQOptional = 62,
            /// Permanent request, for unpaced transfers.
            PermanentRequestForUnpacedTransfers = 63
        ],
        /// When this channel completes, it will trigger the channel indicated by CHAIN_TO.
        CHAIN_TO OFFSET(11) NUMBITS(4) [],
        /// Select whether RING_SIZE applies to read or write addresses.
        /// If 0, read addresses are wrapped on a (1 << RING_SIZ
        RING_SEL OFFSET(10) NUMBITS(1) [],
        /// Size of address wrap region. If 0, don't wrap. For values n > 0, only the lower
        ///
        /// Ring sizes between 2 and 32768 bytes are possible. T
        RING_SIZE OFFSET(6) NUMBITS(4) [
            RING_NONE = 0
        ],
        INCR_WRITE OFFSET(5) NUMBITS(1) [],
        INCR_READ OFFSET(4) NUMBITS(1) [],
        DATA_SIZE OFFSET(2) NUMBITS(2) [
            SIZE_BYTE = 0b00,
            SIZE_HALFWORD = 0b01,
            SIZE_WORD = 0b10
        ],
        HIGH_PRIORITY OFFSET(1) NUMBITS(1) [],
        EN OFFSET(0) NUMBITS(1) []
    ],
    INTR [
        INTR OFFSET(0) NUMBITS(16) []
    ],
    INTE [
        CH11 OFFSET(11) NUMBITS(1) [],
        CH10 OFFSET(10) NUMBITS(1) [],
        CH9 OFFSET(9) NUMBITS(1) [],
        CH8 OFFSET(8) NUMBITS(1) [],
        CH7 OFFSET(7) NUMBITS(1) [],
        CH6 OFFSET(6) NUMBITS(1) [],
        CH5 OFFSET(5) NUMBITS(1) [],
        CH4 OFFSET(4) NUMBITS(1) [],
        CH3 OFFSET(3) NUMBITS(1) [],
        CH2 OFFSET(2) NUMBITS(1) [],
        CH1 OFFSET(1) NUMBITS(1) [],
        CH0 OFFSET(0) NUMBITS(1) [],
    ],
    INTF [
        CH11 OFFSET(11) NUMBITS(1) [],
        CH10 OFFSET(10) NUMBITS(1) [],
        CH9 OFFSET(9) NUMBITS(1) [],
        CH8 OFFSET(8) NUMBITS(1) [],
        CH7 OFFSET(7) NUMBITS(1) [],
        CH6 OFFSET(6) NUMBITS(1) [],
        CH5 OFFSET(5) NUMBITS(1) [],
        CH4 OFFSET(4) NUMBITS(1) [],
        CH3 OFFSET(3) NUMBITS(1) [],
        CH2 OFFSET(2) NUMBITS(1) [],
        CH1 OFFSET(1) NUMBITS(1) [],
        CH0 OFFSET(0) NUMBITS(1) [],
    ],
    INTS [
        /// Indicates active channel interrupt requests which are currently causing IRQ 0 to
        /// Channel interrupts can be cleared by writing a bit m
        CH11 OFFSET(11) NUMBITS(1) [],
        CH10 OFFSET(10) NUMBITS(1) [],
        CH9 OFFSET(9) NUMBITS(1) [],
        CH8 OFFSET(8) NUMBITS(1) [],
        CH7 OFFSET(7) NUMBITS(1) [],
        CH6 OFFSET(6) NUMBITS(1) [],
        CH5 OFFSET(5) NUMBITS(1) [],
        CH4 OFFSET(4) NUMBITS(1) [],
        CH3 OFFSET(3) NUMBITS(1) [],
        CH2 OFFSET(2) NUMBITS(1) [],
        CH1 OFFSET(1) NUMBITS(1) [],
        CH0 OFFSET(0) NUMBITS(1) [],
    ],
    TIMER0 [
        /// Pacing Timer Dividend. Specifies the X value for the (X/Y) fractional timer.
        X OFFSET(16) NUMBITS(16) [],
        /// Pacing Timer Divisor. Specifies the Y value for the (X/Y) fractional timer.
        Y OFFSET(0) NUMBITS(16) []
    ],
    TIMER1 [
        /// Pacing Timer Dividend. Specifies the X value for the (X/Y) fractional timer.
        X OFFSET(16) NUMBITS(16) [],
        /// Pacing Timer Divisor. Specifies the Y value for the (X/Y) fractional timer.
        Y OFFSET(0) NUMBITS(16) []
    ],
    TIMER2 [
        X OFFSET(16) NUMBITS(16) [],
        Y OFFSET(0) NUMBITS(16) []
    ],
    TIMER3 [
        X OFFSET(16) NUMBITS(16) [],
        Y OFFSET(0) NUMBITS(16) []
    ],
    MULTI_CHAN_TRIGGER [
        MULTI_CHAN_TRIGGER OFFSET(0) NUMBITS(16) []
    ],
    SNIFF_CTRL [
        OUT_INV OFFSET(11) NUMBITS(1) [],
        OUT_REV OFFSET(10) NUMBITS(1) [],
        BSWAP OFFSET(9) NUMBITS(1) [],

        CALC OFFSET(5) NUMBITS(4) [
            /// Calculate a CRC-32 (IEEE802.3 polynomial)
            CalculateACRC32IEEE8023Polynomial = 0,
            /// Calculate a CRC-32 (IEEE802.3 polynomial) with bit reversed data
            CalculateACRC32IEEE8023PolynomialWithBitReversedData = 1,
            /// Calculate a CRC-16-CCITT
            CalculateACRC16CCITT = 2,
            /// Calculate a CRC-16-CCITT with bit reversed data
            CalculateACRC16CCITTWithBitReversedData = 3,
            /// XOR reduction over all data. == 1 if the total 1 population count is odd.
            XORReductionOverAllData1IfTheTotal1PopulationCountIsOdd = 14,
            /// Calculate a simple 32-bit checksum (addition with a 32 bit accumulator)
            CalculateASimple32BitChecksumAdditionWithA32BitAccumulator = 15
        ],
        /// DMA channel for Sniffer to observe
        DMACH OFFSET(1) NUMBITS(4) [],
        /// Enable sniffer
        EN OFFSET(0) NUMBITS(1) []
    ],
    SNIFF_DATA [
        SNIFF_DATA OFFSET(0) NUMBITS(32) []
    ],
    FIFO_LEVELS [
        /// Current Read-Address-FIFO fill level
        RAF_LVL OFFSET(16) NUMBITS(8) [],
        /// Current Write-Address-FIFO fill level
        WAF_LVL OFFSET(8) NUMBITS(8) [],
        /// Current Transfer-Data-FIFO fill level
        TDF_LVL OFFSET(0) NUMBITS(8) []
    ],
    CHAN_ABORT [
        CHAN_ABORT OFFSET(0) NUMBITS(16) []
    ],
    N_CHANNELS [
        N_CHANNELS OFFSET(0) NUMBITS(5) []
    ],
    DBG_CTDREQ [
        DBG_CTDREQ OFFSET(0) NUMBITS(6) []
    ],
    DBG_TCR [
        DBG_TCR OFFSET(0) NUMBITS(32) []
    ]
];

#[derive(Clone, Copy)]
pub enum Channel {
    Channel0 = 0,
    Channel1 = 1,
    Channel2 = 2,
    Channel3 = 3,
    Channel4 = 4,
    Channel5 = 5,
    Channel6 = 6,
    Channel7 = 7,
    Channel8 = 8,
    Channel9 = 9,
    Channel10 = 10,
    Channel11 = 11,
}

pub enum Transfer {
    PeripheralToMemory,
    MemoryToPeripheral,
}

pub enum DataSize {
    Byte = 0x0,
    HalfWord = 0x1,
    Word = 0x2,
}

impl From<DataSize> for FieldValue<u32, CTRL_TRIG::Register> {
    fn from(value: DataSize) -> Self {
        match value {
            DataSize::Byte => CTRL_TRIG::DATA_SIZE::SIZE_BYTE,
            DataSize::HalfWord => CTRL_TRIG::DATA_SIZE::SIZE_HALFWORD,
            DataSize::Word => CTRL_TRIG::DATA_SIZE::SIZE_WORD,
        }
    }
}

pub enum DmaPeripheral {
    PioRxFifo(pio::PIONumber, pio::SMNumber),
    PioTxFifo(pio::PIONumber, pio::SMNumber),
}

impl From<DmaPeripheral> for FieldValue<u32, CTRL_TRIG::Register> {
    fn from(value: DmaPeripheral) -> Self {
        match value {
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM0) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO0AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM1) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO1AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM2) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO2AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM3) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO3AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM0) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO0AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM1) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO1AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM2) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO2AsTREQ
            }
            DmaPeripheral::PioRxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM3) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO3AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM0) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO0AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM1) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO1AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM2) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO2AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO0, pio::SMNumber::SM3) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO3AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM0) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO0AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM1) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO1AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM2) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO2AsTREQ
            }
            DmaPeripheral::PioTxFifo(pio::PIONumber::PIO1, pio::SMNumber::SM3) => {
                CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO3AsTREQ
            }
        }
    }
}

pub enum Irq {
    Irq0,
    Irq1,
}

const DMA_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x50000000 as *const DmaRegisters) };

pub trait DmaChannelClient {
    fn transfer_done(&self);
}

pub struct DmaChannel<'a> {
    registers: StaticRef<DmaRegisters>,
    ch: Channel,
    client: OptionalCell<&'a dyn DmaChannelClient>,
}

impl<'a> DmaChannel<'a> {
    pub const fn new(ch: Channel) -> Self {
        Self {
            registers: DMA_BASE,
            ch,
            client: OptionalCell::empty(),
        }
    }
    pub fn set_client(&self, client: &'a dyn DmaChannelClient) {
        self.client.set(client);
    }
}

pub struct Dma<'a> {
    registers: StaticRef<DmaRegisters>,
    interrupt0: Cell<LocalRegisterCopy<u32, INTE::Register>>,
    interrupt1: Cell<LocalRegisterCopy<u32, INTE::Register>>,
    channels: [DmaChannel<'a>; 12],
}

impl<'a> Dma<'a> {
    pub const fn new() -> Self {
        Self {
            registers: DMA_BASE,
            interrupt0: Cell::new(LocalRegisterCopy::new(0)),
            interrupt1: Cell::new(LocalRegisterCopy::new(0)),
            channels: [
                DmaChannel::new(Channel::Channel0),
                DmaChannel::new(Channel::Channel1),
                DmaChannel::new(Channel::Channel2),
                DmaChannel::new(Channel::Channel3),
                DmaChannel::new(Channel::Channel4),
                DmaChannel::new(Channel::Channel5),
                DmaChannel::new(Channel::Channel6),
                DmaChannel::new(Channel::Channel7),
                DmaChannel::new(Channel::Channel8),
                DmaChannel::new(Channel::Channel9),
                DmaChannel::new(Channel::Channel10),
                DmaChannel::new(Channel::Channel11),
            ],
        }
    }

    pub fn channel(&'a self, ch: Channel) -> &'a DmaChannel<'a> {
        &self.channels[ch as usize]
    }
}

impl Dma<'_> {
    pub fn enable_interrupt(&self, channel: Channel, irq: Irq) {
        let inte = match channel {
            Channel::Channel0 => INTE::CH0::SET,
            Channel::Channel1 => INTE::CH1::SET,
            Channel::Channel2 => INTE::CH2::SET,
            Channel::Channel3 => INTE::CH3::SET,
            Channel::Channel4 => INTE::CH4::SET,
            Channel::Channel5 => INTE::CH5::SET,
            Channel::Channel6 => INTE::CH6::SET,
            Channel::Channel7 => INTE::CH7::SET,
            Channel::Channel8 => INTE::CH8::SET,
            Channel::Channel9 => INTE::CH9::SET,
            Channel::Channel10 => INTE::CH10::SET,
            Channel::Channel11 => INTE::CH11::SET,
        };

        match irq {
            Irq::Irq0 => {
                self.registers.inte0.modify(inte);
                let mut interrupt0 = self.interrupt0.get();
                interrupt0.modify(inte);
                self.interrupt0.set(interrupt0);
            }
            Irq::Irq1 => {
                self.registers.inte1.modify(inte);
                let mut interrupt1 = self.interrupt1.get();
                interrupt1.modify(inte);
                self.interrupt1.set(interrupt1);
            }
        }
    }

    pub fn disable_interrupt(&self, channel: Channel, irq: Irq) {
        let inte = match channel {
            Channel::Channel0 => INTE::CH0::CLEAR,
            Channel::Channel1 => INTE::CH1::CLEAR,
            Channel::Channel2 => INTE::CH2::CLEAR,
            Channel::Channel3 => INTE::CH3::CLEAR,
            Channel::Channel4 => INTE::CH4::CLEAR,
            Channel::Channel5 => INTE::CH5::CLEAR,
            Channel::Channel6 => INTE::CH6::CLEAR,
            Channel::Channel7 => INTE::CH7::CLEAR,
            Channel::Channel8 => INTE::CH8::CLEAR,
            Channel::Channel9 => INTE::CH9::CLEAR,
            Channel::Channel10 => INTE::CH10::CLEAR,
            Channel::Channel11 => INTE::CH11::CLEAR,
        };

        match irq {
            Irq::Irq0 => {
                self.registers.inte0.modify(inte);
                let mut interrupt0 = self.interrupt0.get();
                interrupt0.modify(inte);
                self.interrupt0.set(interrupt0);
            }
            Irq::Irq1 => {
                self.registers.inte1.modify(inte);
                let mut interrupt1 = self.interrupt1.get();
                interrupt1.modify(inte);
                self.interrupt1.set(interrupt1);
            }
        }
    }

    pub fn clear_interrupt(&self, channel: Channel, irq: Irq) {
        let ints = match channel {
            Channel::Channel0 => INTS::CH0::SET,
            Channel::Channel1 => INTS::CH1::SET,
            Channel::Channel2 => INTS::CH2::SET,
            Channel::Channel3 => INTS::CH3::SET,
            Channel::Channel4 => INTS::CH4::SET,
            Channel::Channel5 => INTS::CH5::SET,
            Channel::Channel6 => INTS::CH6::SET,
            Channel::Channel7 => INTS::CH7::SET,
            Channel::Channel8 => INTS::CH8::SET,
            Channel::Channel9 => INTS::CH9::SET,
            Channel::Channel10 => INTS::CH10::SET,
            Channel::Channel11 => INTS::CH11::SET,
        };

        match irq {
            Irq::Irq0 => self.registers.ints0.modify(ints),
            Irq::Irq1 => self.registers.ints1.modify(ints),
        }
    }

    pub fn handle_interrupt0(&self) {
        let value = self.registers.ints0.get();
        self.registers.ints0.set(value);

        self.handle_channels(value);
    }

    pub fn handle_interrupt1(&self) {
        let value = self.registers.ints1.get();
        self.registers.ints1.set(value);

        self.handle_channels(value);
    }

    #[inline]
    fn handle_channels(&self, ints: u32) {
        for channel in 0..12u32 {
            if ints & (1 << channel) != 0 {
                self.channels[channel as usize]
                    .client
                    .map(|client| client.transfer_done());
            }
        }
    }
}

impl Dma<'_> {
    pub fn channel_registers(&self, channel: Channel) -> &ChannelRegisters {
        &self.registers.channels[channel as usize]
    }
}

impl DmaChannel<'_> {
    pub fn trans_count(&self) -> u32 {
        let regs = &self.registers.channels[self.ch as usize];
        regs.trans_count.get()
    }

    pub fn busy(&self) -> bool {
        let regs = &self.registers.channels[self.ch as usize];
        match regs.ctrl_trig.read(CTRL_TRIG::BUSY) {
            0 => false,
            _ => true,
        }
    }

    pub fn set_read_addr(&self, addr: u32) {
        let regs = &self.registers.channels[self.ch as usize];
        regs.read_addr.write(READ_ADDR::READ_ADDR.val(addr));
    }

    pub fn set_write_addr(&self, addr: u32) {
        let regs = &self.registers.channels[self.ch as usize];
        regs.write_addr.write(WRITE_ADDR::WRITE_ADDR.val(addr));
    }

    pub fn set_len(&self, len: u32) {
        let regs = &self.registers.channels[self.ch as usize];
        regs.trans_count.write(TRANS_COUNT::TRANS_COUNT.val(len));
    }

    pub fn enable_interrupt(&self, irq: Irq) {
        let irq = match irq {
            Irq::Irq0 => &self.registers.inte0,
            Irq::Irq1 => &self.registers.inte1,
        };
        let mut value = irq.get();
        value |= 1 << (self.ch as usize);
        irq.set(value);
    }

    pub fn disable_interrupt(&self, irq: Irq) {
        let irq = match irq {
            Irq::Irq0 => &self.registers.inte0,
            Irq::Irq1 => &self.registers.inte1,
        };
        let mut value = irq.get();
        value &= !(1 << (self.ch as usize));
        irq.set(value);
    }

    pub fn enable(
        &self,
        treq: DmaPeripheral,
        data_size: DataSize,
        transfer: Transfer,
        bswap: bool,
    ) {
        let regs = &self.registers.channels[self.ch as usize];

        let bswap = match bswap {
            true => CTRL_TRIG::BSWAP::SET,
            false => CTRL_TRIG::BSWAP::CLEAR,
        };
        let (incr_rd, incr_wr) = match transfer {
            Transfer::MemoryToPeripheral => {
                (CTRL_TRIG::INCR_READ::SET, CTRL_TRIG::INCR_WRITE::CLEAR)
            }
            Transfer::PeripheralToMemory => {
                (CTRL_TRIG::INCR_READ::CLEAR, CTRL_TRIG::INCR_WRITE::SET)
            }
        };
        let treq = FieldValue::from(treq);
        let data_size = FieldValue::from(data_size);
        let chain_to = CTRL_TRIG::CHAIN_TO.val(self.ch as u32);

        let fv = treq + data_size + bswap + incr_rd + incr_wr + chain_to + CTRL_TRIG::EN::SET;
        regs.ctrl_trig.write(fv);
    }
}
