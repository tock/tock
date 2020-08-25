//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.
//!
//! Pin Configuration
//! -------------------
//!
//! ### `GPIO`
//!
//! | #  | Pin   | Ix | Header | Arduino |
//! |----|-------|----|--------|---------|
//! | 0  | P1.01 | 33 | P3 1   | D0      |
//! | 1  | P1.02 | 34 | P3 2   | D1      |
//! | 2  | P1.03 | 35 | P3 3   | D2      |
//! | 3  | P1.04 | 36 | P3 4   | D3      |
//! | 4  | P1.05 | 37 | P3 5   | D4      |
//! | 5  | P1.06 | 38 | P3 6   | D5      |
//! | 6  | P1.07 | 39 | P3 7   | D6      |
//! | 7  | P1.08 | 40 | P3 8   | D7      |
//! | 8  | P1.10 | 42 | P4 1   | D8      |
//! | 9  | P1.11 | 43 | P4 2   | D9      |
//! | 10 | P1.12 | 44 | P4 3   | D10     |
//! | 11 | P1.13 | 45 | P4 4   | D11     |
//! | 12 | P1.14 | 46 | P4 5   | D12     |
//! | 13 | P1.15 | 47 | P4 6   | D13     |
//! | 14 | P0.26 | 26 | P4 9   | D14     |
//! | 15 | P0.27 | 27 | P4 10  | D15     |
//!
//! ### `GPIO` / Analog Inputs
//!
//! | #  | Pin        | Header | Arduino |
//! |----|------------|--------|---------|
//! | 16 | P0.03 AIN1 | P2 1   | A0      |
//! | 17 | P0.04 AIN2 | P2 2   | A1      |
//! | 18 | P0.28 AIN4 | P2 3   | A2      |
//! | 19 | P0.29 AIN5 | P2 4   | A3      |
//! | 20 | P0.30 AIN6 | P2 5   | A4      |
//! | 21 | P0.31 AIN7 | P2 6   | A5      |
//! | 22 | P0.02 AIN0 | P4 8   | AVDD    |
//!
//! ### Onboard Functions
//!
//! | Pin   | Header | Function |
//! |-------|--------|----------|
//! | P0.05 | P6 3   | UART RTS |
//! | P0.06 | P6 4   | UART TXD |
//! | P0.07 | P6 5   | UART CTS |
//! | P0.08 | P6 6   | UART RXT |
//! | P0.11 | P24 1  | Button 1 |
//! | P0.12 | P24 2  | Button 2 |
//! | P0.13 | P24 3  | LED 1    |
//! | P0.14 | P24 4  | LED 2    |
//! | P0.15 | P24 5  | LED 3    |
//! | P0.16 | P24 6  | LED 4    |
//! | P0.18 | P24 8  | Reset    |
//! | P0.19 | P24 9  | SPI CLK  |
//! | P0.20 | P24 10 | SPI MOSI |
//! | P0.21 | P24 11 | SPI MISO |
//! | P0.24 | P24 14 | Button 3 |
//! | P0.25 | P24 15 | Button 4 |

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]
#![feature(const_in_array_repeat_expressions)]

use capsules::net::ieee802154::MacAddress;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::time::Counter;
#[allow(unused_imports)]
use kernel::hil::usb::Client;
#[allow(unused_imports)]
use kernel::{capabilities, create_capability, debug, debug_gpio, debug_verbose, static_init};
use nrf52840::gpio::Pin;
use nrf52_components::{self, UartChannel, UartPins};

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_13;
const LED2_PIN: Pin = Pin::P0_14;
const LED3_PIN: Pin = Pin::P0_15;
const LED4_PIN: Pin = Pin::P0_16;

// The nRF52840DK buttons (see back of board)
const BUTTON1_PIN: Pin = Pin::P0_11;
const BUTTON2_PIN: Pin = Pin::P0_12;
const BUTTON3_PIN: Pin = Pin::P0_24;
const BUTTON4_PIN: Pin = Pin::P0_25;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

