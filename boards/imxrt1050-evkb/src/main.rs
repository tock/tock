//! Reference Manual for the Imxrt-1052 development board
//!
//! - <https://www.nxp.com/webapp/Download?colCode=IMXRT1050RM>

#![no_std]
#![no_main]
#![feature(asm)]
#![deny(missing_docs)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::debug;
use kernel::hil::gpio::Configure;
use kernel::hil::time::Alarm;
use kernel::Platform;
use kernel::{create_capability, static_init};

// use components::fxos8700::Fxos8700Component;
// use components::ninedof::NineDofComponent;

use imxrt1050::iomuxc::DriveStrength;
use imxrt1050::iomuxc::MuxMode;
use imxrt1050::iomuxc::OpenDrainEn;
use imxrt1050::iomuxc::PadId;
use imxrt1050::iomuxc::PullKeepEn;
use imxrt1050::iomuxc::PullUpDown;
use imxrt1050::iomuxc::Sion;
use imxrt1050::iomuxc::Speed;

// Unit Tests for drivers.
// #[allow(dead_code)]
// mod virtual_uart_rx_test;

/// Support routines for debugging I/O.
pub mod io;

/// Defines a vector which contains the boot section
pub mod boot_header;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 1;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] = [None];

static mut CHIP: Option<&'static imxrt1050::chip::Imxrt1050> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Manually setting the boot header section that contains the FCB header
#[used]
#[link_section = ".boot_hdr"]
static BOOT_HDR: [u8; 8192] = boot_header::BOOT_HDR;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 65536] = [0; 65536];

// Force the emission of the `.apps` segment in the kernel elf image
// NOTE: This will cause the kernel to overwrite any existing apps when flashed!
#[used]
#[link_section = ".app.hack"]
static APP_HACK: u8 = 0;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

