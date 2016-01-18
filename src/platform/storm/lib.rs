#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(const_fn)]

extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use hil::Controller;
use hil::timer::*;
use hil::spi_master::SpiMaster;

// Uncomment each module to test with respective commented out code block in
// `init`
//
//mod gpio_dummy;
//mod spi_dummy;
#[allow(unused_variables,dead_code)]
pub struct DummyCB {
    val: u8
}
 
pub static mut FLOP: bool = false;
pub static mut buf1: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub static mut buf2: [u8; 8] = [7, 6, 5, 4, 3, 2, 1, 0];

pub struct Firestorm {
    chip: sam4l::chip::Sam4l,
    console: &'static drivers::console::Console<'static, sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static hil::gpio::GPIOPin; 14]>,
    tmp006: &'static drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice>,
    spi: &'static drivers::spi::Spi<'static, sam4l::spi::Spi>,
}

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
            3 => f(Some(self.spi)),
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
    static mut SPI_BUF: [u8; 512] = [0; 512];

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
    *console = drivers::console::Console::new(&sam4l::usart::USART3);

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

    // Configure SPI pins: CLK, MISO, MOSI, CS3
    sam4l::gpio::PC[ 6].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 4].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 5].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 1].configure(Some(sam4l::gpio::PeripheralFunction::A));
    let spi : &mut drivers::spi::Spi<sam4l::spi::Spi> = mem::transmute(&mut SPI_BUF); 
    {
      *spi = drivers::spi::Spi::new(&mut sam4l::spi::SPI);
      sam4l::spi::SPI.init(spi as &hil::spi_master::SpiCallback);
      sam4l::spi::SPI.enable();
    }
    // The SPI clock is now enabled in Spi::enable
    // pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI)); 

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: sam4l::chip::Sam4l::new(),
        console: &*console,
        gpio: drivers::gpio::GPIO::new(
            [ &sam4l::gpio::PC[10], &sam4l::gpio::PC[19]
            , &sam4l::gpio::PC[13], &sam4l::gpio::PA[9]
            , &sam4l::gpio::PA[17], &sam4l::gpio::PC[20]
            , &sam4l::gpio::PA[19], &sam4l::gpio::PA[14]
            , &sam4l::gpio::PA[16], &sam4l::gpio::PA[13]
            , &sam4l::gpio::PA[11], &sam4l::gpio::PA[10]
            , &sam4l::gpio::PA[12], &sam4l::gpio::PC[09]]),
        tmp006: &*tmp006,
        spi: &*spi,
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

    // Uncommenting the following line will cause the device to write
    // [8, 7, 6, 5, 4, 3, 2, 1] once over the SPI then echo the 8 bytes read
    // from the slave continuously.
    //spi_dummy::spi_dummy_test();

    // Uncommenting the following line will toggle the LED whenever the value of
    // Firestorm's pin 8 changes value (e.g., connect a push button to pin 8 and
    // press toggle it).
    //gpio_dummy::gpio_dummy_test();

    firestorm.console.initialize();

    firestorm
}

