#![no_std]
#![no_main]
#![feature(const_fn,lang_items)]

extern crate capsules;
#[macro_use(static_init)]
extern crate kernel;
extern crate sam4l;

use capsules::timer::TimerDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::{Chip, MPU};
use kernel::hil;
use kernel::hil::Controller;
use kernel::hil::spi::SpiMaster;
use kernel::hil::gpio::Pin;

mod io;

// unit test for the HW
#[allow(dead_code)]
mod spi_dummy;


struct Imix {
    console: &'static capsules::console::Console<'static, sam4l::usart::USART>,
    ble_adv: &'static capsules::ble_adv::BleAdv<'static, sam4l::usart::USART>,
    gpio: &'static capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    si7021: &'static capsules::si7021::SI7021<'static,
                                              VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    isl29035: &'static capsules::isl29035::Isl29035<'static>,
    adc: &'static capsules::adc::ADC<'static, sam4l::adc::Adc>,
    led: &'static capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
    spi: &'static capsules::spi::Spi<'static, sam4l::spi::Spi>,
}

impl kernel::Platform for Imix {
    fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),

            3 => f(Some(self.timer)),
            4 => f(Some(self.spi)), 
            6 => f(Some(self.isl29035)),
            7 => f(Some(self.adc)),
            8 => f(Some(self.led)),
            9 => f(Some(self.button)),
            10 => f(Some(self.si7021)),

            0xbe => f(Some(self.ble_adv)),
            _ => f(None),
        }
    }
}

unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::{PA, PB, PC};
    use sam4l::gpio::PeripheralFunction::{A, B, C, E};

    // Right column: Imix pin name
    // Left  column: SAM4L peripheral function
    PA[04].configure(Some(C));  // LI_INT      --  EIC EXTINT2
    PA[05].configure(Some(A));  // AD0         --  ADCIFE AD1
    PA[06].configure(Some(C));  // EXTINT1     --  EIC EXTINT1
    PA[07].configure(Some(A));  // AD1         --  ADCIFE AD2
    PA[08].configure(None);     // RF233 IRQ   --  GPIO pin
    PA[09].configure(None);     // RF233 RST   --  GPIO pin
    PA[10].configure(None);     // RF233 SLP   --  GPIO pin
    PA[13].configure(None);     // TRNG EN     --  GPIO pin
    PA[14].configure(None);     // TRNG_OUT    --  GPIO pin
    PA[17].configure(None);     // NRF INT     -- GPIO pin
    PA[18].configure(Some(A));  // NRF CLK     -- USART2_CLK
    PA[21].configure(Some(E));  // TWI2 SDA    -- TWIM2_SDA
    PA[22].configure(Some(E));  // TWI2 SCL    --  TWIM2 TWCK
    PA[25].configure(Some(A));  // USB_N       --  USB DM
    PA[26].configure(Some(A));  // USB_P       --  USB DP
    PB[00].configure(Some(A));  // TWI1_SDA    --  TWIMS1 TWD
    PB[01].configure(Some(A));  // TWI1_SCL    --  TWIMS1 TWCK
    PB[02].configure(Some(A));  // AD2         --  ADCIFE AD3
    PB[03].configure(Some(A));  // AD3         --  ADCIFE AD4
    PB[04].configure(Some(A));  // AD4         --  ADCIFE AD5
    PB[05].configure(Some(A));  // AD5         --  ADCIFE AD6
    PB[06].configure(Some(A));  // RTS3        --  USART3 RTS
    PB[07].configure(None);     // NRF RESET   --  GPIO
    PB[09].configure(Some(A));  // RX3         --  USART3 RX
    PB[10].configure(Some(A));  // TX3         --  USART3 TX
    PB[11].configure(Some(A));  // CTS0        --  USART0 CTS
    PB[12].configure(Some(A));  // RTS0        --  USART0 RTS
    PB[13].configure(Some(A));  // CLK0        --  USART0 CLK
    PB[14].configure(Some(A));  // RX0         --  USART0 RX
    PB[15].configure(Some(A));  // TX0         --  USART0 TX
    PC[00].configure(Some(A));  // CS2         --  SPI NPCS2
    PC[01].configure(Some(A));  // CS3 (RF233) -- SPI NPCS3
    PC[02].configure(Some(A));  // CS1         --  SPI NPCS1
    PC[03].configure(Some(A));  // CS0         --  SPI NPCS0
    PC[04].configure(Some(A));  // MISO        --  SPI MISO
    PC[05].configure(Some(A));  // MOSI        --  SPI MOSI
    PC[06].configure(Some(A));  // SCK         --  SPI CLK
    PC[07].configure(Some(B));  // RTS2 (BLE)  -- USART2_RTS
    PC[08].configure(Some(B));  // CTS2 (BLE)  -- USART2_CTS
    PC[09].configure(None);     // NRF GPIO    -- GPIO
    PC[10].configure(None);     // USER LED    -- GPIO
    PC[11].configure(Some(B));  // RX2 (BLE)   -- USART2_RX
    PC[12].configure(Some(B));  // TX2 (BLE)   -- USART2_TX
    PC[13].configure(None);     // ACC_INT1    -- GPIO
    PC[14].configure(None);     // ACC_INT2    -- GPIO
    PC[16].configure(None);     // SENSE_PWR   --  GPIO pin
    PC[17].configure(None);     // NRF_PWR     --  GPIO pin
    PC[18].configure(None);     // RF233_PWR   --  GPIO pin
    PC[19].configure(None);     // TRNG_PWR    -- GPIO Pin
    PC[24].configure(None);     // USER_BTN    -- GPIO Pin
    PC[25].configure(None);     // D8          -- GPIO Pin
    PC[26].configure(None);     // D7          -- GPIO Pin
    PC[27].configure(None);     // D6          -- GPIO Pin
    PC[28].configure(None);     // D5          -- GPIO Pin
    PC[29].configure(None);     // D4          -- GPIO Pin
    PC[30].configure(None);     // D3          -- GPIO Pin
    PC[31].configure(None);     // D2          -- GPIO Pin

    // Enable, and disable output for RF233 pins
    // IRQ
    PA[08].enable();
    PA[08].disable_output();
    PA[08].disable_interrupt();
    // RST
    PA[09].enable();
    PA[09].disable_output();
    // SLP 
    PA[10].enable();
    PA[10].disable_output();
}


