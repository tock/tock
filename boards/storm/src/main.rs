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
use kernel::hil::gpio::PinCtl;
use kernel::hil::spi::SpiMaster;
use sam4l::usart;

#[macro_use]
pub mod io;

// HAL unit tests. To enable a particular unit test, uncomment the call to
// start the test in the init function below.
// #[allow(dead_code)]
// mod gpio_dummy;
// #[allow(dead_code)]
// mod spi_dummy;
// #[allow(dead_code)]
// mod i2c_dummy;
// #[allow(dead_code)]
// mod flash_dummy;
//


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

struct Firestorm {
    console: &'static Console<'static, usart::USART>,
    gpio: &'static capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    tmp006: &'static capsules::tmp006::TMP006<'static>,
    isl29035: &'static capsules::isl29035::Isl29035<'static>,
    spi: &'static capsules::spi::Spi<'static, sam4l::spi::Spi>,
    nrf51822: &'static Nrf51822Serialization<'static, usart::USART>,
    adc: &'static capsules::adc::ADC<'static, sam4l::adc::Adc>,
    led: &'static capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
    FXOS8700CQ: &'static capsules::FXOS8700CQ::FXOS8700CQ<'static>,
}

impl Platform for Firestorm {
    // fn mpu(&mut self) -> &mut cortexm4::mpu::MPU {
    // &mut self.chip.mpu
    // }

    fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {

        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),
            2 => f(Some(self.tmp006)),
            3 => f(Some(self.timer)),
            4 => f(Some(self.spi)),
            5 => f(Some(self.nrf51822)),
            6 => f(Some(self.isl29035)),
            7 => f(Some(self.adc)),
            8 => f(Some(self.led)),
            _ => f(None),
        }
    }
}


unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::{PA, PB, PC};
    use sam4l::gpio::PeripheralFunction::{A, B, C, D, E};

    // Configuring pins for RF233
    // SPI
    PC[03].configure(Some(A)); // SPI NPCS0
    PC[02].configure(Some(A)); // SPI NPCS1
    PC[00].configure(Some(A)); // SPI NPCS2
    PC[01].configure(Some(A)); // SPI NPCS3 (RF233)
    PC[06].configure(Some(A)); // SPI CLK
    PC[04].configure(Some(A)); // SPI MISO
    PC[05].configure(Some(A)); // SPI MOSI
    // GIRQ line of RF233
    PA[20].enable();
    PA[20].disable_output();
    PA[20].disable_interrupt();
    // PA00 is RCLK
    // RSLP
    PC[14].configure(None);
    PC[14].enable();
    PC[14].disable_output();
    // RRST
    PC[15].configure(None);
    PC[15].enable();
    PC[15].disable_output();

    // Right column: Firestorm pin name
    // Left  column: SAM4L peripheral function
    PA[04].configure(Some(C));  // LI_INT   --  EIC EXTINT2
    PA[06].configure(Some(C));  // EXTINT1  --  EIC EXTINT1
    PA[08].configure(None);     // PWM 0    --  GPIO pin
    PC[16].configure(None);     // PWM 1    --  GPIO pin
    PC[17].configure(None);     // PWM 2    --  GPIO pin
    PC[18].configure(None);     // PWM 3    --  GPIO pin
    PA[05].configure(Some(A));  // AD5      --  ADCIFE AD1
    PA[07].configure(Some(A));  // AD4      --  ADCIFE AD2
    PB[02].configure(Some(A));  // AD3      --  ADCIFE AD3
    PB[03].configure(Some(A));  // AD2      --  ADCIFE AD4
    PB[04].configure(Some(A));  // AD1      --  ADCIFE AD5
    PB[05].configure(Some(A));  // AD0      --  ADCIFE AD6
    PB[06].configure(Some(A));  // BL_SEL   --  USART3 RTS
    PB[07].configure(Some(A));  //          --  USART3 CTS
    PB[08].configure(Some(A));  //          --  USART3 CLK
    PB[09].configure(Some(A));  // PRI_RX   --  USART3 RX
    PB[10].configure(Some(A));  // PRI_TX   --  USART3 TX
    PB[11].configure(Some(A));  // U1_CTS   --  USART0 CTS
    PB[12].configure(Some(A));  // U1_RTS   --  USART0 RTS
    PB[13].configure(Some(A));  // U1_CLK   --  USART0 CLK
    PB[14].configure(Some(A));  // U1_RX    --  USART0 RX
    PB[15].configure(Some(A));  // U1_TX    --  USART0 TX
    PC[07].configure(Some(B));  // STORMRTS --  USART2 RTS
    PC[08].configure(Some(E));  // STORMCTS --  USART2 CTS
    PC[11].configure(Some(B));  // STORMRX  --  USART2 RX
    PC[12].configure(Some(B));  // STORMTX  --  USART2 TX
    PA[18].configure(Some(A));  // STORMCLK --  USART2 CLK
    PB[00].configure(Some(A));  // ESDA     --  TWIMS1 TWD
    PB[01].configure(Some(A));  // ESCL     --  TWIMS1 TWCK
    PA[21].configure(Some(E));  // SDA      --  TWIM2 TWD
    PA[22].configure(Some(E));  // SCL      --  TWIM2 TWCK
    PA[25].configure(Some(A));  // EPCLK    --  USBC DM
    PA[26].configure(Some(A));  // EPDAT    --  USBC DP
    PC[21].configure(Some(D));  // PCLK     --  PARC PCCK
    PC[22].configure(Some(D));  // PCEN1    --  PARC PCEN1
    PC[23].configure(Some(D));  // EPGP     --  PARC PCEN2
    PC[24].configure(Some(D));  // PCD0     --  PARC PCDATA0
    PC[25].configure(Some(D));  // PCD1     --  PARC PCDATA1
    PC[26].configure(Some(D));  // PCD2     --  PARC PCDATA2
    PC[27].configure(Some(D));  // PCD3     --  PARC PCDATA3
    PC[28].configure(Some(D));  // PCD4     --  PARC PCDATA4
    PC[29].configure(Some(D));  // PCD5     --  PARC PCDATA5
    PC[30].configure(Some(D));  // PCD6     --  PARC PCDATA6
    PC[31].configure(Some(D));  // PCD7     --  PARC PCDATA7
    PA[16].configure(None);     // P2       -- GPIO Pin
    PA[12].configure(None);     // P3       -- GPIO Pin
    PC[09].configure(None);     // P4       -- GPIO Pin
    PA[10].configure(None);     // P5       -- GPIO Pin
    PA[11].configure(None);     // P6       -- GPIO Pin
    PA[19].configure(None);     // P7       -- GPIO Pin
    PA[13].configure(None);     // P8       -- GPIO Pin
    PA[14].configure(None);     // none     -- GPIO Pin
    PC[20].configure(None);     // ACC_INT2 -- GPIO Pin
    PA[17].configure(None);     // STORMINT -- GPIO Pin
    PA[09].configure(None);     // TMP_DRDY -- GPIO Pin
    PC[13].configure(None);     // ACC_INT1 -- GPIO Pin
    PC[19].configure(None);     // ENSEN    -- GPIO Pin
    PC[10].configure(None);     // LED0     -- GPIO Pin
}

