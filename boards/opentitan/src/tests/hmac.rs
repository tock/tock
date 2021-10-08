use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use core::cell::Cell;
use kernel::hil::digest::{self, Digest, HMACSha256};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::{debug, ErrorCode};

static KEY: [u8; 32] = [0xA1; 32];
static mut INPUT: [u8; 32] = [32; 32];
static mut DIGEST: [u8; 32] = [
    0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7, 0x65, 0xbd, 0xe, 0x2, 0x82, 0xf7, 0x8b, 0xe1,
    0xef, 0xd1, 0xb, 0xdc, 0xa8, 0xba, 0xe1, 0xfa, 0x11, 0x3f, 0xf6, 0xeb, 0xaf, 0x58, 0x57, 0x40,
];

struct HmacTestCallback {
    add_data_done: Cell<bool>,
    verification_done: Cell<bool>,
}

unsafe impl Sync for HmacTestCallback {}

impl<'a> HmacTestCallback {
    const fn new() -> Self {
        HmacTestCallback {
            add_data_done: Cell::new(false),
            verification_done: Cell::new(false),
        }
    }

    fn reset(&self) {
        self.add_data_done.set(false);
        self.verification_done.set(false);
    }
}

impl<'a> digest::Client<'a, 32> for HmacTestCallback {
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, _data: &'static mut [u8]) {
        self.add_data_done.set(true);
        assert_eq!(result, Ok(()));
    }

    fn hash_done(&'a self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 32]) {
        unimplemented!()
    }

    fn verification_done(
        &'a self,
        result: Result<bool, ErrorCode>,
        compare: &'static mut [u8; 32],
    ) {
        self.verification_done.set(true);
        assert_eq!(result, Ok(true));
    }
}

static CALLBACK: HmacTestCallback = HmacTestCallback::new();

#[test_case]
fn hmac_check_load_binary() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let hmac = &perf.hmac;
    let buf = unsafe { LeasableBuffer::new(&mut INPUT) };

    debug!("check hmac load binary... ");
    run_kernel_op(100);

    hmac.set_client(&CALLBACK);
    assert_eq!(hmac.add_data(buf), Ok(32));

    run_kernel_op(1000);
    #[cfg(feature = "hardware_tests")]
    assert_eq!(CALLBACK.add_data_done.get(), true);

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn hmac_check_verify() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let hmac = &perf.hmac;
    let buf = unsafe { LeasableBuffer::new(&mut INPUT) };

    debug!("check hmac check verify... ");
    run_kernel_op(100);

    hmac.set_client(&CALLBACK);
    hmac.set_mode_hmacsha256(&KEY).unwrap();
    assert_eq!(hmac.add_data(buf), Ok(32));

    run_kernel_op(1000);
    #[cfg(feature = "hardware_tests")]
    assert_eq!(CALLBACK.add_data_done.get(), true);

    unsafe {
        assert_eq!(hmac.verify(&mut DIGEST), Ok(()));
    }

    run_kernel_op(1000);
    #[cfg(feature = "hardware_tests")]
    assert_eq!(CALLBACK.verification_done.get(), true);

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}
