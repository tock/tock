//! Component for KV System Driver.
//!
//! Usage
//! -----
//! ```rust
//!    let mux_kv = components::kv_system::KVStoreMuxComponent::new(tickv).finalize(
//!        components::kv_store_mux_component_helper!(
//!            capsules::tickv::TicKVStore<
//!                capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!            >,
//!            capsules::tickv::TicKVKeyType,
//!        ),
//!    );
//!
//!    let kv_store = components::kv_system::KVStoreComponent::new(mux_kv).finalize(
//!        components::kv_store_component_helper!(
//!            capsules::tickv::TicKVStore<
//!                capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!            >,
//!            capsules::tickv::TicKVKeyType,
//!        ),
//!    );
//!
//!    let kv_driver_data_buf = static_init!([u8; 32], [0; 32]);
//!    let kv_driver_dest_buf = static_init!(capsules::tickv::TicKVKeyType, [0; 8]);
//!
//!    let kv_driver = components::kv_system::KVDriverComponent::new(
//!        kv_store,
//!        board_kernel,
//!        capsules::kv_driver::DRIVER_NUM,
//!        kv_driver_data_buf,
//!        kv_driver_dest_buf,
//!    )
//!    .finalize(components::kv_driver_component_helper!(
//!        // capsules::kv_store::KVStore<
//!        capsules::tickv::TicKVStore<
//!            capsules::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!        >,
//!        capsules::tickv::TicKVKeyType,
//!    ));
//! ```

use capsules::kv_driver::KVSystemDriver;
use capsules::kv_store::{KVStore, MuxKVStore};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::kv_system::{KVSystem, KeyType};
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_mux_component_helper {
    ($K:ty, $T:ty $(,)?) => {{
        use capsules::kv_store::MuxKVStore;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxKVStore<'static, $K, $T>> = MaybeUninit::uninit();
        &mut BUF1
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

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        static_init_half!(s, MuxKVStore<'static, K, T>, MuxKVStore::new(self.kv))
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_component_helper {
    ($K:ty, $T:ty $(,)?) => {{
        use capsules::kv_store::KVStore;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<KVStore<'static, $K, $T>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct KVStoreComponent<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> {
    kv_store: &'static MuxKVStore<'static, K, T>,
    key_buf: &'static mut T,
    header_buf: &'static mut [u8; 9],
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> KVStoreComponent<K, T> {
    pub fn new(
        kv_store: &'static MuxKVStore<'static, K, T>,
        key_buf: &'static mut T,
        header_buf: &'static mut [u8; 9],
    ) -> Self {
        Self {
            kv_store,
            key_buf,
            header_buf,
        }
    }
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> Component
    for KVStoreComponent<K, T>
{
    type StaticInput = &'static mut MaybeUninit<KVStore<'static, K, T>>;
    type Output = &'static KVStore<'static, K, T>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_init_half!(
            static_buffer,
            KVStore<'static, K, T>,
            KVStore::new(self.kv_store, self.key_buf, self.header_buf)
        )
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_driver_component_helper {
    ($K:ty, $T:ty $(,)?) => {{
        use capsules::kv_driver::KVSystemDriver;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<KVSystemDriver<'static, $K, $T>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct KVDriverComponent<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> {
    kv: &'static KVStore<'static, K, T>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    data_buffer: &'static mut [u8],
    dest_buffer: &'static mut [u8],
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> KVDriverComponent<K, T> {
    pub fn new(
        kv: &'static KVStore<'static, K, T>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8],
    ) -> Self {
        Self {
            kv,
            board_kernel,
            driver_num,
            data_buffer,
            dest_buffer,
        }
    }
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> Component
    for KVDriverComponent<K, T>
{
    type StaticInput = &'static mut MaybeUninit<KVSystemDriver<'static, K, T>>;
    type Output = &'static KVSystemDriver<'static, K, T>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let driver = static_init_half!(
            static_buffer,
            KVSystemDriver<'static, K, T>,
            KVSystemDriver::new(
                self.kv,
                self.data_buffer,
                self.dest_buffer,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );
        self.kv.set_client(driver);
        driver
    }
}
