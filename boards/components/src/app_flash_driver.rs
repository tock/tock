//! Component for any App Flash Driver.
//!
//! Usage
//! -----
//! ```rust
//! let app_flash =
//!     components::app_flash_driver::AppFlashComponent::new(board_kernel, &base_peripherals.nvmc)
//!         .finalize(components::app_flash_component_helper!(
//!             nrf52833::nvmc::Nvmc,
//!             512
//!     ));
//! ```

use capsules::app_flash_driver::AppFlash;
use capsules::nonvolatile_to_pages::NonvolatileToPages;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::nonvolatile_storage::NonvolatileStorage;
use kernel::static_init_half;

#[macro_export]
macro_rules! app_flash_component_helper {
    ($F:ty, $buffer_size: literal) => {{
        static mut BUFFER: [u8; $buffer_size] = [0; $buffer_size];
        use capsules::app_flash_driver::AppFlash;
        use capsules::nonvolatile_to_pages::NonvolatileToPages;
        use core::mem::MaybeUninit;
        use kernel::hil;
        static mut page_buffer: MaybeUninit<<$F as hil::flash::Flash>::Page> =
            MaybeUninit::uninit();
        static mut nv_to_page: MaybeUninit<NonvolatileToPages<'static, $F>> = MaybeUninit::uninit();
        static mut app_flash: MaybeUninit<AppFlash<'static>> = MaybeUninit::uninit();
        (
            &mut BUFFER,
            &mut page_buffer,
            &mut nv_to_page,
            &mut app_flash,
        )
    };};
}

pub struct AppFlashComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    storage: &'static F,
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    > AppFlashComponent<F>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        storage: &'static F,
    ) -> AppFlashComponent<F> {
        AppFlashComponent {
            board_kernel,
            driver_num,
            storage,
        }
    }
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    > Component for AppFlashComponent<F>
{
    type StaticInput = (
        &'static mut [u8],
        &'static mut MaybeUninit<<F as hil::flash::Flash>::Page>,
        &'static mut MaybeUninit<NonvolatileToPages<'static, F>>,
        &'static mut MaybeUninit<AppFlash<'static>>,
    );
    type Output = &'static AppFlash<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let flash_pagebuffer = static_init_half!(
            static_buffer.1,
            <F as hil::flash::Flash>::Page,
            <F as hil::flash::Flash>::Page::default()
        );

        let nv_to_page = static_init_half!(
            static_buffer.2,
            NonvolatileToPages<'static, F>,
            NonvolatileToPages::new(self.storage, flash_pagebuffer)
        );
        self.storage.set_client(nv_to_page);

        let app_flash = static_init_half!(
            static_buffer.3,
            capsules::app_flash_driver::AppFlash<'static>,
            capsules::app_flash_driver::AppFlash::new(
                nv_to_page,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                static_buffer.0
            )
        );

        nv_to_page.set_client(app_flash);

        app_flash
    }
}
