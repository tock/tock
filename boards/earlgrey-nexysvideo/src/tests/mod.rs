use crate::BOARD;
use crate::CHIP;
use crate::MAIN_CAP;
use crate::PLATFORM;
use crate::{NUM_PROCS, NUM_UPCALLS_IPC};
use kernel::debug;

pub fn semihost_command_exit_success() -> ! {
    // Exit QEMU with a return code of 0
    unsafe {
        rv32i::semihost_command(0x18, 0x20026, 0);
    }
    loop {}
}

pub fn semihost_command_exit_failure() -> ! {
    // Exit QEMU with a return code of 1
    unsafe {
        rv32i::semihost_command(0x18, 1, 0);
    }
    loop {}
}

fn run_kernel_op(loops: usize) {
    unsafe {
        for _i in 0..loops {
            BOARD.unwrap().kernel_loop_operation(
                PLATFORM.unwrap(),
                CHIP.unwrap(),
                None::<&kernel::ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
                true,
                MAIN_CAP.unwrap(),
            );
        }
    }
}

#[test_case]
fn trivial_assertion() {
    debug!("trivial assertion... ");
    run_kernel_op(100);

    assert_eq!(1, 1);

    debug!("    [ok]");
    run_kernel_op(100);
}

mod aes_test;
mod hmac;
mod multi_alarm;
// OTBN is no longer included in the FPGA build, so we disable the tests
// For a FPGA build that works with OTBN see lowRISC/opentitan@f50ded219d28c9c669607409cbb7bd1383634e48
// mod otbn;
mod tickv_test;
