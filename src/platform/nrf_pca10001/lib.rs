#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]

extern crate drivers;
extern crate hil;
extern crate nrf51822;
extern crate support;
extern crate process;
extern crate common;


use core::intrinsics::{volatile_load, volatile_store};

use hil::Controller;
use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use hil::gpio::GPIOPin;
use nrf51822::gpio::PORT;
use nrf51822::uart;

use drivers::timer::TimerDriver;
use nrf51822::timer::TimerAlarm;
use nrf51822::timer::ALARM1;


pub mod systick;

pub struct Firestorm {
    chip: nrf51822::chip::Nrf51822,
    gpio: &'static drivers::gpio::GPIO<'static, nrf51822::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, TimerAlarm>>,
}

pub struct DummyMPU;

impl DummyMPU {
    pub fn set_mpu(&mut self, _: u32, _: u32, _: u32, _: bool, _: u32) {
    }
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn mpu(&mut self) -> DummyMPU {
        DummyMPU
    }

    #[inline(never)]
    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {
        match driver_num {
            1 => f(Some(self.gpio)),
           // 3 => f(Some(self.timer)),
            _ => f(None)
        }
    }
}

macro_rules! static_init {
   ($V:ident : $T:ty = $e:expr) => {
        let $V : &mut $T = {
            // Waiting out for size_of to be available at compile-time to avoid
            // hardcoding an abitrary large size...
            static mut BUF : [u8; 1024] = [0; 1024];
            let mut tmp : &mut $T = mem::transmute(&mut BUF);
            *tmp = $e;
            tmp
        };
   }
}

//let nrf_uart = nrf51822::uart::UART::new()
//nrf_uart.init();

//nrf_uart.enable_tx();
//nrf_uart.set_baud_rate(9600); //doesnt matter. I have hard coded the reg value in init
//while(nrf_uart.tx_ready()) {

//    loop{
//    nrf_uart.send_byte('+' as u8);
//    }
//}

#[inline(never)]
	
pub unsafe fn init<'a>() -> &'a mut Firestorm {

    use core::mem;

//	let mut nrf_uart = uart::UART::new();

    //let mut uart_params = hiluart::UARTParams {baud_rate : 115200, data_bits:1, parity: Odd, mode: Normal};
//	nrf_uart.init(9600);
//    nrf_uart.send_byte('h' as u8);
    //let reg : uart::Registers = uart::Registers {starttx : 0x40002008};
	//volatile_store(reg.starttx as *mut usize, 1); //starttx
	//volatile_store(0x40002500 as *mut usize, 100); //enable
	//volatile_store(0x40002524 as *mut usize, 0x00275000); //baudrate
	//volatile_store(0x40002508 as *mut usize, 8); //pselrts
	//volatile_store(0x4000250C as *mut usize, 9); //pseltxd
	//volatile_store(0x40002510 as *mut usize, 10); //pselcts
	//volatile_store(0x40002514 as *mut usize, 11); //pselrxd
	//volatile_store(0x4000251C as *mut usize, 0111); //txd

	
    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];

    static_init!(gpio_pins : [&'static nrf51822::gpio::GPIOPin; 10] = [

            &nrf51822::gpio::PORT[18], // LED_0
            &nrf51822::gpio::PORT[19], // LED_1
            &nrf51822::gpio::PORT[0], // Top left header on EK board
            &nrf51822::gpio::PORT[1], //   |
            &nrf51822::gpio::PORT[2], //   V 
            &nrf51822::gpio::PORT[3], // 
            &nrf51822::gpio::PORT[4], //
            &nrf51822::gpio::PORT[5], // 
            &nrf51822::gpio::PORT[6], // 
            &nrf51822::gpio::PORT[7], // 
            ]);
    static_init!(gpio : drivers::gpio::GPIO<'static, nrf51822::gpio::GPIOPin> =
                 drivers::gpio::GPIO::new(gpio_pins));
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }


    let alarm = &nrf51822::timer::ALARM1;
    static_init!(mux_alarm : MuxAlarm<'static, TimerAlarm> = MuxAlarm::new(&ALARM1));
    alarm.set_client(mux_alarm);

    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, TimerAlarm> =
                                  VirtualMuxAlarm::new(mux_alarm));
    static_init!(timer : TimerDriver<'static, VirtualMuxAlarm<'static, TimerAlarm>> =
                         TimerDriver::new(virtual_alarm1, process::Container::create()));
    virtual_alarm1.set_client(timer);


    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: nrf51822::chip::Nrf51822::new(),
        gpio: gpio,
        timer: timer,
    };

    systick::reset();
    systick::enable(true);
    firestorm
}


use core::fmt::Arguments;
#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern fn rust_begin_unwind(_args: &Arguments,
    _file: &'static str, _line: usize) -> ! {
    use support::nop;
    use hil::gpio::GPIOPin;


    let led0 = &nrf51822::gpio::PORT[18];
    let led1 = &nrf51822::gpio::PORT[19];

    led0.enable_output();
    led1.enable_output();
    loop {
        for _ in 0..100000 {
            led0.set();
            led1.set();
            nop();
        }
        for _ in 0..100000 {
            led0.clear();
            led1.clear();
            nop();
        }
    }
}


