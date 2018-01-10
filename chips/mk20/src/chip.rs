use kernel::Chip;
use kernel::common::{RingBuffer, Queue};
use nvic;
use pit;
use spi;
use gpio;
use uart;

pub struct MK20 {
    pub mpu: (),
    pub systick: (),
}

// Interrupt queue allocation
const IQ_SIZE: usize = 100;
static mut IQ_BUF: [nvic::NvicIdx; IQ_SIZE] = [nvic::NvicIdx::DMA0; IQ_SIZE];
pub static mut INTERRUPT_QUEUE: Option<RingBuffer<'static, nvic::NvicIdx>> = None;

impl MK20 {
    pub unsafe fn new() -> MK20 {
        // Initialize interrupt queue
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));

        // Set up DMA channels
        // TODO: implement

        MK20 {
            mpu: (),
            systick: ()
        }
    }
}

impl Chip for MK20 {
    type MPU = ();
    type SysTick = ();

    fn service_pending_interrupts(&mut self) {
        use nvic::NvicIdx::*;
        unsafe {
            let iq = INTERRUPT_QUEUE.as_mut().unwrap();
            while let Some(interrupt) = iq.dequeue() {
                match interrupt {
                    PCMA => gpio::PA.handle_interrupt(),
                    PCMB => gpio::PB.handle_interrupt(),
                    PCMC => gpio::PC.handle_interrupt(),
                    PCMD => gpio::PD.handle_interrupt(),
                    PCME => gpio::PE.handle_interrupt(),
                    PIT2 => pit::PIT.handle_interrupt(),
                    SPI0 => spi::SPI0.handle_interrupt(),
                    SPI1 => spi::SPI1.handle_interrupt(),
                    SPI2 => spi::SPI2.handle_interrupt(),
                    UART0 => uart::UART0.handle_interrupt(),
                    UART1 => uart::UART1.handle_interrupt(),
                    _ => {}
                }

                nvic::enable(interrupt);
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { INTERRUPT_QUEUE.as_mut().unwrap().has_elements() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }
}
