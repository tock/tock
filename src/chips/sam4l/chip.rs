use common::{RingBuffer,Queue};
use ast;
//use adc;
use dma;
use nvic;
use usart;
use gpio;
use spi;

pub struct Sam4l;

const IQ_SIZE: usize = 100;
static mut IQ_BUF : [nvic::NvicIdx; IQ_SIZE] =
    [nvic::NvicIdx::HFLASHC; IQ_SIZE];
pub static mut INTERRUPT_QUEUE : Option<RingBuffer<'static, nvic::NvicIdx>> = None;


impl Sam4l {
    #[inline(never)]
    pub unsafe fn new() -> Sam4l {
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));
        usart::USART3.set_dma(&mut dma::DMAChannels[0]);
        dma::DMAChannels[0].client = Some(&mut usart::USART3);
        spi::SPI.set_dma(&mut dma::DMAChannels[1], &mut dma::DMAChannels[2]);
        dma::DMAChannels[1].client = Some(&mut spi::SPI);
        dma::DMAChannels[2].client = Some(&mut spi::SPI);
        Sam4l
    }

    pub unsafe fn service_pending_interrupts(&mut self) {
        use nvic::NvicIdx::*;
        INTERRUPT_QUEUE.as_mut().unwrap().dequeue().map(|interrupt| {
            match interrupt {
                ASTALARM => ast::AST.handle_interrupt(),
                USART3   => usart::USART3.handle_interrupt(),

                PDCA0   => dma::DMAChannels[0].handle_interrupt(),
                PDCA1   => dma::DMAChannels[1].handle_interrupt(),
                PDCA2   => dma::DMAChannels[2].handle_interrupt(),

                GPIO0 => gpio::PA.handle_interrupt(),
                GPIO1 => gpio::PA.handle_interrupt(),
                GPIO2 => gpio::PA.handle_interrupt(),
                GPIO3 => gpio::PA.handle_interrupt(),
                GPIO4 => gpio::PB.handle_interrupt(),
                GPIO5 => gpio::PB.handle_interrupt(),
                GPIO6 => gpio::PB.handle_interrupt(),
                GPIO7 => gpio::PB.handle_interrupt(),
                GPIO8 => gpio::PC.handle_interrupt(),
                GPIO9 => gpio::PC.handle_interrupt(),
                GPIO10 => gpio::PC.handle_interrupt(),
                GPIO11 => gpio::PC.handle_interrupt(),

                //NvicIdx::ADCIFE   => self.adc.handle_interrupt(),
                _ => {}
            }
            nvic::enable(interrupt);
       });
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        INTERRUPT_QUEUE.as_mut().unwrap().has_elements()
    }
}

