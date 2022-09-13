//! Test the opentitan Flash Controller
//! Tests: read_page, write_page, erase_page
use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use core::cell::Cell;
use kernel::debug;
use kernel::hil;
#[allow(unused_imports)]
use kernel::hil::flash::Flash;
use kernel::hil::flash::HasClient;
use kernel::static_init;
use kernel::utilities::cells::TakeCell;

#[allow(dead_code)]
struct FlashCtlCallBack {
    read_pending: Cell<bool>,
    write_pending: Cell<bool>,
    // A lowrisc page to for reads/writes
    read_in_page: TakeCell<'static, lowrisc::flash_ctrl::LowRiscPage>,
    write_in_page: TakeCell<'static, lowrisc::flash_ctrl::LowRiscPage>,
    // We recover the callback returned buffer into these
    read_out_buf: TakeCell<'static, [u8]>,
    write_out_buf: TakeCell<'static, [u8]>,
}

impl<'a> FlashCtlCallBack {
    fn new(
        read_in_page: &'static mut lowrisc::flash_ctrl::LowRiscPage,
        write_in_page: &'static mut lowrisc::flash_ctrl::LowRiscPage,
    ) -> Self {
        FlashCtlCallBack {
            read_pending: Cell::new(false),
            write_pending: Cell::new(false),
            read_in_page: TakeCell::new(read_in_page),
            write_in_page: TakeCell::new(write_in_page),
            read_out_buf: TakeCell::empty(),
            write_out_buf: TakeCell::empty(),
        }
    }

    fn reset(&self) {
        self.read_pending.set(false);
        self.write_pending.set(false);
    }
}

impl<'a, F: hil::flash::Flash> hil::flash::Client<F> for FlashCtlCallBack {
    fn read_complete(&self, page: &'static mut F::Page, error: hil::flash::Error) {
        if self.read_pending.get() {
            assert_eq!(error, hil::flash::Error::CommandComplete);
            self.read_out_buf.replace(page.as_mut());
            self.read_pending.set(false);
        }
    }

    fn write_complete(&self, page: &'static mut F::Page, error: hil::flash::Error) {
        if self.write_pending.get() {
            assert_eq!(error, hil::flash::Error::CommandComplete);
            self.write_out_buf.replace(page.as_mut());
            self.write_pending.set(false);
        }
    }

    fn erase_complete(&self, error: hil::flash::Error) {
        // Caller may check by a successive page read to assert the erased
        // page is composed of 0xFF (all erased bits should be 1)
        assert_eq!(error, hil::flash::Error::CommandComplete);
    }
}

unsafe fn static_init_test() -> &'static FlashCtlCallBack {
    let r_in_page = static_init!(
        lowrisc::flash_ctrl::LowRiscPage,
        lowrisc::flash_ctrl::LowRiscPage::default()
    );
    let w_in_page = static_init!(
        lowrisc::flash_ctrl::LowRiscPage,
        lowrisc::flash_ctrl::LowRiscPage::default()
    );
    let mut val: u8 = 0;

    for i in 0..lowrisc::flash_ctrl::PAGE_SIZE {
        val = val.wrapping_add(10);
        r_in_page[i] = 0x00;
        w_in_page[i] = 0xAA; // Arbitrary Data
    }
    static_init!(
        FlashCtlCallBack,
        FlashCtlCallBack::new(r_in_page, w_in_page)
    )
}

/// Tests: Erase Page -> Write Page -> Read Page
///
/// Compare the data we wrote is stored in flash with a
/// successive read.
#[test_case]
fn flash_ctl_read_write_page() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let flash_ctl = &perf.flash_ctrl;

    let cb = unsafe { static_init_test() };
    flash_ctl.set_client(cb);
    cb.reset();

    debug!("[FLASH_CTRL] Test page read/write....");

    #[cfg(feature = "hardware_tests")]
    {
        let page_num: usize = 511;
        run_kernel_op(100);
        // Lets do a page erase
        assert!(flash_ctl.erase_page(page_num).is_ok());
        run_kernel_op(100);

        // Do Page Write
        let write_page = cb.write_in_page.take().unwrap();
        assert!(flash_ctl.write_page(page_num, write_page).is_ok());
        cb.write_pending.set(true);
        run_kernel_op(100);
        // OP Complete, buffer recovered.
        assert!(!cb.write_pending.get());
        cb.reset();

        // Read the same page
        let read_page = cb.read_in_page.take().unwrap();
        assert!(flash_ctl.read_page(page_num, read_page).is_ok());
        cb.read_pending.set(true);
        run_kernel_op(100);
        assert!(!cb.read_pending.get());
        cb.reset();

        // Compare r/w buffer
        let write_in = cb.write_out_buf.take().unwrap(); // Recovered buffer is saved here as &[u8]
        let read_out = cb.read_out_buf.take().unwrap();

        assert_eq!(write_in.len(), read_out.len());
        assert!(
            write_in.iter().zip(read_out.iter()).all(|(i, j)| i == j),
            "[ERR] Read data indicates flash write error on page {}",
            page_num
        );

        cb.write_out_buf.replace(write_in);
        cb.read_out_buf.replace(read_out);
    }

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}

/// Tests: Erase Page -> Write Page -> Erase Page -> Read Page
/// A page erased should set all bits to `1`s or all bytes in page to
/// `0xFF`. Assert this is true after writing data to a page and erasing
/// the page.
#[test_case]
fn flash_ctl_erase_page() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let flash_ctl = &perf.flash_ctrl;

    let cb = unsafe { static_init_test() };
    cb.reset();
    flash_ctl.set_client(cb);

    debug!("[FLASH_CTRL] Test page erase....");

    #[cfg(feature = "hardware_tests")]
    {
        let page_num: usize = 500;
        run_kernel_op(100);
        // Lets do a page erase
        assert!(flash_ctl.erase_page(page_num).is_ok());
        run_kernel_op(100);

        // Do Page Write
        let write_page = cb.write_in_page.take().unwrap();
        assert!(flash_ctl.write_page(page_num, write_page).is_ok());
        cb.write_pending.set(true);
        run_kernel_op(100);
        // OP Complete, buffer recovered.
        assert!(!cb.write_pending.get());
        cb.reset();

        // Erase again
        assert!(flash_ctl.erase_page(page_num).is_ok());
        run_kernel_op(100);

        // Read Page
        let read_page = cb.read_in_page.take().unwrap();
        assert!(flash_ctl.read_page(page_num, read_page).is_ok());
        cb.read_pending.set(true);
        run_kernel_op(100);
        assert!(!cb.read_pending.get());
        cb.reset();

        // Check that the erased paged is all `0xFF` bytes
        let read_out = cb.read_out_buf.take().unwrap();
        assert!(
            read_out.iter().all(|&a| a == 0xFF),
            "[ERR] Read data indicates erase failure on page {}",
            page_num
        );

        cb.read_out_buf.replace(read_out);
    }
    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}
