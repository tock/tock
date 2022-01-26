use kernel::platform::chip::ClockInterface;
use kernel::utilities::StaticRef;

use crate::nvic;
use crate::rcc;
use crate::usart;

use super::{
    ChannelId, Direction, DmaClock, DmaRegisters, FifoSize, Msize, Psize, Size, Stream, StreamId,
    StreamPeripheral, StreamServer, TransferMode,
};

const DMA2_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x40026400 as *const DmaRegisters) };

/// List of peripherals managed by DMA2
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Dma2Peripheral {
    USART1_TX,
    USART1_RX,
}

impl Dma2Peripheral {
    // Returns the IRQ number of the stream associated with the peripheral. Used
    // to enable interrupt on the NVIC.
    pub fn get_stream_irqn(&self) -> u32 {
        match self {
            Dma2Peripheral::USART1_TX => nvic::DMA2_Stream7,
            Dma2Peripheral::USART1_RX => nvic::DMA2_Stream5, // could also be Stream 2, chosen arbitrarily
        }
    }

    pub fn get_stream_idx<'a>(&self) -> usize {
        usize::from(StreamId::from(*self) as u8)
    }
}

impl From<Dma2Peripheral> for StreamId {
    fn from(pid: Dma2Peripheral) -> StreamId {
        match pid {
            Dma2Peripheral::USART1_TX => StreamId::Stream7,
            Dma2Peripheral::USART1_RX => StreamId::Stream5,
        }
    }
}

impl StreamPeripheral for Dma2Peripheral {
    fn transfer_mode(&self) -> TransferMode {
        TransferMode::Fifo(FifoSize::Full)
    }

    fn data_width(&self) -> (Msize, Psize) {
        (Msize(Size::Byte), Psize(Size::Byte))
    }

    fn channel_id(&self) -> ChannelId {
        match self {
            // USART1_TX Stream 7, Channel 4
            Dma2Peripheral::USART1_TX => ChannelId::Channel4,
            // USART1_RX Stream 5, Channel 4
            Dma2Peripheral::USART1_RX => ChannelId::Channel4,
        }
    }

    fn direction(&self) -> Direction {
        match self {
            Dma2Peripheral::USART1_TX => Direction::MemoryToPeripheral,
            Dma2Peripheral::USART1_RX => Direction::PeripheralToMemory,
        }
    }

    fn address(&self) -> u32 {
        match self {
            Dma2Peripheral::USART1_TX => usart::get_address_dr(usart::USART1_BASE),
            Dma2Peripheral::USART1_RX => usart::get_address_dr(usart::USART1_BASE),
        }
    }
}

pub fn new_dma2_stream<'a>(dma: &'a Dma2) -> [Stream<'a, Dma2<'a>>; 8] {
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

pub struct Dma2<'a> {
    registers: StaticRef<DmaRegisters>,
    clock: DmaClock<'a>,
}

impl<'a> Dma2<'a> {
    pub const fn new(rcc: &'a rcc::Rcc) -> Dma2 {
        Dma2 {
            registers: DMA2_BASE,
            clock: DmaClock(rcc::PeripheralClock::new(
                rcc::PeripheralClockType::AHB1(rcc::HCLK1::DMA2),
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

impl<'a> StreamServer<'a> for Dma2<'a> {
    type Peripheral = Dma2Peripheral;

    fn registers(&self) -> &DmaRegisters {
        &*self.registers
    }
}
