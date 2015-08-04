#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std)]

extern crate core;
extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use core::prelude::*;
use hil::Controller;
use hil::timer::*;
use hil::led::*;

pub static mut TIMER: TimerRequest = TimerRequest {
    next: None,
    is_active: false,
    is_repeat: false,
    when: 0,
    interval: 0,
    callback: None
};
pub static mut TIMERCB: Option<TestTimer> = None;

pub struct TestTimer {
    firestorm: &'static mut Firestorm
}

#[allow(unused_variables)]
impl TimerCB for TestTimer {
    fn fired(&'static mut self,
             request: &'static mut TimerRequest,
             now: u32) {
        self.firestorm.led.toggle();
        self.firestorm.console.putstr("Timer fired!\n");
    }
}

pub fn print_val(firestorm: &'static mut Firestorm, val: u32) {
     firestorm.console.putstr("0x");
     for x in 0..4 {
          let hdigit = (val >> ((3-x) * 4)) & 0xf;
          let char = match hdigit {
              0  => "0",
              1  => "1",
              2  => "2",
              3  => "3",
              4  => "4",
              5  => "5",
              6  => "6",
              7  => "7",
              8  => "8",
              9  => "9",
              10 => "A",
              11 => "B",
              12 => "C",
              13 => "D",
              14 => "E",
              15 => "F",
              _  => "?",
          };
          firestorm.console.putstr(char);
     }
}

pub static mut ADC: Option<sam4l::adc::Adc> = None;
pub static mut REQ: Option<TestRequest> = None;

pub struct TestRequest {
    firestorm: &'static mut Firestorm
}

impl hil::adc::Request for TestRequest {
  fn sample_done(&'static mut self, val: u16) {
      unsafe {
        self.firestorm.console.putstr("ADC reading: ");
        print_val(self.firestorm, val as u32);
        self.firestorm.console.putstr("\n");
        let adc = ADC.as_mut().unwrap() as &'static mut hil::adc::AdcInternal;
        adc.sample(1, self);
        self.firestorm.led.toggle();
      }
  }
}

pub struct Firestorm {
    chip: &'static mut sam4l::chip::Sam4l,
    console: drivers::console::Console<sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>,
    timer: TimerMux,
    led: LedHigh
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn with_driver<F, R>(&mut self, driver_num: usize, mut f: F) -> R where
            F: FnMut(Option<&mut hil::Driver>) -> R {

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

    static mut CHIP_BUF : [u8; 2048] = [0; 2048];
    /* TODO(alevy): replace above line with this. Currently, over allocating to make development
     * easier, but should be obviated when `size_of` at compile time hits.
    static mut CHIP_BUF : [u8; 924] = [0; 924];
    // Just test that CHIP_BUF is correct size
    // (will throw compiler error if too large or small)
    let _ : sam4l::chip::Sam4l = mem::transmute(CHIP_BUF);*/

    let chip : &'static mut sam4l::chip::Sam4l = mem::transmute(&mut CHIP_BUF);
    *chip = sam4l::chip::Sam4l::new();
    sam4l::chip::INTERRUPT_QUEUE = Some(&mut chip.queue);

    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];
    /* TODO(alevy): replace above line with this. Currently, over allocating to make development
     * easier, but should be obviated when `size_of` at compile time hits.
    static mut FIRESTORM_BUF : [u8; 172] = [0; 172];
    // Just test that FIRESTORM_BUF is correct size
    // (will throw compiler error if too large or small)
    let _ : Firestorm = mem::transmute(FIRESTORM_BUF);*/

    chip.ast.select_clock(sam4l::ast::Clock::ClockRCSys);
    chip.ast.set_prescalar(0);
    chip.ast.clear_alarm();

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: chip,
        console: drivers::console::Console::new(&mut chip.usarts[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09]),
        tmp006: drivers::tmp006::TMP006::new(&mut chip.i2c[2]),
        timer: hil::timer::TimerMux::new(&mut chip.ast),
        led: hil::led::LedHigh::new(&mut chip.pc10)
    };

    TIMERCB = Some(TestTimer {firestorm: firestorm});
    TIMER = TimerRequest::new(TIMERCB.as_mut().unwrap());

    firestorm.led.init();

    chip.usarts[3].configure(sam4l::usart::USARTParams {
        client: &mut firestorm.console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    chip.pb09.configure(Some(sam4l::gpio::PeripheralFunction::A));
    chip.pb10.configure(Some(sam4l::gpio::PeripheralFunction::A));

    chip.pa21.configure(Some(sam4l::gpio::PeripheralFunction::E));
    chip.pa22.configure(Some(sam4l::gpio::PeripheralFunction::E));

    firestorm.console.initialize();

    firestorm.timer.repeat(32768, &mut TIMER);

    // Configure pin to be ADC (channel 1)
    chip.pa21.configure(Some(sam4l::gpio::PeripheralFunction::A));
/*
    chip.scif.general_clock_enable(sam4l::scif::GenericClock::GCLK10,
                                   sam4l::scif::ClockSource::RCSYS);
    ADC = Some(sam4l::adc::Adc::new());
    let adc = ADC.as_mut().unwrap() as &'static mut hil::adc::AdcInternal;
    adc.initialize();
    REQ = Some(TestRequest { firestorm: firestorm});
    let req = REQ.as_mut().unwrap() as &'static mut hil::adc::Request;
    adc.sample(1, req);
    */
    firestorm.console.putstr("Booted. Requested ADC.\n");
    firestorm.led.on();
    firestorm
}
