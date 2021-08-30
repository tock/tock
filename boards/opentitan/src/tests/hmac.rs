use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use kernel::hil::digest::{self, Digest};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::{debug, ErrorCode};

static mut BUF: [u8; 32] = [0; 32];

struct HmacTestCallback {}

impl<'a> digest::Client<'a, 32> for HmacTestCallback {
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, _data: &'static mut [u8]) {
        assert_eq!(result, Ok(()));
    }

    fn hash_done(&'a self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 32]) {
        unimplemented!()
    }
}

static CALLBACK: HmacTestCallback = HmacTestCallback {};

#[test_case]
fn hmac_check_load_binary() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let hmac = &perf.hmac;
    let buf = unsafe { LeasableBuffer::new(&mut BUF) };

    debug!("check hmac load binary... ");
    run_kernel_op(100);

    hmac.set_client(&CALLBACK);
    assert_eq!(hmac.add_data(buf), Ok(32));

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}
