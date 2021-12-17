use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::StaticRef;
use kernel::ClockInterface;

use crate::nvic;
use crate::rcc;
use crate::spi;
use crate::usart;

use super::*;

const DMA1_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x40026000 as *const DmaRegisters) };

/// List of peripherals managed by DMA1
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Copy, Clone, PartialEq)]
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

    pub fn get_stream<'a>(&self) -> &'a Stream<'static> {
        unsafe { &DMA1_STREAM[usize::from(StreamId::from(*self) as u8)] }
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

pub struct Stream<'a> {
    streamid: StreamId,
    client: OptionalCell<&'a dyn StreamClient>,
    buffer: TakeCell<'static, [u8]>,
    peripheral: OptionalCell<Dma1Peripheral>,
}

pub static mut DMA1_STREAM: [Stream<'static>; 8] = [
    Stream::new(StreamId::Stream0),
    Stream::new(StreamId::Stream1),
    Stream::new(StreamId::Stream2),
    Stream::new(StreamId::Stream3),
    Stream::new(StreamId::Stream4),
    Stream::new(StreamId::Stream5),
    Stream::new(StreamId::Stream6),
    Stream::new(StreamId::Stream7),
];

pub trait StreamClient {
    fn transfer_done(&self, pid: Dma1Peripheral);
}

impl<'a> Stream<'a> {
    const fn new(streamid: StreamId) -> Stream<'a> {
        Stream {
            streamid: streamid,
            buffer: TakeCell::empty(),
            client: OptionalCell::empty(),
            peripheral: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'a dyn StreamClient) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.clear_transfer_complete_flag();

        self.client.map(|client| {
            self.peripheral.map(|pid| {
                client.transfer_done(*pid);
            });
        });
    }

    pub fn setup(&self, pid: Dma1Peripheral) {
        self.peripheral.set(pid);

        // Setup is called before interrupts are enabled on the NVIC
        self.disable_interrupt();
        self.disable();

        // The numbers below are from Section 1.2 of AN4031. It looks like these
        // settings can be set only once. Trying to set them again, seems to
        // generate a hard-fault even when the stream is disabled.
        //
        // 8
        self.set_transfer_mode_for_peripheral();
        // 9
        self.set_data_width_for_peripheral();
    }

    pub fn do_transfer(&self, buf: &'static mut [u8], len: usize) {
        self.disable_interrupt();

        // The numbers below are from Section 1.2 of AN4031
        //
        // NOTE: We only clear TC flag here. Trying to clear any other flag,
        //       generates a hard-fault
        // 1
        self.disable();
        self.clear_transfer_complete_flag();
        // 2
        self.set_peripheral_address();
        // 3
        self.set_memory_address(&buf[0] as *const u8 as u32);
        // 4
        self.set_data_items(len as u32);
        // 5
        self.set_channel();
        // 9
        self.set_direction();
        self.set_peripheral_address_increment();
        self.set_memory_address_increment();
        self.interrupt_enable();
        // 10
        self.enable();

        // NOTE: We still have to enable DMA on the peripheral side
        self.buffer.replace(buf);
    }

    pub fn abort_transfer(&self) -> (Option<&'static mut [u8]>, u32) {
        self.disable_interrupt();

        self.disable();

        (self.buffer.take(), self.get_data_items())
    }

