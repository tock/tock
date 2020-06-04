//! Tock kernel for the Arduino Nano 33 BLE.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::ActivationMode::ActiveLow;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;

// Three-color LED.
const LED_RED_PIN: Pin = Pin::P0_24;
const LED_GREEN_PIN: Pin = Pin::P0_16;
const LED_BLUE_PIN: Pin = Pin::P0_06;

const LED_KERNEL_PIN: Pin = Pin::P0_13;

// const BUTTON_RST_PIN: Pin = Pin::P0_18;

const GPIO_D2: Pin = Pin::P1_11;
const GPIO_D3: Pin = Pin::P1_12;
const GPIO_D4: Pin = Pin::P1_15;
const GPIO_D5: Pin = Pin::P1_13;
const GPIO_D6: Pin = Pin::P1_14;
const GPIO_D7: Pin = Pin::P0_23;
const GPIO_D8: Pin = Pin::P0_21;
const GPIO_D9: Pin = Pin::P0_27;
const GPIO_D10: Pin = Pin::P1_02;

const UART_TX_PIN: Pin = Pin::P1_03;
const UART_RX_PIN: Pin = Pin::P1_10;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 245760] = [0; 245760];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] = [None; 8];

static mut CHIP: Option<&'static nrf52840::chip::Chip> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    // ble_radio: &'static capsules::ble_advertising_driver::BLE<
    //     'static,
    //     nrf52::ble_radio::Radio,
    //     VirtualMuxAlarm<'static, Rtc<'static>>,
    // >,
    // ieee802154_radio: &'static capsules::ieee802154::RadioDriver<'static>,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf52::gpio::GPIOPin>,
    rng: &'static capsules::rng::RngDriver<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    >,
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
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            // capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            // capsules::ieee802154::DRIVER_NUM => f(Some(radio)),
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

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    //--------------------------------------------------------------------------
    // CAPABILITIES
    //--------------------------------------------------------------------------

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    //--------------------------------------------------------------------------
    // DEBUG GPIO
    //--------------------------------------------------------------------------

    // Configure kernel debug GPIOs as early as possible. These are used by the
    // `debug_gpio!(0, toggle)` macro. We configure these early so that the
    // macro is available during most of the setup code and kernel execution.
    kernel::debug::assign_gpios(Some(&nrf52840::gpio::PORT[LED_KERNEL_PIN]), None, None);

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            0 => &nrf52840::gpio::PORT[GPIO_D2],
            1 => &nrf52840::gpio::PORT[GPIO_D3],
            2 => &nrf52840::gpio::PORT[GPIO_D4],
            3 => &nrf52840::gpio::PORT[GPIO_D5],
            4 => &nrf52840::gpio::PORT[GPIO_D6],
            5 => &nrf52840::gpio::PORT[GPIO_D7],
            6 => &nrf52840::gpio::PORT[GPIO_D8],
            7 => &nrf52840::gpio::PORT[GPIO_D9],
            8 => &nrf52840::gpio::PORT[GPIO_D10]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        nrf52840::gpio::GPIOPin,
        (&nrf52840::gpio::PORT[LED_RED_PIN], ActiveLow),
        (&nrf52840::gpio::PORT[LED_GREEN_PIN], ActiveLow),
        (&nrf52840::gpio::PORT[LED_BLUE_PIN], ActiveLow)
    ))
    .finalize(components::led_component_buf!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Deferred Call (Dynamic) Setup
    //--------------------------------------------------------------------------

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &nrf52::rtc::RTC;
    rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52::rtc::Rtc));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    // Create a shared UART channel for the console and for kernel debug.
    // `nrf52::uart::UARTE0` uses the UART pinned out on the castellated header,
    // _not_ the micro USB.
    let uart_mux = components::console::UartMuxComponent::new(
        &nrf52::uart::UARTE0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Configure the UART pins on this specific board.
    nrf52::uart::UARTE0.initialize(
        nrf52::pinmux::Pinmux::new(UART_TX_PIN as u32),
        nrf52::pinmux::Pinmux::new(UART_RX_PIN as u32),
        None,
        None,
    );

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    //--------------------------------------------------------------------------
    // RANDOM NUMBERS
    //--------------------------------------------------------------------------

    let rng = components::rng::RngComponent::new(board_kernel, &nrf52::trng::TRNG).finalize(());

    //--------------------------------------------------------------------------
    // WIRELESS
    //--------------------------------------------------------------------------

    // let ble_radio =
    //     BLEComponent::new(board_kernel, &nrf52::ble_radio::RADIO, mux_alarm).finalize(());

    // let (ieee802154_radio, _) = Ieee802154Component::new(
    //     board_kernel,
    //     &nrf52::ieee802154_radio::RADIO,
    //     PAN_ID,
    //     SRC_MAC,
    // )
    // .finalize(());

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52dk_base::nrf52_components::NrfClockComponent::new().finalize(());

    let platform = Platform {
        // ble_radio: ble_radio,
        // ieee802154_radio: ieee802154_radio,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(nrf52840::chip::Chip, nrf52840::chip::new());
    CHIP = Some(chip);

    debug!("Initialization complete. Entering main loop.");

    //--------------------------------------------------------------------------
    // PROCESSES AND MAIN LOOP
    //--------------------------------------------------------------------------

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
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
