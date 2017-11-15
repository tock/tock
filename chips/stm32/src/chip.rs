use kernel::common::{RingBuffer, Queue};
use nvic::NvicIdx;

const IQ_SIZE: usize = 100;
static mut IQ_BUF: [NvicIdx; IQ_SIZE] = [NvicIdx::WWDG; IQ_SIZE];
pub static mut INTERRUPT_QUEUE: Option<RingBuffer<'static, NvicIdx>> = None;

pub unsafe fn init() {
    INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));
}

pub unsafe fn dequeue_interrupt() -> Option<NvicIdx> {
    INTERRUPT_QUEUE.as_mut().unwrap().dequeue()
}

pub unsafe fn has_pending_interrupts() -> bool {
    INTERRUPT_QUEUE.as_mut().unwrap().has_elements()
}
