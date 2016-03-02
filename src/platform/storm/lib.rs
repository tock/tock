#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(const_fn,lang_items)]

extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;
extern crate support;

use hil::Controller;
use hil::spi_master::SpiMaster;
use drivers::timer::AlarmToTimer;
use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

#[macro_use]
pub mod io;

// HAL unit tests. To enable a particular unit test, uncomment
// the module here and uncomment the call to start the test in
// the init function below.
//mod gpio_dummy;
//mod spi_dummy;
//mod i2c_dummy;

static mut spi_read_buf:  [u8; 64] = [0; 64];
static mut spi_write_buf: [u8; 64] = [0; 64];

pub struct Firestorm {
    chip: sam4l::chip::Sam4l,
    console: &'static drivers::console::Console<'static, sam4l::usart::USART>,
    gpio: &'static drivers::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
    timer: &'static drivers::timer::TimerDriver<'static, AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>>>,
    tmp006: &'static drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice, sam4l::gpio::GPIOPin>,
    spi: &'static drivers::spi::Spi<'static, sam4l::spi::Spi>,
    nrf51822: &'static drivers::nrf51822_serialization::Nrf51822Serialization<'static, sam4l::usart::USART>,
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {

        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),
            2 => f(Some(self.tmp006)),
            3 => f(Some(self.timer)),
            4 => f(Some(self.spi)),
            5 => f(Some(self.nrf51822)),
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

    // Workaround for SB.02 hardware bug
    // TODO(alevy): Get rid of this when we think SB.02 are out of circulation
    sam4l::gpio::PA[14].enable();
    sam4l::gpio::PA[14].set();
    sam4l::gpio::PA[14].enable_output();

    static_init!(console : drivers::console::Console<sam4l::usart::USART> =
                    drivers::console::Console::new(&sam4l::usart::USART3,
                                       &mut drivers::console::WRITE_BUF));
    sam4l::usart::USART3.set_client(console);

    // Create the Nrf51822Serialization driver for passing BLE commands
    // over UART to the nRF51822 radio.
    static_init!(nrf_serialization : drivers::nrf51822_serialization::Nrf51822Serialization<sam4l::usart::USART> =
                    drivers::nrf51822_serialization::Nrf51822Serialization::new(&sam4l::usart::USART2,
                                                                                &mut drivers::nrf51822_serialization::WRITE_BUF));
    sam4l::usart::USART2.set_client(nrf_serialization);

    let ast = &sam4l::ast::AST;

    static_init!(mux_alarm : MuxAlarm<'static, sam4l::ast::Ast> =
                    MuxAlarm::new(&sam4l::ast::AST));
    ast.configure(mux_alarm);


