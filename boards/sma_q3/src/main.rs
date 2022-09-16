//! Tock kernel for the SMA Q3 smartwatch.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! SWD as I/O and many peripherals.
//!
//! Reverse-engineered documentation available at:
//! <https://hackaday.io/project/175577-hackable-nrf52840-smart-watch>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use capsules::virtual_aes_ccm::MuxAES128CCM;
use capsules::virtual_alarm::VirtualMuxAlarm;
use components::bmp280::Bmp280Component;
use components::bmp280_component_helper;
use kernel::component::Component;
use kernel::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::symmetric_encryption::AES128;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{capabilities, create_capability, debug, debug_gpio, debug_verbose, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52_components;

// The backlight LED
const LED1_PIN: Pin = Pin::P0_08;

// Vibration motor
const VIBRA1_PIN: Pin = Pin::P0_19;

// The side button
const BUTTON_PIN: Pin = Pin::P0_17;

/// I2C pins for the temp/pressure sensor
const I2C_TEMP_SDA_PIN: Pin = Pin::P1_15;
const I2C_TEMP_SCL_PIN: Pin = Pin::P0_02;

// Constants related to the configuration of the 15.4 network stack
const SRC_MAC: u16 = 0xf00f;
const PAN_ID: u16 = 0xABCD;

/// UART Writer
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Static reference to chip for panic dumps
static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;
// Static reference to process printer for panic dumps
static mut PROCESS_PRINTER: Option<&'static kernel::process::ProcessPrinterText> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    temperature: &'static capsules::temperature::TemperatureSensor<'static>,
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52840::ble_radio::Radio<'static>,
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    ieee802154_radio: &'static capsules::ieee802154::RadioDriver<'static>,
    button: &'static capsules::button::Button<'static, nrf52840::gpio::GPIOPin<'static>>,
    pconsole: &'static capsules::process_console::ProcessConsole<
        'static,
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
        components::process_console::Capability,
    >,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52840::gpio::GPIOPin<'static>>,
    led: &'static capsules::led::LedDriver<
        'static,
        LedHigh<'static, nrf52840::gpio::GPIOPin<'static>>,
        2,
    >,
    rng: &'static capsules::rng::RngDriver<'static>,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    analog_comparator: &'static capsules::analog_comparator::AnalogComparator<
        'static,
        nrf52840::acomp::Comparator<'static>,
    >,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
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
            capsules::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules::analog_comparator::DRIVER_NUM => f(Some(self.analog_comparator)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type CredentialsCheckingPolicy = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        &self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn credentials_checking_policy(&self) -> &'static Self::CredentialsCheckingPolicy {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.systick
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn get_peripherals() -> &'static mut Nrf52840DefaultPeripherals<'static> {
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new()
    );

    nrf52840_peripherals
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    nrf52840::init();

    let nrf52840_peripherals = get_peripherals();

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // GPIOs
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            0 => &nrf52840_peripherals.gpio_port[Pin::P0_29],
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840_peripherals.gpio_port[BUTTON_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_buf!(nrf52840::gpio::GPIOPin));

    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
        LedHigh<'static, nrf52840::gpio::GPIOPin>,
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED1_PIN]),
        LedHigh::new(&nrf52840_peripherals.gpio_port[VIBRA1_PIN]),
    ));

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    nrf52_components::startup::NrfStartupComponent::new(
        false,
        // the button pin cannot be used to reset the device,
        // but the API expects some pin,
        // so might as well give a useless one.
        BUTTON_PIN,
        nrf52840::uicr::Regulator0Output::V3_0,
        &base_peripherals.nvmc,
    )
    .finalize(());

    // Create capabilities that the board needs to call certain protected kernel
    // functions.

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let gpio_port = &nrf52840_peripherals.gpio_port;

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(Some(&gpio_port[LED1_PIN]), None, None);

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52840::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_helper!(nrf52840::rtc::Rtc));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 8], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let process_printer =
        components::process_printer::ProcessPrinterTextComponent::new().finalize(());
    PROCESS_PRINTER = Some(process_printer);

    // Initialize early so any panic beyond this point can use the RTT memory object.
    let uart_channel = {
        // RTT communication channel
        let mut rtt_memory = components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());

        // TODO: This is inherently unsafe as it aliases the mutable reference to rtt_memory. This
        // aliases reference is only used inside a panic handler, which should be OK, but maybe we
        // should use a const reference to rtt_memory and leverage interior mutability instead.
        self::io::set_rtt_memory(&mut *rtt_memory.get_rtt_memory_ptr());

        components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
            .finalize(components::segger_rtt_component_helper!(nrf52840::rtc::Rtc))
    };

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux =
        components::console::UartMuxComponent::new(uart_channel, 115200, dynamic_deferred_caller)
            .finalize(());

    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
    )
    .finalize(components::process_console_component_helper!(
        nrf52840::rtc::Rtc<'static>
    ));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_helper!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let ble_radio = nrf52_components::BLEComponent::new(
        board_kernel,
        capsules::ble_advertising_driver::DRIVER_NUM,
        &base_peripherals.ble_radio,
        mux_alarm,
    )
    .finalize(());

    let aes_mux = static_init!(
        MuxAES128CCM<'static, nrf52840::aes::AesECB>,
        MuxAES128CCM::new(&base_peripherals.ecb, dynamic_deferred_caller)
    );
    base_peripherals.ecb.set_client(aes_mux);
    aes_mux.initialize_callback_handle(
        dynamic_deferred_caller.register(aes_mux).unwrap(), // Unwrap fail = no deferred call slot available for ccm mux
    );

    let (ieee802154_radio, _mux_mac) = components::ieee802154::Ieee802154Component::new(
        board_kernel,
        capsules::ieee802154::DRIVER_NUM,
        &base_peripherals.ieee802154_radio,
        aes_mux,
        PAN_ID,
        SRC_MAC,
        dynamic_deferred_caller,
    )
    .finalize(components::ieee802154_component_helper!(
        nrf52840::ieee802154_radio::Radio,
        nrf52840::aes::AesECB<'static>
    ));

    // Not exposed in favor of the BMP280, but present.
    // Possibly needs power management all the same.
    let _temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules::temperature::DRIVER_NUM,
        &base_peripherals.temp,
    )
    .finalize(());

    let sensors_i2c_bus = static_init!(
        capsules::virtual_i2c::MuxI2C<'static>,
        capsules::virtual_i2c::MuxI2C::new(&base_peripherals.twi1, None, dynamic_deferred_caller)
    );

    base_peripherals.twi1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_TEMP_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_TEMP_SDA_PIN as u32),
    );
    base_peripherals.twi1.set_master_client(sensors_i2c_bus);

    let bmp280 = Bmp280Component::new(sensors_i2c_bus, mux_alarm)
        .finalize(bmp280_component_helper!(nrf52840::rtc::Rtc<'static>));

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules::temperature::DRIVER_NUM,
        bmp280,
    )
    .finalize(());

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules::rng::DRIVER_NUM,
        &base_peripherals.trng,
    )
    .finalize(());

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let analog_comparator = components::analog_comparator::AcComponent::new(
        &base_peripherals.acomp,
        components::acomp_component_helper!(
            nrf52840::acomp::Channel,
            &nrf52840::acomp::CHANNEL_AC0
        ),
        board_kernel,
        capsules::analog_comparator::DRIVER_NUM,
    )
    .finalize(components::acomp_component_buf!(
        nrf52840::acomp::Comparator
    ));

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    let periodic_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    periodic_virtual_alarm.setup();

    let platform = Platform {
        temperature,
        button,
        ble_radio,
        ieee802154_radio,
        pconsole,
        console,
        led,
        gpio,
        rng,
        alarm,
        analog_comparator,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    /// I split this out to be able to start applications with a delay
    /// after the board is initialized.
    /// The benefit to debugging is that if I want to print
    /// some debug information while the board initalizes,
    /// it won't be affected by an application that prints so much
    /// that it overflows the output buffer.
    ///
    /// It's also useful for a future "fake off" functionality,
    /// where if a button is pressed, processes are stopped,
    /// but when pressed again, they are loaded anew.
    fn load_processes(
        board_kernel: &'static kernel::Kernel,
        chip: &'static nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>,
    ) {
        let process_management_capability =
            create_capability!(capabilities::ProcessManagementCapability);
        unsafe {
            kernel::process::load_processes(
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
                &FAULT_RESPONSE,
                &process_management_capability,
            )
            .unwrap_or_else(|err| {
                debug!("Error loading processes!");
                debug!("{:?}", err);
            });
        }
    }

    let _ = platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52840::ficr::FICR_INSTANCE);

    load_processes(board_kernel, chip);
    // These symbols are defined in the linker script.
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

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
