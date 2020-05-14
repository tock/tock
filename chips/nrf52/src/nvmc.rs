//! Non-Volatile Memory Controller
//!
//! Used in order read and write to internal flash.

use core::cell::Cell;
use core::convert::TryFrom;
use core::ops::{Index, IndexMut};
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::deferred_call::DeferredCall;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

use crate::deferred_call_tasks::DeferredCallTask;

const NVMC_BASE: StaticRef<NvmcRegisters> =
    unsafe { StaticRef::new(0x4001E400 as *const NvmcRegisters) };

#[repr(C)]
struct NvmcRegisters {
    /// Ready flag
    /// Address 0x400 - 0x404
    pub ready: ReadOnly<u32, Ready::Register>,
    /// Reserved
    _reserved1: [u32; 64],
    /// Configuration register
    /// Address: 0x504 - 0x508
    pub config: ReadWrite<u32, Configuration::Register>,
    /// Register for erasing a page in Code area
    /// Address: 0x508 - 0x50C
    pub erasepage: ReadWrite<u32, ErasePage::Register>,
    /// Register for erasing all non-volatile user memory
    /// Address: 0x50C - 0x510
    pub eraseall: ReadWrite<u32, EraseAll::Register>,
    _reserved2: u32,
    /// Register for erasing User Information Configuration Registers
    /// Address: 0x514 - 0x518
    pub eraseuicr: ReadWrite<u32, EraseUicr::Register>,
    /// Reserved
    _reserved3: [u32; 10],
    /// Configuration register
    /// Address: 0x540 - 0x544
    pub icachecnf: ReadWrite<u32, CacheConfiguration::Register>,
    /// Reserved
    _reserved4: u32,
    /// Configuration register
    /// Address: 0x548 - 0x54c
    pub ihit: ReadWrite<u32, CacheHit::Register>,
    /// Configuration register
    /// Address: 0x54C - 0x550
    pub imiss: ReadWrite<u32, CacheMiss::Register>,
}

register_bitfields! [u32,
    /// Ready flag
    Ready [
        /// NVMC is ready or busy
        READY OFFSET(0) NUMBITS(1) [
            /// NVMC is busy (on-going write or erase operation)
            BUSY = 0,
            /// NVMC is ready
            READY = 1
        ]
    ],
    /// Configuration register
    Configuration [
        /// Program memory access mode. It is strongly recommended
        /// to only activate erase and write modes when they are actively
        /// used. Enabling write or erase will invalidate the cache and keep
        /// it invalidated.
        WEN OFFSET(0) NUMBITS(2) [
            /// Read only access
            Ren = 0,
            /// Write Enabled
            Wen = 1,
            /// Erase enabled
            Een = 2
        ]
    ],
    /// Register for erasing a page in Code area
    ErasePage [
        /// Register for starting erase of a page in Code area
        ERASEPAGE OFFSET(0) NUMBITS(32) []
    ],
    /// Register for erasing all non-volatile user memory
    EraseAll [
        /// Erase all non-volatile memory including UICR registers. Note
        /// that code erase has to be enabled by CONFIG.EEN before the
        /// UICR can be erased
        ERASEALL OFFSET(0) NUMBITS(1) [
            /// No operation
            NOOPERATION = 0,
            /// Start chip erase
            ERASE = 1
        ]
    ],
    /// Register for erasing User Information Configuration Registers
    EraseUicr [
        /// Register starting erase of all User Information Configuration Registers.
        /// Note that code erase has to be enabled by CONFIG.EEN before the UICR can be erased
        ERASEUICR OFFSET(0) NUMBITS(1) [
            /// No operation
            NOOPERATION = 0,
            /// Start erase of UICR
            ERASE = 1
        ]
    ],
    /// I-Code cache configuration register
    CacheConfiguration [
        /// Cache enabled
        CACHEEN OFFSET(0) NUMBITS(1) [
            /// Disable cache. Invalidates all cache entries
            DISABLED = 0,
            /// Enable cache
            ENABLED = 1
        ],
        /// Cache profiling enable
        CACHEPROFEN OFFSET(8) NUMBITS(1) [
            /// Disable cache profiling
            DISABLED = 0,
            /// Enable cache profiling
            ENABLED = 1
        ]
    ],
    /// I-Code cache hit counter
    CacheHit [
        /// Number of cache hits
        HITS OFFSET(0) NUMBITS(32) []
    ],
    /// I-Code cache miss counter
    CacheMiss [
        /// Number of cache misses
        MISSES OFFSET(0) NUMBITS(32) []
    ]
];

