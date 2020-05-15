#![no_std]
#![no_main]
#![feature(asm, core_intrinsics)]
// #![deny(missing_docs)]

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::watchdog::*;
use kernel::Platform;
use kernel::{create_capability, static_init};

pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static msp432::chip::Msp432> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

// Force the emission of the `.apps` segment in the kernel elf image
// NOTE: This will cause the kernel to overwrite any existing apps when flashed!
// #[used]
// #[link_section = ".app.hack"]
// static APP_HACK: u8 = 0;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct MspExp432P401R {
    led: &'static capsules::led::LED<'static>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for MspExp432P401R {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    msp432::init();
    msp432::wdt::WATCHDOG.stop();
    msp432::sysctl::SYSCTL.enable_all_sram_banks();
    msp432::pcm::PCM.set_high_power();
    msp432::flctl::FLCTL.set_waitstates(msp432::flctl::WaitStates::_1);
    msp432::cs::CS.set_clk_48mhz();
    msp432::flctl::FLCTL.set_buffering(true);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let chip = static_init!(msp432::chip::Msp432, msp432::chip::Msp432::new());
    CHIP = Some(chip);

    let leds = components::led::LedsComponent::new().finalize(components::led_component_helper!(
        (
            &msp432::gpio::PINS[msp432::gpio::PinNr::P02_0 as usize],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &msp432::gpio::PINS[msp432::gpio::PinNr::P02_1 as usize],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            &msp432::gpio::PINS[msp432::gpio::PinNr::P02_2 as usize],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ));

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let msp_exp432p4014 = MspExp432P401R { led: leds };

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
    .unwrap();

    board_kernel.kernel_loop(&msp_exp432p4014, chip, None, &main_loop_capability);
    panic!();
}
