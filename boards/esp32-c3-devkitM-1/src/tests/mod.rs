use crate::BOARD;
use crate::CHIP;
use crate::MAIN_CAP;
use crate::NUM_PROCS;
use crate::PLATFORM;
use kernel::debug;

fn run_kernel_op(loops: usize) {
    unsafe {
        for _i in 0..loops {
            BOARD.unwrap().kernel_loop_operation(
                PLATFORM.unwrap(),
                CHIP.unwrap(),
                None::<&kernel::ipc::IPC<0>>,
                true,
                MAIN_CAP.unwrap(),
            );
        }
    }
}

#[test_case]
fn trivial_assertion() {
    debug!("trivial assertion... ");
    run_kernel_op(10000);

    assert_eq!(1, 1);

    debug!("    [ok]");
    run_kernel_op(10000);
}

mod multi_alarm;
