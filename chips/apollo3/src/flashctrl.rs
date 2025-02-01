// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Flash Controller

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

pub const PAGE_SIZE: usize = 8 * 1024;
pub const FLASH_INSTANCE_SIZE: usize = 512 * 1024;
pub const FLASH_NUM_INSTANCES: usize = 2;
pub const FLASH_PAGES_PER_INSTANCE: usize = FLASH_INSTANCE_SIZE / PAGE_SIZE;
pub const FLASH_MAX_PAGES: usize = FLASH_NUM_INSTANCES * FLASH_PAGES_PER_INSTANCE;

const FLASH_PROGRAM_KEY: u32 = 0x12344321;

/// There are two flash instances, each is 512KiB.
#[derive(PartialEq)]
enum FlashInstance {
    MAIN0 = 0,
    MAIN1 = 1,
}

pub struct Apollo3Page(pub [u8; PAGE_SIZE]);

impl Default for Apollo3Page {
    fn default() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Index<usize> for Apollo3Page {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Apollo3Page {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for Apollo3Page {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// This function can be used to read / write arbitrary flash memory, and thus
/// arbitrary program code. As such, they are unsafe operations. We also can't
/// confirm that the functions are safe in the context of Rust.
///
/// The below documentation is based on the official HAL documentation.
///
/// This function will call the chip ROM code to perform the below operation.
///
/// Use this function to safely read a value from peripheral or memory locations.
///
/// `addr` - The location to be read.
///
/// @return The value read from the given address.
unsafe fn flash_util_read_word(addr: *mut u32) -> u32 {
    // Call `uint32_t flash_util_read_word(uint32_t *)` in the ROM code.
    let flash_util_read_word: unsafe extern "C" fn(*mut u32) -> u32 =
        unsafe { core::mem::transmute(0x08000075 as *const ()) };

    flash_util_read_word(addr)
}

/// This function can be used to read / write arbitrary flash memory, and thus
/// arbitrary program code. As such, they are unsafe operations. We also can't
/// confirm that the functions are safe in the context of Rust.
///
/// The below documentation is based on the official HAL documentation.
///
/// This function will call the chip ROM code to perform the below operation.
///
/// This function will program multiple words in main flash.
///
/// `program_key` - The programming key, AM_HAL_FLASH_PROGRAM_KEY.
/// `src_addr` - Pointer to word aligned array of data to program into the flash instance.
/// `dst_addr` - Pointer to the word aligned flash location where
/// programming of the flash instance is to begin.
/// `num_words` - The number of words to be programmed.
///
/// @return 0 for success, non-zero for failure.
///     Failing return code indicates:
///     1   ui32ProgramKey is invalid.
///     2   pui32Dst is invalid.
///     3   Flash addressing range would be exceeded.  That is, (pui32Dst +
///         (ui32NumWords * 4)) is greater than the last valid address.
///     4   pui32Src is invalid.
///     5   Unused - will never be returned.
///     6   Flash controller hardware timeout.
unsafe fn flash_program_main(
    program_key: u32,
    src_addr: *mut u32,
    dst_addr: *mut u32,
    num_words: u32,
) -> i32 {
    use core::ffi::c_int;

    // Call `int flash_program_main(uint32_t, uint32_t *, uint32_t *, uint32_t)` in the ROM code.
    let flash_program_main: unsafe extern "C" fn(u32, *mut u32, *mut u32, u32) -> c_int =
        unsafe { core::mem::transmute(0x08000055 as *const ()) };

    flash_program_main(program_key, src_addr, dst_addr, num_words)
}

/// This function can be used to read / write arbitrary flash memory, and thus
/// arbitrary program code. As such, they are unsafe operations. We also can't
/// confirm that the functions are safe in the context of Rust.
///
/// The below documentation is based on the official HAL documentation.
///
/// This function will call the chip ROM code to perform the below operation.
///
/// This function will erase the desired flash page in the desired instance of
/// flash.
///
/// `program_key` - The flash program key.
/// `flash_instance` - The flash instance to reference the page number with.
/// `page_num` - The flash page relative to the specified instance.
///
/// @return 0 for success, non-zero for failure.
///     Failing return code indicates:
///     1   ui32ProgramKey is invalid.
///     2   ui32FlashInst is invalid.
///     3   ui32PageNum is invalid.
///     4   Flash controller hardware timeout.
unsafe fn flash_page_erase(program_key: u32, flash_instance: u32, page_num: u32) -> i32 {
    use core::ffi::c_int;

    // Call `int flash_page_erase(uint32_t, uint32_t, uint32_t)` in the ROM code.
    let flash_page_erase: unsafe extern "C" fn(u32, u32, u32) -> c_int =
        unsafe { core::mem::transmute(0x08000051 as *const ()) };

    flash_page_erase(program_key, flash_instance, page_num)
}

#[derive(Copy, Clone, PartialEq)]
enum Operation {
    None,
    Read,
    Write,
    Erase,
}

pub struct FlashCtrl<'a> {
    flash_client: OptionalCell<&'a dyn hil::flash::Client<FlashCtrl<'a>>>,
    read_buf: TakeCell<'static, Apollo3Page>,
    write_buf: TakeCell<'static, Apollo3Page>,

    deferred_call: DeferredCall,
    op: Cell<Operation>,
}

impl FlashCtrl<'_> {
    pub fn new() -> Self {
        FlashCtrl {
            flash_client: OptionalCell::empty(),
            read_buf: TakeCell::empty(),
            write_buf: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            op: Cell::new(Operation::None),
        }
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for FlashCtrl<'_> {
    fn set_client(&self, client: &'static C) {
        self.flash_client.set(client);
    }
}

impl hil::flash::Flash for FlashCtrl<'_> {
    type Page = Apollo3Page;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if page_number >= FLASH_MAX_PAGES {
            return Err((ErrorCode::INVAL, buf));
        }

        if self.op.get() != Operation::None {
            return Err((ErrorCode::BUSY, buf));
        }

        if self.deferred_call.is_pending() {
            return Err((ErrorCode::BUSY, buf));
        }

        let addr = (page_number * PAGE_SIZE) as u32;
        let addr_ptr = addr as *mut u32;

        for i in 0..(PAGE_SIZE / 4) {
            let val = unsafe { flash_util_read_word(addr_ptr.wrapping_add(i)).to_le_bytes() };
            let offset = i * 4;

            buf[offset] = val[0];
            buf[offset + 1] = val[1];
            buf[offset + 2] = val[2];
            buf[offset + 3] = val[3];
        }

        self.read_buf.replace(buf);
        self.op.set(Operation::Read);
        self.deferred_call.set();
        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if page_number >= FLASH_MAX_PAGES {
            return Err((ErrorCode::INVAL, buf));
        }

        if self.op.get() != Operation::None {
            return Err((ErrorCode::BUSY, buf));
        }

        if self.deferred_call.is_pending() {
            return Err((ErrorCode::BUSY, buf));
        }

        let addr = (page_number * PAGE_SIZE) as u32;
        let addr_ptr = addr as *mut u32;

        let source_ptr = buf.0.as_mut_ptr() as *mut u32;

        let ret = unsafe {
            flash_program_main(
                FLASH_PROGRAM_KEY,
                source_ptr,
                addr_ptr,
                PAGE_SIZE as u32 / 4,
            )
        };

        match ret {
            0 => {
                self.write_buf.replace(buf);
                self.op.set(Operation::Write);
                self.deferred_call.set();
                Ok(())
            }
            1 => {
                // ProgramKey is invalid
                Err((ErrorCode::NOSUPPORT, buf))
            }
            2 => {
                // Dst is invalid
                Err((ErrorCode::INVAL, buf))
            }
            3 => {
                // Flash addressing range would be exceeded
                Err((ErrorCode::INVAL, buf))
            }
            4 => {
                // Src is invalid.
                Err((ErrorCode::INVAL, buf))
            }
            6 => {
                // Flash controller hardware timeout.
                Err((ErrorCode::BUSY, buf))
            }
            _ => Err((ErrorCode::FAIL, buf)),
        }
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        if page_number >= FLASH_MAX_PAGES {
            return Err(ErrorCode::INVAL);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        if self.deferred_call.is_pending() {
            return Err(ErrorCode::BUSY);
        }

        let ret = if page_number <= FLASH_PAGES_PER_INSTANCE {
            unsafe {
                flash_page_erase(
                    FLASH_PROGRAM_KEY,
                    FlashInstance::MAIN0 as u32,
                    page_number as u32,
                )
            }
        } else {
            unsafe {
                flash_page_erase(
                    FLASH_PROGRAM_KEY,
                    FlashInstance::MAIN1 as u32,
                    (page_number - FLASH_PAGES_PER_INSTANCE) as u32,
                )
            }
        };

        match ret {
            0 => {
                self.op.set(Operation::Erase);
                self.deferred_call.set();
                Ok(())
            }
            1 => {
                // ProgramKey is invalid
                Err(ErrorCode::NOSUPPORT)
            }
            2 => {
                // FlashInst is invalid.
                Err(ErrorCode::NOSUPPORT)
            }
            3 => {
                // PageNum is invalid.
                Err(ErrorCode::INVAL)
            }
            4 => {
                // Flash controller hardware timeout.
                Err(ErrorCode::BUSY)
            }
            _ => Err(ErrorCode::FAIL),
        }
    }
}

impl DeferredCallClient for FlashCtrl<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        let prev_op = self.op.get();

        self.op.set(Operation::None);

        self.flash_client.map(|client| match prev_op {
            Operation::None => unreachable!(),
            Operation::Read => client.read_complete(self.read_buf.take().unwrap(), Ok(())),
            Operation::Write => client.write_complete(self.write_buf.take().unwrap(), Ok(())),
            Operation::Erase => client.erase_complete(Ok(())),
        });
    }
}
