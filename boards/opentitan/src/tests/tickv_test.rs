// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test TicKV

use crate::tests::run_kernel_op;
use crate::{SIPHASH, TICKV};
use capsules_core::virtualizers::virtual_flash::FlashUser;
use capsules_extra::test::kv_system::KVSystemTest;
use capsules_extra::tickv::KVSystem;
use capsules_extra::tickv::{TicKVKeyType, TicKVSystem};
use kernel::debug;
use kernel::hil::hasher::Hasher;
use kernel::static_init;
use kernel::utilities::leasable_buffer::SubSliceMut;

#[test_case]
fn tickv_append_key() {
    debug!("start TicKV append key test...");

    unsafe {
        let tickv = TICKV.unwrap();
        let sip_hasher = SIPHASH.unwrap();

        let key_input = static_init!(
            [u8; 16],
            [
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC,
                0xDE, 0xF0
            ]
        );
        let key = static_init!([u8; 8], [0; 8]);
        let value = static_init!([u8; 3], [0x10, 0x20, 0x30]);
        let ret = static_init!([u8; 4], [0; 4]);

        let test = static_init!(
            KVSystemTest<
                'static,
                TicKVSystem<
                    'static,
                    FlashUser<'static, lowrisc::flash_ctrl::FlashCtrl<'static>>,
                    capsules_extra::sip_hash::SipHasher24,
                    2048,
                >,
                TicKVKeyType,
            >,
            KVSystemTest::new(tickv, SubSliceMut::new(value), ret)
        );

        sip_hasher.set_client(tickv);
        tickv.set_client(test);

        // Kick start the tests by generating a key
        tickv
            .generate_key(SubSliceMut::new(key_input), key)
            .unwrap();
    }
    run_kernel_op(100000);

    debug!("    [ok]");
    run_kernel_op(100);
}
