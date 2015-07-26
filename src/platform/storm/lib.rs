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
use hil::adc::AdcImpl;
use hil::Controller;
use sam4l::*;
use hil::adc::AdcMux;

pub fn print_val(firestorm: &'static mut Firestorm, val: u32) {
  firestorm.console.putstr("0x");
  for x in 0..8 {
    let hdigit = (val >> ((7-x) * 4)) & 0xf;
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

pub struct TestTimer {
  firestorm: &'static mut Firestorm,
  led: &'static mut hil::led::Led
}

impl hil::timer::TimerCB for TestTimer {
  fn fired(&'static mut self,
           request: &'static mut hil::timer::TimerRequest,
           now: u32) {
    self.firestorm.led.toggle();  
    self.firestorm.console.putstr("tick: ");
    print_val(self.firestorm, now);
    self.firestorm.console.putstr(" -> ");
    print_val(self.firestorm, now + 2048);	
    self.firestorm.console.putstr("\n");
    unsafe {
//      let mytimer = &mut (FIRESTORM.as_mut().unwrap().timer) as &'static mut hil::timer::Timer;
      //      mytimer.oneshot(2048, request);
      }
  }
}

pub struct TestAdcRequest {
  chan: u8
}
impl hil::adc::ImplRequest for TestAdcRequest {
  fn read_done(&mut self, val: u16) {}
  fn channel(&self) -> u8 {
    self.chan
  }
}

pub struct TestAdcMuxRequest;
impl hil::adc::Request for TestAdcMuxRequest {
  fn read_done(&'static mut self, val: u16, req: &'static mut hil::adc::Request) {
  }
}

pub struct TestAlarmRequest {
  val: u8
}
impl hil::alarm::Request for TestAlarmRequest {
  fn fired(&'static mut self) {
    unsafe {
    let mut ast: &'static mut hil::alarm::Alarm = &mut FIRESTORM.as_mut().unwrap().chip.ast;
    FIRESTORM.as_mut().unwrap().led.toggle();
    let time = ast.now();
    let val = time % 10;
    let digit = match val {
       0 => "0 ",
       1 => "1 ",	
       2 => "2 ",
       3 => "3 ",
       4 => "4 ",
       5 => "5 ",
       6 => "6 ",
       7 => "7 ",
       8 => "8 ",
       9 => "9 ",
	 _ => "? "
      };
    FIRESTORM.as_mut().unwrap().console.putstr(digit);
    ast.set_alarm(time + 16000, self);
  }
  }
}

pub static mut PINC10: sam4l::gpio::GPIOPin = sam4l::gpio::GPIOPin {pin: sam4l::gpio::Pin::PC10};
pub static mut LED: Option<hil::led::LedHigh> = None;
pub static mut TIMER_REQUEST: Option<hil::timer::TimerRequest> = None;
pub static MREQI: Option<&'static mut hil::adc::RequestInternal> = None;
pub static mut TESTTIMER: Option<TestTimer> = None;
pub static mut FIRESTORM : Option<Firestorm> = None;
pub static mut ALARMREQ: TestAlarmRequest = TestAlarmRequest{val:0};

pub struct Firestorm {
    chip: &'static mut chip::Sam4l,
    console: drivers::console::Console<usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    led: &'static mut hil::led::Led,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>,
    timer: hil::timer::TimerMux
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

pub unsafe fn init() -> &'static mut Firestorm {
    chip::CHIP = Some(chip::Sam4l::new());
    let chip = chip::CHIP.as_mut().unwrap();
    LED = Some(hil::led::LedHigh {pin: &mut PINC10});
    let mut led = LED.as_mut().unwrap() as &mut hil::led::Led;
    let mut ast: &'static mut hil::alarm::Alarm  = &mut chip.ast;
    led.init();
    chip.ast.select_clock(sam4l::ast::Clock::ClockRCSys);
    chip.ast.set_prescalar(0);
    chip.ast.clear_alarm();
    FIRESTORM = Some(Firestorm {
        chip: chip,
        console: drivers::console::Console::new(&mut chip.usarts[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09]),
        led: led,
        tmp006: drivers::tmp006::TMP006::new(&mut chip.i2c[2]),
        timer: hil::timer::TimerMux::new(&mut chip.ast)
    });

    let firestorm : &'static mut Firestorm = FIRESTORM.as_mut().unwrap();

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

    FIRESTORM.as_mut().unwrap().led.init();
    firestorm.console.initialize();

    TESTTIMER = Some(TestTimer {firestorm: FIRESTORM.as_mut().unwrap(),
                                led: led});
    TIMER_REQUEST = Some(hil::timer::TimerRequest::new(TESTTIMER.as_mut().unwrap()));
    let mytimer = &mut (FIRESTORM.as_mut().unwrap().timer) as &'static mut hil::timer::Timer;
    let myrequest = TIMER_REQUEST.as_mut().unwrap() as &'static mut hil::timer::TimerRequest;


    // Make sure CLK_AST is enabled in the power manager
    // Internal clock must be active, enabled through SCIF
    // RCSYS always enabled
    chip.ast.enable();
    mytimer.repeat(2048, myrequest);
    //ast.set_alarm(ast.now() + 1000, &mut ALARMREQ);
    firestorm
}