#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    sam4l::pm::setup_system_clock(sam4l::pm::SystemClockSource::DfllRc32k, 48000000);

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();
    
    
    // # CONSOLE

    let console = static_init!(
        capsules::console::Console<sam4l::usart::USART>,
        capsules::console::Console::new(&sam4l::usart::USART3,
                     115200,
                     &mut capsules::console::WRITE_BUF,
                     kernel::Container::create()),
        224/8);
    hil::uart::UART::set_client(&sam4l::usart::USART3, console);
    console.initialize();

    let ble_adv = static_init!(
        capsules::ble_adv::BleAdv<sam4l::usart::USART>,
        capsules::ble_adv::BleAdv::new(&sam4l::usart::USART2,
                     &sam4l::gpio::PA[17],
                     &mut capsules::ble_adv::BUF,
                     kernel::Container::create()),
        192/8);
    hil::uart::UART::set_client(&sam4l::usart::USART2, ble_adv);
    ble_adv.initialize();
    {
        use kernel::hil::gpio::Pin;
        sam4l::gpio::PC[17].make_output();
        sam4l::gpio::PC[17].clear();
    }

    // # TIMER

    let ast = &sam4l::ast::AST;

    let mux_alarm = static_init!(
        MuxAlarm<'static, sam4l::ast::Ast>,
        MuxAlarm::new(&sam4l::ast::AST),
        16);
    ast.configure(mux_alarm);

    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        TimerDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        TimerDriver::new(virtual_alarm1, kernel::Container::create()),
        12);
    virtual_alarm1.set_client(timer);

    // # I2C Sensors

    let mux_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(&sam4l::i2c::I2C2), 20);
    sam4l::i2c::I2C2.set_master_client(mux_i2c);

    // Configure the ISL29035, device address 0x44
    let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(mux_i2c, 0x44), 32);
    let isl29035 = static_init!(
        capsules::isl29035::Isl29035<'static>,
        capsules::isl29035::Isl29035::new(isl29035_i2c, &mut capsules::isl29035::BUF),
        36);
    isl29035_i2c.set_client(isl29035);

    static mut spi_read_buf: [u8; 64] = [0; 64];
    static mut spi_write_buf: [u8; 64] = [0; 64];
    
    // Initialize and enable SPI HAL
    let chip_selects = static_init!([u8; 4], [0, 1, 2, 3], 4);
    let spi = static_init!(
        capsules::spi::Spi<'static, sam4l::spi::Spi>,
        capsules::spi::Spi::new(&mut sam4l::spi::SPI, chip_selects),
        92);
    spi.config_buffers(&mut spi_read_buf, &mut spi_write_buf);
    sam4l::spi::SPI.set_client(spi);
    sam4l::spi::SPI.init();
    sam4l::spi::SPI.enable();

    // Configure the SI7021, device address 0x40
    let si7021_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm),
        24);
    let si7021_i2c = static_init!(I2CDevice, I2CDevice::new(mux_i2c, 0x40), 32);
    let si7021 = static_init!(
        capsules::si7021::SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        capsules::si7021::SI7021::new(si7021_i2c, si7021_alarm, &mut capsules::si7021::BUFFER),
        36);
    si7021_i2c.set_client(si7021);
    si7021_alarm.set_client(si7021);

    // Clear sensors enable pin to enable sensor rail
    sam4l::gpio::PC[16].enable_output();
    sam4l::gpio::PC[16].clear();

    // # ADC

    // Setup ADC
    let adc = static_init!(
        capsules::adc::ADC<'static, sam4l::adc::Adc>,
        capsules::adc::ADC::new(&mut sam4l::adc::ADC),
        160/8);
    sam4l::adc::ADC.set_client(adc);

    // # GPIO

    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 11],
        [&sam4l::gpio::PC[31], // P2
         &sam4l::gpio::PC[30], // P3
         &sam4l::gpio::PC[29], // P4
         &sam4l::gpio::PC[28], // P5
         &sam4l::gpio::PC[27], // P6
         &sam4l::gpio::PC[26], // P7
         &sam4l::gpio::PC[25], // P8
         &sam4l::gpio::PC[25], // Dummy Pin (regular GPIO)
         &sam4l::gpio::PA[10], // RSLP 
         &sam4l::gpio::PA[09], // RRST
         &sam4l::gpio::PA[08]], // RIRQ
        11 * 4
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        20);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // # LEDs

    let led_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 1],
        [&sam4l::gpio::PC[10]],
        1 * 4);
    let led = static_init!(
        capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins, capsules::led::ActivationMode::ActiveHigh),
        96/8);
    // # BUTTONs

    let button_pins = static_init!(
        [&'static sam4l::gpio::GPIOPin; 1],
        [&sam4l::gpio::PC[24]],
        1 * 4);

    let button = static_init!(
        capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Container::create()),
        96/8);
    for btn in button_pins.iter() {
        btn.set_client(button);
    }


    let mut imix = Imix {
        console: console,
        ble_adv: ble_adv,
        timer: timer,
        gpio: gpio,
        si7021: si7021,
        isl29035: isl29035,
        adc: adc,
        led: led,
        button: button,
        spi: spi,
    };


    let mut chip = sam4l::chip::Sam4l::new();
    //spi_dummy::spi_dummy_test(); 
    chip.mpu().enable_mpu();
    kernel::main(&mut imix, &mut chip, load_processes());
}

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
        panic!("Exceeded maximum NUM_PROCS. {:#x}", *(addr as *const usize));
    }

    &mut processes
}
