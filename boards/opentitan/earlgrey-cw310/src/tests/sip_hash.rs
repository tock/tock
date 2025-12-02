// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::tests::run_kernel_op;
use crate::SIPHASH;
use core::cell::Cell;
use kernel::hil::hasher::{self, Hasher};
use kernel::static_init;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::{debug, ErrorCode};

struct SipHashTestCallback {
    data_add_done: Cell<bool>,
    hash_done: Cell<bool>,
    input_buf: [TakeCell<'static, [u8; 8]>; 20],
    output_buf: TakeCell<'static, [u8; 8]>,
    cb_count: Cell<usize>,
    run_count: Cell<usize>,
}

unsafe impl Sync for SipHashTestCallback {}

impl<'a> SipHashTestCallback {
    fn new(input_buf: [TakeCell<'static, [u8; 8]>; 20], output: &'static mut [u8; 8]) -> Self {
        SipHashTestCallback {
            data_add_done: Cell::new(false),
            hash_done: Cell::new(false),
            input_buf,
            output_buf: TakeCell::new(output),
            cb_count: Cell::new(0),
            run_count: Cell::new(0),
        }
    }

    fn run_reset(&self) {
        self.data_add_done.set(false);
        self.hash_done.set(false);
    }

    fn full_reset(&self) {
        self.cb_count.set(0);
        self.run_count.set(self.run_count.get() + 1);
        self.run_reset();
    }
}

impl<'a> hasher::Client<8> for SipHashTestCallback {
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        unimplemented!()
    }

    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        assert_eq!(result, Ok(()));
        self.data_add_done.set(true);

        // Stay within valid ranges
        assert_eq!(self.cb_count.get() < 20, true);

        // Replace the input buffer with all of cb data
        self.input_buf[self.cb_count.get()].replace(data.take().try_into().unwrap());

        self.cb_count.set(self.cb_count.get() + 1);
    }

    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; 8]) {
        let ret = u64::from_le_bytes(*digest);

        assert_eq!(result, Ok(()));
        // Value calculated from:
        //    https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=08a45842acb9dc1abf9dbc303b5eaa50
        if self.run_count.get() == 0 {
            assert_eq!(ret, 0x9ed5975598371f51);
        } else if self.run_count.get() == 1 {
            assert_eq!(ret, 0xB5326A7D96F8A5B7);
        } else {
            panic!("Unsupported number of callbacks");
        }

        self.hash_done.set(true);
        self.output_buf.replace(digest);
    }
}

unsafe fn static_init_test_cb() -> &'static SipHashTestCallback {
    let output = static_init!([u8; 8], [0; 8]);
    let input = [
        TakeCell::new(static_init!(
            [u8; 8],
            [0x31, 0x0e, 0x0e, 0xdd, 0x47, 0xdb, 0x6f, 0x72]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xfd, 0x67, 0xdc, 0x93, 0xc5, 0x39, 0xf8, 0x74]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x5a, 0x4f, 0xa9, 0xd9, 0x09, 0x80, 0x6c, 0x0d]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x2d, 0x7e, 0xfb, 0xd7, 0x96, 0x66, 0x67, 0x85]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xb7, 0x87, 0x71, 0x27, 0xe0, 0x94, 0x27, 0xcf]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x8d, 0xa6, 0x99, 0xcd, 0x64, 0x55, 0x76, 0x18]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xce, 0xe3, 0xfe, 0x58, 0x6e, 0x46, 0xc9, 0xcb]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x37, 0xd1, 0x01, 0x8b, 0xf5, 0x00, 0x02, 0xab]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x62, 0x24, 0x93, 0x9a, 0x79, 0xf5, 0xf5, 0x93]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xb0, 0xe4, 0xa9, 0x0b, 0xdf, 0x82, 0x00, 0x9e]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xf3, 0xb9, 0xdd, 0x94, 0xc5, 0xbb, 0x5d, 0x7a]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xa7, 0xad, 0x6b, 0x22, 0x46, 0x2f, 0xb3, 0xf4]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xfb, 0xe5, 0x0e, 0x86, 0xbc, 0x8f, 0x1e, 0x75]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x90, 0x3d, 0x84, 0xc0, 0x27, 0x56, 0xea, 0x14]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xee, 0xf2, 0x7a, 0x8e, 0x90, 0xca, 0x23, 0xf7]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xe5, 0x45, 0xbe, 0x49, 0x61, 0xca, 0x29, 0xa1]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0xdb, 0x9b, 0xc2, 0x57, 0x7f, 0xcc, 0x2a, 0x3f]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x94, 0x47, 0xbe, 0x2c, 0xf5, 0xe9, 0x9a, 0x69]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x9c, 0xd3, 0x8d, 0x96, 0xf0, 0xb3, 0xc1, 0x4b]
        )),
        TakeCell::new(static_init!(
            [u8; 8],
            [0x28, 0xef, 0x49, 0x5c, 0x53, 0xa3, 0x87, 0xad]
        )),
    ];

    static_init!(SipHashTestCallback, SipHashTestCallback::new(input, output))
}

#[test_case]
fn sip_hasher_2_4() {
    let sip_hasher = unsafe { SIPHASH.unwrap() };
    let cb = unsafe { static_init_test_cb() };

    debug!("check SipHash 2-4... ");
    run_kernel_op(100);

    sip_hasher.set_client(cb);

    for slice in cb.input_buf.iter() {
        // Data add done should be reset per each slice
        cb.run_reset();
        assert_eq!(
            sip_hasher.add_mut_data(SubSliceMut::new(slice.take().unwrap())),
            Ok(8)
        );

        run_kernel_op(100);
        assert_eq!(cb.data_add_done.get(), true);
    }

    assert_eq!(sip_hasher.run(cb.output_buf.take().unwrap()), Ok(()));
    run_kernel_op(100);
    assert_eq!(cb.hash_done.get(), true);

    cb.full_reset();

    for slice in cb.input_buf.iter() {
        // Data add done should be reset per each slice
        cb.run_reset();
        assert_eq!(
            sip_hasher.add_mut_data(SubSliceMut::new(slice.take().unwrap())),
            Ok(8)
        );

        run_kernel_op(100);
        assert_eq!(cb.data_add_done.get(), true);
    }

    assert_eq!(sip_hasher.run(cb.output_buf.take().unwrap()), Ok(()));
    run_kernel_op(100);
    assert_eq!(cb.hash_done.get(), true);

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}