/// This mechanism allows us to schedule "interrupts" even if the hardware
/// does not support them.
static DEFERRED_CALL: DeferredCall<DeferredCallTask> =
    unsafe { DeferredCall::new(DeferredCallTask::Nvmc) };

type WORD = u32;
const WORD_SIZE: usize = core::mem::size_of::<WORD>();
const PAGE_SIZE: usize = 4096;
const MAX_WORD_WRITES: usize = 2;
const MAX_PAGE_ERASES: usize = 10000;
const WORD_MASK: usize = WORD_SIZE - 1;
const PAGE_MASK: usize = PAGE_SIZE - 1;

/// This is a wrapper around a u8 array that is sized to a single page for the
/// nrf. Users of this module must pass an object of this type to use the
/// `hil::flash::Flash` interface.
///
/// An example looks like:
///
/// ```rust
/// # extern crate nrf52;
/// # use nrf52::nvmc::NrfPage;
///
/// static mut PAGEBUFFER: NrfPage = NrfPage::new();
/// ```
pub struct NrfPage(pub [u8; PAGE_SIZE as usize]);

impl NrfPage {
    pub const fn new() -> NrfPage {
        NrfPage([0; PAGE_SIZE as usize])
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for NrfPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for NrfPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for NrfPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// FlashState is used to track the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    Ready, // Flash is ready to complete a command.
    Read,  // Performing a read operation.
    Write, // Performing a write operation.
    Erase, // Performing an erase operation.
}

pub static mut NVMC: Nvmc = Nvmc::new();

pub struct Nvmc {
    registers: StaticRef<NvmcRegisters>,
    client: OptionalCell<&'static dyn hil::flash::Client<Nvmc>>,
    buffer: TakeCell<'static, NrfPage>,
    state: Cell<FlashState>,
}

impl Nvmc {
    pub const fn new() -> Nvmc {
        Nvmc {
            registers: NVMC_BASE,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            state: Cell::new(FlashState::Ready),
        }
    }

    pub fn configure_readonly(&self) {
        let regs = &*self.registers;
        regs.config.write(Configuration::WEN::Ren);
    }

    /// Configure the NVMC to allow writes to flash.
    pub fn configure_writeable(&self) {
        let regs = &*self.registers;
        regs.config.write(Configuration::WEN::Wen);
    }

    pub fn configure_eraseable(&self) {
        let regs = &*self.registers;
        regs.config.write(Configuration::WEN::Een);
    }

    pub fn erase_uicr(&self) {
        let regs = &*self.registers;
        regs.config.write(Configuration::WEN::Een);
        while !self.is_ready() {}
        regs.erasepage.write(ErasePage::ERASEPAGE.val(0x10001000));
        while !self.is_ready() {}
    }

    /// Check if there is an ongoing operation with the NVMC peripheral.
    pub fn is_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.ready.is_set(Ready::READY)
    }

