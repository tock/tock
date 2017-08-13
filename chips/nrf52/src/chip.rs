use kernel;
use kernel::common::{RingBuffer, Queue};
use nrf5x;
use nrf5x::peripheral_interrupts::NvicIdx;
use radio;
use uart;

const IQ_SIZE: usize = 100;
static mut IQ_BUF: [NvicIdx; IQ_SIZE] = [NvicIdx::POWER_CLOCK; IQ_SIZE];
pub static mut INTERRUPT_QUEUE: Option<RingBuffer<'static, NvicIdx>> = None;

pub struct NRF52(());

impl NRF52 {
    pub unsafe fn new() -> NRF52 {
        INTERRUPT_QUEUE = Some(RingBuffer::new(&mut IQ_BUF));
        NRF52(())
    }
}


impl kernel::Chip for NRF52 {
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
                    NvicIdx::ECB => nrf5x::aes::AESECB.handle_interrupt(),
                    NvicIdx::GPIOTE => nrf5x::gpio::PORT.handle_interrupt(),
                    NvicIdx::RADIO => radio::RADIO.handle_interrupt(),
                    NvicIdx::RNG => nrf5x::trng::TRNG.handle_interrupt(),
                    NvicIdx::RTC1 => nrf5x::rtc::RTC.handle_interrupt(),
                    NvicIdx::TEMP => nrf5x::temperature::TEMP.handle_interrupt(),
                    NvicIdx::TIMER0 => nrf5x::timer::TIMER0.handle_interrupt(),
                    NvicIdx::TIMER1 => nrf5x::timer::ALARM1.handle_interrupt(),
                    NvicIdx::TIMER2 => nrf5x::timer::TIMER2.handle_interrupt(),
                    NvicIdx::UART0 => uart::UART0.handle_interrupt(),
                    _ => debug!("NvicIdx not supported by Tock\r\n"),
                }
                nrf5x::nvic::enable(interrupt);
            });
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { INTERRUPT_QUEUE.as_mut().unwrap().has_elements() }
    }
}
