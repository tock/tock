//! Board file for Imxrt-1052 development board
//!
//! - <https://www.nxp.com/webapp/Download?colCode=IMXRT1050RM>

#![no_std]
#![no_main]
#![feature(asm)]
#![deny(missing_docs)]

mod imxrt1050_components;

use kernel::hil::time::Alarm;
// use kernel::hil::gpio::Output;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use imxrt1050_components::fxos8700::NineDofComponent;
use kernel::debug;
// use capsules::fxos8700cq;
// use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
// use kernel::hil::time::Alarm;
use kernel::hil::gpio::Configure;
use kernel::Platform;
use kernel::{create_capability, static_init};

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

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 1;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] = [None];

static mut CHIP: Option<&'static imxrt1050::chip::Imxrt1050> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

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
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static, imxrt1050::gpio::Pin<'static>>,
    // button: &'static capsules::button::Button<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, imxrt1050::gpt1::Gpt1<'static>>,
    >,
    ninedof: &'static capsules::ninedof::NineDof<'static>, // accel: &'static capsules::fxos8700cq::Fxos8700cq<'static>,
                                                           // gpio: &'static capsules::gpio::GPIO<'static>
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for Imxrt1050EVKB {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            // capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            // capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
// unsafe fn setup_dma() {
//     use imxr::dma1::{Dma1Peripheral, DMA1};
//     use stm32f4xx::usart;
//     use stm32f4xx::usart::USART3;

//     DMA1.enable_clock();

//     let usart3_tx_stream = Dma1Peripheral::USART3_TX.get_stream();
//     let usart3_rx_stream = Dma1Peripheral::USART3_RX.get_stream();

//     USART3.set_dma(
//         usart::TxDMA(usart3_tx_stream),
//         usart::RxDMA(usart3_rx_stream),
//     );

//     usart3_tx_stream.set_client(&USART3);
//     usart3_rx_stream.set_client(&USART3);

//     usart3_tx_stream.setup(Dma1Peripheral::USART3_TX);
//     usart3_rx_stream.setup(Dma1Peripheral::USART3_RX);

//     cortexm4::nvic::Nvic::new(Dma1Peripheral::USART3_TX.get_stream_irqn()).enable();
//     cortexm4::nvic::Nvic::new(Dma1Peripheral::USART3_RX.get_stream_irqn()).enable();
// }

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    // use kernel::hil::gpio::Configure;
    // use stm32f4xx::exti::{LineId, EXTI};
    use imxrt1050::gpio::{PinId, PORT};
    // use imxrt1050::iomuxc::PortId;
    // use stm32f4xx::syscfg::SYSCFG;
    use imxrt1050::ccm::CCM;

    CCM.enable_iomuxc_clock();
    CCM.enable_gpio1_clock();
    // SYSCFG.enable_clock();

    PORT[0].enable_clock();

    // User_LED is connected to GPIO_AD_B0_09. Configure P1_09 as `debug_gpio!(0, ...)`
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT5,
        Sion::Disabled,
        9,
    );
    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        9,
        PullUpDown::Pus0_100kOhmPullDown,
        PullKeepEn::Pke1PullKeeperEnabled,
        OpenDrainEn::Ode0OpenDrainDisabled,
        Speed::Medium2,
        DriveStrength::DSE6,
    );

    PinId::AdB0_09.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    // PORT[PortId::D as usize].enable_clock();

    // // pd8 and pd9 (USART3) is connected to ST-LINK virtual COM port
    // PinId::PD08.get_pin().as_ref().map(|pin| {
    //     pin.set_mode(Mode::AlternateFunctionMode);
    //     // AF7 is USART2_TX
    //     pin.set_alternate_function(AlternateFunction::AF7);
    // });
    // PinId::PD09.get_pin().as_ref().map(|pin| {
    //     pin.set_mode(Mode::AlternateFunctionMode);
    //     // AF7 is USART2_RX
    //     pin.set_alternate_function(AlternateFunction::AF7);
    // });

    // PORT[PortId::C as usize].enable_clock();

    // // button is connected on pc13
    // PinId::PC13.get_pin().as_ref().map(|pin| {
    //     // By default, upon reset, the pin is in input mode, with no internal
    //     // pull-up, no internal pull-down (i.e., floating).
    //     //
    //     // Only set the mapping between EXTI line and the Pin and let capsule do
    //     // the rest.
    //     EXTI.associate_line_gpiopin(LineId::Exti13, pin);
    // });
    // // EXTI13 interrupts is delivered at IRQn 40 (EXTI15_10)
    // cortexm4::nvic::Nvic::new(stm32f4xx::nvic::EXTI15_10).enable();

    // // Enable clocks for GPIO Ports
    // // Disable some of them if you don't need some of the GPIOs
    // PORT[PortId::P1 as usize].enable_clock();
    // // Ports B, C and D are already enabled
    // PORT[PortId::E as usize].enable_clock();
    // PORT[PortId::F as usize].enable_clock();
    // PORT[PortId::G as usize].enable_clock();
    // PORT[PortId::H as usize].enable_clock();
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
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    imxrt1050::init();
    imxrt1050::lpuart::LPUART1.set_baud();

    // We use the default HSI 16Mhz clock

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
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT2,
        Sion::Disabled,
        13,
    );
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB0,
        MuxMode::ALT2,
        Sion::Disabled,
        14,
    );

    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        13,
        PullUpDown::Pus0_100kOhmPullDown,
        PullKeepEn::Pke1PullKeeperEnabled,
        OpenDrainEn::Ode0OpenDrainDisabled,
        Speed::Medium2,
        DriveStrength::DSE6,
    );
    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB0,
        14,
        PullUpDown::Pus0_100kOhmPullDown,
        PullKeepEn::Pke1PullKeeperEnabled,
        OpenDrainEn::Ode0OpenDrainDisabled,
        Speed::Medium2,
        DriveStrength::DSE6,
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

    // Setup the process inspection console
    // let process_console_uart = static_init!(UartDevice, UartDevice::new(mux_uart, true));
    // process_console_uart.setup();
    // pub struct ProcessConsoleCapability;
    // unsafe impl capabilities::ProcessManagementCapability for ProcessConsoleCapability {}
    // let process_console = static_init!(
    //     capsules::process_console::ProcessConsole<'static, ProcessConsoleCapability>,
    //     capsules::process_console::ProcessConsole::new(
    //         process_console_uart,
    //         &mut capsules::process_console::WRITE_BUF,
    //         &mut capsules::process_console::READ_BUF,
    //         &mut capsules::process_console::COMMAND_BUF,
    //         board_kernel,
    //         ProcessConsoleCapability,
    //     )
    // );
    // hil::uart::Transmit::set_transmit_client(process_console_uart, process_console);
    // hil::uart::Receive::set_receive_client(process_console_uart, process_console);
    // process_console.start();

    // LEDs

    // Clock to Port A is enabled in `set_pin_primary_functions()
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        imxrt1050::gpio::Pin<'static>,
        (
            imxrt1050::gpio::PinId::AdB0_09.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    )).finalize(components::led_component_buf!(
        imxrt1050::gpio::Pin<'static>
    ));

    // BUTTONs
    // let button = components::button::ButtonComponent::new(board_kernel).finalize(
    //     components::button_component_helper!((
    //         stm32f4xx::gpio::PinId::PC13.get_pin().as_ref().unwrap(),
    //         capsules::button::GpioMode::LowWhenPressed,
    //         kernel::hil::gpio::FloatingState::PullNone
    //     )),
    // );

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
    // let gpio = GpioComponent::new(board_kernel).finalize(components::gpio_component_helper!(
    //     // Arduino like RX/TX
    // ));

    // stm32f3xx::i2c::I2C1.enable_clock();
    // stm32f3xx::i2c::I2C1.set_speed(stm32f3xx::i2c::I2CSpeed::Speed400k, 8);

    // let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f3xx::i2c::I2C1)
    //     .finalize(components::i2c_mux_component_helper!());
    // let sensor_accelerometer_i2c = components::i2c::I2CComponent::new(mux_i2c, 0x19)
    //     .finalize(components::i2c_component_helper!());
    // let sensor_magnetometer_i2c = components::i2c::I2CComponent::new(mux_i2c, 0x1e)
    //     .finalize(components::i2c_component_helper!());

    // PinId::PB06.get_pin().as_ref().map(|pin| {
    //     pin.set_mode(Mode::AlternateFunctionMode);
    //     pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
    //     // AF4 is I2C
    //     pin.set_alternate_function(AlternateFunction::AF4);
    // });
    // PinId::PB07.get_pin().as_ref().map(|pin| {
    //     pin.make_output();
    //     pin.set_floating_state(kernel::hil::gpio::FloatingState::PullNone);
    //     pin.set_mode(Mode::AlternateFunctionMode);
    //     // AF4 is I2C
    //     pin.set_alternate_function(AlternateFunction::AF4);
    // });

    // static mut BUFFER: [u8; 120] = [0; 120];

    // LPI2C
    // AD_B1_00 is LPI2C1_SCL
    // AD_B1_01 is LPI2C1_SDA
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB1,
        MuxMode::ALT3,
        Sion::Enabled,
        0,
    );
    imxrt1050::iomuxc::IOMUXC.enable_lpi2c_scl_select_input();
    imxrt1050::iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio(
        PadId::AdB1,
        MuxMode::ALT3,
        Sion::Enabled,
        1,
    );
    imxrt1050::iomuxc::IOMUXC.enable_lpi2c_sda_select_input();

    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB1,
        0,
        PullUpDown::Pus3_22kOhmPullUp,
        PullKeepEn::Pke1PullKeeperEnabled,
        OpenDrainEn::Ode1OpenDrainEnabled,
        Speed::Medium2,
        DriveStrength::DSE6,
    );

    imxrt1050::iomuxc::IOMUXC.configure_sw_pad_ctl_pad_gpio(
        PadId::AdB1,
        1,
        PullUpDown::Pus3_22kOhmPullUp,
        PullKeepEn::Pke1PullKeeperEnabled,
        OpenDrainEn::Ode1OpenDrainEnabled,
        Speed::Medium2,
        DriveStrength::DSE6,
    );

    imxrt1050::lpi2c::LPI2C1.enable_clock();
    imxrt1050::lpi2c::LPI2C1.set_speed(imxrt1050::lpi2c::Lpi2cSpeed::Speed100k, 8);

    // let lsm303dlhc = static_init!(
    //     capsules::lsm303dlhc::Lsm303dlhc,
    //     capsules::lsm303dlhc::Lsm303dlhc::new(
    //         sensor_accelerometer_i2c,
    //         sensor_magnetometer_i2c,
    //         &mut BUFFER
    //     )
    // );
    // sensor_accelerometer_i2c.set_client(lsm303dlhc);
    // sensor_magnetometer_i2c.set_client(lsm303dlhc);
    let mux_i2c = components::i2c::I2CMuxComponent::new(&imxrt1050::lpi2c::LPI2C1)
        .finalize(components::i2c_mux_component_helper!());

    // let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new()
    //     .finalize(components::lsm303dlhc_i2c_component_helper!(mux_i2c));

    // lsm303dlhc.configure(
    //     lsm303dlhc::Lsm303dlhcAccelDataRate::DataRate25Hz,
    //     false,
    //     lsm303dlhc::Lsm303dlhcScale::Scale2G,
    //     false,
    //     true,
    //     lsm303dlhc::Lsm303dlhcMagnetoDataRate::DataRate3_0Hz,
    //     lsm303dlhc::Lsm303dlhcRange::Range4_7G,
    // );
    use imxrt1050::gpio::PinId;
    let ninedof = NineDofComponent::new(
        board_kernel,
        mux_i2c,
        PinId::AdB1_00.get_pin().as_ref().unwrap(),
    )
    .finalize(());

    let imxrt1050 = Imxrt1050EVKB {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        led: led,
        ninedof: ninedof,
        // button: button,
        alarm: alarm,
        // gpio: gpio,
    };

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

    // let pin = imxrt1050::gpio::PinId::AdB0_09.get_pin().as_ref().unwrap();
    // pin.make_output();
    // pin.clear();

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
