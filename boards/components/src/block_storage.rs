//! Component for block storage drivers.
//!
//! This provides one component, BlockStorageComponent, which provides
//! a system call inteface to block storage.
//!
//! Usage
//! -----
//! ```rust
//! let block_storage = components::block_storage::BlockStorageComponent::new(
//!     &flash_device,
//! )
//! .finalize(components::block_storage_component_helper!());
//! ```

use capsules::block_storage_driver;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init_half;

/// Setup static space for the objects.
/// B: the block device type.
/// W: Write block size.
/// E: Discard block size
#[macro_export]
macro_rules! block_storage_component_helper {
    ($B:ty, $W: literal, $E: literal $(,)?) => {{
        use capsules::block_storage_driver::BlockStorage;
        use core::mem::MaybeUninit;
        use kernel::hil;
        static mut BUF2: MaybeUninit<[u8; $W]> = MaybeUninit::uninit();
        static mut BUF3: MaybeUninit<BlockStorage<'static, $B, $W, $E>> = MaybeUninit::uninit();
        (&mut BUF2, &mut BUF3)
    };};
}

pub struct BlockStorageComponent<B, const W: usize, const E: usize>
where
    B: 'static + hil::block_storage::Storage<W, E>,
{
    pub board_kernel: &'static kernel::Kernel,
    pub driver_num: usize,
    pub device: &'static B,
}

impl<B, const W: usize, const E: usize> Component for BlockStorageComponent<B, W, E>
where
    B: 'static
        + hil::block_storage::Storage<W, E>
        + hil::block_storage::HasClient<'static, block_storage_driver::BlockStorage<'static, B, W, E>>,
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; W]>,
        &'static mut MaybeUninit<block_storage_driver::BlockStorage<'static, B, W, E>>,
    );
    type Output = &'static block_storage_driver::BlockStorage<'static, B, W, E>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let write_buffer = static_init_half!(static_buffer.0, [u8; W], [0; W],);

        let syscall_driver = static_init_half!(
            static_buffer.1,
            block_storage_driver::BlockStorage<'static, B, W, E>,
            block_storage_driver::BlockStorage::new(
                self.device,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                write_buffer,
            )
        );

        hil::block_storage::HasClient::set_client(self.device, syscall_driver);
        syscall_driver
    }
}