#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    sam4l::pm::setup_system_clock(sam4l::pm::SystemClockSource::DfllRc32k, 48000000);

    // Workaround for SB.02 hardware bug
    // TODO(alevy): Get rid of this when we think SB.02 are out of circulation
    sam4l::gpio::PA[14].enable();
    sam4l::gpio::PA[14].set();
    sam4l::gpio::PA[14].enable_output();


    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);



    set_pin_primary_functions();

    let console = static_init!(
        Console<usart::USART>,
        Console::new(&usart::USART3,
                     115200,
                     &mut console::WRITE_BUF,
                     kernel::Container::create()),
        224/8);
    hil::uart::UART::set_client(&usart::USART3, console);

    // Create the Nrf51822Serialization driver for passing BLE commands
    // over UART to the nRF51822 radio.
    let nrf_serialization = static_init!(
        Nrf51822Serialization<usart::USART>,
        Nrf51822Serialization::new(&usart::USART2,
                                   &mut nrf51822_serialization::WRITE_BUF,
                                   &mut nrf51822_serialization::READ_BUF),
        608/8);
    hil::uart::UART::set_client(&usart::USART2, nrf_serialization);

    let ast = &sam4l::ast::AST;

    let mux_alarm = static_init!(
        MuxAlarm<'static, sam4l::ast::Ast>,
        MuxAlarm::new(&sam4l::ast::AST),
        16);
    ast.configure(mux_alarm);

    let mux_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(&sam4l::i2c::I2C2), 20);
    sam4l::i2c::I2C2.set_master_client(mux_i2c);

    // Configure the TMP006. Device address 0x40
    let tmp006_i2c = static_init!(I2CDevice, I2CDevice::new(mux_i2c, 0x40), 32);
    sam4l::gpio::PA[9].set_input_mode(kernel::hil::gpio::InputMode::PullUp);
    let tmp006 = static_init!(
        capsules::tmp006::TMP006<'static>,
        capsules::tmp006::TMP006::new(tmp006_i2c,
                                     &sam4l::gpio::PA[9],
                                     &mut capsules::tmp006::BUFFER),
        52);
    tmp006_i2c.set_client(tmp006);
    sam4l::gpio::PA[9].set_client(tmp006);

    // Configure the ISL29035, device address 0x44
    let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(mux_i2c, 0x44), 32);
    let isl29035 = static_init!(
        capsules::isl29035::Isl29035<'static>,
        capsules::isl29035::Isl29035::new(isl29035_i2c, &mut capsules::isl29035::BUF),
        36);
    isl29035_i2c.set_client(isl29035);

    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        TimerDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        TimerDriver::new(virtual_alarm1, kernel::Container::create()),
        12);
    virtual_alarm1.set_client(timer);

    // Initialize and enable SPI HAL
    let chip_selects = static_init!([u8; 4], [0, 1, 2, 3], 4);
    let spi = static_init!(
        capsules::spi::Spi<'static, sam4l::spi::Spi>,
        capsules::spi::Spi::new(&mut sam4l::spi::SPI, chip_selects),
        92);
    spi.config_buffers(&mut spi_read_buf, &mut spi_write_buf);
    sam4l::spi::SPI.set_client(spi);
    sam4l::spi::SPI.init();

    // LEDs
    let led_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 1],
        [&sam4l::gpio::PC[10]],
        1 * 4);
    let led = static_init!(
        capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins, capsules::led::ActivationMode::ActiveHigh),
        96/8);

    // accelerometer on 0x1C, 0x1D, 0x1E, or 0x1F?? 
    let fx0_i2c = static_init!(I2CDevice, I2CDevice::new(mux_i2c, 0x1C), 32);
    let fx0 = static_init!(
        capsules::FXOS8700CQ::FXOS8700CQ<'static>,
        capsules::FXOS8700CQ::FXOS8700CQ::new(fx0_i2c, &mut capsules::FXOS8700CQ::BUF),
        48);
    fx0_i2c.set_client(fx0);

    // Setup ADC
    let adc = static_init!(
        capsules::adc::ADC<'static, sam4l::adc::Adc>,
        capsules::adc::ADC::new(&mut sam4l::adc::ADC),
        160/8);
    sam4l::adc::ADC.set_client(adc);


    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 11],
        [&sam4l::gpio::PA[16], // P2
         &sam4l::gpio::PA[12], // P3
         &sam4l::gpio::PC[9], // P4
         &sam4l::gpio::PA[10], // P5
         &sam4l::gpio::PA[11], // P6
         &sam4l::gpio::PA[19], // P7
         &sam4l::gpio::PA[13], // P8
         &sam4l::gpio::PA[17], /* STORM_INT (nRF51822) */
         &sam4l::gpio::PC[14], /* RSLP (RF233 sleep line) */
         &sam4l::gpio::PC[15], /* RRST (RF233 reset line) */
         &sam4l::gpio::PA[20]], /* RIRQ (RF233 interrupt) */
        11 * 4
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        20);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // Note: The following GPIO pins aren't assigned to anything:
    // &sam4l::gpio::PC[19] // !ENSEN
    // &sam4l::gpio::PC[13] // ACC_INT1
    // &sam4l::gpio::PC[20] // ACC_INT2
    // &sam4l::gpio::PA[14] // No Connection
    //

    let firestorm = static_init!(
        Firestorm,
        Firestorm {
            console: console,
            gpio: gpio,
            timer: timer,
            tmp006: tmp006,
            isl29035: isl29035,
            spi: spi,
            nrf51822: nrf_serialization,
            adc: adc,
            led: led,
            FXOS8700CQ: fx0,
        },
        320/8);

    // Configure USART2 Pins for connection to nRF51822
    // NOTE: the SAM RTS pin is not working for some reason. Our hypothesis is
    //  that it is because RX DMA is not set up. For now, just having it always
    //  enabled works just fine
    sam4l::gpio::PC[07].enable();
    sam4l::gpio::PC[07].enable_output();
    sam4l::gpio::PC[07].clear();


    // Uncommenting the following line will cause the device to use the
    // SPI HAL to write [8, 7, 6, 5, 4, 3, 2, 1] once over the SPI then
    // echo the 8 bytes read from the slave continuously.
    // spi_dummy::spi_dummy_test();

    // Uncommenting the following line will toggle the LED whenever the value of
    // Firestorm's pin 8 changes value (e.g., connect a push button to pin 8 and
    // press toggle it).
    // gpio_dummy::gpio_dummy_test();

    // Uncommenting the following line will test the I2C
    // i2c_dummy::i2c_scan_slaves();
    // i2c_dummy::i2c_tmp006_test();
    // i2c_dummy::i2c_accel_test();
    // i2c_dummy::i2c_li_test();

    // Uncommenting the following lines will test the Flash Controller
    // flash_dummy::meta_test();
    // flash_dummy::set_read_write_test();

    firestorm.console.initialize();
    firestorm.nrf51822.initialize();

    let mut chip = sam4l::chip::Sam4l::new();
    chip.mpu().enable_mpu();


    kernel::main(firestorm, &mut chip, load_processes());
}
