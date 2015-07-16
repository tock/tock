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

pub static mut ADC: Option<adc::Adc> = None;
pub static mut LED: Option<hil::led::LedHigh> = None;
pub static mut PINC10: sam4l::gpio::GPIOPin = sam4l::gpio::GPIOPin {pin: sam4l::gpio::Pin::PC10};
pub static mut ADCM: Option<AdcMux> = None;

pub struct TestRequest {
  chan: u8
}

impl hil::adc::ImplRequest for TestRequest {
  fn read_done(&mut self, val: u16) {
    unsafe {
    }
  }
  fn channel(&self) -> u8 {
    self.chan
  }
}

pub static mut REQ: TestRequest = TestRequest {
  chan: 0
};

pub struct MuxRequest;

impl hil::adc::Request for MuxRequest {
  fn read_done(&'static mut self, val: u16, req: &'static mut hil::adc::Request) {
  }
}
pub static mut MREQ: MuxRequest = MuxRequest;

pub static mut FIRESTORM : Option<Firestorm> = None;

pub struct Firestorm {
    chip: &'static mut chip::Sam4l,
    console: drivers::console::Console<usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    led: &'static mut hil::led::Led,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>
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

pub static MREQI: Option<&'static mut hil::adc::RequestInternal> = None;

pub unsafe fn init() -> &'static mut Firestorm {
    chip::CHIP = Some(chip::Sam4l::new());
    let chip = chip::CHIP.as_mut().unwrap();
    LED = Some(hil::led::LedHigh {pin: &mut PINC10}) ;
    
    FIRESTORM = Some(Firestorm {
        chip: chip,
        console: drivers::console::Console::new(&mut chip.usarts[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09]),
        led: LED.as_mut().unwrap()
        tmp006: drivers::tmp006::TMP006::new(&mut chip.i2c[2]),
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

    ADC = Some(sam4l::adc::Adc::new());
    let adc = ADC.as_mut().unwrap();
    adc.initialize();
    REQ.chan = 1;
//    adc.sample(&mut REQ);

    FIRESTORM.as_mut().unwrap().led.init();
    firestorm.console.initialize();
    firestorm
}

