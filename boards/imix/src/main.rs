//! Board file for Imix development platform.
//!
//! - <https://github.com/tock/tock/tree/master/boards/imix>
//! - <https://github.com/tock/imix>

#![no_std]
#![no_main]
#![feature(in_band_lifetimes)]
#![feature(infer_outlives_requirements)]
#![feature(panic_implementation)]
#![deny(missing_docs)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;
extern crate cortexm4;
extern crate sam4l;

mod components;

use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::MuxI2C;
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::radio;
#[allow(unused_imports)]
use kernel::hil::radio::{RadioConfig, RadioData};
use kernel::hil::spi::SpiMaster;
use kernel::hil::Controller;

use components::adc::AdcComponent;
use components::alarm::AlarmDriverComponent;
use components::button::ButtonComponent;
use components::console::ConsoleComponent;
use components::crc::CrcComponent;
use components::fxos8700::NineDofComponent;
use components::gpio::GpioComponent;
use components::isl29035::AmbientLightComponent;
use components::led::LedComponent;
use components::nonvolatile_storage::NonvolatileStorageComponent;
use components::nrf51822::Nrf51822Component;
use components::radio::RadioComponent;
use components::rf233::RF233Component;
use components::si7021::{HumidityComponent, SI7021Component, TemperatureComponent};
use components::spi::{SpiComponent, SpiSyscallComponent};
use components::usb::UsbComponent;

/// Support routines for debugging I/O.
///
/// Note: Use of this module will trample any other USART3 configuration.
#[macro_use]
pub mod io;

// Unit Tests for drivers.
#[allow(dead_code)]
mod i2c_dummy;
#[allow(dead_code)]
mod icmp_lowpan_test;
#[allow(dead_code)]
mod ipv6_lowpan_test;
#[allow(dead_code)]
mod spi_dummy;
#[allow(dead_code)]
mod udp_lowpan_test;

#[allow(dead_code)]
mod aes_test;

#[allow(dead_code)]
mod aes_ccm_test;

#[allow(dead_code)]
mod power;

// State for loading apps.

const NUM_PROCS: usize = 2;

// how should the kernel respond when a process faults
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 16384] = [0; 16384];

static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] =
    [None, None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

struct Imix {
    console: &'static capsules::console::Console<'static, sam4l::usart::USART>,
    gpio: &'static capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
    alarm: &'static AlarmDriver<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    humidity: &'static capsules::humidity::HumiditySensor<'static>,
    ambient_light: &'static capsules::ambient_light::AmbientLight<'static>,
    adc: &'static capsules::adc::Adc<'static, sam4l::adc::Adc>,
    led: &'static capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
    spi: &'static capsules::spi::Spi<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>,
    ipc: kernel::ipc::IPC,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
    radio_driver: &'static capsules::ieee802154::RadioDriver<'static>,
    crc: &'static capsules::crc::Crc<'static, sam4l::crccu::Crccu<'static>>,
    usb_driver: &'static capsules::usb_user::UsbSyscallDriver<
        'static,
        capsules::usbc_client::Client<'static, sam4l::usbc::Usbc<'static>>,
    >,
    nrf51822: &'static capsules::nrf51822_serialization::Nrf51822Serialization<
        'static,
        sam4l::usart::USART,
    >,
    nonvolatile_storage: &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
}

// The RF233 radio stack requires our buffers for its SPI operations:
//
//   1. buf: a packet-sized buffer for SPI operations, which is
//      used as the read buffer when it writes a packet passed to it and the write
//      buffer when it reads a packet into a buffer passed to it.
//   2. rx_buf: buffer to receive packets into
//   3 + 4: two small buffers for performing registers
//      operations (one read, one write).

static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];
static mut RF233_REG_WRITE: [u8; 2] = [0x00; 2];
static mut RF233_REG_READ: [u8; 2] = [0x00; 2];

