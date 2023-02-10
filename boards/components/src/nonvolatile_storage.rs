//! Component for non-volatile storage Drivers.
//!
//! This provides one component, NonvolatileStorageComponent, which provides
//! a system call interface to non-volatile storage.
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
//! .finalize(components::nonvolatile_storage_component_static!(
//!     sam4l::flashcalw::FLASHCALW
//! ));
//! ```

use core::mem::MaybeUninit;
use extra_capsules::nonvolatile_storage_driver::NonvolatileStorage;
use extra_capsules::nonvolatile_to_pages::NonvolatileToPages;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! nonvolatile_storage_component_static {
    ($F:ty $(,)?) => {{
        let page = kernel::static_buf!(<$F as kernel::hil::flash::Flash>::Page);
        let ntp = kernel::static_buf!(
            extra_capsules::nonvolatile_to_pages::NonvolatileToPages<'static, $F>
        );
        let ns = kernel::static_buf!(
            extra_capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>
        );
        let buffer = kernel::static_buf!([u8; extra_capsules::nonvolatile_storage_driver::BUF_LEN]);

        (page, ntp, ns, buffer)
    };};
}

pub struct NonvolatileStorageComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
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
        driver_num: usize,
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
        &'static mut MaybeUninit<NonvolatileStorage<'static>>,
        &'static mut MaybeUninit<[u8; extra_capsules::nonvolatile_storage_driver::BUF_LEN]>,
    );
    type Output = &'static NonvolatileStorage<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let buffer = static_buffer
            .3
            .write([0; extra_capsules::nonvolatile_storage_driver::BUF_LEN]);

        let flash_pagebuffer = static_buffer
            .0
            .write(<F as hil::flash::Flash>::Page::default());

        let nv_to_page = static_buffer
            .1
            .write(NonvolatileToPages::new(self.flash, flash_pagebuffer));
        hil::flash::HasClient::set_client(self.flash, nv_to_page);

        let nonvolatile_storage = static_buffer.2.write(NonvolatileStorage::new(
            nv_to_page,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.userspace_start, // Start address for userspace accessible region
            self.userspace_length, // Length of userspace accessible region
            self.kernel_start,    // Start address of kernel region
            self.kernel_length,   // Length of kernel region
            buffer,
        ));
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        nonvolatile_storage
    }
}