    pub fn return_buffer(&self) -> Option<&'static mut [u8]> {
        self.buffer.take()
    }

    fn set_channel(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_RX Stream 7, Channel 0
                    DMA1.registers
                        .s7cr
                        .modify(S7CR::CHSEL.val(ChannelId::Channel0 as u32));
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6, Channel 4
                    DMA1.registers
                        .s6cr
                        .modify(S6CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5, Channel 4
                    DMA1.registers
                        .s5cr
                        .modify(S5CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3, Channel 4
                    DMA1.registers
                        .s3cr
                        .modify(S3CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2, Channel 0
                    DMA1.registers
                        .s2cr
                        .modify(S2CR::CHSEL.val(ChannelId::Channel0 as u32));
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1, Channel 4
                    DMA1.registers
                        .s1cr
                        .modify(S1CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
            }
        });
    }

    fn set_direction(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers
                        .s7cr
                        .modify(S7CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers
                        .s6cr
                        .modify(S6CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers
                        .s5cr
                        .modify(S5CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers
                        .s3cr
                        .modify(S3CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers
                        .s2cr
                        .modify(S2CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers
                        .s1cr
                        .modify(S1CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
            }
        });
    }

    fn set_peripheral_address(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7par.set(spi::SPI3.get_address_dr());
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6par.set(usart::USART2.get_address_dr());
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5par.set(usart::USART2.get_address_dr());
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3par.set(usart::USART3.get_address_dr());
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2par.set(spi::SPI3.get_address_dr());
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1par.set(usart::USART3.get_address_dr());
                },
            }
        });
    }

    fn set_peripheral_address_increment(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7cr.modify(S7CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6cr.modify(S6CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5cr.modify(S5CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3cr.modify(S3CR::PINC::CLEAR);
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2cr.modify(S2CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1cr.modify(S1CR::PINC::CLEAR);
                },
            }
        });
    }

    fn set_memory_address(&self, buf_addr: u32) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3m0ar.set(buf_addr);
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1m0ar.set(buf_addr);
                },
            }
        });
    }

    fn set_memory_address_increment(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7cr.modify(S7CR::MINC::SET);
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6cr.modify(S6CR::MINC::SET);
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5cr.modify(S5CR::MINC::SET);
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3cr.modify(S3CR::MINC::SET);
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2cr.modify(S2CR::MINC::SET);
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1cr.modify(S1CR::MINC::SET);
                },
            }
        });
    }

    fn get_data_items(&self) -> u32 {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0ndtr.get() },
            StreamId::Stream1 => unsafe { DMA1.registers.s1ndtr.get() },
            StreamId::Stream2 => unsafe { DMA1.registers.s2ndtr.get() },
            StreamId::Stream3 => unsafe { DMA1.registers.s3ndtr.get() },
            StreamId::Stream4 => unsafe { DMA1.registers.s4ndtr.get() },
            StreamId::Stream5 => unsafe { DMA1.registers.s5ndtr.get() },
            StreamId::Stream6 => unsafe { DMA1.registers.s6ndtr.get() },
            StreamId::Stream7 => unsafe { DMA1.registers.s7ndtr.get() },
        }
    }

    fn set_data_items(&self, data_items: u32) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA1.registers.s0ndtr.set(data_items);
            },
            StreamId::Stream1 => unsafe {
                DMA1.registers.s1ndtr.set(data_items);
            },
            StreamId::Stream2 => unsafe {
                DMA1.registers.s2ndtr.set(data_items);
            },
            StreamId::Stream3 => unsafe {
                DMA1.registers.s3ndtr.set(data_items);
            },
            StreamId::Stream4 => unsafe {
                DMA1.registers.s4ndtr.set(data_items);
            },
            StreamId::Stream5 => unsafe {
                DMA1.registers.s5ndtr.set(data_items);
            },
            StreamId::Stream6 => unsafe {
                DMA1.registers.s6ndtr.set(data_items);
            },
            StreamId::Stream7 => unsafe {
                DMA1.registers.s7ndtr.set(data_items);
            },
        }
    }

    fn set_data_width_for_peripheral(&self) {
        self.peripheral.map(|pid| match pid {
            Dma1Peripheral::SPI3_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART2_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART2_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART3_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::SPI3_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART3_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
        });
    }

    fn stream_set_data_width(&self, msize: Msize, psize: Psize) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA1.registers.s0cr.modify(S0CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s0cr.modify(S0CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream1 => unsafe {
                DMA1.registers.s1cr.modify(S1CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s1cr.modify(S1CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream2 => unsafe {
                DMA1.registers.s2cr.modify(S2CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s2cr.modify(S2CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream3 => unsafe {
                DMA1.registers.s3cr.modify(S3CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s3cr.modify(S3CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream4 => unsafe {
                DMA1.registers.s4cr.modify(S4CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s4cr.modify(S4CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream5 => unsafe {
                DMA1.registers.s5cr.modify(S5CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s5cr.modify(S5CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream6 => unsafe {
                DMA1.registers.s6cr.modify(S6CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s6cr.modify(S6CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream7 => unsafe {
                DMA1.registers.s7cr.modify(S7CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s7cr.modify(S7CR::MSIZE.val(msize.0 as u32));
            },
        }
    }

    fn set_transfer_mode_for_peripheral(&self) {
        self.peripheral.map(|pid| match pid {
            Dma1Peripheral::SPI3_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART2_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART2_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART3_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::SPI3_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART3_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
        });
    }

    fn stream_set_transfer_mode(&self, transfer_mode: TransferMode) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s0fcr.modify(S0FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s0fcr.modify(S0FCR::DMDIS::SET);
                        DMA1.registers.s0fcr.modify(S0FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream1 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s1fcr.modify(S1FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s1fcr.modify(S1FCR::DMDIS::SET);
                        DMA1.registers.s1fcr.modify(S1FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream2 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s2fcr.modify(S2FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s2fcr.modify(S2FCR::DMDIS::SET);
                        DMA1.registers.s2fcr.modify(S2FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream3 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s3fcr.modify(S3FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s3fcr.modify(S3FCR::DMDIS::SET);
                        DMA1.registers.s3fcr.modify(S3FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream4 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s4fcr.modify(S4FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s4fcr.modify(S4FCR::DMDIS::SET);
                        DMA1.registers.s4fcr.modify(S4FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream5 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s5fcr.modify(S5FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s5fcr.modify(S5FCR::DMDIS::SET);
                        DMA1.registers.s5fcr.modify(S5FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream6 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s6fcr.modify(S6FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s6fcr.modify(S6FCR::DMDIS::SET);
                        DMA1.registers.s6fcr.modify(S6FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream7 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s7fcr.modify(S7FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s7fcr.modify(S7FCR::DMDIS::SET);
                        DMA1.registers.s7fcr.modify(S7FCR::FTH.val(s as u32));
                    }
                }
            },
        }
    }

    fn enable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::EN::SET) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::EN::SET) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::EN::SET) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::EN::SET) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::EN::SET) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::EN::SET) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::EN::SET) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::EN::SET) },
        }
    }

    fn disable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::EN::CLEAR) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::EN::CLEAR) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::EN::CLEAR) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::EN::CLEAR) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::EN::CLEAR) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::EN::CLEAR) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::EN::CLEAR) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::EN::CLEAR) },
        }
    }

    fn clear_transfer_complete_flag(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF0::SET);
            },
            StreamId::Stream1 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF1::SET);
            },
            StreamId::Stream2 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF2::SET);
            },
            StreamId::Stream3 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF3::SET);
            },
            StreamId::Stream4 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF4::SET);
            },
            StreamId::Stream5 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF5::SET);
            },
            StreamId::Stream6 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF6::SET);
            },
            StreamId::Stream7 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF7::SET);
            },
        }
    }

    // We only interrupt on TC (Transfer Complete)
    fn interrupt_enable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::TCIE::SET) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::TCIE::SET) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::TCIE::SET) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::TCIE::SET) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::TCIE::SET) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::TCIE::SET) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::TCIE::SET) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::TCIE::SET) },
        }
    }

    // We only interrupt on TC (Transfer Complete)
    fn disable_interrupt(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::TCIE::CLEAR) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::TCIE::CLEAR) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::TCIE::CLEAR) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::TCIE::CLEAR) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::TCIE::CLEAR) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::TCIE::CLEAR) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::TCIE::CLEAR) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::TCIE::CLEAR) },
        }
    }
}

pub struct Dma1 {
    registers: StaticRef<DmaRegisters>,
    clock: Dma1Clock,
}

pub static mut DMA1: Dma1 = Dma1::new();

impl Dma1 {
    const fn new() -> Dma1 {
        Dma1 {
            registers: DMA1_BASE,
            clock: Dma1Clock(rcc::PeripheralClock::AHB1(rcc::HCLK1::DMA1)),
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
