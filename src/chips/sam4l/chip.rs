use common::{RingBuffer,Queue};
use ast;
//use adc;
use dma;
use nvic;
use usart;

pub struct Sam4l;

const IQ_SIZE: usize = 100;
static mut IQ_BUF : [nvic::NvicIdx; IQ_SIZE] =
    [nvic::NvicIdx::HFLASHC; IQ_SIZE];
pub static mut INTERRUPT_QUEUE : Option<RingBuffer<'static, nvic::NvicIdx>> = None;


impl Sam4l {
    #[inline(never)]
    pub unsafe fn new() -> Sam4l {
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));
        usart::USARTS[3].set_dma(&mut dma::DMAChannels[0]);
        dma::DMAChannels[0].client = Some(&mut usart::USARTS[3]);

        Sam4l
    }

    pub unsafe fn service_pending_interrupts(&mut self) {
        use nvic::NvicIdx::*;
        INTERRUPT_QUEUE.as_mut().unwrap().dequeue().map(|interrupt| {
            match interrupt {
                ASTALARM => ast::AST.handle_interrupt(),
                USART3   => usart::USARTS[3].handle_interrupt(),
                PDCA0   => dma::DMAChannels[0].handle_interrupt(),
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

