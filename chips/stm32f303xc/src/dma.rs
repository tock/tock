use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

use crate::rcc;

/// DMA controller
#[repr(C)]
struct Dma1Registers {
    /// interrupt status register
    isr: ReadOnly<u32, ISR::Register>,
    /// interrupt flag clear register
    ifcr: ReadWrite<u32, IFCR::Register>,

    /// channel 1 configuration register
    ccr1: ReadWrite<u32, CCR::Register>,
    /// channel 1 number of data register
    cndtr1: ReadWrite<u32, CNDTR::Register>,
    /// channel 1 peripheral address register
    cpar1: ReadWrite<u32, CPAR::Register>,
    /// channel 1 memory address register
    cmar1: ReadWrite<u32, CMAR::Register>,

    _reserved0: [u8; 4],

    /// channel 2 configuration register
    ccr2: ReadWrite<u32, CCR::Register>,
    /// channel 2 number of data register
    cndtr2: ReadWrite<u32, CNDTR::Register>,
    /// channel 2 peripheral address register
    cpar2: ReadWrite<u32, CPAR::Register>,
    /// channel 2 memory address register
    cmar2: ReadWrite<u32, CMAR::Register>,

    _reserved1: [u8; 4],

    /// channel 3 configuration register
    ccr3: ReadWrite<u32, CCR::Register>,
    /// channel 3 number of data register
    cndtr3: ReadWrite<u32, CNDTR::Register>,
    /// channel 3 peripheral address register
    cpar3: ReadWrite<u32, CPAR::Register>,
    /// channel 3 memory address register
    cmar3: ReadWrite<u32, CMAR::Register>,

    _reserved2: [u8; 4],

    /// channel 4 configuration register
    ccr4: ReadWrite<u32, CCR::Register>,
    /// channel 4 number of data register
    cndtr4: ReadWrite<u32, CNDTR::Register>,
    /// channel 4 peripheral address register
    cpar4: ReadWrite<u32, CPAR::Register>,
    /// channel 4 memory address register
    cmar4: ReadWrite<u32, CMAR::Register>,

    _reserved3: [u8; 4],

    /// channel 5 configuration register
    ccr5: ReadWrite<u32, CCR::Register>,
    /// channel 5 number of data register
    cndtr5: ReadWrite<u32, CNDTR::Register>,
    /// channel 5 peripheral address register
    cpar5: ReadWrite<u32, CPAR::Register>,
    /// channel 5 memory address register
    cmar5: ReadWrite<u32, CMAR::Register>,

    _reserved4: [u8; 4],

    /// channel 6 configuration register
    ccr6: ReadWrite<u32, CCR::Register>,
    /// channel 6 number of data register
    cndtr6: ReadWrite<u32, CNDTR::Register>,
    /// channel 6 peripheral address register
    cpar6: ReadWrite<u32, CPAR::Register>,
    /// channel 6 memory address register
    cmar6: ReadWrite<u32, CMAR::Register>,

    _reserved5: [u8; 4],

    /// channel 7 configuration register
    ccr7: ReadWrite<u32, CCR::Register>,
    /// channel 7 number of data register
    cndtr7: ReadWrite<u32, CNDTR::Register>,
    /// channel 7 peripheral address register
    cpar7: ReadWrite<u32, CPAR::Register>,
    /// channel 7 memory address register
    cmar7: ReadWrite<u32, CMAR::Register>,
}