const SPI_MOSI: Pin = Pin::P0_20;
const SPI_MISO: Pin = Pin::P0_21;
const SPI_CLK: Pin = Pin::P0_19;

const SPI_MX25R6435F_CHIP_SELECT: Pin = Pin::P0_17;
const SPI_MX25R6435F_WRITE_PROTECT_PIN: Pin = Pin::P0_22;
const SPI_MX25R6435F_HOLD_PIN: Pin = Pin::P0_23;

// Constants related to the configuration of the 15.4 network stack
const PAN_ID: u16 = 0xABCD;
const DST_MAC_ADDR: capsules::net::ieee802154::MacAddress =
    capsules::net::ieee802154::MacAddress::Short(49138);
const DEFAULT_CTX_PREFIX_LEN: u8 = 8; //Length of context for 6LoWPAN compression
const DEFAULT_CTX_PREFIX: [u8; 16] = [0x0 as u8; 16]; //Context for 6LoWPAN Compression

/// Debug Writer
pub mod io;

// Whether to use UART debugging or Segger RTT (USB) debugging.
// - Set to false to use UART.
// - Set to true to use Segger RTT over USB.
const USB_DEBUGGING: bool = false;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

nrf52840::create_default_nrf52840_peripherals!(Nrf52840Peripherals);
static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840Peripherals>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52840::ble_radio::Radio<'static>,
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    ieee802154_radio: &'static capsules::ieee802154::RadioDriver<'static>,
    button: &'static capsules::button::Button<'static, nrf52840::gpio::GPIOPin<'static>>,
    pconsole: &'static capsules::process_console::ProcessConsole<
        'static,
        components::process_console::Capability,
    >,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52840::gpio::GPIOPin<'static>>,
    led: &'static capsules::led::LED<'static, nrf52840::gpio::GPIOPin<'static>>,
    rng: &'static capsules::rng::RngDriver<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    analog_comparator: &'static capsules::analog_comparator::AnalogComparator<
        'static,
        nrf52840::acomp::Comparator<'static>,
    >,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    nonvolatile_storage: &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    udp_driver: &'static capsules::net::udp::UDPDriver<'static>,
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
            capsules::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_radio)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::analog_comparator::DRIVER_NUM => f(Some(self.analog_comparator)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => f(Some(self.nonvolatile_storage)),
            capsules::net::udp::DRIVER_NUM => f(Some(self.udp_driver)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52840::init();
    let ppi = static_init!(nrf52840::ppi::Ppi, nrf52840::ppi::Ppi::new());
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(Nrf52840Peripherals, Nrf52840Peripherals::new(ppi));

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52_base;

    let uart_channel = if USB_DEBUGGING {
        // Initialize early so any panic beyond this point can use the RTT memory object.
        let mut rtt_memory_refs =
            components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());

        // XXX: This is inherently unsafe as it aliases the mutable reference to rtt_memory. This
        // aliases reference is only used inside a panic handler, which should be OK, but maybe we
        // should use a const reference to rtt_memory and leverage interior mutability instead.
        self::io::set_rtt_memory(&mut *rtt_memory_refs.get_rtt_memory_ptr());

        UartChannel::Rtt(rtt_memory_refs)
    } else {
        UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD))
    };

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            0 => &base_peripherals.gpio_port[Pin::P1_01],
            1 => &base_peripherals.gpio_port[Pin::P1_02],
            2 => &base_peripherals.gpio_port[Pin::P1_03],
            3 => &base_peripherals.gpio_port[Pin::P1_04],
            4 => &base_peripherals.gpio_port[Pin::P1_05],
            5 => &base_peripherals.gpio_port[Pin::P1_06],
            6 => &base_peripherals.gpio_port[Pin::P1_07],
            7 => &base_peripherals.gpio_port[Pin::P1_08],
            8 => &base_peripherals.gpio_port[Pin::P1_10],
            9 => &base_peripherals.gpio_port[Pin::P1_11],
            10 => &base_peripherals.gpio_port[Pin::P1_12],
            11 => &base_peripherals.gpio_port[Pin::P1_13],
            12 => &base_peripherals.gpio_port[Pin::P1_14],
            13 => &base_peripherals.gpio_port[Pin::P1_15],
            14 => &base_peripherals.gpio_port[Pin::P0_26],
            15 => &base_peripherals.gpio_port[Pin::P0_27]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &base_peripherals.gpio_port[BUTTON1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //13
            (
                &base_peripherals.gpio_port[BUTTON2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //14
            (
                &base_peripherals.gpio_port[BUTTON3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //15
            (
                &base_peripherals.gpio_port[BUTTON4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ) //16
        ),
    )
    .finalize(components::button_component_buf!(nrf52840::gpio::GPIOPin));

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        nrf52840::gpio::GPIOPin,
        (
            &base_peripherals.gpio_port[LED1_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &base_peripherals.gpio_port[LED2_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &base_peripherals.gpio_port[LED3_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &base_peripherals.gpio_port[LED4_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    ))
    .finalize(components::led_component_buf!(nrf52840::gpio::GPIOPin));

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840Peripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    nrf52_components::startup::NrfStartupComponent::new(
        false,
        BUTTON_RST_PIN,
        nrf52840::uicr::Regulator0Output::DEFAULT,
    )
    .finalize(());

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let gpio_port = &base_peripherals.gpio_port;
    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&gpio_port[LED1_PIN]),
        Some(&gpio_port[LED2_PIN]),
        Some(&gpio_port[LED3_PIN]),
    );

    let rtc = &base_peripherals.rtc;
    rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52840::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52840::rtc::Rtc));

    let channel = nrf52_components::UartChannelComponent::new(
        uart_channel,
        mux_alarm,
        &base_peripherals.uarte0,
    )
    .finalize(());

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
        nrf52_components::BLEComponent::new(board_kernel, &base_peripherals.ble_radio, mux_alarm)
            .finalize(());

    let serial_num = nrf52840::ficr::FICR_INSTANCE.address();
    let serial_num_bottom_16 = serial_num[0] as u16 + ((serial_num[1] as u16) << 8);
    let src_mac_from_serial_num: MacAddress = MacAddress::Short(serial_num_bottom_16);
    let (ieee802154_radio, mux_mac) = components::ieee802154::Ieee802154Component::new(
        board_kernel,
        &base_peripherals.ieee802154_radio,
        &base_peripherals.ecb,
        PAN_ID,
        serial_num_bottom_16,
    )
    .finalize(components::ieee802154_component_helper!(
        nrf52840::ieee802154_radio::Radio,
        nrf52840::aes::AesECB<'static>
    ));

    let local_ip_ifaces = static_init!(
        [IPAddr; 3],
        [
            IPAddr([
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f,
            ]),
            IPAddr([
                0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
                0x1e, 0x1f,
            ]),
            IPAddr::generate_from_mac(capsules::net::ieee802154::MacAddress::Short(
                serial_num_bottom_16
            )),
        ]
    );

    let (udp_send_mux, udp_recv_mux, udp_port_table) = components::udp_mux::UDPMuxComponent::new(
        mux_mac,
        DEFAULT_CTX_PREFIX_LEN,
        DEFAULT_CTX_PREFIX,
        DST_MAC_ADDR,
        src_mac_from_serial_num,
        local_ip_ifaces,
        mux_alarm,
    )
    .finalize(components::udp_mux_component_helper!(nrf52840::rtc::Rtc));

    // UDP driver initialization happens here
    let udp_driver = components::udp_driver::UDPDriverComponent::new(
        board_kernel,
        udp_send_mux,
        udp_recv_mux,
        udp_port_table,
        local_ip_ifaces,
    )
    .finalize(components::udp_driver_component_helper!(nrf52840::rtc::Rtc));

    let temp =
        components::temperature::TemperatureComponent::new(board_kernel, &base_peripherals.temp)
            .finalize(());

    let rng = components::rng::RngComponent::new(board_kernel, &base_peripherals.trng).finalize(());

    // SPI
    let mux_spi = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_helper!(nrf52840::spi::SPIM));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(SPI_MOSI as u32),
        nrf52840::pinmux::Pinmux::new(SPI_MISO as u32),
        nrf52840::pinmux::Pinmux::new(SPI_CLK as u32),
    );

    let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
        &gpio_port[SPI_MX25R6435F_WRITE_PROTECT_PIN],
        &gpio_port[SPI_MX25R6435F_HOLD_PIN],
        &gpio_port[SPI_MX25R6435F_CHIP_SELECT] as &dyn kernel::hil::gpio::Pin,
        mux_alarm,
        mux_spi,
    )
    .finalize(components::mx25r6435f_component_helper!(
        nrf52840::spi::SPIM,
        nrf52840::gpio::GPIOPin,
        nrf52840::rtc::Rtc
    ));

    let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
        board_kernel,
        mx25r6435f,
        0x60000, // Start address for userspace accessible region
        0x20000, // Length of userspace accessible region
        0,       // Start address of kernel region
        0x60000, // Length of kernel region
    )
    .finalize(components::nv_storage_component_helper!(
        capsules::mx25r6435f::MX25R6435F<
            'static,
            capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
            nrf52840::gpio::GPIOPin,
            VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        >
    ));

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let analog_comparator = components::analog_comparator::AcComponent::new(
        &base_peripherals.acomp,
        components::acomp_component_helper!(
            nrf52840::acomp::Channel,
            &nrf52840::acomp::CHANNEL_AC0
        ),
    )
    .finalize(components::acomp_component_buf!(
        nrf52840::acomp::Comparator
    ));

    nrf52_components::NrfClockComponent::new().finalize(());

    // let alarm_test_component =
    //     components::test::multi_alarm_test::MultiAlarmTestComponent::new(&mux_alarm).finalize(
    //         components::multi_alarm_test_component_buf!(nrf52840::rtc::Rtc),
    //     );

    //--------------------------------------------------------------------------
    // USB CTAP EXAMPLE
    //--------------------------------------------------------------------------
    // Uncomment to experiment with this.

    // // Create the strings we include in the USB descriptor.
    // let strings = static_init!(
    //     [&str; 3],
    //     [
    //         "Nordic Semiconductor", // Manufacturer
    //         "nRF52840dk - TockOS",  // Product
    //         "serial0001",           // Serial number
    //     ]
    // );

    // let ctap_send_buffer = static_init!([u8; 64], [0; 64]);
    // let ctap_recv_buffer = static_init!([u8; 64], [0; 64]);

    // let (ctap, _ctap_driver) = components::ctap::CtapComponent::new(
    //     &peripherals.usbd,
    //     0x1915, // Nordic Semiconductor
    //     0x503a, // lowRISC generic FS USB
    //     strings,
    //     board_kernel,
    //     ctap_send_buffer,
    //     ctap_recv_buffer,
    // )
    // .finalize(components::usb_ctap_component_helper!(nrf52840::usbd::Usbd));

    // ctap.enable();
    // ctap.attach();

    let platform = Platform {
        button,
        ble_radio,
        ieee802154_radio,
        pconsole,
        console,
        led,
        gpio,
        rng,
        temp,
        alarm,
        analog_comparator,
        nonvolatile_storage,
        udp_driver,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52840::ficr::FICR_INSTANCE);

    // alarm_test_component.run();

    /// These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.ipc),
        scheduler,
        &main_loop_capability,
    );
}
