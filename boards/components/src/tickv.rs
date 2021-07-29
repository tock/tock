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
//!        components::flash_user_component_helper!(lowrisc::flash_ctrl::FlashCtrl),
//!    );
//!
//!    let kvstore = components::tickv::TicKVComponent::new(
//!        &mux_flash,
//!        0x20040000 / lowrisc::flash_ctrl::PAGE_SIZE,
//!        0x40000,
//!        flash_ctrl_read_buf,
//!        page_buffer,
//!    )
//!    .finalize(components::tickv_component_helper!(
//!        lowrisc::flash_ctrl::FlashCtrl
//!    ));
//!    hil::flash::HasClient::set_client(&peripherals.flash_ctrl, mux_flash);
//! ```

use capsules::tickv::TicKVStore;
use capsules::virtual_flash::FlashUser;
use capsules::virtual_flash::MuxFlash;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::flash::HasClient;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! tickv_component_helper {
    ($F:ty) => {{
        use capsules::tickv::TicKVStore;
        use capsules::virtual_flash::FlashUser;
        use core::mem::MaybeUninit;
        use kernel::hil;
        static mut BUF1: MaybeUninit<FlashUser<'static, $F>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<TicKVStore<'static, FlashUser<'static, $F>>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct TicKVComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
> {
    mux_flash: &'static MuxFlash<'static, F>,
    region_offset: usize,
    flash_size: usize,
    tickfs_read_buf: &'static mut [u8; 64],
    flash_read_buffer: &'static mut F::Page,
}

impl<F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>>
    TicKVComponent<F>
{
    pub fn new(
        mux_flash: &'static MuxFlash<'static, F>,
        region_offset: usize,
        flash_size: usize,
        tickfs_read_buf: &'static mut [u8; 64],
        flash_read_buffer: &'static mut F::Page,
    ) -> Self {
        Self {
            mux_flash,
            region_offset,
            flash_size,
            tickfs_read_buf,
            flash_read_buffer,
        }
    }
}

impl<F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>>
    Component for TicKVComponent<F>
{
    type StaticInput = (
        &'static mut MaybeUninit<FlashUser<'static, F>>,
        &'static mut MaybeUninit<TicKVStore<'static, FlashUser<'static, F>>>,
    );
    type Output = &'static TicKVStore<'static, FlashUser<'static, F>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let _grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_flash = static_init_half!(
            static_buffer.0,
            FlashUser<'static, F>,
            FlashUser::new(self.mux_flash)
        );

        let driver = static_init_half!(
            static_buffer.1,
            TicKVStore<'static, FlashUser<'static, F>>,
            TicKVStore::new(
                virtual_flash,
                self.tickfs_read_buf,
                self.flash_read_buffer,
                self.region_offset,
                self.flash_size,
            )
        );
        virtual_flash.set_client(driver);
        driver.initalise();
        driver
    }
}
