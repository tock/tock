//! Shared setup for nrf52dk boards.

#![no_std]

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use capsules::virtual_alarm::VirtualMuxAlarm;
use capsules::virtual_spi::MuxSpiMaster;
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::hil;
use kernel::hil::entropy::Entropy32;
use kernel::hil::rng::Rng;
use nrf5x::rtc::Rtc;

use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};

/// Pins for SPI for the flash chip MX25R6435F
#[derive(Debug)]
pub struct SpiMX25R6435FPins {
    chip_select: usize,
    write_protect_pin: usize,
    hold_pin: usize,
}

impl SpiMX25R6435FPins {
    pub fn new(chip_select: usize, write_protect_pin: usize, hold_pin: usize) -> Self {
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
    mosi: usize,
    miso: usize,
    clk: usize,
}

impl SpiPins {
    pub fn new(mosi: usize, miso: usize, clk: usize) -> Self {
        Self { mosi, miso, clk }
    }
}

/// Pins for the UART
#[derive(Debug)]
pub struct UartPins {
    rts: usize,
    txd: usize,
    cts: usize,
    rxd: usize,
}

impl UartPins {
    pub fn new(rts: usize, txd: usize, cts: usize, rxd: usize) -> Self {
        Self { rts, txd, cts, rxd }
    }
}

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52::radio::Radio,
        VirtualMuxAlarm<'static, Rtc>,
    >,
    button: &'static capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
    rng: &'static capsules::rng::RngDriver<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
    >,
    // The nRF52dk does not have the flash chip on it, so we make this optional.
    nonvolatile_storage:
        Option<&'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
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
pub unsafe fn setup_board(
    board_kernel: &'static kernel::Kernel,
    button_rst_pin: usize,
    gpio_pins: &'static mut [&'static nrf5x::gpio::GPIOPin],
    debug_pin1_index: usize,
    debug_pin2_index: usize,
    debug_pin3_index: usize,
    led_pins: &'static mut [(&'static nrf5x::gpio::GPIOPin, capsules::led::ActivationMode)],
    uart_pins: &UartPins,
    spi_pins: &SpiPins,
    mx25r6435f: &Option<SpiMX25R6435FPins>,
    button_pins: &'static mut [(&'static nrf5x::gpio::GPIOPin, capsules::button::GpioMode)],
    app_memory: &mut [u8],
    process_pointers: &'static mut [Option<&'static kernel::procs::ProcessType>],
    app_fault_response: kernel::procs::FaultResponse,
) {
    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52::uicr::Uicr::new();
    nrf52::nvmc::NVMC.erase_uicr();
    nrf52::nvmc::NVMC.configure_writeable();
    while !nrf52::nvmc::NVMC.is_ready() {}
    uicr.set_psel0_reset_pin(button_rst_pin);
    while !nrf52::nvmc::NVMC.is_ready() {}
    uicr.set_psel1_reset_pin(button_rst_pin);

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&nrf5x::gpio::PORT[debug_pin1_index]),
        Some(&nrf5x::gpio::PORT[debug_pin2_index]),
        Some(&nrf5x::gpio::PORT[debug_pin3_index]),
    );

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(
            gpio_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // LEDs
    let led = static_init!(
        capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // Buttons
    let button = static_init!(
        capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    for &(btn, _) in button_pins.iter() {
        use kernel::hil::gpio::PinCtl;
        btn.set_input_mode(kernel::hil::gpio::InputMode::PullUp);
        btn.set_client(button);
    }

    let rtc = &nrf5x::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf5x::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm1,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    virtual_alarm1.set_client(alarm);
    let ble_radio_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &nrf52::uart::UARTE0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    uart_mux.initialize();
    hil::uart::Transmit::set_transmit_client(&nrf52::uart::UARTE0, uart_mux);
    hil::uart::Receive::set_receive_client(&nrf52::uart::UARTE0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

    nrf52::uart::UARTE0.initialize(
        nrf5x::pinmux::Pinmux::new(uart_pins.txd as u32),
        nrf5x::pinmux::Pinmux::new(uart_pins.rxd as u32),
        nrf5x::pinmux::Pinmux::new(uart_pins.cts as u32),
        nrf5x::pinmux::Pinmux::new(uart_pins.rts as u32),
    );
    let console = static_init!(
        capsules::console::Console<'static>,
        capsules::console::Console::new(
            console_uart,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::uart::Transmit::set_transmit_client(console_uart, console);
    kernel::hil::uart::Receive::set_receive_client(console_uart, console);

    // Create virtual device for kernel debug.
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart,
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::Transmit::set_transmit_client(debugger_uart, debugger);

    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    let ble_radio = static_init!(
        capsules::ble_advertising_driver::BLE<
            'static,
            nrf52::radio::Radio,
            VirtualMuxAlarm<'static, Rtc>,
        >,
        capsules::ble_advertising_driver::BLE::new(
            &mut nrf52::radio::RADIO,
            board_kernel.create_grant(&memory_allocation_capability),
            &mut capsules::ble_advertising_driver::BUF,
            ble_radio_virtual_alarm
        )
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
        &nrf52::radio::RADIO,
        ble_radio,
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
        &nrf52::radio::RADIO,
        ble_radio,
    );
    ble_radio_virtual_alarm.set_client(ble_radio);

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(
            &mut nrf5x::temperature::TEMP,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf5x::temperature::TEMP, temp);

    let entropy_to_random = static_init!(
        capsules::rng::Entropy32ToRandom<'static>,
        capsules::rng::Entropy32ToRandom::new(&nrf5x::trng::TRNG)
    );

    let rng = static_init!(
        capsules::rng::RngDriver<'static>,
        capsules::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    nrf5x::trng::TRNG.set_client(entropy_to_random);
    entropy_to_random.set_client(rng);

    // SPI
    let mux_spi = static_init!(
        MuxSpiMaster<'static, nrf52::spi::SPIM>,
        MuxSpiMaster::new(&nrf52::spi::SPIM0)
    );
    hil::spi::SpiMaster::set_client(&nrf52::spi::SPIM0, mux_spi);
    hil::spi::SpiMaster::init(&nrf52::spi::SPIM0);
    nrf52::spi::SPIM0.configure(
        nrf5x::pinmux::Pinmux::new(spi_pins.mosi as u32),
        nrf5x::pinmux::Pinmux::new(spi_pins.miso as u32),
        nrf5x::pinmux::Pinmux::new(spi_pins.clk as u32),
    );

    let nonvolatile_storage: Option<
        &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    > = if let Some(driver) = mx25r6435f {
        // Create a SPI device for the mx25r6435f flash chip.
        let mx25r6435f_spi = static_init!(
            capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
            capsules::virtual_spi::VirtualSpiMasterDevice::new(
                mux_spi,
                &nrf5x::gpio::PORT[driver.chip_select]
            )
        );
        // Create an alarm for this chip.
        let mx25r6435f_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
            VirtualMuxAlarm::new(mux_alarm)
        );
        // Setup the actual MX25R6435F driver.
        let mx25r6435f = static_init!(
            capsules::mx25r6435f::MX25R6435F<
                'static,
                capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
                nrf5x::gpio::GPIOPin,
                VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
            >,
            capsules::mx25r6435f::MX25R6435F::new(
                mx25r6435f_spi,
                mx25r6435f_virtual_alarm,
                &mut capsules::mx25r6435f::TXBUFFER,
                &mut capsules::mx25r6435f::RXBUFFER,
                Some(&nrf5x::gpio::PORT[driver.write_protect_pin]),
                Some(&nrf5x::gpio::PORT[driver.hold_pin])
            )
        );
        mx25r6435f_spi.set_client(mx25r6435f);
        mx25r6435f_virtual_alarm.set_client(mx25r6435f);

        pub static mut FLASH_PAGEBUFFER: capsules::mx25r6435f::Mx25r6435fSector =
            capsules::mx25r6435f::Mx25r6435fSector::new();
        let nv_to_page = static_init!(
            capsules::nonvolatile_to_pages::NonvolatileToPages<
                'static,
                capsules::mx25r6435f::MX25R6435F<
                    'static,
                    capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
                    nrf5x::gpio::GPIOPin,
                    VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
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

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_call = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_call);

    let platform = Platform {
        button: button,
        ble_radio: ble_radio,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        temp: temp,
        alarm: alarm,
        nonvolatile_storage: nonvolatile_storage,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(nrf52::chip::NRF52, nrf52::chip::NRF52::new());

    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52::ficr::FICR_INSTANCE);

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        app_memory,
        process_pointers,
        app_fault_response,
        &process_management_capability,
    );

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
