#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(const_fn)]

extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use sam4l::pm;
use hil::Controller;
use hil::timer::*;
use hil::spi_master::*;

pub struct Firestorm {
    chip: sam4l::chip::Sam4l,
    console: &'static drivers::console::Console<'static, sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static hil::gpio::GPIOPin; 14]>,
    tmp006: &'static drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice>,
}

#[allow(unused_variables,dead_code)]
pub struct DummyCB {
  val: u8
}

pub static mut FLOP: bool = false;
pub static mut buf1: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub static mut buf2: [u8; 8] = [7, 6, 5, 4, 3, 2, 1, 0];

impl hil::spi_master::SpiCallback for DummyCB {
#[allow(unused_variables,dead_code)]
    fn read_write_done(&'static self) {
        unsafe {
            FLOP = !FLOP;
            let len: usize = buf1.len();
            if FLOP {
                sam4l::spi_dma::SPI.read_write_bytes(Some(&mut buf1), Some(&mut buf2), len);
            } else {
                sam4l::spi_dma::SPI.read_write_bytes(Some(&mut buf2), Some(&mut buf1), len);
            }
        }
    }
}

pub static mut SPICB: DummyCB = DummyCB{val: 0x55 as u8};

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    #[inline(never)]
    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {

        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(&self.gpio)),
            2 => f(Some(self.tmp006)),
            _ => f(None)
        }
    }
}


pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;

    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];
    static mut CONSOLE_BUF : [u8; 1024] = [0; 1024];
    static mut TIMER_BUF : [u8; 1024] = [0; 1024];
    static mut MUX_ALARM_BUF : [u8; 256] = [0; 256];
    static mut VIRT_ALARM_BUF : [u8; 256] = [0; 256];
    static mut TMP006_BUF : [u8; 1028] = [0; 1028];

    pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI)); 
    /* TODO(alevy): replace above line with this. Currently, over allocating to make development
     * easier, but should be obviated when `size_of` at compile time hits.
    static mut FIRESTORM_BUF : [u8; 192] = [0; 192];
    // Just test that FIRESTORM_BUF is correct size
    // (will throw compiler error if too large or small)
    let _ : Firestorm = mem::transmute(FIRESTORM_BUF);
    let _ : Firestorm = mem::transmute(CONSOLE_BUF);
    */

    let ast = &sam4l::ast::AST;
    ast.select_clock(sam4l::ast::Clock::ClockRCSys);
    ast.set_prescalar(0);
    ast.clear_alarm();
   
    let console : &mut drivers::console::Console<sam4l::usart::USART> = mem::transmute(&mut CONSOLE_BUF);
    *console = drivers::console::Console::new(&mut sam4l::usart::USART3);

    let mut mux_alarm : &mut MuxAlarm<'static, sam4l::ast::Ast> = mem::transmute(&mut MUX_ALARM_BUF);
    *mux_alarm = MuxAlarm::new(ast);
    ast.configure(mux_alarm);

    let mut virtual_alarm : &mut VirtualMuxAlarm<'static, sam4l::ast::Ast> = mem::transmute(&mut VIRT_ALARM_BUF);
    *virtual_alarm = VirtualMuxAlarm::new(mux_alarm);

    let mut timer : &mut SingleTimer<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>> = mem::transmute(&mut TIMER_BUF);
    *timer = SingleTimer::new(virtual_alarm);
    virtual_alarm.set_client(timer);

    let tmp006 : &mut drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice> = mem::transmute(&mut TMP006_BUF);
    *tmp006 = drivers::tmp006::TMP006::new(&sam4l::i2c::I2C2, timer);

    timer.set_client(tmp006);

    sam4l::usart::USART3.set_client(&*console);

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: sam4l::chip::Sam4l::new(),
        console: &*console,
        gpio: drivers::gpio::GPIO::new(
            [ &mut sam4l::gpio::PC[10], &mut sam4l::gpio::PC[19]
            , &mut sam4l::gpio::PC[13], &mut sam4l::gpio::PA[9]
            , &mut sam4l::gpio::PA[17], &mut sam4l::gpio::PC[20]
            , &mut sam4l::gpio::PA[19], &mut sam4l::gpio::PA[14]
            , &mut sam4l::gpio::PA[16], &mut sam4l::gpio::PA[13]
            , &mut sam4l::gpio::PA[11], &mut sam4l::gpio::PA[10]
            , &mut sam4l::gpio::PA[12], &mut sam4l::gpio::PC[09]]),
        tmp006: &*tmp006
    };
    
    sam4l::usart::USART3.configure(sam4l::usart::USARTParams {
        //client: &console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    sam4l::gpio::PB[09].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PB[10].configure(Some(sam4l::gpio::PeripheralFunction::A));

    sam4l::gpio::PA[21].configure(Some(sam4l::gpio::PeripheralFunction::E));
    sam4l::gpio::PA[22].configure(Some(sam4l::gpio::PeripheralFunction::E));


    /*
    let pc6 = &sam4l::gpio::PC[6] as &hil::gpio::GPIOPin;
    pc6.enable_output();
    loop {
        pc6.toggle();
    }
    */


    // Configure SPI pins: CLK, MISO, MOSI, CS3 
    sam4l::gpio::PC[ 6].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 4].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 5].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 1].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::spi_dma::SPI.set_active_peripheral(sam4l::spi_dma::Peripheral::Peripheral1);
    sam4l::spi_dma::SPI.init(&SPICB);
    sam4l::spi_dma::SPI.enable();

    let len = buf1.len();
    sam4l::spi_dma::SPI.read_write_bytes(Some(&mut buf2), Some(&mut buf1), len);
    // This is a simple byte-level test of SPI.
    /*let mut counter: u8 = 0;
    loop {
      let val1 = SPI.read_write_byte(0x55);
      let val2 = SPI.read_write_byte(counter);
      let val3 = SPI.read_write_byte(val1);
      let val4 = SPI.read_write_byte(val2);
      let val5 = SPI.read_write_byte(0);
      counter = counter + 1;
      for x in 1..2000 {
        SPI.disable();
        SPI.enable();
      }
    }*/

    
    firestorm.console.initialize();

    firestorm
 
}

