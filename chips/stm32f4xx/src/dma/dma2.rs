use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::StaticRef;
use kernel::ClockInterface;

use crate::nvic;
use crate::rcc;
use crate::usart;

use super::*;

const DMA2_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x40026400 as *const DmaRegisters) };

/// List of peripherals managed by DMA2
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Copy, Clone, PartialEq)]
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

    pub fn get_stream<'a>(&self) -> &'a Stream<'static> {
        unsafe { &DMA2_STREAM[usize::from(StreamId::from(*self) as u8)] }
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

pub struct Stream<'a> {
    streamid: StreamId,
    client: OptionalCell<&'a dyn StreamClient>,
    buffer: TakeCell<'static, [u8]>,
    peripheral: OptionalCell<Dma2Peripheral>,
}

pub static mut DMA2_STREAM: [Stream<'static>; 8] = [
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
    fn transfer_done(&self, pid: Dma2Peripheral);
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

    pub fn setup(&self, pid: Dma2Peripheral) {
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
                Dma2Peripheral::USART1_TX => unsafe {
                    // USART1_TX Stream 7, Channel 4
                    DMA2.registers
                        .s7cr
                        .modify(S7CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma2Peripheral::USART1_RX => unsafe {
                    // USART1_RX Stream 5, Channel 4
                    DMA2.registers
                        .s5cr
                        .modify(S5CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
            }
        });
    }

    fn set_direction(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma2Peripheral::USART1_TX => unsafe {
                    // USART1_TX Stream 7
                    DMA2.registers
                        .s7cr
                        .modify(S7CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma2Peripheral::USART1_RX => unsafe {
                    // USART1_RX Stream 5
                    DMA2.registers
                        .s5cr
                        .modify(S5CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
            }
        });
    }

    fn set_peripheral_address(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma2Peripheral::USART1_TX => unsafe {
                    // USART1_TX Stream 7
                    DMA2.registers.s7par.set(usart::USART1.get_address_dr());
                },
                Dma2Peripheral::USART1_RX => unsafe {
                    // USART1_RX Stream 5
                    DMA2.registers.s5par.set(usart::USART1.get_address_dr());
                },
            }
        });
    }

    fn set_peripheral_address_increment(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma2Peripheral::USART1_TX => unsafe {
                    // USART1_TX Stream 7
                    DMA2.registers.s7cr.modify(S7CR::PINC::CLEAR);
                },
                Dma2Peripheral::USART1_RX => unsafe {
                    // USART1_RX Stream 5
                    DMA2.registers.s5cr.modify(S5CR::PINC::CLEAR);
                },
            }
        });
    }

    fn set_memory_address(&self, buf_addr: u32) {
        self.peripheral.map(|pid| {
            match pid {
                Dma2Peripheral::USART1_TX => unsafe {
                    // USART1_TX Stream 7
                    DMA2.registers.s7m0ar.set(buf_addr);
                },
                Dma2Peripheral::USART1_RX => unsafe {
                    // USART1_RX Stream 5
                    DMA2.registers.s5m0ar.set(buf_addr);
                },
            }
        });
    }

    fn set_memory_address_increment(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma2Peripheral::USART1_TX => unsafe {
                    // USART1_TX Stream 7
                    DMA2.registers.s7cr.modify(S7CR::MINC::SET);
                },
                Dma2Peripheral::USART1_RX => unsafe {
                    // USART1_RX Stream 5
                    DMA2.registers.s5cr.modify(S5CR::MINC::SET);
                },
            }
        });
    }

    fn get_data_items(&self) -> u32 {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA2.registers.s0ndtr.get() },
            StreamId::Stream1 => unsafe { DMA2.registers.s1ndtr.get() },
            StreamId::Stream2 => unsafe { DMA2.registers.s2ndtr.get() },
            StreamId::Stream3 => unsafe { DMA2.registers.s3ndtr.get() },
            StreamId::Stream4 => unsafe { DMA2.registers.s4ndtr.get() },
            StreamId::Stream5 => unsafe { DMA2.registers.s5ndtr.get() },
            StreamId::Stream6 => unsafe { DMA2.registers.s6ndtr.get() },
            StreamId::Stream7 => unsafe { DMA2.registers.s7ndtr.get() },
        }
    }

    fn set_data_items(&self, data_items: u32) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA2.registers.s0ndtr.set(data_items);
            },
            StreamId::Stream1 => unsafe {
                DMA2.registers.s1ndtr.set(data_items);
            },
            StreamId::Stream2 => unsafe {
                DMA2.registers.s2ndtr.set(data_items);
            },
            StreamId::Stream3 => unsafe {
                DMA2.registers.s3ndtr.set(data_items);
            },
            StreamId::Stream4 => unsafe {
                DMA2.registers.s4ndtr.set(data_items);
            },
            StreamId::Stream5 => unsafe {
                DMA2.registers.s5ndtr.set(data_items);
            },
            StreamId::Stream6 => unsafe {
                DMA2.registers.s6ndtr.set(data_items);
            },
            StreamId::Stream7 => unsafe {
                DMA2.registers.s7ndtr.set(data_items);
            },
        }
    }

    fn set_data_width_for_peripheral(&self) {
        self.peripheral.map(|pid| match pid {
            Dma2Peripheral::USART1_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma2Peripheral::USART1_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
        });
    }

    fn stream_set_data_width(&self, msize: Msize, psize: Psize) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA2.registers.s0cr.modify(S0CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s0cr.modify(S0CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream1 => unsafe {
                DMA2.registers.s1cr.modify(S1CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s1cr.modify(S1CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream2 => unsafe {
                DMA2.registers.s2cr.modify(S2CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s2cr.modify(S2CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream3 => unsafe {
                DMA2.registers.s3cr.modify(S3CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s3cr.modify(S3CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream4 => unsafe {
                DMA2.registers.s4cr.modify(S4CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s4cr.modify(S4CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream5 => unsafe {
                DMA2.registers.s5cr.modify(S5CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s5cr.modify(S5CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream6 => unsafe {
                DMA2.registers.s6cr.modify(S6CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s6cr.modify(S6CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream7 => unsafe {
                DMA2.registers.s7cr.modify(S7CR::PSIZE.val(psize.0 as u32));
                DMA2.registers.s7cr.modify(S7CR::MSIZE.val(msize.0 as u32));
            },
        }
    }

    fn set_transfer_mode_for_peripheral(&self) {
        self.peripheral.map(|pid| match pid {
            Dma2Peripheral::USART1_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma2Peripheral::USART1_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
        });
    }

    fn stream_set_transfer_mode(&self, transfer_mode: TransferMode) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s0fcr.modify(S0FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s0fcr.modify(S0FCR::DMDIS::SET);
                        DMA2.registers.s0fcr.modify(S0FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream1 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s1fcr.modify(S1FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s1fcr.modify(S1FCR::DMDIS::SET);
                        DMA2.registers.s1fcr.modify(S1FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream2 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s2fcr.modify(S2FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s2fcr.modify(S2FCR::DMDIS::SET);
                        DMA2.registers.s2fcr.modify(S2FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream3 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s3fcr.modify(S3FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s3fcr.modify(S3FCR::DMDIS::SET);
                        DMA2.registers.s3fcr.modify(S3FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream4 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s4fcr.modify(S4FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s4fcr.modify(S4FCR::DMDIS::SET);
                        DMA2.registers.s4fcr.modify(S4FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream5 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s5fcr.modify(S5FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s5fcr.modify(S5FCR::DMDIS::SET);
                        DMA2.registers.s5fcr.modify(S5FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream6 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s6fcr.modify(S6FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s6fcr.modify(S6FCR::DMDIS::SET);
                        DMA2.registers.s6fcr.modify(S6FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream7 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA2.registers.s7fcr.modify(S7FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA2.registers.s7fcr.modify(S7FCR::DMDIS::SET);
                        DMA2.registers.s7fcr.modify(S7FCR::FTH.val(s as u32));
                    }
                }
            },
        }
    }

    fn enable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA2.registers.s0cr.modify(S0CR::EN::SET) },
            StreamId::Stream1 => unsafe { DMA2.registers.s1cr.modify(S1CR::EN::SET) },
            StreamId::Stream2 => unsafe { DMA2.registers.s2cr.modify(S2CR::EN::SET) },
            StreamId::Stream3 => unsafe { DMA2.registers.s3cr.modify(S3CR::EN::SET) },
            StreamId::Stream4 => unsafe { DMA2.registers.s4cr.modify(S4CR::EN::SET) },
            StreamId::Stream5 => unsafe { DMA2.registers.s5cr.modify(S5CR::EN::SET) },
            StreamId::Stream6 => unsafe { DMA2.registers.s6cr.modify(S6CR::EN::SET) },
            StreamId::Stream7 => unsafe { DMA2.registers.s7cr.modify(S7CR::EN::SET) },
        }
    }

    fn disable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA2.registers.s0cr.modify(S0CR::EN::CLEAR) },
            StreamId::Stream1 => unsafe { DMA2.registers.s1cr.modify(S1CR::EN::CLEAR) },
            StreamId::Stream2 => unsafe { DMA2.registers.s2cr.modify(S2CR::EN::CLEAR) },
            StreamId::Stream3 => unsafe { DMA2.registers.s3cr.modify(S3CR::EN::CLEAR) },
            StreamId::Stream4 => unsafe { DMA2.registers.s4cr.modify(S4CR::EN::CLEAR) },
            StreamId::Stream5 => unsafe { DMA2.registers.s5cr.modify(S5CR::EN::CLEAR) },
            StreamId::Stream6 => unsafe { DMA2.registers.s6cr.modify(S6CR::EN::CLEAR) },
            StreamId::Stream7 => unsafe { DMA2.registers.s7cr.modify(S7CR::EN::CLEAR) },
        }
    }

    fn clear_transfer_complete_flag(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA2.registers.lifcr.write(LIFCR::CTCIF0::SET);
            },
            StreamId::Stream1 => unsafe {
                DMA2.registers.lifcr.write(LIFCR::CTCIF1::SET);
            },
            StreamId::Stream2 => unsafe {
                DMA2.registers.lifcr.write(LIFCR::CTCIF2::SET);
            },
            StreamId::Stream3 => unsafe {
                DMA2.registers.lifcr.write(LIFCR::CTCIF3::SET);
            },
            StreamId::Stream4 => unsafe {
                DMA2.registers.hifcr.write(HIFCR::CTCIF4::SET);
            },
            StreamId::Stream5 => unsafe {
                DMA2.registers.hifcr.write(HIFCR::CTCIF5::SET);
            },
            StreamId::Stream6 => unsafe {
                DMA2.registers.hifcr.write(HIFCR::CTCIF6::SET);
            },
            StreamId::Stream7 => unsafe {
                DMA2.registers.hifcr.write(HIFCR::CTCIF7::SET);
            },
        }
    }

    // We only interrupt on TC (Transfer Complete)
    fn interrupt_enable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA2.registers.s0cr.modify(S0CR::TCIE::SET) },
            StreamId::Stream1 => unsafe { DMA2.registers.s1cr.modify(S1CR::TCIE::SET) },
            StreamId::Stream2 => unsafe { DMA2.registers.s2cr.modify(S2CR::TCIE::SET) },
            StreamId::Stream3 => unsafe { DMA2.registers.s3cr.modify(S3CR::TCIE::SET) },
            StreamId::Stream4 => unsafe { DMA2.registers.s4cr.modify(S4CR::TCIE::SET) },
            StreamId::Stream5 => unsafe { DMA2.registers.s5cr.modify(S5CR::TCIE::SET) },
            StreamId::Stream6 => unsafe { DMA2.registers.s6cr.modify(S6CR::TCIE::SET) },
            StreamId::Stream7 => unsafe { DMA2.registers.s7cr.modify(S7CR::TCIE::SET) },
        }
    }

    // We only interrupt on TC (Transfer Complete)
    fn disable_interrupt(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA2.registers.s0cr.modify(S0CR::TCIE::CLEAR) },
            StreamId::Stream1 => unsafe { DMA2.registers.s1cr.modify(S1CR::TCIE::CLEAR) },
            StreamId::Stream2 => unsafe { DMA2.registers.s2cr.modify(S2CR::TCIE::CLEAR) },
            StreamId::Stream3 => unsafe { DMA2.registers.s3cr.modify(S3CR::TCIE::CLEAR) },
            StreamId::Stream4 => unsafe { DMA2.registers.s4cr.modify(S4CR::TCIE::CLEAR) },
            StreamId::Stream5 => unsafe { DMA2.registers.s5cr.modify(S5CR::TCIE::CLEAR) },
            StreamId::Stream6 => unsafe { DMA2.registers.s6cr.modify(S6CR::TCIE::CLEAR) },
            StreamId::Stream7 => unsafe { DMA2.registers.s7cr.modify(S7CR::TCIE::CLEAR) },
        }
    }
}

pub struct Dma2 {
    registers: StaticRef<DmaRegisters>,
    clock: Dma2Clock,
}

pub static mut DMA2: Dma2 = Dma2::new();

impl Dma2 {
    const fn new() -> Dma2 {
        Dma2 {
            registers: DMA2_BASE,
            clock: Dma2Clock(rcc::PeripheralClock::AHB1(rcc::HCLK1::DMA2)),
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

struct Dma2Clock(rcc::PeripheralClock);

impl ClockInterface for Dma2Clock {
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