    pub fn handle_interrupt(&self) {
        let state = self.state.get();
        self.state.set(FlashState::Ready);

        match state {
            FlashState::Read => {
                self.client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, hil::flash::Error::CommandComplete);
                    });
                });
            }
            FlashState::Write => {
                self.client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.write_complete(buffer, hil::flash::Error::CommandComplete);
                    });
                });
            }
            FlashState::Erase => {
                self.client.map(|client| {
                    client.erase_complete(hil::flash::Error::CommandComplete);
                });
            }
            _ => {}
        }
    }

    fn erase_page_helper(&self, page_number: usize) {
        let regs = &*self.registers;

        // Put the NVMC in erase mode.
        regs.config.write(Configuration::WEN::Een);

        // Tell the NVMC to erase the correct page by passing in the correct
        // address.
        regs.erasepage
            .write(ErasePage::ERASEPAGE.val((page_number * PAGE_SIZE) as u32));

        // Make sure that the NVMC is done. The CPU should be blocked while the
        // erase is happening, but it doesn't hurt to check too.
        while !regs.ready.is_set(Ready::READY) {}
    }

    fn read_range(
        &self,
        page_number: usize,
        buffer: &'static mut NrfPage,
    ) -> Result<(), (ReturnCode, &'static mut NrfPage)> {
        // Actually do a copy from flash into the buffer.
        let mut byte: *const u8 = (page_number * PAGE_SIZE) as *const u8;
        unsafe {
            for i in 0..buffer.len() {
                buffer[i] = *byte;
                byte = byte.offset(1);
            }
        }

        // Hold on to the buffer for the callback.
        self.buffer.replace(buffer);

        // Mark the need for an interrupt so we can call the read done
        // callback.
        self.state.set(FlashState::Read);
        DEFERRED_CALL.set();

        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        data: &'static mut NrfPage,
    ) -> Result<(), (ReturnCode, &'static mut NrfPage)> {
        let regs = &*self.registers;

        // Need to erase the page first.
        self.erase_page_helper(page_number);

        // Put the NVMC in write mode.
        regs.config.write(Configuration::WEN::Wen);

        for i in (0..data.len()).step_by(WORD_SIZE) {
            let word: u32 = (data[i + 0] as u32) << 0
                | (data[i + 1] as u32) << 8
                | (data[i + 2] as u32) << 16
                | (data[i + 3] as u32) << 24;

            let address = ((page_number * PAGE_SIZE) + i) as u32;
            let location = unsafe { &*(address as *const VolatileCell<u32>) };
            location.set(word);
        }

        // Make sure that the NVMC is done. The CPU should be blocked while the
        // write is happening, but it doesn't hurt to check too.
        while !regs.ready.is_set(Ready::READY) {}

        // Save the buffer so we can return it with the callback.
        self.buffer.replace(data);

        // Mark the need for an interrupt so we can call the write done
        // callback.
        self.state.set(FlashState::Write);
        DEFERRED_CALL.set();

        Ok(())
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        // Do the basic erase.
        self.erase_page_helper(page_number);

        // Mark that we want to trigger a pseudo interrupt so that we can issue
        // the callback even though the NVMC is completely blocking.
        self.state.set(FlashState::Erase);
        DEFERRED_CALL.set();

        ReturnCode::SUCCESS
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for Nvmc {
    fn set_client(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl hil::flash::Flash for Nvmc {
    type Page = NrfPage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.read_range(page_number, buf)
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.write_page(page_number, buf)
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.erase_page(page_number)
    }
}

/// Provides access to the writeable flash regions of the application.
///
/// The purpose of this driver is to provide low-level access to the embedded flash of nRF52 boards
/// to allow applications to implement flash-aware (like wear-leveling) data-structures. The driver
/// only permits applications to operate on their writeable flash regions. The API is blocking since
/// the CPU is halted during write and erase operations.
///
/// Supported boards:
/// - nRF52840 (tested)
/// - nRF52833
/// - nRF52811
/// - nRF52810
///
/// The maximum number of writes for the nRF52832 board is not per word but per block (512 bytes)
/// and as such doesn't exactly fit this API. However, it could be safely supported by returning
/// either 1 for the maximum number of word writes (i.e. the flash can only be written once before
/// being erased) or 8 for the word size (i.e. the write granularity is doubled). In both cases,
/// only 128 writes per block are permitted while the flash supports 181.
///
/// # Syscalls
///
/// - COMMAND(0): Check the driver.
/// - COMMAND(1, 0): Get the word size (always 4).
/// - COMMAND(1, 1): Get the page size (always 4096).
/// - COMMAND(1, 2): Get the maximum number of word writes between page erasures (always 2).
/// - COMMAND(1, 3): Get the maximum number page erasures in the lifetime of the flash (always
///     10000).
/// - COMMAND(2, ptr, len): Write the allow slice to the flash region starting at `ptr`.
///   - `ptr` must be word-aligned.
///   - `len` must be word-aligned.
///   - `len` must be equal to the allow slice length.
///   - The flash slice starting at `ptr` of length `len` must be writable.
/// - COMMAND(3, ptr, len): Erase a page.
///   - `ptr` must be page-aligned.
///   - `len` must be equal to the page size.
///   - The page starting at `ptr` must be erasable.
/// - ALLOW(0): The allow slice for COMMAND(2).
pub struct SyscallDriver {
    nvmc: &'static Nvmc,
    apps: Grant<App>,
}

pub const DRIVER_NUM: usize = 0x50003;

#[derive(Default)]
pub struct App {
    /// The allow slice for COMMAND(2).
    slice: Option<AppSlice<Shared, u8>>,
}

impl SyscallDriver {
    pub fn new(nvmc: &'static Nvmc, apps: Grant<App>) -> SyscallDriver {
        SyscallDriver { nvmc, apps }
    }
}

fn is_write_needed(old: u32, new: u32) -> bool {
    // No need to write if it would not modify the current value.
    old & new != old
}

impl SyscallDriver {
    /// Writes a word-aligned slice at a word-aligned address.
    ///
    /// Words are written only if necessary, i.e. if writing the new value would change the current
    /// value. This can be used to simplify recovery operations (e.g. if power is lost during a
    /// write operation). The application doesn't need to check which prefix has already been
    /// written and may repeat the complete write that was interrupted.
    ///
    /// # Safety
    ///
    /// The words in this range must have been written less than `MAX_WORD_WRITES` since their last
    /// page erasure.
    ///
    /// # Errors
    ///
    /// Fails with `EINVAL` if any of the following conditions does not hold:
    /// - `ptr` must be word-aligned.
    /// - `slice.len()` must be word-aligned.
    /// - The flash slice starting at `ptr` of length `slice.len()` must be writable.
    fn write_slice(&self, ptr: usize, slice: &[u8]) -> ReturnCode {
        if ptr & WORD_MASK != 0 || slice.len() & WORD_MASK != 0 {
            return ReturnCode::EINVAL;
        }
        self.nvmc.configure_writeable();
        for (i, chunk) in slice.chunks(WORD_SIZE).enumerate() {
            // `unwrap` cannot fail because `slice.len()` is word-aligned (see above).
            let val = WORD::from_ne_bytes(<[u8; WORD_SIZE]>::try_from(chunk).unwrap());
            let loc = unsafe { &*(ptr as *const VolatileCell<u32>).add(i) };
            if is_write_needed(loc.get(), val) {
                loc.set(val);
            }
        }
        while !self.nvmc.is_ready() {}
        self.nvmc.configure_readonly();
        ReturnCode::SUCCESS
    }

    /// Erases a page at a page-aligned address.
    ///
    /// # Errors
    ///
    /// Fails with `EINVAL` if any of the following conditions does not hold:
    /// - `ptr` must be page-aligned.
    /// - The page starting at `ptr` must be erasable.
    fn erase_page(&self, ptr: usize) -> ReturnCode {
        if ptr & PAGE_MASK != 0 {
            return ReturnCode::EINVAL;
        }
        self.nvmc.erase_page_helper(ptr / PAGE_SIZE);
        self.nvmc.configure_readonly();
        ReturnCode::SUCCESS
    }
}

impl Driver for SyscallDriver {
    fn subscribe(&self, _: usize, _: Option<Callback>, _: AppId) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn command(&self, cmd: usize, arg0: usize, arg1: usize, appid: AppId) -> ReturnCode {
        match (cmd, arg0, arg1) {
            (0, _, _) => ReturnCode::SUCCESS,

            (1, 0, _) => ReturnCode::SuccessWithValue { value: WORD_SIZE },
            (1, 1, _) => ReturnCode::SuccessWithValue { value: PAGE_SIZE },
            (1, 2, _) => ReturnCode::SuccessWithValue {
                value: MAX_WORD_WRITES,
            },
            (1, 3, _) => ReturnCode::SuccessWithValue {
                value: MAX_PAGE_ERASES,
            },
            (1, _, _) => ReturnCode::EINVAL,

            (2, ptr, len) => self
                .apps
                .enter(appid, |app, _| {
                    let slice = match app.slice.take() {
                        None => return ReturnCode::EINVAL,
                        Some(slice) => slice,
                    };
                    if len != slice.len() {
                        return ReturnCode::EINVAL;
                    }
                    self.write_slice(ptr, slice.as_ref())
                })
                .unwrap_or_else(|err| err.into()),

            (3, ptr, len) => {
                if len != PAGE_SIZE {
                    return ReturnCode::EINVAL;
                }
                self.erase_page(ptr)
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