register_bitfields![u32,
    ISR [
        /// Channel 7 transfer error flag
        TEIF7 OFFSET(27) NUMBITS(1) [],
        /// Channel 7 half transfer flag
        HTIF7 OFFSET(26) NUMBITS(1) [],
        /// Channel 7 transfer complete flag
        TCIF7 OFFSET(25) NUMBITS(1) [],
        /// Channel 7 global interrupt flag
        GIF7 OFFSET(24) NUMBITS(1) [],

        /// Channel 6 transfer error flag
        TEIF6 OFFSET(23) NUMBITS(1) [],
        /// Channel 6 half transfer flag
        HTIF6 OFFSET(22) NUMBITS(1) [],
        /// Channel 6 transfer complete flag
        TCIF6 OFFSET(21) NUMBITS(1) [],
        /// Channel 6 global interrupt flag
        GIF6 OFFSET(20) NUMBITS(1) [],

        /// Channel 5 transfer error flag
        TEIF5 OFFSET(19) NUMBITS(1) [],
        /// Channel 5 half transfer flag
        HTIF5 OFFSET(18) NUMBITS(1) [],
        /// Channel 5 transfer complete flag
        TCIF5 OFFSET(17) NUMBITS(1) [],
        /// Channel 5 global interrupt flag
        GIF5 OFFSET(16) NUMBITS(1) [],

        /// Channel 4 transfer error flag
        TEIF4 OFFSET(15) NUMBITS(1) [],
        /// Channel 4 half transfer flag
        HTIF4 OFFSET(14) NUMBITS(1) [],
        /// Channel 4 transfer complete flag
        TCIF4 OFFSET(13) NUMBITS(1) [],
        /// Channel 4 global interrupt flag
        GIF4 OFFSET(12) NUMBITS(1) [],

        /// Channel 3 transfer error flag
        TEIF3 OFFSET(11) NUMBITS(1) [],
        /// Channel 3 half transfer flag
        HTIF3 OFFSET(10) NUMBITS(1) [],
        /// Channel 3 transfer complete flag
        TCIF3 OFFSET(9) NUMBITS(1) [],
        /// Channel 3 global interrupt flag
        GIF3 OFFSET(8) NUMBITS(1) [],

        /// Channel 2 transfer error flag
        TEIF2 OFFSET(7) NUMBITS(1) [],
        /// Channel 2 half transfer flag
        HTIF2 OFFSET(6) NUMBITS(1) [],
        /// Channel 2 transfer complete flag
        TCIF2 OFFSET(5) NUMBITS(1) [],
        /// Channel 2 global interrupt flag
        GIF2 OFFSET(4) NUMBITS(1) [],

        /// Channel 1 transfer error flag
        TEIF1 OFFSET(3) NUMBITS(1) [],
        /// Channel 1 half transfer flag
        HTIF1 OFFSET(2) NUMBITS(1) [],
        /// Channel 1 transfer complete flag
        TCIF1 OFFSET(1) NUMBITS(1) [],
        /// Channel 1 global interrupt flag
        GIF1 OFFSET(0) NUMBITS(1) []
    ],
    IFCR [
        /// Channel 7 transfer error clear
        CTEIF7 OFFSET(27) NUMBITS(1) [],
        /// Channel 7 half transfer clear
        CHTIF7 OFFSET(26) NUMBITS(1) [],
        /// Channel 7 transfer complete clear
        CTCIF7 OFFSET(25) NUMBITS(1) [],
        /// Channel 7 global interrupt clear
        CGIF7 OFFSET(24) NUMBITS(1) [],

        /// Channel 6 transfer error clear
        CTEIF6 OFFSET(23) NUMBITS(1) [],
        /// Channel 6 half transfer clear
        CHTIF6 OFFSET(22) NUMBITS(1) [],
        /// Channel 6 transfer complete clear
        CTCIF6 OFFSET(21) NUMBITS(1) [],
        /// Channel 6 global interrupt clear
        CGIF6 OFFSET(20) NUMBITS(1) [],

        /// Channel 5 transfer error clear
        CTEIF5 OFFSET(19) NUMBITS(1) [],
        /// Channel 5 half transfer clear
        CHTIF5 OFFSET(18) NUMBITS(1) [],
        /// Channel 5 transfer complete clear
        CTCIF5 OFFSET(17) NUMBITS(1) [],
        /// Channel 5 global interrupt clear
        CGIF5 OFFSET(16) NUMBITS(1) [],

        /// Channel 4 transfer error clear
        CTEIF4 OFFSET(15) NUMBITS(1) [],
        /// Channel 4 half transfer clear
        CHTIF4 OFFSET(14) NUMBITS(1) [],
        /// Channel 4 transfer complete clear
        CTCIF4 OFFSET(13) NUMBITS(1) [],
        /// Channel 4 global interrupt clear
        CGIF4 OFFSET(12) NUMBITS(1) [],

        /// Channel 3 transfer error clear
        CTEIF3 OFFSET(11) NUMBITS(1) [],
        /// Channel 3 half transfer clear
        CHTIF3 OFFSET(10) NUMBITS(1) [],
        /// Channel 3 transfer complete clear
        CTCIF3 OFFSET(9) NUMBITS(1) [],
        /// Channel 3 global interrupt clear
        CGIF3 OFFSET(8) NUMBITS(1) [],

        /// Channel 2 transfer error clear
        CTEIF2 OFFSET(7) NUMBITS(1) [],
        /// Channel 2 half transfer clear
        CHTIF2 OFFSET(6) NUMBITS(1) [],
        /// Channel 2 transfer complete clear
        CTCIF2 OFFSET(5) NUMBITS(1) [],
        /// Channel 2 global interrupt clear
        CGIF2 OFFSET(4) NUMBITS(1) [],

        /// Channel 1 transfer error clear
        CTEIF1 OFFSET(3) NUMBITS(1) [],
        /// Channel 1 half transfer clear
        CHTIF1 OFFSET(2) NUMBITS(1) [],
        /// Channel 1 transfer complete clear
        CTCIF1 OFFSET(1) NUMBITS(1) [],
        /// Channel 1 global interrupt clear
        CGIF1 OFFSET(0) NUMBITS(1) []
    ],
    CCR [
        /// Memory to memory mode
        MEM2MEM OFFSET(14) NUMBITS(1) [],
        /// Channel priority level
        PL OFFSET(12) NUMBITS(2) [],
        /// Memory size
        MSIZE OFFSET(10) NUMBITS(2) [],
        /// Peripheral size
        PSIZE OFFSET(8) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(7) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(6) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(5) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(4) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(3) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(2) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(1) NUMBITS(1) [],
        /// Channel enable
        EN OFFSET(0) NUMBITS(1) []
    ],
    CNDTR [
        /// Number of data to transfer
        NDT OFFSET(0) NUMBITS(16) []
    ],
    CPAR [
        /// Peripheral address
        PA OFFSET(0) NUMBITS(32) []
    ],
    CMAR [
        /// Memory address
        MA OFFSET(0) NUMBITS(32) []
    ]
];

