#![no_std]
#![no_main]
#![feature(const_fn,lang_items)]

extern crate capsules;
extern crate cortexm4;
#[macro_use(static_init)]
extern crate kernel;
extern crate sam4l;

use capsules::console::{self, Console};
use capsules::nrf51822_serialization::{self, Nrf51822Serialization};
use capsules::timer::TimerDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::{Chip, MPU, Platform};
use kernel::hil;
use kernel::hil::Controller;
use kernel::hil::spi::SpiMaster;
use sam4l::usart;

#[macro_use]
pub mod io;

static mut spi_read_buf: [u8; 64] = [0; 64];
static mut spi_write_buf: [u8; 64] = [0; 64];

unsafe fn load_processes() -> &'static mut [Option<kernel::process::Process<'static>>] {
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    const NUM_PROCS: usize = 2;

    #[link_section = ".app_memory"]
    static mut MEMORIES: [[u8; 8192]; NUM_PROCS] = [[0; 8192]; NUM_PROCS];

    static mut processes: [Option<kernel::process::Process<'static>>; NUM_PROCS] = [None, None];

    let mut addr = &_sapps as *const u8;
    for i in 0..NUM_PROCS {
        // The first member of the LoadInfo header contains the total size of each process image. A
        // sentinel value of 0 (invalid because it's smaller than the header itself) is used to
        // mark the end of the list of processes.
        let total_size = *(addr as *const usize);
        if total_size == 0 {
            break;
        }

        let process = &mut processes[i];
        let memory = &mut MEMORIES[i];
        *process = Some(kernel::process::Process::create(addr, total_size, memory));
        // TODO: panic if loading failed?

        addr = addr.offset(total_size as isize);
    }

    if *(addr as *const usize) != 0 {
        panic!("Exceeded maximum NUM_PROCS.");
    }

    &mut processes
}

struct Hail {
    console: &'static Console<'static, usart::USART>,
    gpio: &'static capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    isl29035: &'static capsules::isl29035::Isl29035<'static,
                                                    VirtualMuxAlarm<'static,
                                                                    sam4l::ast::Ast<'static>>>,
    si7021: &'static capsules::si7021::SI7021<'static,
                                              VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    fxos8700: &'static capsules::fxos8700_cq::Fxos8700cq<'static>,
    spi: &'static capsules::spi::Spi<'static, sam4l::spi::Spi>,
    nrf51822: &'static Nrf51822Serialization<'static, usart::USART>,
    adc: &'static capsules::adc::ADC<'static, sam4l::adc::Adc>,
    led: &'static capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
    ipc: kernel::ipc::IPC,
}

impl Platform for Hail {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {

        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),

            3 => f(Some(self.timer)),
            4 => f(Some(self.spi)),
            5 => f(Some(self.nrf51822)),
            6 => f(Some(self.isl29035)),
            7 => f(Some(self.adc)),
            8 => f(Some(self.led)),
            9 => f(Some(self.button)),
            10 => f(Some(self.si7021)),
            11 => f(Some(self.fxos8700)),

            0xff => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}


unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::{PA, PB};
    use sam4l::gpio::PeripheralFunction::{A, B};

    PA[04].configure(Some(A)); // A0 - ADC0
    PA[05].configure(Some(A)); // A1 - ADC1
    PA[06].configure(Some(A)); // DAC
    PA[07].configure(None);    // WKP - Wakeup
    PA[08].configure(Some(A)); // FTDI_RTS - USART0 RTS
    PA[09].configure(None);    // ACC_INT1 - FXOS8700CQ Interrupt 1
    PA[10].configure(None);    // unused
    PA[11].configure(Some(A)); // FTDI_OUT - USART0 RX FTDI->SAM4L
    PA[12].configure(Some(A)); // FTDI_IN - USART0 TX SAM4L->FTDI
    PA[13].configure(None);    // RED_LED
    PA[14].configure(None);    // BLUE_LED
    PA[15].configure(None);    // GREEN_LED
    PA[16].configure(None);    // BUTTON - User Button
    PA[17].configure(None);    // !NRF_RESET - Reset line for nRF51822
    PA[18].configure(None);    // ACC_INT2 - FXOS8700CQ Interrupt 2
    PA[19].configure(None);    // unused
    PA[20].configure(None);    // !LIGHT_INT - ISL29035 Light Sensor Interrupt
    // SPI Mode
    PA[21].configure(Some(A)); // D3 - SPI MISO
    PA[22].configure(Some(A)); // D2 - SPI MOSI
    PA[23].configure(Some(A)); // D4 - SPI SCK
    PA[24].configure(Some(A)); // D5 - SPI CS
    // // I2C MODE
    // PA[21].configure(None); // D3
    // PA[22].configure(None); // D2
    // PA[23].configure(Some(B)); // D4 - TWIMS0 SDA
    // PA[24].configure(Some(B)); // D5 - TWIMS0 SCL
    // UART Mode
    PA[25].configure(Some(B)); // RX - USART2 RXD
    PA[26].configure(Some(B)); // TX - USART2 TXD

    PB[00].configure(Some(A)); // SENSORS_SDA - TWIMS1 SDA
    PB[01].configure(Some(A)); // SENSORS_SCL - TWIMS1 SCL
    PB[02].configure(Some(A)); // A2 - ADC3
    PB[03].configure(Some(A)); // A3 - ADC4
    PB[04].configure(Some(A)); // A4 - ADC5
    PB[05].configure(Some(A)); // A5 - ADC6
    PB[06].configure(Some(A)); // NRF_CTS - USART3 RTS
    PB[07].configure(Some(A)); // NRF_RTS - USART3 CTS
    PB[08].configure(None);    // NRF_INT - Interrupt line nRF->SAM4L
    PB[09].configure(Some(A)); // NRF_OUT - USART3 RXD
    PB[10].configure(Some(A)); // NRF_IN - USART3 TXD
    PB[11].configure(None);    // D6
    PB[12].configure(None);    // D7
    PB[13].configure(None);    // unused
    PB[14].configure(None);    // D0
    PB[15].configure(None);    // D1
}

