//! Shared setup for nrf52dk boards.

#![no_std]

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use capsules::analog_comparator;
use capsules::virtual_alarm::VirtualMuxAlarm;
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use capsules::lora::radio::{RadioConfig};
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::radio;
use nrf52::gpio::{GPIOPin, Pin};
use nrf52::rtc::Rtc;
use nrf52::uicr::Regulator0Output;

use components::spi::{SpiComponent, SpiMuxComponent, SpiSyscallComponent};
pub mod nrf52_components;
use nrf52_components::ble::BLEComponent;
use nrf52_components::ieee802154::Ieee802154Component;
use nrf52_components::lora::LoraComponent;
// Constants related to the configuration of the 15.4 network stack
const SRC_MAC: u16 = 0xf00f;
const PAN_ID: u16 = 0xABCD;

// The LoRa radio requires buffers for its SPI operations:
static mut LORA_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];
static mut LORA_REG_WRITE: [u8; 2] = [0x00; 2];
static mut LORA_REG_READ: [u8; 2] = [0x00; 2];

/// Pins for SPI for the flash chip MX25R6435F
#[derive(Debug)]
pub struct SpiMX25R6435FPins {
    chip_select: Pin,
    write_protect_pin: Pin,
    hold_pin: Pin,
}

impl SpiMX25R6435FPins {
    pub fn new(chip_select: Pin, write_protect_pin: Pin, hold_pin: Pin) -> Self {
        Self {
            chip_select,
            write_protect_pin,
            hold_pin,
        }
    }
}

/// Pins for the SPI driver
#[derive(Debug)]
pub struct SpiPins {
    mosi: Pin,
    miso: Pin,
    clk: Pin,
}

impl SpiPins {
    pub fn new(mosi: Pin, miso: Pin, clk: Pin) -> Self {
        Self { mosi, miso, clk }
    }
}

/// Pins for the UART
#[derive(Debug)]
pub struct UartPins {
    rts: Option<Pin>,
    txd: Pin,
    cts: Option<Pin>,
    rxd: Pin,
}

impl UartPins {
    pub fn new(rts: Option<Pin>, txd: Pin, cts: Option<Pin>, rxd: Pin) -> Self {
        Self { rts, txd, cts, rxd }
    }
}

pub enum UartChannel<'a> {
    Pins(UartPins),
    Rtt(components::segger_rtt::SeggerRttMemoryRefs<'a>),
}

