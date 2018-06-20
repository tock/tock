use sam4l;
use capsules;
use capsules::nonvolatile_storage_driver::NonvolatileStorage;
use capsules::nonvolatile_to_pages::NonvolatileToPages;
use kernel;
use kernel::component::Component;
use kernel::hil;

pub struct NonvolatileStorageComponent;

impl NonvolatileStorageComponent {
    pub fn new() -> Self {
        NonvolatileStorageComponent {}
    }
}

impl Component for NonvolatileStorageComponent {
    type Output = &'static NonvolatileStorage<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        sam4l::flashcalw::FLASH_CONTROLLER.configure();
        pub static mut FLASH_PAGEBUFFER: sam4l::flashcalw::Sam4lPage =
            sam4l::flashcalw::Sam4lPage::new();
        let nv_to_page = static_init!(
            NonvolatileToPages<'static, sam4l::flashcalw::FLASHCALW>,
            NonvolatileToPages::new(
                &mut sam4l::flashcalw::FLASH_CONTROLLER,
                &mut FLASH_PAGEBUFFER
            )
        );
        hil::flash::HasClient::set_client(&sam4l::flashcalw::FLASH_CONTROLLER, nv_to_page);

        extern "C" {
            /// Beginning on the ROM region containing app images.
            static _sstorage: u8;
            static _estorage: u8;
        }

        // Kernel storage region, allocated with the storage_volume!
        // macro in common/utils.rs
        let kernel_start = &_sstorage as *const u8 as usize;
        let kernel_end = &_estorage as *const u8 as usize;
        let kernel_len = kernel_end - kernel_start;

        let nonvolatile_storage = static_init!(
            NonvolatileStorage<'static>,
            NonvolatileStorage::new(
                nv_to_page,
                kernel::Grant::create(),
                0x60000, // Start address for userspace accessible region
                0x20000, // Length of userspace accessible region
                kernel_start, // Start address of kernel region
                kernel_len, // Length of kernel region
                &mut capsules::nonvolatile_storage_driver::BUFFER
            )
        );
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        nonvolatile_storage
    }
}
