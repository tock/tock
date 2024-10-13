// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::tests::run_kernel_op;
use crate::ATECC508A;
use core::cell::Cell;
use kernel::hil::digest::{self, DigestData, DigestHash};
use kernel::static_init;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{debug, ErrorCode};

struct ShaTestCallback {
    add_mut_data_done: Cell<bool>,
    digest_done: Cell<bool>,
    input_buffer: TakeCell<'static, [u8]>,
    digest_buffer: TakeCell<'static, [u8; 32]>,
}

unsafe impl Sync for ShaTestCallback {}

impl<'a> ShaTestCallback {
    fn new(input_buffer: &'static mut [u8], digest_buffer: &'static mut [u8; 32]) -> Self {
        ShaTestCallback {
            add_mut_data_done: Cell::new(false),
            digest_done: Cell::new(false),
            input_buffer: TakeCell::new(input_buffer),
            digest_buffer: TakeCell::new(digest_buffer),
        }
    }

    fn reset(&self) {
        self.add_mut_data_done.set(false);
        self.digest_done.set(false);
    }
}

impl<'a> digest::ClientData<32> for ShaTestCallback {
    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        self.add_mut_data_done.set(true);
        // Check that all of the data was accepted and the active slice is length 0
        assert_eq!(data.len(), 0);
        // Input data has been loaded, hold copy of data
        self.input_buffer.replace(data.take());
        assert_eq!(result, Ok(()));
    }

    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        unimplemented!()
    }
}

impl<'a> digest::ClientHash<32> for ShaTestCallback {
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; 32]) {
        debug!("Hash: {digest:x?}");

        self.digest_buffer.replace(digest);
        self.digest_done.set(true);
        assert_eq!(result, Ok(()));
    }
}

/// Static init an ShaTestCallback, with
/// respective buffers allocated for data fields.
macro_rules! static_init_test_cb {
    () => {{
        let input_data = static_init!([u8; 32], [32; 32]);
        let digest_data = static_init!(
            [u8; 32],
            [
                0x23, 0x85, 0x1d, 0x3e, 0x42, 0x62, 0xc4, 0x94, 0xb5, 0xe2, 0x6e, 0xd6, 0x4f, 0xf4,
                0xaf, 0xb9, 0xf7, 0x80, 0xfb, 0xc8, 0xd1, 0x22, 0x13, 0x4a, 0xce, 0xfb, 0xea, 0x75,
                0x0a, 0x41, 0xf7, 0x1a
            ]
        );

        static_init!(
            ShaTestCallback,
            ShaTestCallback::new(input_data, digest_data)
        )
    }};
}

#[test_case]
fn hmac_check_load_binary() {
    let atecc508a = unsafe { ATECC508A.unwrap() };

    let callback = unsafe { static_init_test_cb!() };

    debug!("check hmac load binary... ");
    run_kernel_op(100);

    digest::DigestDataHash::set_client(atecc508a, callback);
    callback.reset();

    debug!("    adding 1st data... ");
    let buf = SubSliceMut::new(callback.input_buffer.take().unwrap());
    assert_eq!(atecc508a.add_mut_data(buf), Ok(()));
    run_kernel_op(30_000);
    assert_eq!(callback.add_mut_data_done.get(), true);
    callback.reset();

    debug!("    adding 2nd data... ");
    let buf = SubSliceMut::new(callback.input_buffer.take().unwrap());
    assert_eq!(atecc508a.add_mut_data(buf), Ok(()));
    run_kernel_op(350_000);
    assert_eq!(callback.add_mut_data_done.get(), true);
    callback.reset();

    debug!("    adding 3rd data... ");
    let buf = SubSliceMut::new(callback.input_buffer.take().unwrap());
    assert_eq!(atecc508a.add_mut_data(buf), Ok(()));
    run_kernel_op(30_000);
    assert_eq!(callback.add_mut_data_done.get(), true);
    callback.reset();

    debug!("    performing hash... ");
    assert_eq!(
        atecc508a.run(callback.digest_buffer.take().unwrap()),
        Ok(())
    );

    run_kernel_op(150_000);
    assert_eq!(callback.digest_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
