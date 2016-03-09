use common::{RingBuffer,Queue};
use nvic;
use peripheral_interrupts::NvicIdx;

const IQ_SIZE: usize = 100;
static mut IQ_BUF : [NvicIdx; IQ_SIZE] = [NvicIdx::POWER_CLOCK; IQ_SIZE];
pub static mut INTERRUPT_QUEUE : Option<RingBuffer<'static, NvicIdx>> = None;

pub struct Nrf51822;

impl Nrf51822 {
    pub unsafe fn new() -> Nrf51822 {
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));
        Nrf51822
    }

    pub unsafe fn service_pending_interrupts(&mut self) {
        INTERRUPT_QUEUE.as_mut().unwrap().dequeue().map(|interrupt| {
            match interrupt {
                _ => {}
            }
            nvic::enable(interrupt);
        });
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        INTERRUPT_QUEUE.as_mut().unwrap().has_elements()
    }
}
