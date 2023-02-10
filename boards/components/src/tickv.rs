//! Component for TicKV KV System Driver.
//!
//! This provides one component, TicKVComponent, which provides
//! a system call inteface to non-volatile storage.
//!
//! Usage
//! -----
//! ```rust
//!    let flash_ctrl_read_buf = static_init!(
//!        [u8; lowrisc::flash_ctrl::PAGE_SIZE],
//!        [0; lowrisc::flash_ctrl::PAGE_SIZE]
//!    );
//!    let page_buffer = static_init!(
//!        lowrisc::flash_ctrl::LowRiscPage,
//!        lowrisc::flash_ctrl::LowRiscPage::default()
//!    );
//!
//!    let mux_flash = components::tickv::FlashMuxComponent::new(&peripherals.flash_ctrl).finalize(
//!        components::flash_user_component_static!(lowrisc::flash_ctrl::FlashCtrl),
//!    );
//!
//!    let kvstore = components::tickv::TicKVComponent::new(
//!        &mux_flash,
//!        0x20040000 / lowrisc::flash_ctrl::PAGE_SIZE,
//!        0x40000,
//!        flash_ctrl_read_buf,
//!        page_buffer,
//!    )
//!    .finalize(components::tickv_component_static!(
//!        lowrisc::flash_ctrl::FlashCtrl
//!    ));
//!    hil::flash::HasClient::set_client(&peripherals.flash_ctrl, mux_flash);
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_flash::FlashUser;
use core_capsules::virtual_flash::MuxFlash;
use extra_capsules::tickv::TicKVStore;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::flash::HasClient;
use kernel::hil::hasher::Hasher;

// Setup static space for the objects.
#[macro_export]
macro_rules! tickv_component_static {
    ($F:ty, $H:ty) => {{
        let flash = kernel::static_buf!(core_capsules::virtual_flash::FlashUser<'static, $F>);
        let tickv = kernel::static_buf!(
            extra_capsules::tickv::TicKVStore<
                'static,
                core_capsules::virtual_flash::FlashUser<'static, $F>,
                $H,
            >
        );

        (flash, tickv)
    };};
}

pub struct TicKVComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
    H: 'static + Hasher<'static, 8>,
> {
    mux_flash: &'static MuxFlash<'static, F>,
    hasher: &'static H,
    region_offset: usize,
    flash_size: usize,
    tickfs_read_buf: &'static mut [u8; 2048],
    flash_read_buffer: &'static mut F::Page,
}

impl<
        F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
        H: Hasher<'static, 8>,
    > TicKVComponent<F, H>
{
    pub fn new(
        hasher: &'static H,
        mux_flash: &'static MuxFlash<'static, F>,
        region_offset: usize,
        flash_size: usize,
        tickfs_read_buf: &'static mut [u8; 2048],
        flash_read_buffer: &'static mut F::Page,
    ) -> Self {
        Self {
            hasher,
            mux_flash,
            region_offset,
            flash_size,
            tickfs_read_buf,
            flash_read_buffer,
        }
    }
}

impl<
        F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
        H: 'static + Hasher<'static, 8>,
    > Component for TicKVComponent<F, H>
{
    type StaticInput = (
        &'static mut MaybeUninit<FlashUser<'static, F>>,
        &'static mut MaybeUninit<TicKVStore<'static, FlashUser<'static, F>, H>>,
    );
    type Output = &'static TicKVStore<'static, FlashUser<'static, F>, H>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let _grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_flash = static_buffer.0.write(FlashUser::new(self.mux_flash));

        let driver = static_buffer.1.write(TicKVStore::new(
            virtual_flash,
            self.hasher,
            self.tickfs_read_buf,
            self.flash_read_buffer,
            self.region_offset,
            self.flash_size,
        ));
        virtual_flash.set_client(driver);
        driver.initialise();
        driver
    }
}
