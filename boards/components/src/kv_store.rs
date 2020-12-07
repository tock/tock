//! Component for non-volatile storage Drivers.
//!
//! This provides one component, KVStoreComponent, which provides
//! a system call inteface to non-volatile storage.
//!
//! Usage
//! -----
//! ```rust
//! let nonvolatile_storage = components::nonvolatile_storage::KVStoreComponent::new(
//!     board_kernel,
//!     &sam4l::flashcalw::FLASH_CONTROLLER,
//!     0x60000,
//!     0x20000,
//!     &_sstorage as *const u8 as usize,
//!     &_estorage as *const u8 as usize,
//! )
//! .finalize(components::nv_storage_component_helper!(
//!     sam4l::flashcalw::FLASHCALW
//! ));
//! ```

use capsules::kv_store::KVStoreDriver;
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
macro_rules! flash_user_component_helper {
    ($F:ty) => {{
        use capsules::virtual_flash::MuxFlash;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxFlash<'static, $F>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct FlashMuxComponent<F: 'static + hil::flash::Flash> {
    flash: &'static F,
}

impl<F: 'static + hil::flash::Flash> FlashMuxComponent<F> {
    pub fn new(flash: &'static F) -> FlashMuxComponent<F> {
        FlashMuxComponent { flash }
    }
}

impl<F: 'static + hil::flash::Flash> Component for FlashMuxComponent<F> {
    type StaticInput = &'static mut MaybeUninit<MuxFlash<'static, F>>;
    type Output = &'static MuxFlash<'static, F>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_flash = static_init_half!(s, MuxFlash<'static, F>, MuxFlash::new(self.flash));

        mux_flash
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! kv_store_component_helper {
    ($F:ty) => {{
        use capsules::kv_store::KVStoreDriver;
        use capsules::virtual_flash::FlashUser;
        use core::mem::MaybeUninit;
        use kernel::hil;
        static mut BUF1: MaybeUninit<FlashUser<'static, $F>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<KVStoreDriver<'static, FlashUser<'static, $F>>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct KVStoreComponent<F: 'static + hil::flash::Flash> {
    board_kernel: &'static kernel::Kernel,
    mux_flash: &'static MuxFlash<'static, F>,
    region_offset: usize,
    tickfs_read_buf: &'static mut [u8; 512],
    flash_read_buffer: &'static mut F::Page,
    static_key_buf: &'static mut [u8; 64],
    static_value_buf: &'static mut [u8; 64],
}

impl<F: 'static + hil::flash::Flash> KVStoreComponent<F> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        mux_flash: &'static MuxFlash<'static, F>,
        region_offset: usize,
        tickfs_read_buf: &'static mut [u8; 512],
        flash_read_buffer: &'static mut F::Page,
        static_key_buf: &'static mut [u8; 64],
        static_value_buf: &'static mut [u8; 64],
    ) -> Self {
        Self {
            board_kernel,
            mux_flash,
            region_offset,
            tickfs_read_buf,
            flash_read_buffer,
            static_key_buf,
            static_value_buf,
        }
    }
}

impl<F: 'static + hil::flash::Flash> Component for KVStoreComponent<F> {
    type StaticInput = (
        &'static mut MaybeUninit<FlashUser<'static, F>>,
        &'static mut MaybeUninit<KVStoreDriver<'static, FlashUser<'static, F>>>,
    );
    type Output = &'static KVStoreDriver<'static, FlashUser<'static, F>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_flash = static_init_half!(
            static_buffer.0,
            FlashUser<'static, F>,
            FlashUser::new(self.mux_flash)
        );

        let driver = static_init_half!(
            static_buffer.1,
            KVStoreDriver<'static, FlashUser<'static, F>>,
            KVStoreDriver::new(
                virtual_flash,
                self.board_kernel.create_grant(&grant_cap),
                self.tickfs_read_buf,
                self.flash_read_buffer,
                self.region_offset,
                self.static_key_buf,
                self.static_value_buf,
            )
        );
        virtual_flash.set_client(driver);
        driver.initalise();
        driver
    }
}
