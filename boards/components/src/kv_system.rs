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

use capsules_extra::kv_driver::KVStoreDriver;
use capsules_extra::kv_store::KVStore;
use capsules_extra::kv_store_permissions::KVStorePermissions;
use capsules_extra::tickv::{KVSystem, KeyType};
use capsules_extra::virtual_kv::{MuxKVPermissions, VirtualKVPermissions};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

//////////////
// KV Mux
//////////////

#[macro_export]
macro_rules! kv_permissions_mux_component_static {
    ($V:ty $(,)?) => {{
        let mux = kernel::static_buf!(capsules_extra::virtual_kv::MuxKVPermissions<'static, $V>);

        mux
    };};
}

pub struct KVPermissionsMuxComponent<V: hil::kv::KVPermissions<'static> + 'static> {
    kv: &'static V,
}

impl<'a, V: hil::kv::KVPermissions<'static>> KVPermissionsMuxComponent<V> {
    pub fn new(kv: &'static V) -> KVPermissionsMuxComponent<V> {
        Self { kv }
    }
}

impl<V: hil::kv::KVPermissions<'static> + 'static> Component for KVPermissionsMuxComponent<V> {
    type StaticInput = &'static mut MaybeUninit<MuxKVPermissions<'static, V>>;
    type Output = &'static MuxKVPermissions<'static, V>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mux = static_buffer.write(MuxKVPermissions::new(self.kv));
        self.kv.set_client(mux);
        mux
    }
}

/////////////////////
// Virtual KV Object
/////////////////////

#[macro_export]
macro_rules! virtual_kv_permissions_component_static {
    ($V:ty $(,)?) => {{
        let virtual_kv =
            kernel::static_buf!(capsules_extra::virtual_kv::VirtualKVPermissions<'static, $V>);

        virtual_kv
    };};
}

pub struct VirtualKVPermissionsComponent<V: hil::kv::KVPermissions<'static> + 'static> {
    mux_kv: &'static MuxKVPermissions<'static, V>,
}

impl<'a, V: hil::kv::KVPermissions<'static>> VirtualKVPermissionsComponent<V> {
    pub fn new(mux_kv: &'static MuxKVPermissions<'static, V>) -> VirtualKVPermissionsComponent<V> {
        Self { mux_kv }
    }
}

impl<V: hil::kv::KVPermissions<'static> + 'static> Component for VirtualKVPermissionsComponent<V> {
    type StaticInput = &'static mut MaybeUninit<VirtualKVPermissions<'static, V>>;
    type Output = &'static VirtualKVPermissions<'static, V>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let virtual_kv = static_buffer.write(VirtualKVPermissions::new(self.mux_kv));
        virtual_kv.setup();
        virtual_kv
    }
}

/////////////////////
// KV Store Permissions
/////////////////////

#[macro_export]
macro_rules! kv_store_permissions_component_static {
    ($V:ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::kv_store_permissions::HEADER_LENGTH]);
        let kv_store = kernel::static_buf!(
            capsules_extra::kv_store_permissions::KVStorePermissions<'static, $V>
        );

        (kv_store, buffer)
    };};
}

pub struct KVStorePermissionsComponent<V: hil::kv::KV<'static> + 'static> {
    kv: &'static V,
}

impl<V: hil::kv::KV<'static> + 'static> KVStorePermissionsComponent<V> {
    pub fn new(kv: &'static V) -> Self {
        Self { kv }
    }
}

impl<V: hil::kv::KV<'static> + 'static> Component for KVStorePermissionsComponent<V> {
    type StaticInput = (
        &'static mut MaybeUninit<KVStorePermissions<'static, V>>,
        &'static mut MaybeUninit<[u8; capsules_extra::kv_store_permissions::HEADER_LENGTH]>,
    );
    type Output = &'static KVStorePermissions<'static, V>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .1
            .write([0; capsules_extra::kv_store_permissions::HEADER_LENGTH]);

        let kv_store_permissions = static_buffer
            .0
            .write(KVStorePermissions::new(self.kv, buffer));

        self.kv.set_client(kv_store_permissions);

        kv_store_permissions
    }
}

/////////////////////
// KV Store
/////////////////////

#[macro_export]
macro_rules! kv_store_component_static {
    ($K:ty, $T:ty $(,)?) => {{
        let key = kernel::static_buf!($T);
        let kv_store = kernel::static_buf!(capsules_extra::kv_store::KVStore<'static, $K, $T>);

        (kv_store, key)
    };};
}

pub struct KVStoreComponent<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> {
    kv_system: &'static K,
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType> KVStoreComponent<K, T> {
    pub fn new(kv_system: &'static K) -> Self {
        Self { kv_system }
    }
}

impl<K: 'static + KVSystem<'static, K = T>, T: 'static + KeyType + Default> Component
    for KVStoreComponent<K, T>
{
    type StaticInput = (
        &'static mut MaybeUninit<KVStore<'static, K, T>>,
        &'static mut MaybeUninit<T>,
    );
    type Output = &'static KVStore<'static, K, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let key_buf = static_buffer.1.write(T::default());

        let kv_store = static_buffer.0.write(KVStore::new(self.kv_system, key_buf));

        self.kv_system.set_client(kv_store);

        kv_store
    }
}

///////////////////////
// KV Userspace Driver
///////////////////////

#[macro_export]
macro_rules! kv_driver_component_static {
    ($V:ty $(,)?) => {{
        let kv = kernel::static_buf!(capsules_extra::kv_driver::KVStoreDriver<'static, $V>);
        let data_buffer = kernel::static_buf!([u8; 32]);
        let dest_buffer = kernel::static_buf!([u8; 48]);

        (kv, data_buffer, dest_buffer)
    };};
}

pub struct KVDriverComponent<V: hil::kv::KVPermissions<'static> + 'static> {
    kv: &'static V,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<V: hil::kv::KVPermissions<'static>> KVDriverComponent<V> {
    pub fn new(kv: &'static V, board_kernel: &'static kernel::Kernel, driver_num: usize) -> Self {
        Self {
            kv,
            board_kernel,
            driver_num,
        }
    }
}

impl<V: hil::kv::KVPermissions<'static>> Component for KVDriverComponent<V> {
    type StaticInput = (
        &'static mut MaybeUninit<KVStoreDriver<'static, V>>,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; 48]>,
    );
    type Output = &'static KVStoreDriver<'static, V>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let data_buffer = static_buffer.1.write([0; 32]);
        let dest_buffer = static_buffer.2.write([0; 48]);

        let driver = static_buffer.0.write(KVStoreDriver::new(
            self.kv,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        self.kv.set_client(driver);
        driver
    }
}
