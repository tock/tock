//! Component for KV System Driver.
//!
//! Usage
//! -----
//! ```rust
//!    let mux_kv = components::kv_system::KVStoreMuxComponent::new(tickv).finalize(
//!        components::kv_store_mux_component_static!(
//!            extra_capsules::tickv::TicKVStore<
//!                core_capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!            >,
//!            extra_capsules::tickv::TicKVKeyType,
//!        ),
//!    );
//!
//!    let kv_store = components::kv_system::KVStoreComponent::new(mux_kv).finalize(
//!        components::kv_store_component_static!(
//!            extra_capsules::tickv::TicKVStore<
//!                core_capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!            >,
//!            extra_capsules::tickv::TicKVKeyType,
//!        ),
//!    );
//!
//!    let kv_driver = components::kv_system::KVDriverComponent::new(
//!        kv_store,
//!        board_kernel,
//!        extra_capsules::kv_driver::DRIVER_NUM,
//!    )
//!    .finalize(components::kv_driver_component_static!(
//!        // extra_capsules::kv_store::KVStore<
//!        extra_capsules::tickv::TicKVStore<
//!            core_capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!        >,
//!        extra_capsules::tickv::TicKVKeyType,
//!    ));
//! ```

use core::mem::MaybeUninit;
use extra_capsules::kv_driver::KVSystemDriver;
use extra_capsules::kv_store::{KVStore, MuxKVStore};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::kv_system::{KVSystem, KeyType};

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_mux_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        kernel::static_buf!(extra_capsules::kv_store::MuxKVStore<'static, $K, $T>)
    };};
}

pub struct KVStoreMuxComponent<
    K: 'static + KVSystem<'static> + KVSystem<'static, K = T>,
    T: 'static + KeyType,
> {
    kv: &'static K,
}

impl<'a, K: 'static + KVSystem<'static> + KVSystem<'static, K = T>, T: 'static + KeyType>
    KVStoreMuxComponent<K, T>
{
    pub fn new(kv: &'static K) -> KVStoreMuxComponent<K, T> {
        Self { kv }
    }
}

impl<K: 'static + KVSystem<'static> + KVSystem<'static, K = T>, T: 'static + KeyType> Component
    for KVStoreMuxComponent<K, T>
{
    type StaticInput = &'static mut MaybeUninit<MuxKVStore<'static, K, T>>;
    type Output = &'static MuxKVStore<'static, K, T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(MuxKVStore::new(self.kv))
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        let kv_store = kernel::static_buf!(extra_capsules::kv_store::KVStore<'static, $K, $T>);
        let key = kernel::static_buf!($T);
        let buffer = kernel::static_buf!([u8; 9]);

        (kv_store, key, buffer)
    };};
}

pub struct KVStoreComponent<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> {
    kv_store: &'static MuxKVStore<'static, K, T>,
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> KVStoreComponent<K, T> {
    pub fn new(kv_store: &'static MuxKVStore<'static, K, T>) -> Self {
        Self { kv_store }
    }
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType + Default> Component
    for KVStoreComponent<K, T>
{
    type StaticInput = (
        &'static mut MaybeUninit<KVStore<'static, K, T>>,
        &'static mut MaybeUninit<T>,
        &'static mut MaybeUninit<[u8; 9]>,
    );
    type Output = &'static KVStore<'static, K, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let key_buf = static_buffer.1.write(T::default());
        let buffer = static_buffer.2.write([0; 9]);
        static_buffer
            .0
            .write(KVStore::new(self.kv_store, key_buf, buffer))
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_driver_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        let kv = kernel::static_buf!(extra_capsules::kv_driver::KVSystemDriver<'static, $K, $T>);
        let data_buffer = kernel::static_buf!([u8; 32]);
        let dest_buffer = kernel::static_buf!([u8; 48]);

        (kv, data_buffer, dest_buffer)
    };};
}

pub struct KVDriverComponent<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> {
    kv: &'static KVStore<'static, K, T>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> KVDriverComponent<K, T> {
    pub fn new(
        kv: &'static KVStore<'static, K, T>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Self {
            kv,
            board_kernel,
            driver_num,
        }
    }
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> Component
    for KVDriverComponent<K, T>
{
    type StaticInput = (
        &'static mut MaybeUninit<KVSystemDriver<'static, K, T>>,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; 48]>,
    );
    type Output = &'static KVSystemDriver<'static, K, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let data_buffer = static_buffer.1.write([0; 32]);
        let dest_buffer = static_buffer.2.write([0; 48]);

        let driver = static_buffer.0.write(KVSystemDriver::new(
            self.kv,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        self.kv.set_client(driver);
        driver
    }
}
