use kernel::platform::chip::ClockInterface;
use kernel::utilities::StaticRef;

use crate::nvic;
use crate::rcc;
use crate::spi;
use crate::usart;

use super::{
    ChannelId, Direction, DmaClock, DmaRegisters, FifoSize, Msize, Psize, Size, Stream, StreamId,
    StreamPeripheral, StreamServer, TransferMode,
};

/// List of peripherals managed by DMA1
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Dma1Peripheral {
    USART2_TX,
    USART2_RX,
    USART3_TX,
    USART3_RX,
    SPI3_TX,
    SPI3_RX,
}

impl Dma1Peripheral {
    // Returns the IRQ number of the stream associated with the peripheral. Used
    // to enable interrupt on the NVIC.
    pub fn get_stream_irqn(&self) -> u32 {
        match self {
            Dma1Peripheral::SPI3_TX => nvic::DMA1_Stream7,
            Dma1Peripheral::USART2_TX => nvic::DMA1_Stream6,
            Dma1Peripheral::USART2_RX => nvic::DMA1_Stream5,
            Dma1Peripheral::USART3_TX => nvic::DMA1_Stream3,
            Dma1Peripheral::SPI3_RX => nvic::DMA1_Stream2,
            Dma1Peripheral::USART3_RX => nvic::DMA1_Stream1,
        }
    }

    pub fn get_stream_idx<'a>(&self) -> usize {
        usize::from(StreamId::from(*self) as u8)
    }
}

impl From<Dma1Peripheral> for StreamId {
    fn from(pid: Dma1Peripheral) -> StreamId {
        match pid {
            Dma1Peripheral::SPI3_TX => StreamId::Stream7,
            Dma1Peripheral::USART2_TX => StreamId::Stream6,
            Dma1Peripheral::USART2_RX => StreamId::Stream5,
            Dma1Peripheral::USART3_TX => StreamId::Stream3,
            Dma1Peripheral::SPI3_RX => StreamId::Stream2,
            Dma1Peripheral::USART3_RX => StreamId::Stream1,
        }
    }
}

impl StreamPeripheral for Dma1Peripheral {
    fn transfer_mode(&self) -> TransferMode {
        TransferMode::Fifo(FifoSize::Full)
    }

    fn data_width(&self) -> (Msize, Psize) {
        (Msize(Size::Byte), Psize(Size::Byte))
    }

    fn channel_id(&self) -> ChannelId {
        match self {
            Dma1Peripheral::SPI3_TX => {
                // SPI3_RX Stream 7, Channel 0
                ChannelId::Channel0
            }
            Dma1Peripheral::USART2_TX => {
                // USART2_TX Stream 6, Channel 4
                ChannelId::Channel4
            }
            Dma1Peripheral::USART2_RX => {
                // USART2_RX Stream 5, Channel 4
                ChannelId::Channel4
            }
            Dma1Peripheral::USART3_TX => {
                // USART3_TX Stream 3, Channel 4
                ChannelId::Channel4
            }
            Dma1Peripheral::SPI3_RX => {
                // SPI3_RX Stream 2, Channel 0
                ChannelId::Channel0
            }
            Dma1Peripheral::USART3_RX => {
                // USART3_RX Stream 1, Channel 4
                ChannelId::Channel4
            }
        }
    }

    fn direction(&self) -> Direction {
        match self {
            Dma1Peripheral::SPI3_TX => Direction::MemoryToPeripheral,
            Dma1Peripheral::USART2_TX => Direction::MemoryToPeripheral,
            Dma1Peripheral::USART2_RX => Direction::PeripheralToMemory,
            Dma1Peripheral::USART3_TX => Direction::MemoryToPeripheral,
            Dma1Peripheral::SPI3_RX => Direction::PeripheralToMemory,
            Dma1Peripheral::USART3_RX => Direction::PeripheralToMemory,
        }
    }

    fn address(&self) -> u32 {
        match self {
            Dma1Peripheral::SPI3_TX => spi::get_address_dr(spi::SPI3_BASE),
            Dma1Peripheral::USART2_TX => usart::get_address_dr(usart::USART2_BASE),
            Dma1Peripheral::USART2_RX => usart::get_address_dr(usart::USART2_BASE),
            Dma1Peripheral::USART3_TX => usart::get_address_dr(usart::USART3_BASE),
            Dma1Peripheral::SPI3_RX => spi::get_address_dr(spi::SPI3_BASE),
            Dma1Peripheral::USART3_RX => usart::get_address_dr(usart::USART3_BASE),
        }
    }
}

pub fn new_dma1_stream<'a>(dma: &'a Dma1) -> [Stream<'a, Dma1<'a>>; 8] {
    [
        Stream::new(StreamId::Stream0, dma),
        Stream::new(StreamId::Stream1, dma),
        Stream::new(StreamId::Stream2, dma),
        Stream::new(StreamId::Stream3, dma),
        Stream::new(StreamId::Stream4, dma),
        Stream::new(StreamId::Stream5, dma),
        Stream::new(StreamId::Stream6, dma),
        Stream::new(StreamId::Stream7, dma),
    ]
}

const DMA1_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x40026000 as *const DmaRegisters) };

pub struct Dma1<'a> {
    registers: StaticRef<DmaRegisters>,
    clock: DmaClock<'a>,
}

impl<'a> Dma1<'a> {
    pub const fn new(rcc: &'a rcc::Rcc) -> Dma1 {
        Dma1 {
            registers: DMA1_BASE,
            clock: DmaClock(rcc::PeripheralClock::new(
                rcc::PeripheralClockType::AHB1(rcc::HCLK1::DMA1),
                rcc,
            )),
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

impl<'a> StreamServer<'a> for Dma1<'a> {
    type Peripheral = Dma1Peripheral;

    fn registers(&self) -> &DmaRegisters {
        &*self.registers
    }
}
