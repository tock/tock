use crate::BOARD;
use crate::CHIP;
use crate::MAIN_CAP;
use crate::NUM_PROCS;
use crate::PLATFORM;
use crate::SCHEDULER;
use kernel::{debug, Chip};

extern "C" {
    pub(crate) fn semihost_command(command: usize, arg0: usize, arg1: usize) -> !;
}

fn run_kernel_op(loops: usize) {
    unsafe {
        for _i in 0..loops {
            BOARD.unwrap().kernel_loop_operation(
                PLATFORM.unwrap(),
                CHIP.unwrap(),
                None::<&kernel::ipc::IPC<NUM_PROCS>>,
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
    run_kernel_op(10);

    assert_eq!(1, 1);

    debug!("    [ok]");
    run_kernel_op(10);
}

#[test_case]
fn check_epmp_regions() {
    debug!("check epmp regions... ");
    run_kernel_op(10);
    unsafe {
        debug!("{}", CHIP.unwrap().pmp);
    }
    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(10);
}

#[test_case]
fn check_pending_interrupts() {
    debug!("check pending interrupts... ");
    run_kernel_op(10);
    unsafe {
        assert_eq!(CHIP.unwrap().has_pending_interrupts(), false);
    }
    run_kernel_op(1000);

    debug!("    [ok]");
    run_kernel_op(10);
}