    // the i2c address of the device is 0x40
    static_init!(tmp006 : drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice, sam4l::gpio::GPIOPin> =
                    drivers::tmp006::TMP006::new(&sam4l::i2c::I2C2, 0x40, &sam4l::gpio::PA[9],
                                                 &mut drivers::tmp006::BUFFER));
    sam4l::i2c::I2C2.set_client(tmp006);
    sam4l::gpio::PA[9].set_client(tmp006);

    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, sam4l::ast::Ast> =
                    VirtualMuxAlarm::new(mux_alarm));
    static_init!(vtimer1 : AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>> =
                            AlarmToTimer::new(virtual_alarm1));
    virtual_alarm1.set_client(vtimer1);
    static_init!(timer : drivers::timer::TimerDriver<AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>>> =
                            drivers::timer::TimerDriver::new(vtimer1));
    vtimer1.set_client(timer);

    // Configure SPI pins: CLK, MISO, MOSI, CS3
    sam4l::gpio::PC[ 6].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 4].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 5].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 1].configure(Some(sam4l::gpio::PeripheralFunction::A));
    // Initialize and enable SPI HAL
    static_init!(spi: drivers::spi::Spi<'static, sam4l::spi::Spi> =
                      drivers::spi::Spi::new(&mut sam4l::spi::SPI));
    spi.config_buffers(&mut spi_read_buf, &mut spi_write_buf);
    sam4l::spi::SPI.set_active_peripheral(sam4l::spi::Peripheral::Peripheral1);
    sam4l::spi::SPI.init(spi as &hil::spi_master::SpiCallback);
    sam4l::spi::SPI.enable();


    // set GPIO driver controlling remaining GPIO pins
    static_init!(gpio_pins : [&'static sam4l::gpio::GPIOPin; 9] = [
            &sam4l::gpio::PC[10], // LED_0
            &sam4l::gpio::PA[16], // P2
            &sam4l::gpio::PA[12], // P3
            &sam4l::gpio::PC[ 9], // P4
            &sam4l::gpio::PA[10], // P5
            &sam4l::gpio::PA[11], // P6
            &sam4l::gpio::PA[19], // P7
            &sam4l::gpio::PA[13], // P8
            &sam4l::gpio::PA[17], // STORM_INT (nRF51822)
            ]);
    static_init!(gpio : drivers::gpio::GPIO<'static, sam4l::gpio::GPIOPin> =
                 drivers::gpio::GPIO::new(gpio_pins));
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    /* Note: The following GPIO pins aren't assigned to anything:
    &sam4l::gpio::PC[19] // !ENSEN
    &sam4l::gpio::PC[13] // ACC_INT1
    &sam4l::gpio::PC[20] // ACC_INT2
    &sam4l::gpio::PA[14] // No Connection
    */

    static_init!(firestorm : Firestorm = Firestorm {
        chip: sam4l::chip::Sam4l::new(),
        console: &*console,
        gpio: gpio,
        timer: timer,
        tmp006: &*tmp006,
        spi: &*spi,
        nrf51822: &*nrf_serialization,
    });

    sam4l::usart::USART3.configure(sam4l::usart::USARTParams {
        //client: &console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    // Setup USART2 for the nRF51822 connection
    sam4l::usart::USART2.configure(sam4l::usart::USARTParams {
        baud_rate: 250000,
        data_bits: 8,
        parity: hil::uart::Parity::Even
    });

    sam4l::gpio::PB[09].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PB[10].configure(Some(sam4l::gpio::PeripheralFunction::A));

    // Configure I2C SDA and SCL pins
    sam4l::gpio::PA[21].configure(Some(sam4l::gpio::PeripheralFunction::E));
    sam4l::gpio::PA[22].configure(Some(sam4l::gpio::PeripheralFunction::E));

    // Uncommenting the following line will cause the device to use the
    // SPI HAL to write [8, 7, 6, 5, 4, 3, 2, 1] once over the SPI then
    // echo the 8 bytes read from the slave continuously.
    //spi_dummy::spi_dummy_test();

    // Configure USART2 Pins for connection to nRF51822
    sam4l::gpio::PC[ 7].configure(Some(sam4l::gpio::PeripheralFunction::B));
    sam4l::gpio::PC[ 8].configure(Some(sam4l::gpio::PeripheralFunction::B));
    sam4l::gpio::PC[11].configure(Some(sam4l::gpio::PeripheralFunction::B));
    sam4l::gpio::PC[12].configure(Some(sam4l::gpio::PeripheralFunction::B));

    // Uncommenting the following line will toggle the LED whenever the value of
    // Firestorm's pin 8 changes value (e.g., connect a push button to pin 8 and
    // press toggle it).
    //gpio_dummy::gpio_dummy_test();

    // Uncommenting the following line will test the I2C
    //i2c_dummy::i2c_scan_slaves();
    //i2c_dummy::i2c_tmp006_test();
    //i2c_dummy::i2c_accel_test();
    //i2c_dummy::i2c_li_test();

    firestorm.console.initialize();
    firestorm.nrf51822.initialize();
    firestorm
}