#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    sam4l::pm::setup_system_clock(sam4l::pm::SystemClockSource::ExternalOscillatorPll,
                                  48000000);

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();

    let console = static_init!(
        Console<usart::USART>,
        Console::new(&usart::USART0,
                     115200,
                     &mut console::WRITE_BUF,
                     kernel::Container::create()),
        224/8);
    hil::uart::UART::set_client(&usart::USART0, console);

    // Create the Nrf51822Serialization driver for passing BLE commands
    // over UART to the nRF51822 radio.
    let nrf_serialization = static_init!(
        Nrf51822Serialization<usart::USART>,
        Nrf51822Serialization::new(&usart::USART3,
                                   &mut nrf51822_serialization::WRITE_BUF,
                                   &mut nrf51822_serialization::READ_BUF),
        608/8);
    hil::uart::UART::set_client(&usart::USART3, nrf_serialization);

    let ast = &sam4l::ast::AST;

    let mux_alarm = static_init!(
        MuxAlarm<'static, sam4l::ast::Ast>,
        MuxAlarm::new(&sam4l::ast::AST),
        16);
    ast.configure(mux_alarm);

    let sensors_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(&sam4l::i2c::I2C1), 20);
    sam4l::i2c::I2C1.set_master_client(sensors_i2c);

    // SI7021 Temperature / Humidity Sensor, address: 0x40
    let si7021_i2c = static_init!(
        capsules::virtual_i2c::I2CDevice,
        capsules::virtual_i2c::I2CDevice::new(sensors_i2c, 0x40),
        32);
    let si7021_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm),
        192/8);
    let si7021 = static_init!(
        capsules::si7021::SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        capsules::si7021::SI7021::new(si7021_i2c,
            si7021_virtual_alarm,
            &mut capsules::si7021::BUFFER),
        288/8);
    si7021_i2c.set_client(si7021);
    si7021_virtual_alarm.set_client(si7021);

    // Configure the ISL29035, device address 0x44
    let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(sensors_i2c, 0x44), 32);
    let isl29035_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm),
        192/8);
    let isl29035 = static_init!(
        capsules::isl29035::Isl29035<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        capsules::isl29035::Isl29035::new(isl29035_i2c, isl29035_virtual_alarm,
                                          &mut capsules::isl29035::BUF),
        320/8);
    isl29035_i2c.set_client(isl29035);
    isl29035_virtual_alarm.set_client(isl29035);

    // Timer
    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        TimerDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        TimerDriver::new(virtual_alarm1, kernel::Container::create()),
        12);
    virtual_alarm1.set_client(timer);

    // FXOS8700CQ accelerometer, device address 0x
    let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(sensors_i2c, 0x1e), 32);
    let fxos8700 = static_init!(
        capsules::fxos8700_cq::Fxos8700cq<'static>,
        capsules::fxos8700_cq::Fxos8700cq::new(fxos8700_i2c, &mut capsules::fxos8700_cq::BUF),
        288/8);
    fxos8700_i2c.set_client(fxos8700);

    // Initialize and enable SPI HAL
    let chip_selects = static_init!([u8; 1], [0], 1);
    let spi = static_init!(
        capsules::spi::Spi<'static, sam4l::spi::Spi>,
        capsules::spi::Spi::new(&mut sam4l::spi::SPI, chip_selects),
        92);
    spi.config_buffers(&mut spi_read_buf, &mut spi_write_buf);
    sam4l::spi::SPI.set_client(spi);
    sam4l::spi::SPI.init();

    // LEDs
    let led_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 3],
        [&sam4l::gpio::PA[13],  // Red
         &sam4l::gpio::PA[15],  // Green
         &sam4l::gpio::PA[14]], // Blue
        3 * 4);
    let led = static_init!(
        capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins, capsules::led::ActivationMode::ActiveLow),
        96/8);

    // BUTTONs
    let button_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 1],
        [&sam4l::gpio::PA[16]],
        1 * 4);
    let button = static_init!(
        capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Container::create()),
        96/8);
    for btn in button_pins.iter() {
        btn.set_client(button);
    }

    // Setup ADC
    let adc = static_init!(
        capsules::adc::ADC<'static, sam4l::adc::Adc>,
        capsules::adc::ADC::new(&mut sam4l::adc::ADC),
        160/8);
    sam4l::adc::ADC.set_client(adc);


    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 4],
        [&sam4l::gpio::PB[14],  // D0
         &sam4l::gpio::PB[15],  // D1
         &sam4l::gpio::PB[11],  // D6
         &sam4l::gpio::PB[12]], // D7
        4 * 4
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        20);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let hail = Hail {
        console: console,
        gpio: gpio,
        timer: timer,
        si7021: si7021,
        isl29035: isl29035,
        fxos8700: fxos8700,
        spi: spi,
        nrf51822: nrf_serialization,
        adc: adc,
        led: led,
        button: button,
        ipc: kernel::ipc::IPC::new(),
    };

    // Need to reset the nRF on boot
    sam4l::gpio::PA[17].enable();
    sam4l::gpio::PA[17].enable_output();
    sam4l::gpio::PA[17].clear();
    sam4l::gpio::PA[17].set();

    hail.console.initialize();
    hail.nrf51822.initialize();

    let mut chip = sam4l::chip::Sam4l::new();
    chip.mpu().enable_mpu();

    kernel::main(&hail, &mut chip, load_processes(), &hail.ipc);
}
