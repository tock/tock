#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(no_std)]

extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use hil::Controller;
use hil::timer::*;

pub struct Firestorm {
    chip: sam4l::chip::Sam4l,
    console: drivers::console::Console<'static, sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>,
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&mut hil::Driver>) -> R {

        f(match driver_num {
            0 => Some(&mut self.console),
            1 => Some(&mut self.gpio),
            2 => Some(&mut self.tmp006),
            _ => None
        })
    }
}

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;

    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];
    /* TODO(alevy): replace above line with this. Currently, over allocating to make development
     * easier, but should be obviated when `size_of` at compile time hits.
    static mut FIRESTORM_BUF : [u8; 192] = [0; 192];
    // Just test that FIRESTORM_BUF is correct size
    // (will throw compiler error if too large or small)
    let _ : Firestorm = mem::transmute(FIRESTORM_BUF);*/

    let ast = &mut sam4l::ast::AST;
    ast.select_clock(sam4l::ast::Clock::ClockRCSys);
    ast.set_prescalar(0);
    ast.clear_alarm();

    static mut TIMER_MUX : Option<TimerMux> = None;
    TIMER_MUX = Some(TimerMux::new(ast));

    let timer_mux = TIMER_MUX.as_mut().unwrap();

    static mut TIMER_REQUEST: TimerRequest = TimerRequest {
        next: None,
        is_active: false,
        is_repeat: false,
        when: 0,
        interval: 0,
        callback: None
    };
    let timer_request = &mut TIMER_REQUEST;

    let virtual_timer0 = VirtualTimer::new(timer_mux, timer_request);


    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: sam4l::chip::Sam4l::new(),
        console: drivers::console::Console::new(&mut sam4l::usart::USARTS[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut sam4l::gpio::PC[10], &mut sam4l::gpio::PC[19]
            , &mut sam4l::gpio::PC[13], &mut sam4l::gpio::PA[9]
            , &mut sam4l::gpio::PA[17], &mut sam4l::gpio::PC[20]
            , &mut sam4l::gpio::PA[19], &mut sam4l::gpio::PA[14]
            , &mut sam4l::gpio::PA[16], &mut sam4l::gpio::PA[13]
            , &mut sam4l::gpio::PA[11], &mut sam4l::gpio::PA[10]
            , &mut sam4l::gpio::PA[12], &mut sam4l::gpio::PC[09]]),
        tmp006: drivers::tmp006::TMP006::new(&mut sam4l::i2c::I2C2, virtual_timer0)
    };

    timer_request.callback = Some(&mut firestorm.tmp006);

    ast.configure(timer_mux);

    sam4l::usart::USARTS[3].configure(sam4l::usart::USARTParams {
        client: &mut firestorm.console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    sam4l::gpio::PB[09].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PB[10].configure(Some(sam4l::gpio::PeripheralFunction::A));

    sam4l::gpio::PA[21].configure(Some(sam4l::gpio::PeripheralFunction::E));
    sam4l::gpio::PA[22].configure(Some(sam4l::gpio::PeripheralFunction::E));

    firestorm.console.initialize();

    firestorm
}

