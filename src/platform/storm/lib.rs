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
use hil::adc::AdcInternal;
use hil::Controller;
use sam4l::*;

pub static mut ADC  : Option<adc::Adc> = None;

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

pub struct TestRequest {
    val: u32
}

impl hil::adc::Request for TestRequest {
  fn sample_done(&'static mut self, val: u16) {
      unsafe {
        let fs: &'static mut Firestorm = FIRESTORM.as_mut().unwrap();
        fs.console.putstr("ADC reading: ");
        print_val(fs, val as u32);
        fs.console.putstr("\n");
        let adc = ADC.as_mut().unwrap();
        adc.sample(1, self);
        let led: &'static mut hil::gpio::GPIOPin = &mut fs.chip.pc10;
        led.toggle();
      }
  }
}

pub static mut REQ: TestRequest = TestRequest {
    val: 0
};


pub static mut FIRESTORM : Option<Firestorm> = None;

pub struct Firestorm {
    chip: &'static mut chip::Sam4l,
    console: drivers::console::Console<sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
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

pub unsafe fn init() -> &'static mut Firestorm {
    chip::CHIP = Some(chip::Sam4l::new());
    let chip = chip::CHIP.as_mut().unwrap();

    FIRESTORM = Some(Firestorm {
        chip: chip,
        console: drivers::console::Console::new(&mut chip.usarts[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09]),
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

    // LED pin is an output
    let led: &'static mut hil::gpio::GPIOPin = &mut chip.pc10;
    led.enable_output();

    firestorm.console.initialize();
    led.toggle();
    // Configure pin to be ADC (channel 1)
    chip.pa21.configure(Some(sam4l::gpio::PeripheralFunction::A));
    ADC = Some(sam4l::adc::Adc::new());
    let adc = ADC.as_mut().unwrap();
    adc.initialize();
    adc.sample(1, &mut REQ);


    firestorm.console.putstr("Booting.\n");
    firestorm
}

