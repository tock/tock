//! Test TicKV

use crate::tests::run_kernel_op;
use crate::TICKV;
use capsules::test::kv_system::KVSystemTest;
use capsules::tickv::{TicKVKeyType, TicKVStore};
use capsules::virtual_flash::FlashUser;
use kernel::debug;
use kernel::hil::kv_system::KVSystem;
use kernel::static_init;

#[test_case]
fn tickv_append_key() {
    debug!("start TicKV append key test...");

    unsafe {
        let tickv = TICKV.unwrap();
        let key = static_init!([u8; 8], [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]);
        let value = static_init!([u8; 3], [0x10, 0x20, 0x30]);
        let ret = static_init!([u8; 4], [0; 4]);

        let test = static_init!(
            KVSystemTest<
                'static,
                TicKVStore<'static, FlashUser<'static, lowrisc::flash_ctrl::FlashCtrl<'static>>>,
                TicKVKeyType,
            >,
            KVSystemTest::new(tickv, ret)
        );

        tickv.set_client(test);

        // Kick start the tests by adding a key
        tickv.append_key(key, value).unwrap();
    }
    run_kernel_op(100000);

    debug!("    [ok]");
    run_kernel_op(100);
}