impl kernel::Platform for Imix {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::spi::DRIVER_NUM => f(Some(self.spi)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::ambient_light::DRIVER_NUM => f(Some(self.ambient_light)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            capsules::crc::DRIVER_NUM => f(Some(self.crc)),
            capsules::usb_user::DRIVER_NUM => f(Some(self.usb_driver)),
            capsules::ieee802154::DRIVER_NUM => f(Some(self.radio_driver)),
            capsules::nrf51822_serialization::DRIVER_NUM => f(Some(self.nrf51822)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => f(Some(self.nonvolatile_storage)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::PeripheralFunction::{A, B, C, E};
    use sam4l::gpio::{PA, PB, PC};

    // Right column: Imix pin name
    // Left  column: SAM4L peripheral function
    PA[04].configure(Some(A)); // AD0         --  ADCIFE AD0
    PA[05].configure(Some(A)); // AD1         --  ADCIFE AD1
    PA[06].configure(Some(C)); // EXTINT1     --  EIC EXTINT1
    PA[07].configure(Some(A)); // AD1         --  ADCIFE AD2
    PA[08].configure(None); //... RF233 IRQ   --  GPIO pin
    PA[09].configure(None); //... RF233 RST   --  GPIO pin
    PA[10].configure(None); //... RF233 SLP   --  GPIO pin
    PA[13].configure(None); //... TRNG EN     --  GPIO pin
    PA[14].configure(None); //... TRNG_OUT    --  GPIO pin
    PA[17].configure(None); //... NRF INT     -- GPIO pin
    PA[18].configure(Some(A)); // NRF CLK     -- USART2_CLK
    PA[20].configure(None); //... D8          -- GPIO pin
    PA[21].configure(Some(E)); // TWI2 SDA    -- TWIM2_SDA
    PA[22].configure(Some(E)); // TWI2 SCL    --  TWIM2 TWCK
    PA[25].configure(Some(A)); // USB_N       --  USB DM
    PA[26].configure(Some(A)); // USB_P       --  USB DP
    PB[00].configure(Some(A)); // TWI1_SDA    --  TWIMS1 TWD
    PB[01].configure(Some(A)); // TWI1_SCL    --  TWIMS1 TWCK
    PB[02].configure(Some(A)); // AD3         --  ADCIFE AD3
    PB[03].configure(Some(A)); // AD4         --  ADCIFE AD4
    PB[04].configure(Some(A)); // AD5         --  ADCIFE AD5
    PB[05].configure(Some(A)); // VHIGHSAMPLE --  ADCIFE AD6
    PB[06].configure(Some(A)); // RTS3        --  USART3 RTS
    PB[07].configure(None); //... NRF RESET   --  GPIO
    PB[09].configure(Some(A)); // RX3         --  USART3 RX
    PB[10].configure(Some(A)); // TX3         --  USART3 TX
    PB[11].configure(Some(A)); // CTS0        --  USART0 CTS
    PB[12].configure(Some(A)); // RTS0        --  USART0 RTS
    PB[13].configure(Some(A)); // CLK0        --  USART0 CLK
    PB[14].configure(Some(A)); // RX0         --  USART0 RX
    PB[15].configure(Some(A)); // TX0         --  USART0 TX
    PC[00].configure(Some(A)); // CS2         --  SPI NPCS2
    PC[01].configure(Some(A)); // CS3 (RF233) --  SPI NPCS3
    PC[02].configure(Some(A)); // CS1         --  SPI NPCS1
    PC[03].configure(Some(A)); // CS0         --  SPI NPCS0
    PC[04].configure(Some(A)); // MISO        --  SPI MISO
    PC[05].configure(Some(A)); // MOSI        --  SPI MOSI
    PC[06].configure(Some(A)); // SCK         --  SPI CLK
    PC[07].configure(Some(B)); // RTS2 (BLE)  -- USART2_RTS
    PC[08].configure(Some(E)); // CTS2 (BLE)  -- USART2_CTS
    PC[09].configure(None); //... NRF GPIO    -- GPIO
    PC[10].configure(None); //... USER LED    -- GPIO
    PC[11].configure(Some(B)); // RX2 (BLE)   -- USART2_RX
    PC[12].configure(Some(B)); // TX2 (BLE)   -- USART2_TX
    PC[13].configure(None); //... ACC_INT1    -- GPIO
    PC[14].configure(None); //... ACC_INT2    -- GPIO
    PC[16].configure(None); //... SENSE_PWR   --  GPIO pin
    PC[17].configure(None); //... NRF_PWR     --  GPIO pin
    PC[18].configure(None); //... RF233_PWR   --  GPIO pin
    PC[19].configure(None); //... TRNG_PWR    -- GPIO Pin
    PC[22].configure(None); //... KERNEL LED  -- GPIO Pin
    PC[24].configure(None); //... USER_BTN    -- GPIO Pin
    PC[25].configure(Some(B)); // LI_INT      --  EIC EXTINT2
    PC[26].configure(None); //... D7          -- GPIO Pin
    PC[27].configure(None); //... D6          -- GPIO Pin
    PC[28].configure(None); //... D5          -- GPIO Pin
    PC[29].configure(None); //... D4          -- GPIO Pin
    PC[30].configure(None); //... D3          -- GPIO Pin
    PC[31].configure(None); //... D2          -- GPIO Pin
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the SAM4L chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    sam4l::pm::PM.setup_system_clock(sam4l::pm::SystemClockSource::PllExternalOscillatorAt48MHz {
        frequency: sam4l::pm::OscillatorFrequency::Frequency16MHz,
        startup_mode: sam4l::pm::OscillatorStartup::FastStart,
    });

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();

    power::configure_submodules(power::SubmoduleConfig {
        rf233: true,
        nrf51422: true,
        sensors: true,
        trng: true,
    });

    let console = ConsoleComponent::new(&sam4l::usart::USART3, 115200).finalize();

    // Allow processes to communicate over BLE through the nRF51822
    let nrf_serialization = Nrf51822Component::new(&sam4l::usart::USART2).finalize();

    // # TIMER
    let ast = &sam4l::ast::AST;
    let mux_alarm = static_init!(
        MuxAlarm<'static, sam4l::ast::Ast>,
        MuxAlarm::new(&sam4l::ast::AST)
    );
    ast.configure(mux_alarm);
    let alarm = AlarmDriverComponent::new(mux_alarm).finalize();

    // # I2C and I2C Sensors
    let mux_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(&sam4l::i2c::I2C2));
    sam4l::i2c::I2C2.set_master_client(mux_i2c);

    let ambient_light = AmbientLightComponent::new(mux_i2c, mux_alarm).finalize();
    let si7021 = SI7021Component::new(mux_i2c, mux_alarm).finalize();
    let temp = TemperatureComponent::new(si7021).finalize();
    let humidity = HumidityComponent::new(si7021).finalize();
    let ninedof = NineDofComponent::new(mux_i2c, &sam4l::gpio::PC[13]).finalize();

    // SPI MUX, SPI syscall driver and RF233 radio
    let mux_spi = static_init!(
        MuxSpiMaster<'static, sam4l::spi::SpiHw>,
        MuxSpiMaster::new(&sam4l::spi::SPI)
    );
    sam4l::spi::SPI.set_client(mux_spi);
    sam4l::spi::SPI.init();

    let spi_syscalls = SpiSyscallComponent::new(mux_spi).finalize();
    let rf233_spi = SpiComponent::new(mux_spi).finalize();
    let rf233 = RF233Component::new(
        rf233_spi,
        &sam4l::gpio::PA[09], // reset
        &sam4l::gpio::PA[10], // sleep
        &sam4l::gpio::PA[08], // irq
        &sam4l::gpio::PA[08],
    ).finalize();

    // Clear sensors enable pin to enable sensor rail
    // sam4l::gpio::PC[16].enable_output();
    // sam4l::gpio::PC[16].clear();

    let adc = AdcComponent::new().finalize();
    let gpio = GpioComponent::new().finalize();
    let led = LedComponent::new().finalize();
    let button = ButtonComponent::new().finalize();
    let crc = CrcComponent::new().finalize();

    // Can this initialize be pushed earlier, or into component? -pal
    rf233.initialize(&mut RF233_BUF, &mut RF233_REG_WRITE, &mut RF233_REG_READ);
    let radio_driver = RadioComponent::new(rf233, 0xABCD, 0x1008).finalize();

    let usb_driver = UsbComponent::new().finalize();
    let nonvolatile_storage = NonvolatileStorageComponent::new().finalize();

    let imix = Imix {
        console: console,
        alarm: alarm,
        gpio: gpio,
        temp: temp,
        humidity: humidity,
        ambient_light: ambient_light,
        adc: adc,
        led: led,
        button: button,
        crc: crc,
        spi: spi_syscalls,
        ipc: kernel::ipc::IPC::new(),
        ninedof: ninedof,
        radio_driver: radio_driver,
        usb_driver: usb_driver,
        nrf51822: nrf_serialization,
        nonvolatile_storage: nonvolatile_storage,
    };

    let mut chip = sam4l::chip::Sam4l::new();

    // Need to reset the nRF on boot, toggle it's SWDIO
    sam4l::gpio::PB[07].enable();
    sam4l::gpio::PB[07].enable_output();
    sam4l::gpio::PB[07].clear();
    // minimum hold time is 200ns, ~20ns per instruction, so overshoot a bit
    for _ in 0..10 {
        cortexm4::support::nop();
    }
    sam4l::gpio::PB[07].set();

    imix.nrf51822.initialize();

    // These two lines need to be below the creation of the chip for
    // initialization to work.
    rf233.reset();
    rf233.start();

    debug!("Initialization complete. Entering main loop");

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new());

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    board_kernel.kernel_loop(&imix, &mut chip, &mut PROCESSES, Some(&imix.ipc));
}
