//! Test TicKV

use capsules::test::kv_system::KVSystemTest;
use capsules::tickv::{TicKVKeyType, TicKVStore};
use capsules::virtual_flash::FlashUser;
use kernel::hil::kv_system::KVSystem;
use kernel::static_init;

static mut KEY: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
static mut VALUE: [u8; 3] = [0x10, 0x20, 0x30];

pub unsafe fn run_tickv_tests(
    tickv: &'static TicKVStore<
        'static,
        FlashUser<'static, lowrisc::flash_ctrl::FlashCtrl<'static>>,
    >,
) {
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
