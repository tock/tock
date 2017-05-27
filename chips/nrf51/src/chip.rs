use aes;
use gpio;
use kernel;
use kernel::common::{RingBuffer, Queue};
use nvic;
use peripheral_interrupts::NvicIdx;
use rtc;
use temperature;
use timer;
use trng;
use uart;

const IQ_SIZE: usize = 100;
static mut IQ_BUF: [NvicIdx; IQ_SIZE] = [NvicIdx::POWER_CLOCK; IQ_SIZE];
pub static mut INTERRUPT_QUEUE: Option<RingBuffer<'static, NvicIdx>> = None;

pub struct NRF51(());

impl NRF51 {
    pub unsafe fn new() -> NRF51 {
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));
        NRF51(())
    }
}


impl kernel::Chip for NRF51 {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &self.0
    }

    fn systick(&self) -> &Self::SysTick {
        &self.0
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            INTERRUPT_QUEUE.as_mut().unwrap().dequeue().map(|interrupt| {
                match interrupt {
                    NvicIdx::RTC1 => rtc::RTC.handle_interrupt(),
                    NvicIdx::GPIOTE => gpio::PORT.handle_interrupt(),
                    NvicIdx::TIMER0 => timer::TIMER0.handle_interrupt(),
                    NvicIdx::TIMER1 => timer::ALARM1.handle_interrupt(),
                    NvicIdx::TIMER2 => timer::TIMER2.handle_interrupt(),
                    NvicIdx::UART0 => uart::UART0.handle_interrupt(),
                    NvicIdx::TEMP => temperature::TEMP.handle_interrupt(),
                    NvicIdx::RNG => trng::TRNG.handle_interrupt(),
                    NvicIdx::ECB => aes::AESECB.handle_interrupt(),
                    _ => {}
                }
                nvic::enable(interrupt);
            });
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { INTERRUPT_QUEUE.as_mut().unwrap().has_elements() }
    }
}
