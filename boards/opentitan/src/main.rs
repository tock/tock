//! Board file for LowRISC OpenTitan RISC-V development platform.
//!
//! - <https://opentitan.org/>

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(my_test_runner)]
#![reexport_test_harness_main = "test_main"]

use earlgrey_cw310::setup;
use kernel::capabilities;
use kernel::create_capability;

pub const NUM_PROCS: usize = 4;
/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        let (board_kernel, earlgrey_nexysvideo, chip, _peripherals) = setup::setup();

        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(
            earlgrey_nexysvideo,
            chip,
            None::<&kernel::ipc::IPC<NUM_PROCS>>,
            &main_loop_cap,
        );
    }
}
