//! Component for any App Flash Driver.
//!
//! Usage
//! -----
//! ```rust
//! let app_flash =
//!     components::app_flash_driver::AppFlashComponent::new(board_kernel, &base_peripherals.nvmc)
//!         .finalize(components::app_flash_component_static!(
//!             nrf52833::nvmc::Nvmc,
//!             512
//!     ));
//! ```

use core::mem::MaybeUninit;
use extra_capsules::app_flash_driver::AppFlash;
use extra_capsules::nonvolatile_to_pages::NonvolatileToPages;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::nonvolatile_storage::NonvolatileStorage;

#[macro_export]
macro_rules! app_flash_component_static {
    ($F:ty, $buffer_size: literal) => {{
        let buffer = kernel::static_buf!([u8; $buffer_size]);
        let page_buffer = kernel::static_buf!(<$F as kernel::hil::flash::Flash>::Page);
        let nv_to_page = kernel::static_buf!(
            extra_capsules::nonvolatile_to_pages::NonvolatileToPages<'static, $F>
        );
        let app_flash = kernel::static_buf!(extra_capsules::app_flash_driver::AppFlash<'static>);
        (buffer, page_buffer, nv_to_page, app_flash)
    };};
}

pub struct AppFlashComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    const BUF_LEN: usize,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    storage: &'static F,
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
        const BUF_LEN: usize,
    > AppFlashComponent<F, BUF_LEN>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        storage: &'static F,
    ) -> AppFlashComponent<F, BUF_LEN> {
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
        const BUF_LEN: usize,
    > Component for AppFlashComponent<F, BUF_LEN>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; BUF_LEN]>,
        &'static mut MaybeUninit<<F as hil::flash::Flash>::Page>,
        &'static mut MaybeUninit<NonvolatileToPages<'static, F>>,
        &'static mut MaybeUninit<AppFlash<'static>>,
    );
    type Output = &'static AppFlash<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let buffer = static_buffer.0.write([0; BUF_LEN]);

        let flash_pagebuffer = static_buffer
            .1
            .write(<F as hil::flash::Flash>::Page::default());

        let nv_to_page = static_buffer
            .2
            .write(NonvolatileToPages::new(self.storage, flash_pagebuffer));
        self.storage.set_client(nv_to_page);

        let app_flash = static_buffer
            .3
            .write(extra_capsules::app_flash_driver::AppFlash::new(
                nv_to_page,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                buffer,
            ));

        nv_to_page.set_client(app_flash);

        app_flash
    }
}
