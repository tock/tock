// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for KV System Driver.
//!
//! Usage
//! -----
//! ```rust
//!    let mux_kv = components::kv_system::KVStoreMuxComponent::new(tickv).finalize(
//!        components::kv_store_mux_component_static!(
//!            capsules_extra::tickv::TicKVStore<
//!                capsules_core::virtualizers::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!            >,
//!            capsules_extra::tickv::TicKVKeyType,
//!        ),
//!    );
//!
//!    let kv_store = components::kv_system::KVStoreComponent::new(mux_kv).finalize(
//!        components::kv_store_component_static!(
//!            capsules_extra::tickv::TicKVStore<
//!                capsules_core::virtualizers::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!            >,
//!            capsules_extra::tickv::TicKVKeyType,
//!        ),
//!    );
//!
//!    let kv_driver = components::kv_system::KVDriverComponent::new(
//!        kv_store,
//!        board_kernel,
//!        capsules_extra::kv_driver::DRIVER_NUM,
//!    )
//!    .finalize(components::kv_driver_component_static!(
//!        // capsules_extra::kv_store::KVStore<
//!        capsules_extra::tickv::TicKVStore<
//!            capsules_core::virtualizers::virtual_flash::FlashUser<lowrisc::flash_ctrl::FlashCtrl>,
//!        >,
//!        capsules_extra::tickv::TicKVKeyType,
//!    ));
//! ```

use capsules_extra::kv_driver::KVSystemDriver;
use capsules_extra::kv_store::{KVStore, MuxKVStore};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::kv_system::{KVSystem, KeyType};

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_mux_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        let key = kernel::static_buf!($T);
        let mux = kernel::static_buf!(capsules_extra::kv_store::MuxKVStore<'static, $K, $T>);
        let buffer = kernel::static_buf!([u8; 9]);

        (mux, key, buffer)
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

impl<K: 'static + KVSystem<'static> + KVSystem<'static, K = T>, T: 'static + KeyType + Default>
    Component for KVStoreMuxComponent<K, T>
{
    type StaticInput = (
        &'static mut MaybeUninit<MuxKVStore<'static, K, T>>,
        &'static mut MaybeUninit<T>,
        &'static mut MaybeUninit<[u8; 9]>,
    );
    type Output = &'static MuxKVStore<'static, K, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let key_buf = static_buffer.1.write(T::default());
        let buffer = static_buffer.2.write([0; 9]);

        let mux = static_buffer
            .0
            .write(MuxKVStore::new(self.kv, key_buf, buffer));
        self.kv.set_client(mux);
        mux
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        let kv_store = kernel::static_buf!(capsules_extra::kv_store::KVStore<'static, $K, $T>);

        kv_store
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

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> Component
    for KVStoreComponent<K, T>
{
    type StaticInput = &'static mut MaybeUninit<KVStore<'static, K, T>>;
    type Output = &'static KVStore<'static, K, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let kv_store = static_buffer.write(KVStore::new(self.kv_store));
        kv_store.setup();
        kv_store
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_driver_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        let kv = kernel::static_buf!(capsules_extra::kv_driver::KVSystemDriver<'static, $K, $T>);
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
