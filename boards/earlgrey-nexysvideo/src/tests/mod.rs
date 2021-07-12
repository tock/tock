use crate::BOARD;
use crate::CHIP;
use crate::MAIN_CAP;
use crate::NUM_PROCS;
use crate::PLATFORM;
use crate::SCHEDULER;
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
                None::<&kernel::ipc::IPC<NUM_PROCS>>,
                None::<&kernel::ros::ROSDriver<earlgrey::timer::RvTimer>>,
                SCHEDULER.unwrap(),
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
mod otbn;
