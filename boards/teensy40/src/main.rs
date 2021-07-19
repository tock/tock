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
use kernel::component::Component;
use kernel::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::hil::{gpio::Configure, led::LedHigh};
use kernel::platform::chip::ClockInterface;
use kernel::platform::{KernelResources, SyscallDispatch};
use kernel::scheduler::round_robin::RoundRobinSched;
use kernel::{create_capability, static_init};

/// Number of concurrent processes this platform supports
const NUM_PROCS: usize = 4;
const NUM_UPCALLS_IPC: usize = NUM_PROCS + 1;

/// Actual process memory
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

/// What should we do if a process faults?
const FAULT_RESPONSE: kernel::process::PanicFaultPolicy = kernel::process::PanicFaultPolicy {};

/// Teensy 4 platform
struct Teensy40 {
    led:
        &'static capsules::led::LedDriver<'static, LedHigh<'static, imxrt1060::gpio::Pin<'static>>>,
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, imxrt1060::gpt::Gpt1<'static>>,
    >,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm7::systick::SysTick,
}

impl SyscallDispatch for Teensy40 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
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

impl KernelResources<imxrt1060::chip::Imxrt10xx<imxrt1060::chip::Imxrt10xxDefaultPeripherals>>
    for Teensy40
{
    type SyscallDispatch = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
    type SchedulerTimer = cortexm7::systick::SysTick;
    type WatchDog = ();

    fn syscall_dispatch(&self) -> &Self::SyscallDispatch {
        &self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
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
}

/// Static configurations for DMA channels.
///
/// All DMA channels must be unique.
mod dma_config {
    use super::imxrt1060::nvic;

    /// DMA channel for LPUART2_RX (arbitrary).
    pub const LPUART2_RX: usize = 7;
    /// DMA channel for LPUART2_TX (arbitrary).
    pub const LPUART2_TX: usize = 8;

    /// Add your DMA interrupt vector numbers here.
    const DMA_INTERRUPTS: &[u32] = &[nvic::DMA7_23, nvic::DMA8_24];

    /// Enable DMA interrupts for the selected channels.
    #[inline(always)]
    pub fn enable_interrupts() {
        DMA_INTERRUPTS
            .iter()
            .copied()
            // Safety: creating NVIC vector in platform code. Vector is valid.
            .map(|vector| unsafe { cortexm7::nvic::Nvic::new(vector) })
            .for_each(|intr| intr.enable());
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn get_peripherals() -> &'static mut imxrt1060::chip::Imxrt10xxDefaultPeripherals {
    let ccm = static_init!(imxrt1060::ccm::Ccm, imxrt1060::ccm::Ccm::new());
    let peripherals = static_init!(
        imxrt1060::chip::Imxrt10xxDefaultPeripherals,
        imxrt1060::chip::Imxrt10xxDefaultPeripherals::new(ccm)
    );

    peripherals
}

type Chip = imxrt1060::chip::Imxrt10xx<imxrt1060::chip::Imxrt10xxDefaultPeripherals>;
static mut CHIP: Option<&'static Chip> = None;

/// Set the ARM clock frequency to 600MHz
///
/// You should use this early in program initialization, before there's a chance
/// for preemption.
fn set_arm_clock(ccm: &imxrt1060::ccm::Ccm, ccm_analog: &imxrt1060::ccm_analog::CcmAnalog) {
    use imxrt1060::ccm::{
        PeripheralClock2Selection, PeripheralClockSelection, PrePeripheralClockSelection,
    };

    // Switch AHB clock root to 24MHz oscillator
    ccm.set_peripheral_clock2_divider(1);
    ccm.set_peripheral_clock2_selection(PeripheralClock2Selection::Oscillator);
    ccm.set_peripheral_clock_selection(PeripheralClockSelection::PeripheralClock2Divided);

    // Set PLL1 output frequency, which is
    //
    //      24MHz * DIV_SEL / 2
    //
    // 24MHz is from crystal oscillator.
    // PLL1 output == 120MHz
    ccm_analog.restart_pll1(100);

    // ARM divider is right after the PLL1 output,
    // bringing down the clock to 600MHz
    ccm.set_arm_divider(2);

    // Divider just before the AHB clock root
    ccm.set_ahb_divider(1);

    // Switch AHB clock (back) to PLL1
    ccm.set_pre_peripheral_clock_selection(PrePeripheralClockSelection::Pll1);
    ccm.set_peripheral_clock_selection(PeripheralClockSelection::PrePeripheralClock);
}

#[no_mangle]
pub unsafe fn main() {
    imxrt1060::init();

    let peripherals = get_peripherals();
    peripherals.ccm.set_low_power_mode();

    peripherals.dcdc.clock().enable();
    peripherals.dcdc.set_target_vdd_soc(1250);
    set_arm_clock(&peripherals.ccm, &peripherals.ccm_analog);
    // IPG clock is 600MHz / 4 == 150MHz
    peripherals.ccm.set_ipg_divider(4);

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

    peripherals.dma.clock().enable();
    peripherals.dma.reset_tcds();
    peripherals
        .lpuart2
        .set_rx_dma_channel(&peripherals.dma.channels[dma_config::LPUART2_RX]);
    peripherals
        .lpuart2
        .set_tx_dma_channel(&peripherals.dma.channels[dma_config::LPUART2_TX]);

    cortexm7::nvic::Nvic::new(imxrt1060::nvic::GPT1).enable();
    dma_config::enable_interrupts();

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
        capsules::console::DRIVER_NUM,
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
        capsules::alarm::DRIVER_NUM,
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
        kernel::ipc::DRIVER_NUM,
        &memory_allocation_capability,
    );

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    //
    // Platform
    //
    let teensy40 = Teensy40 {
        led,
        console,
        ipc,
        alarm,

        scheduler,
        systick: cortexm7::systick::SysTick::new_with_calibration(792_000_000),
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
    .unwrap();

    board_kernel.kernel_loop(&teensy40, chip, Some(&teensy40.ipc), &main_loop_capability);
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