const DMA1_BASE: StaticRef<Dma1Registers> =
    unsafe { StaticRef::new(0x4002_0000 as *const Dma1Registers) };

#[allow(dead_code)]
#[repr(u32)]
enum ChannelId {
    Channel1 = 0b000,
    Channel2 = 0b001,
    Channel3 = 0b010,
    Channel4 = 0b011,
    Channel5 = 0b100,
    Channel6 = 0b101,
    Channel7 = 0b110,
}

/// DMA transfer priority. Section 13.5.3
#[allow(dead_code)]
#[repr(u32)]
enum Priority {
    Low = 0b00,
    Medium = 0b01,
    High = 0b10,
    VeryHigh = 0b11,
}

/// DMA data size. Section 13.5.3
#[allow(dead_code)]
#[repr(u32)]
enum Size {
    Byte = 0b00,
    HalfWord = 0b01,
    Word = 0b10,
}

struct Msize(Size);
struct Psize(Size);

/// List of peripherals managed by DMA1
/// not complete yet
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Copy, Clone, PartialEq)]
pub enum Dma1Peripheral {
    USART1_TX,
    USART1_RX,
}

pub struct Dma1 {
    registers: StaticRef<Dma1Registers>,
    clock: Dma1Clock,
}

pub static mut DMA1: Dma1 = Dma1::new();

impl Dma1 {
    const fn new() -> Dma1 {
        Dma1 {
            registers: DMA1_BASE,
            clock: Dma1Clock(rcc::PeripheralClock::AHB(rcc::HCLK::DMA1)),
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
}

struct Dma1Clock(rcc::PeripheralClock);

impl ClockInterface for Dma1Clock {
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