// const NUM_LEDS: usize = 1;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct Imxrt1050EVKB {
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, imxrt1050::gpt1::Gpt1<'static>>,
    >,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, imxrt1050::gpio::Pin<'static>>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static, imxrt1050::gpio::Pin<'static>>,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for Imxrt1050EVKB {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
/// DMA for imxrt1050-evkb is not implemented yet.
// unsafe fn setup_dma() {
// }

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use imxrt1050::ccm::CCM;
    use imxrt1050::gpio::{PinId, PORT};

    CCM.enable_iomuxc_clock();

    PORT[0].enable_clock();

    // User_LED is connected to GPIO_AD_B0_09.
    // Values set accordingly to the evkbimxrt1050_iled_blinky SDK example

    // First we configure the pin in GPIO mode and disable the Software Input
    // on Field, so that the Input Path is determined by functionality.
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT5, // ALT5 for AdB0_09: GPIO1_IO09 of instance: gpio1
        Sion::Disabled,
        9,
    );

    // Configure the pin resistance value, pull up or pull down and other
    // physical aspects.
    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        9,
        PullUpDown::Pus0_100kOhmPullDown,   // 100K Ohm Pull Down
        PullKeepEn::Pke1PullKeeperEnabled,  // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode0OpenDrainDisabled, // Output is CMOS, either 0 logic or 1 logic
        Speed::Medium2,                     // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    // Configuring the GPIO_AD_B0_09 as output
    PinId::AdB0_09.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use imxrt1050::gpt1::GPT1;

    // LPUART1 IRQn is 20
    cortexm7::nvic::Nvic::new(imxrt1050::nvic::LPUART1).enable();

    // TIM2 IRQn is 28
    GPT1.enable_clock();
    GPT1.start();
    cortexm7::nvic::Nvic::new(imxrt1050::nvic::GPT1).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the IMXRT1050 chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    imxrt1050::init();
    imxrt1050::lpuart::LPUART1.set_baud();

    set_pin_primary_functions();
    // setup_dma();

    setup_peripherals();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let chip = static_init!(
        imxrt1050::chip::Imxrt1050,
        imxrt1050::chip::Imxrt1050::new()
    );
    CHIP = Some(chip);

    // LPUART1

    // Enable tx and rx from iomuxc
    // TX is on pad GPIO_AD_B0_12
    // RX is on pad GPIO_AD_B0_13
    // Values set accordingly to the evkbimxrt1050_hello_world SDK example

    // First we configure the pin in LPUART mode and disable the Software Input
    // on Field, so that the Input Path is determined by functionality.
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT2, // ALT2: LPUART1_TXD of instance: lpuart1
        Sion::Disabled,
        13,
    );
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT2, // ALT2: LPUART1_RXD of instance: lpuart1
        Sion::Disabled,
        14,
    );

    // Configure the pin resistance value, pull up or pull down and other
    // physical aspects.
    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        13,
        PullUpDown::Pus0_100kOhmPullDown,   // 100K Ohm Pull Down
        PullKeepEn::Pke1PullKeeperEnabled,  // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode0OpenDrainDisabled, // Output is CMOS, either 0 logic or 1 logic
        Speed::Medium2,                     // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );
    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        14,
        PullUpDown::Pus0_100kOhmPullDown,   // 100K Ohm Pull Down
        PullKeepEn::Pke1PullKeeperEnabled,  // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode0OpenDrainDisabled, // Output is CMOS, either 0 logic or 1 logic
        Speed::Medium2,                     // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    // Enable clock
    imxrt1050::lpuart::LPUART1.enable_clock();

    let lpuart_mux = components::console::UartMuxComponent::new(
        &imxrt1050::lpuart::LPUART1,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());
    io::WRITER.set_initialized();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, lpuart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(lpuart_mux).finalize(());

    // LEDs

    // Clock to Port A is enabled in `set_pin_primary_functions()
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        imxrt1050::gpio::Pin<'static>,
        (
            imxrt1050::gpio::PinId::AdB0_09.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    ))
    .finalize(components::led_component_buf!(
        imxrt1050::gpio::Pin<'static>
    ));

    // ALARM
    let mux_alarm = static_init!(
        MuxAlarm<'static, imxrt1050::gpt1::Gpt1>,
        MuxAlarm::new(&imxrt1050::gpt1::GPT1)
    );
    imxrt1050::gpt1::GPT1.set_client(mux_alarm);

    let virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, imxrt1050::gpt1::Gpt1>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, imxrt1050::gpt1::Gpt1>>,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    virtual_alarm.set_client(alarm);

    // GPIO
    // For now we expose only one pin
    let gpio = GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            imxrt1050::gpio::Pin<'static>,
            // The User Led
            0 => imxrt1050::gpio::PinId::AdB0_09.get_pin().as_ref().unwrap()
        ),
    )
    .finalize(components::gpio_component_buf!(
        imxrt1050::gpio::Pin<'static>
    ));

    // LPI2C
    // AD_B1_00 is LPI2C1_SCL
    // AD_B1_01 is LPI2C1_SDA
    // Values set accordingly to the evkbimxrt1050_bubble_peripheral SDK example

    // First we configure the pin in LPUART mode and enable the Software Input
    // on Field, so that we force input path of the pad.
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB1,
        MuxMode::ALT3, // ALT3:  LPI2C1_SCL of instance: lpi2c1
        Sion::Enabled,
        0,
    );
    // Selecting AD_B1_00 for LPI2C1_SCL in the Daisy Chain.
    imxrt1050::iomuxc::IOMUXC.enable_lpi2c_scl_select_input();

    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB1,
        MuxMode::ALT3, // ALT3:  LPI2C1_SDA of instance: lpi2c1
        Sion::Enabled,
        1,
    );
    // Selecting AD_B1_01 for LPI2C1_SDA in the Daisy Chain.
    imxrt1050::iomuxc::IOMUXC.enable_lpi2c_sda_select_input();

    // Configure the pin resistance value, pull up or pull down and other
    // physical aspects.
    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB1,
        0,
        PullUpDown::Pus3_22kOhmPullUp,     // 22K Ohm Pull Up
        PullKeepEn::Pke1PullKeeperEnabled, // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode1OpenDrainEnabled, // Open Drain Enabled (Output is Open Drain)
        Speed::Medium2,                    // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB1,
        1,
        PullUpDown::Pus3_22kOhmPullUp,     // 22K Ohm Pull Up
        PullKeepEn::Pke1PullKeeperEnabled, // Pull-down resistor or keep the previous value
        OpenDrainEn::Ode1OpenDrainEnabled, // Open Drain Enabled (Output is Open Drain)
        Speed::Medium2,                    // Operating frequency: 100MHz - 150MHz
        DriveStrength::DSE6, // Dual/Single voltage: 43/43 Ohm @ 1.8V, 40/26 Ohm @ 3.3V
    );

    // Enabling the lpi2c1 clock and setting the speed.
    imxrt1050::lpi2c::LPI2C1.enable_clock();
    imxrt1050::lpi2c::LPI2C1.set_speed(imxrt1050::lpi2c::Lpi2cSpeed::Speed100k, 8);

    use imxrt1050::gpio::PinId;
    let mux_i2c = components::i2c::I2CMuxComponent::new(
        &imxrt1050::lpi2c::LPI2C1,
        None,
        dynamic_deferred_caller,
    )
    .finalize(components::i2c_mux_component_helper!());

    // Fxos8700 sensor
    let fxos8700 = components::fxos8700::Fxos8700Component::new(
        mux_i2c,
        PinId::AdB1_00.get_pin().as_ref().unwrap(),
    )
    .finalize(());

    // Ninedof
    let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
        .finalize(components::ninedof_component_helper!(fxos8700));

    let imxrt1050 = Imxrt1050EVKB {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        led: led,
        ninedof: ninedof,
        alarm: alarm,
        gpio: gpio,
    };

    // Optional kernel tests
    //
    // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

    debug!("Tock OS initialization complete. Entering main loop");
    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
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

    board_kernel.kernel_loop(
        &imxrt1050,
        chip,
        Some(&imxrt1050.ipc),
        &main_loop_capability,
    );
}
