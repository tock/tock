//! Gives access to the writeable embedded flash regions of the application.
//!
//! The purpose of this capsule is to provide low-level control of the embedded flash to allow
//! applications to implement flash-efficient data-structures using their writeable flash regions.
//! The API is blocking since most flash either halt the CPU during write and erase operations or
//! ask the application to wait until the operation is finished. A blocking API is also simpler to
//! reason and less error-prone.
//!
//! # Syscalls
//!
//! - COMMAND(0): Check the driver.
//! - COMMAND(1, 0): Get the word size.
//! - COMMAND(1, 1): Get the page size.
//! - COMMAND(1, 2): Get the maximum number of word writes between page erasures.
//! - COMMAND(1, 3): Get the maximum number page erasures in the lifetime of the flash.
//! - COMMAND(2, ptr): Write the allow slice to the flash region starting at `ptr`.
//!   - `ptr` must be word-aligned.
//!   - The allow slice length must be word aligned.
//!   - The region starting at `ptr` of the same length as the allow slice must be in a writeable
//!     flash region.
//! - COMMAND(3, ptr): Erase a page.
//!   - `ptr` must be page-aligned.
//!   - The page starting at `ptr` must be in a writeable flash region.
//! - ALLOW(0): The allow slice for COMMAND(2).

use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

pub const DRIVER_NUM: usize = crate::driver::NUM::EmbeddedFlash as usize;

#[derive(Default)]
pub struct App {
    /// The allow slice for COMMAND(2).
    slice: Option<AppSlice<Shared, u8>>,
}

pub struct EmbeddedFlash {
    driver: &'static dyn hil::embedded_flash::EmbeddedFlash,
    apps: Grant<App>,
}

impl EmbeddedFlash {
    pub fn new(
        driver: &'static dyn hil::embedded_flash::EmbeddedFlash,
        apps: Grant<App>,
    ) -> EmbeddedFlash {
        EmbeddedFlash { driver, apps }
    }
}

impl Driver for EmbeddedFlash {
    fn subscribe(&self, _: usize, _: Option<Callback>, _: AppId) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn command(&self, cmd: usize, arg: usize, _: usize, appid: AppId) -> ReturnCode {
        match (cmd, arg) {
            (0, _) => ReturnCode::SUCCESS,

            (1, 0) => ReturnCode::SuccessWithValue {
                value: self.driver.word_size(),
            },
            (1, 1) => ReturnCode::SuccessWithValue {
                value: self.driver.page_size(),
            },
            (1, 2) => ReturnCode::SuccessWithValue {
                value: self.driver.max_word_writes(),
            },
            (1, 3) => ReturnCode::SuccessWithValue {
                value: self.driver.max_page_erases(),
            },
            (1, _) => ReturnCode::EINVAL,

            (2, ptr) => self
                .apps
                .enter(appid, |app, _| {
                    let slice = match app.slice.take() {
                        None => return ReturnCode::EINVAL,
                        Some(slice) => slice,
                    };
                    if !appid.in_writeable_flash_region(ptr, slice.len()) {
                        return ReturnCode::EINVAL;
                    }
                    self.driver.write_slice(ptr, slice.as_ref())
                })
                .unwrap_or_else(|err| err.into()),

            (3, ptr) => {
                if !appid.in_writeable_flash_region(ptr, self.driver.page_size()) {
                    return ReturnCode::EINVAL;
                }
                self.driver.erase_page(ptr)
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            0 => self
                .apps
                .enter(appid, |app, _| {
                    app.slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
