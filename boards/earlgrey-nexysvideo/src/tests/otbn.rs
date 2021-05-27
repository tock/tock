use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::{debug, ErrorCode};
use lowrisc::otbn::Client;

static mut BUF: [u8; 32] = [0; 32];
static mut OUTPUT: [u8; 1024] = [0; 1024];

struct OtbnTestCallback {}

impl<'a> Client<'a, 1024> for OtbnTestCallback {
    fn binary_load_done(&'a self, result: Result<(), ErrorCode>, _input: &'static mut [u8]) {
        debug!("  OTBN binary load callback received");
        assert_eq!(result, Ok(()));
    }

    fn op_done(&'a self, result: Result<(), ErrorCode>, _output: &'static mut [u8; 1024]) {
        debug!("  OTBN op done callback received");
        assert_eq!(result, Err(ErrorCode::FAIL));
    }
}

static CALLBACK: OtbnTestCallback = OtbnTestCallback {};

#[test_case]
fn otbn_check_load_empty_binary() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let otbn = &perf.otbn;
    let buf = unsafe { LeasableBuffer::new(&mut BUF) };

    debug!("check otbn load empty binary... ");
    run_kernel_op(100);

    otbn.set_client(&CALLBACK);
    assert_eq!(otbn.load_binary(buf), Ok(()));

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn otbn_check_invalid_prop() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let otbn = &perf.otbn;

    debug!("check otbn invalid prop... ");
    run_kernel_op(100);

    assert_eq!(otbn.set_property(100, 0), Err(ErrorCode::NOSUPPORT));

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn otbn_check_start_addr_prop() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let otbn = &perf.otbn;

    debug!("check otbn start addr prop... ");
    run_kernel_op(100);

    assert_eq!(otbn.set_property(0, 0x100), Ok(()));

    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn otbn_check_run_empty_binary() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let otbn = &perf.otbn;

    debug!("check otbn run empty binary... ");
    run_kernel_op(100);

    otbn.set_client(&CALLBACK);
    assert_eq!(unsafe { otbn.run(&mut OUTPUT) }, Ok(()));

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}
