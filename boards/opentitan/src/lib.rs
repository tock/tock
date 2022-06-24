//! Board file for LowRISC OpenTitan RISC-V development platform.
//!
//! - <https://opentitan.org/>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(earlgrey_test_runner)]

pub mod io;
pub mod otbn;
pub mod setup;
pub mod usb;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
use kernel::platform::watchdog::WatchDog;
#[cfg(test)]
use kernel::platform::KernelResources;

#[cfg(test)]
pub fn earlgrey_test_runner(tests: &[&dyn Fn()]) {
    unsafe {
        let (board_kernel, earlgrey_nexysvideo, chip, peripherals) = setup::setup();

        setup::BOARD = Some(board_kernel);
        setup::PLATFORM = Some(&earlgrey_nexysvideo);
        setup::PERIPHERALS = Some(peripherals);
        setup::MAIN_CAP = Some(&kernel::create_capability!(
            kernel::capabilities::MainLoopCapability
        ));

        setup::PLATFORM.map(|p| {
            p.watchdog().setup();
        });

        for test in tests {
            test();
        }

        board_kernel.kernel_loop(
            earlgrey_nexysvideo,
            chip,
            None::<&kernel::ipc::IPC<{ setup::NUM_PROCS }>>,
            *setup::MAIN_CAP.as_ref().unwrap(),
        );
    }
    // Exit QEMU with a return code of 0
    crate::tests::semihost_command_exit_success()
}

#[cfg(test)]
#[no_mangle]
pub unsafe fn main() {
    test_main();
}
