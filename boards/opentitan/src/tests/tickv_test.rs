//! Test TicKV

use crate::tests::run_kernel_op;
use crate::{SIPHASH, TICKV};
use core_capsules::virtual_flash::FlashUser;
use extra_capsules::test::kv_system::KVSystemTest;
use extra_capsules::tickv::{TicKVKeyType, TicKVStore};
use kernel::debug;
use kernel::hil::hasher::Hasher;
use kernel::hil::kv_system::KVSystem;
use kernel::static_init;

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
                TicKVStore<
                    'static,
                    FlashUser<'static, lowrisc::flash_ctrl::FlashCtrl<'static>>,
                    extra_capsules::sip_hash::SipHasher24,
                >,
                TicKVKeyType,
            >,
            KVSystemTest::new(tickv, value, ret)
        );

        sip_hasher.set_client(tickv);
        tickv.set_client(test);

        // Kick start the tests by generating a key
        tickv.generate_key(key_input, key).unwrap();
    }
    run_kernel_op(100000);

    debug!("    [ok]");
    run_kernel_op(100);
}
