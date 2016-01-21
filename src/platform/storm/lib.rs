#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(const_fn)]

extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use hil::Controller;
use drivers::timer::AlarmToTimer;
use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

// Uncomment each module to test with respective commented out code block in
// `init`
//
//mod gpio_dummy;
//mod spi_dummy;

pub struct Firestorm {
    chip: sam4l::chip::Sam4l,
    console: &'static drivers::console::Console<'static, sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static hil::gpio::GPIOPin; 14]>,
    timer: &'static drivers::timer::TimerDriver<'static, AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>>>,
    tmp006: &'static drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice>,
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
            3 => f(Some(self.timer)),
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

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;

    static_init!(console : drivers::console::Console<sam4l::usart::USART> =
                    drivers::console::Console::new(&sam4l::usart::USART3));
    sam4l::usart::USART3.set_client(console);

    let ast = &sam4l::ast::AST;

    static_init!(mux_alarm : MuxAlarm<'static, sam4l::ast::Ast> =
                    MuxAlarm::new(&sam4l::ast::AST));
    ast.configure(mux_alarm);


    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, sam4l::ast::Ast> =
                    VirtualMuxAlarm::new(mux_alarm));
    static_init!(vtimer1 : AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>> =
                            AlarmToTimer::new(virtual_alarm1));
    virtual_alarm1.set_client(vtimer1);
    static_init!(tmp006 : drivers::tmp006::TMP006<'static,
                                sam4l::i2c::I2CDevice> =
                    drivers::tmp006::TMP006::new(&sam4l::i2c::I2C2, vtimer1));
    vtimer1.set_client(tmp006);


    static_init!(virtual_alarm2 : VirtualMuxAlarm<'static, sam4l::ast::Ast> =
                    VirtualMuxAlarm::new(mux_alarm));
    static_init!(vtimer2 : AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>> =
                            AlarmToTimer::new(virtual_alarm2));
    virtual_alarm2.set_client(vtimer2);
    static_init!(timer : drivers::timer::TimerDriver<AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>>> =
                            drivers::timer::TimerDriver::new(vtimer2));
    vtimer2.set_client(timer);

    static_init!(firestorm : Firestorm = Firestorm {
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
        timer: timer,
        tmp006: &*tmp006
    });

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

    // Configure SPI pins: CLK, MISO, MOSI, CS3
    sam4l::gpio::PC[ 6].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 4].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 5].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 1].configure(Some(sam4l::gpio::PeripheralFunction::A));

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