/// Pins for the LoRa Module
pub struct LoraPins {
    chip_select: Pin,
    reset: Pin,
    interrupt: Pin,
}
impl LoraPins {
    pub fn new(
    chip_select: Pin,
    reset: Pin,
    interrupt: Pin,
) -> Self {
        Self { chip_select, reset, interrupt }
    }
}

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52::ble_radio::Radio,
        VirtualMuxAlarm<'static, Rtc<'static>>,
    >,
    ieee802154_radio: Option<&'static capsules::ieee802154::RadioDriver<'static>>,
    lora_radio: Option<&'static capsules::lora::driver::RadioDriver<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>>,
    //spi: &'static capsules::spi::Spi<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
    button: &'static capsules::button::Button<'static>,
    pconsole: &'static capsules::process_console::ProcessConsole<
        'static,
        components::process_console::Capability,
    >,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    rng: &'static capsules::rng::RngDriver<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    analog_comparator:
        &'static capsules::analog_comparator::AnalogComparator<'static, nrf52::acomp::Comparator>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
    // The nRF52dk does not have the flash chip on it, so we make this optional.
    nonvolatile_storage:
        Option<&'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::ieee802154::DRIVER_NUM => match self.ieee802154_radio {
                Some(radio) => f(Some(radio)),
                None => f(None),
            },
            capsules::lora::driver::DRIVER_NUM => match self.lora_radio {
                Some(radio) => f(Some(radio)),
                None => f(None),
            },
            //capsules::spi::DRIVER_NUM => f(Some(self.spi)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::analog_comparator::DRIVER_NUM => f(Some(self.analog_comparator)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => {
                f(self.nonvolatile_storage.map_or(None, |nv| Some(nv)))
            }
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Generic function for starting an nrf52dk board.
#[inline]
pub unsafe fn setup_board<I: nrf52::interrupt_service::InterruptService>(
    board_kernel: &'static kernel::Kernel,
    button_rst_pin: Pin,
    gpio_port: &'static nrf52::gpio::Port,
    gpio: &'static capsules::gpio::GPIO<'static>,
    debug_pin1_index: Pin,
    debug_pin2_index: Pin,
    debug_pin3_index: Pin,
    led: &'static capsules::led::LED<'static>,
    uart_channel: UartChannel<'static>,
    spi_pins: &SpiPins,
    mx25r6435f: &Option<SpiMX25R6435FPins>,
    button: &'static capsules::button::Button<'static>,
    ieee802154: bool,
    lora: &Option<LoraPins>,
    app_memory: &mut [u8],
    process_pointers: &'static mut [Option<&'static dyn kernel::procs::ProcessType>],
    app_fault_response: kernel::procs::FaultResponse,
    reg_vout: Regulator0Output,
    nfc_as_gpios: bool,
    chip: &'static nrf52::chip::NRF52<I>,
) {
    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52::uicr::Uicr::new();

    // Check if we need to erase UICR memory to re-program it
    // This only needs to be done when a bit needs to be flipped from 0 to 1.
    let psel0_reset: u32 = uicr.get_psel0_reset_pin().map_or(0, |pin| pin as u32);
    let psel1_reset: u32 = uicr.get_psel1_reset_pin().map_or(0, |pin| pin as u32);
    let mut erase_uicr = ((!psel0_reset & (button_rst_pin as u32))
        | (!psel1_reset & (button_rst_pin as u32))
        | (!(uicr.get_vout() as u32) & (reg_vout as u32)))
        != 0;

    // Only enabling the NFC pin protection requires an erase.
    if nfc_as_gpios {
        erase_uicr |= !uicr.is_nfc_pins_protection_enabled();
    }

    if erase_uicr {
        nrf52::nvmc::NVMC.erase_uicr();
    }

    nrf52::nvmc::NVMC.configure_writeable();
    while !nrf52::nvmc::NVMC.is_ready() {}

    let mut needs_soft_reset: bool = false;

    // Configure reset pins
    if uicr
        .get_psel0_reset_pin()
        .map_or(true, |pin| pin != button_rst_pin)
    {
        uicr.set_psel0_reset_pin(button_rst_pin);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }
    if uicr
        .get_psel1_reset_pin()
        .map_or(true, |pin| pin != button_rst_pin)
    {
        uicr.set_psel1_reset_pin(button_rst_pin);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Configure voltage regulator output
    if uicr.get_vout() != reg_vout {
        uicr.set_vout(reg_vout);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Check if we need to free the NFC pins for GPIO
    if nfc_as_gpios {
        uicr.set_nfc_pins_protection(true);
        while !nrf52::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Any modification of UICR needs a soft reset for the changes to be taken into account.
    if needs_soft_reset {
        cortexm4::scb::reset();
    }

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&gpio_port[debug_pin1_index]),
        Some(&gpio_port[debug_pin2_index]),
        Some(&gpio_port[debug_pin3_index]),
    );

    let rtc = &nrf52::rtc::RTC;
    rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52::rtc::Rtc));

    let channel: &dyn kernel::hil::uart::Uart = match uart_channel {
        UartChannel::Pins(uart_pins) => {
            nrf52::uart::UARTE0.initialize(
                nrf52::pinmux::Pinmux::new(uart_pins.txd as u32),
                nrf52::pinmux::Pinmux::new(uart_pins.rxd as u32),
                uart_pins.cts.map(|x| nrf52::pinmux::Pinmux::new(x as u32)),
                uart_pins.rts.map(|x| nrf52::pinmux::Pinmux::new(x as u32)),
            );
            &nrf52::uart::UARTE0
        }
        UartChannel::Rtt(rtt_memory) => {
            let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
                .finalize(components::segger_rtt_component_helper!(nrf52::rtc::Rtc));
            rtt
        }
    };

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux =
        components::console::UartMuxComponent::new(channel, 115200, dynamic_deferred_caller)
            .finalize(());

    let pconsole =
        components::process_console::ProcessConsoleComponent::new(board_kernel, uart_mux)
            .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let ble_radio =
        BLEComponent::new(board_kernel, &nrf52::ble_radio::RADIO, mux_alarm).finalize(());

    let ieee802154_radio = if ieee802154 {
        let (radio, _) = Ieee802154Component::new(
            board_kernel,
            &nrf52::ieee802154_radio::RADIO,
            PAN_ID,
            SRC_MAC,
        )
        .finalize(());
        Some(radio)
    } else {
        None
    };

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(
            &nrf52::temperature::TEMP,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf52::temperature::TEMP, temp);

    let rng = components::rng::RngComponent::new(board_kernel, &nrf52::trng::TRNG).finalize(());

    let mux_spi = SpiMuxComponent::new(&nrf52::spi::SPIM0)
        .finalize(components::spi_mux_component_helper!(nrf52::spi::SPIM));
    nrf52::spi::SPIM0.configure(
        nrf52::pinmux::Pinmux::new(spi_pins.mosi as u32),
        nrf52::pinmux::Pinmux::new(spi_pins.miso as u32),
        nrf52::pinmux::Pinmux::new(spi_pins.clk as u32),
    );

    //userland SPI --- for test
    //let spi = SpiSyscallComponent::new(mux_spi, &gpio_port[Pin::P1_10])   //fix it
    //      .finalize(components::spi_syscall_component_helper!(nrf52::spi::SPIM));

    // SPI and Lora radio
    let lora_radio = if let Some(pins) = lora {
      let lora_spi = SpiComponent::new(mux_spi, &gpio_port[pins.chip_select])
          .finalize(components::spi_component_helper!(nrf52::spi::SPIM)); //virtual

      let RADIO = static_init!(
        capsules::lora::radio::Radio<'static, VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>>,
        capsules::lora::radio::Radio::new(
          lora_spi,
          &gpio_port[pins.reset],
          &gpio_port[pins.interrupt],
        )
      );

      let (radio,) = LoraComponent::new(
        board_kernel,
        RADIO,
      )
      .finalize(());
      //lora_spi.set_client(radio);      
    Some(radio)
    } else {
      None
    };
    match lora_radio {
      Some(radio) => { 
        radio.device.initialize(&mut LORA_BUF, &mut LORA_REG_WRITE, &mut LORA_REG_READ);
      },
      None => ()
    }

    let nonvolatile_storage: Option<
        &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    > = if let Some(driver) = mx25r6435f {
        // Create a SPI device for the mx25r6435f flash chip.
        let mx25r6435f_spi = static_init!(
            capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
            capsules::virtual_spi::VirtualSpiMasterDevice::new(
                mux_spi,
                &gpio_port[driver.chip_select]
            )
        );
        // Create an alarm for this chip.
        let mx25r6435f_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
            VirtualMuxAlarm::new(mux_alarm)
        );
        // Setup the actual MX25R6435F driver.
        let mx25r6435f = static_init!(
            capsules::mx25r6435f::MX25R6435F<
                'static,
                capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
                nrf52::gpio::GPIOPin,
                VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
            >,
            capsules::mx25r6435f::MX25R6435F::new(
                mx25r6435f_spi,
                mx25r6435f_virtual_alarm,
                &mut capsules::mx25r6435f::TXBUFFER,
                &mut capsules::mx25r6435f::RXBUFFER,
                Some(&gpio_port[driver.write_protect_pin]),
                Some(&gpio_port[driver.hold_pin])
            )
        );
        mx25r6435f_spi.set_client(mx25r6435f);
        hil::time::Alarm::set_client(mx25r6435f_virtual_alarm, mx25r6435f);

        pub static mut FLASH_PAGEBUFFER: capsules::mx25r6435f::Mx25r6435fSector =
            capsules::mx25r6435f::Mx25r6435fSector::new();
        let nv_to_page = static_init!(
            capsules::nonvolatile_to_pages::NonvolatileToPages<
                'static,
                capsules::mx25r6435f::MX25R6435F<
                    'static,
                    capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
                    nrf52::gpio::GPIOPin,
                    VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
                >,
            >,
            capsules::nonvolatile_to_pages::NonvolatileToPages::new(
                mx25r6435f,
                &mut FLASH_PAGEBUFFER
            )
        );
        hil::flash::HasClient::set_client(mx25r6435f, nv_to_page);

        let nonvolatile_storage = static_init!(
            capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
            capsules::nonvolatile_storage_driver::NonvolatileStorage::new(
                nv_to_page,
                board_kernel.create_grant(&memory_allocation_capability),
                0x60000, // Start address for userspace accessible region
                0x20000, // Length of userspace accessible region
                0,       // Start address of kernel accessible region
                0x60000, // Length of kernel accessible region
                &mut capsules::nonvolatile_storage_driver::BUFFER
            )
        );
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        Some(nonvolatile_storage)
    } else {
        None
    };

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let ac_channels = static_init!(
        [&'static nrf52::acomp::Channel; 1],
        [&nrf52::acomp::CHANNEL_AC0,]
    );
    let analog_comparator = static_init!(
        analog_comparator::AnalogComparator<'static, nrf52::acomp::Comparator>,
        analog_comparator::AnalogComparator::new(&mut nrf52::acomp::ACOMP, ac_channels)
    );
    nrf52::acomp::ACOMP.set_client(analog_comparator);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();

    nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_set_source(nrf52::clock::HighClockSource::XTAL);
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}

    let platform = Platform {
        button: button,
        ble_radio: ble_radio,
        ieee802154_radio: ieee802154_radio,
        lora_radio: lora_radio,
        //spi: spi,
        pconsole: pconsole,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        temp: temp,
        alarm: alarm,
        analog_comparator: analog_comparator,
        nonvolatile_storage: nonvolatile_storage,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    // These two lines need to be below the creation of the chip for
    // initialization to work.
    match lora_radio {
      Some(radio) => {
      radio.device.reset();
      radio.device.start();
      },
      None => ()
    }
    
    platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52::ficr::FICR_INSTANCE);

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        app_memory,
        process_pointers,
        app_fault_response,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
