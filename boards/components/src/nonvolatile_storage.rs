//! Component for non-volatile storage Drivers.
//!
//! This provides one component, NonvolatileStorageComponent, which provides
//! a system call inteface to non-volatile storage.
//!
//! Usage
//! -----
//! ```rust
//! let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
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

use capsules::nonvolatile_storage_driver::NonvolatileStorage;
use capsules::nonvolatile_to_pages::NonvolatileToPages;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! nv_storage_component_helper {
    ($F:ty $(,)?) => {{
        use capsules::nonvolatile_to_pages::NonvolatileToPages;
        use core::mem::MaybeUninit;
        use kernel::hil;
        static mut BUF1: MaybeUninit<<$F as hil::flash::Flash>::Page> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<NonvolatileToPages<'static, $F>> = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct NonvolatileStorageComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    flash: &'static F,
    userspace_start: usize,
    userspace_length: usize,
    kernel_start: usize,
    kernel_length: usize,
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    > NonvolatileStorageComponent<F>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        flash: &'static F,
        userspace_start: usize,
        userspace_length: usize,
        kernel_start: usize,
        kernel_length: usize,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            flash,
            userspace_start,
            userspace_length,
            kernel_start,
            kernel_length,
        }
    }
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    > Component for NonvolatileStorageComponent<F>
{
    type StaticInput = (
        &'static mut MaybeUninit<<F as hil::flash::Flash>::Page>,
        &'static mut MaybeUninit<NonvolatileToPages<'static, F>>,
    );
    type Output = &'static NonvolatileStorage<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let flash_pagebuffer = static_init_half!(
            static_buffer.0,
            <F as hil::flash::Flash>::Page,
            <F as hil::flash::Flash>::Page::default()
        );

        let nv_to_page = static_init_half!(
            static_buffer.1,
            NonvolatileToPages<'static, F>,
            NonvolatileToPages::new(self.flash, flash_pagebuffer)
        );
        hil::flash::HasClient::set_client(self.flash, nv_to_page);

        let nonvolatile_storage = static_init!(
            NonvolatileStorage<'static>,
            NonvolatileStorage::new(
                nv_to_page,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                self.userspace_start, // Start address for userspace accessible region
                self.userspace_length, // Length of userspace accessible region
                self.kernel_start,    // Start address of kernel region
                self.kernel_length,   // Length of kernel region
                &mut capsules::nonvolatile_storage_driver::BUFFER
            )
        );
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        nonvolatile_storage
    }
}
