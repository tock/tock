//! Component for Flash
//!
//! Provides `FlashMux` and `FlashUser` (virtual flash)
//! Usage
//! -----
//! ```rust
//!    let mux_flash = components::flash::FlashMuxComponent::new(&base_peripherals.nvmc).finalize(
//!       components::flash_mux_component_helper!(nrf52833::nvmc::Nvmc),
//!    );
//!
//!    let virtual_app_flash = components::flash::FlashUserComponent::new(mux_flash).finalize(
//!       components::flash_user_component_helper!(nrf52833::nvmc::Nvmc),
//!    );
//! ```

use capsules::virtual_flash::FlashUser;
use capsules::virtual_flash::MuxFlash;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::flash::{Flash, HasClient};
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! flash_user_component_helper {
    ($F:ty) => {{
        use capsules::virtual_flash::FlashUser;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<FlashUser<'static, $F>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

#[macro_export]
macro_rules! flash_mux_component_helper {
    ($F:ty) => {{
        use capsules::virtual_flash::MuxFlash;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxFlash<'static, $F>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct FlashMuxComponent<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> {
    flash: &'static F,
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> FlashMuxComponent<F> {
    pub fn new(flash: &'static F) -> FlashMuxComponent<F> {
        FlashMuxComponent { flash }
    }
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> Component
    for FlashMuxComponent<F>
{
    type StaticInput = &'static mut MaybeUninit<MuxFlash<'static, F>>;
    type Output = &'static MuxFlash<'static, F>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_flash = static_init_half!(s, MuxFlash<'static, F>, MuxFlash::new(self.flash));
        HasClient::set_client(self.flash, mux_flash);

        mux_flash
    }
}

pub struct FlashUserComponent<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> {
    mux_flash: &'static MuxFlash<'static, F>,
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> FlashUserComponent<F> {
    pub fn new(mux_flash: &'static MuxFlash<'static, F>) -> Self {
        Self { mux_flash }
    }
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> Component
    for FlashUserComponent<F>
{
    type StaticInput = &'static mut MaybeUninit<FlashUser<'static, F>>;
    type Output = &'static FlashUser<'static, F>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let virtual_flash =
            static_init_half!(s, FlashUser<'static, F>, FlashUser::new(self.mux_flash));

        virtual_flash
    }
}
