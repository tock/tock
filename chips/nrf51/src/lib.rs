#![feature(asm,concat_idents,const_fn,const_cell_new)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;
extern crate nrf5x;
use kernel::common::Queue;

extern "C" {
    pub fn init();
}

mod peripheral_registers;

pub mod chip;
pub mod uart;
pub use chip::NRF51;
pub mod radio;


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn ECB_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::ECB);
    chip::INTERRUPT_QUEUE.as_mut()
        .unwrap()
        .enqueue(nrf5x::peripheral_interrupts::NvicIdx::ECB);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn RNG_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::RNG);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::RNG);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn GPIOTE_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::GPIOTE);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::GPIOTE);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn TEMP_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::TEMP);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::TEMP);
}


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn TIMER0_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::TIMER0);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::TIMER0);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn TIMER1_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::TIMER1);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::TIMER1);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn TIMER2_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::TIMER2);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::TIMER2);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn RTC1_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::RTC1);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nrf5x::peripheral_interrupts::NvicIdx::RTC1);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn RADIO_Handler() {
    nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::RADIO);
    chip::INTERRUPT_QUEUE.as_mut()
        .unwrap()
        .enqueue(nrf5x::peripheral_interrupts::NvicIdx::RADIO);
}
