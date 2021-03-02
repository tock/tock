//! System configuration
//!
//! - LED on pin 13
//! - UART2 allocated for a debug console on pins 14 and 15
//! - GPT1 is the alarm source

#![no_std]
#![no_main]

mod fcb;
mod io;

use imxrt1060::gpio::PinId;
use imxrt1060::iomuxc::{MuxMode, PadId, Sion};
use imxrt10xx as imxrt1060;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::{gpio::Configure, led::LedHigh};
use kernel::{create_capability, static_init};

/// Number of concurrent processes this platform supports
const NUM_PROCS: usize = 4;

/// Actual process memory
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

/// What should we do if a process faults?
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Teensy 4 platform
struct Teensy40 {
    led:
        &'static capsules::led::LedDriver<'static, LedHigh<'static, imxrt1060::gpio::Pin<'static>>>,
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC<NUM_PROCS>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, imxrt1060::gpt::Gpt1<'static>>,
    >,
}

impl kernel::Platform for Teensy40 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

type Chip = imxrt1060::chip::Imxrt10xx<imxrt1060::chip::Imxrt10xxDefaultPeripherals>;
static mut CHIP: Option<&'static Chip> = None;

#[no_mangle]
pub unsafe fn reset_handler() {
    imxrt1060::init();
    let ccm = static_init!(imxrt1060::ccm::Ccm, imxrt1060::ccm::Ccm::new());
    let peripherals = static_init!(
        imxrt1060::chip::Imxrt10xxDefaultPeripherals,
        imxrt1060::chip::Imxrt10xxDefaultPeripherals::new(ccm)
    );
    peripherals.ccm.set_low_power_mode();
    peripherals.lpuart1.disable_clock();
    peripherals.lpuart2.disable_clock();
    peripherals
        .ccm
        .set_uart_clock_sel(imxrt1060::ccm::UartClockSelection::PLL3);
    peripherals.ccm.set_uart_clock_podf(1);

    peripherals.ccm.enable_iomuxc_clock();
    peripherals.ccm.enable_iomuxc_snvs_clock();

    peripherals
        .ccm
        .set_perclk_sel(imxrt1060::ccm::PerclkClockSel::Oscillator);
    peripherals.ccm.set_perclk_divider(8);

    peripherals.ports.pin(PinId::B0_03).make_output();

    // Pin 13 is an LED
    peripherals
        .iomuxc
        .enable_sw_mux_ctl_pad_gpio(PadId::B0, MuxMode::ALT5, Sion::Disabled, 3);

    // Pins 14 and 15 are UART TX and RX
    peripherals
        .iomuxc
        .enable_sw_mux_ctl_pad_gpio(PadId::AdB1, MuxMode::ALT2, Sion::Disabled, 2);
    peripherals
        .iomuxc
        .enable_sw_mux_ctl_pad_gpio(PadId::AdB1, MuxMode::ALT2, Sion::Disabled, 3);

    peripherals.iomuxc.enable_lpuart2_tx_select_input();
    peripherals.iomuxc.enable_lpuart2_rx_select_input();

    peripherals.lpuart2.enable_clock();
    peripherals.lpuart2.set_baud();

    peripherals.gpt1.enable_clock();
    peripherals.gpt1.start(
        peripherals.ccm.perclk_sel(),
        peripherals.ccm.perclk_divider(),
    );

    cortexm7::nvic::Nvic::new(imxrt1060::nvic::GPT1).enable();
    cortexm7::nvic::Nvic::new(imxrt1060::nvic::LPUART2).enable();

    let chip = static_init!(Chip, Chip::new(peripherals));
    CHIP = Some(chip);

    // Start loading the kernel
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    // TODO how many of these should there be...?
    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let uart_mux = components::console::UartMuxComponent::new(
        &peripherals.lpuart2,
        115_200,
        dynamic_deferred_caller,
    )
    .finalize(());
    // Create the debugger object that handles calls to `debug!()`
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // Setup the console
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM as u32,
        uart_mux,
    )
    .finalize(());

    // LED
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        LedHigh<imxrt1060::gpio::Pin>,
        LedHigh::new(peripherals.ports.pin(PinId::B0_03))
    ))
    .finalize(components::led_component_buf!(
        LedHigh<'static, imxrt1060::gpio::Pin>
    ));

    // Alarm
    let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.gpt1).finalize(
        components::alarm_mux_component_helper!(imxrt1060::gpt::Gpt1),
    );
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules::alarm::DRIVER_NUM as u32,
        mux_alarm,
    )
    .finalize(components::alarm_component_helper!(imxrt1060::gpt::Gpt1));

    //
    // Capabilities
    //
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let ipc = kernel::ipc::IPC::new(
        board_kernel,
        kernel::ipc::DRIVER_NUM as u32,
        &memory_allocation_capability,
    );

    //
    // Platform
    //
    let teensy40 = Teensy40 {
        led,
        console,
        ipc,
        alarm,
    };

    //
    // Kernel startup
    //
    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
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
    .unwrap();

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(
        &teensy40,
        chip,
        Some(&teensy40.ipc),
        scheduler,
        &main_loop_capability,
    );
}

/// Space for the stack buffer
///
/// Justified in tock's `kernel_layout.ld`.
#[no_mangle]
#[link_section = ".stack_buffer"]
#[used]
static mut STACK_BUFFER: [u8; 0x2000] = [0; 0x2000];

const FCB_SIZE: usize = core::mem::size_of::<fcb::FCB>();

/// Buffer between FCB and IVT
///
/// The FCB is put at the start of flash. We then need to add a 4K buffer in between
/// the start of flash to the IVT. This buffer provides that padding.
///
/// See justification for the `".stack_buffer"` section to understand why we need
/// explicit padding for the FCB.
#[no_mangle]
#[link_section = ".fcb_buffer"]
#[used]
static mut FCB_BUFFER: [u8; 0x1000 - FCB_SIZE] = [0xFF; 0x1000 - FCB_SIZE];
