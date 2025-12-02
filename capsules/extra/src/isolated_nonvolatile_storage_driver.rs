// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! This provides userspace access to nonvolatile storage.
//!
//! This driver provides isolation between individual userland applications.
//! Each application only has access to its region of nonvolatile memory and
//! cannot read/write to nonvolatile memory of other applications.
//!
//! Each app is assigned a fixed amount of nonvolatile memory. This amount is
//! set at compile time.
//!
//! ## Storage Layout
//!
//! Example nonvolatile storage layout (note that `|` indicates bitwise
//! concatenation):
//!
//! ```text
//!     ╒════════ ← Start of nonvolatile region
//!     ├──────── ← Start of App 1's region header
//!     │ Region version number (8 bits) | Region length (24 bits)
//!     │ App 1's ShortID (u32)
//!     │ XOR of previous two u32 fields (u32)
//!     ├──────── ← Start of App 1's Region          ═╗
//!     │                                             ║
//!     │
//!     │                                            region 1
//!     │                                            length
//!     │
//!     │                                             ║
//!     │                                            ═╝
//!     ├──────── ← Start of App 2's region header
//!     │ Region version number (8 bits) | Region length (24 bits)
//!     │ App 2's ShortID (u32)
//!     │ XOR of previous two u32 fields (u32)
//!     ├──────── ← Start of App 2's Region          ═╗
//!     │                                             ║
//!     │
//!     │
//!     │                                            region 2
//!     │                                            length
//!     │
//!     │
//!     │                                             ║
//!     ...                                          ═╝
//!     ╘════════ ← End of userspace region
//! ```
//!
//! ## Storage Initialization
//!
//! This capsule caches the location of an application's storage region in
//! grant. This cached location is set on the first usage of this capsule.
//!
//! Here is a general high-level overview of what happens when an app makes its
//! first syscall:
//! 1. App engages with the capsule by making any syscall.
//! 2. Capsule searches through storage to see if that app has an existing
//!    region.
//! 3. a. If the capsule finds a matching region:
//!    - Cache the app's region information in its grant.
//!    b. If the capsule DOESN'T find a matching region:
//!    - Allocate a new region for that app.
//!    - Erase the region's usable area.
//! 4. Handle the syscall that the app originally made.
//! 5. When the syscall finishes, notify the app via upcall.
//!
//! ## Example Software Stack
//!
//! Here is a diagram of the expected stack with this capsule: Boxes are
//! components and between the boxes are the traits that are the interfaces
//! between components. This capsule only provides a userspace interface.
//!
//! ```text
//! +------------------------------------------------------------------------+
//! |                                                                        |
//! |                             userspace                                  |
//! |                                                                        |
//! +------------------------------------------------------------------------+
//!                             kernel::Driver
//! +------------------------------------------------------------------------+
//! |                                                                        |
//! | isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage (this) |
//! |                                                                        |
//! +------------------------------------------------------------------------+
//!            hil::nonvolatile_storage::NonvolatileStorage
//! +------------------------------------------------------------------------+
//! |                                                                        |
//! |               Physical nonvolatile storage driver                      |
//! |                                                                        |
//! +------------------------------------------------------------------------+
//! ```
//!

use core::cmp;

use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::copy_slice::CopyOrErr;
use kernel::{ErrorCode, ProcessId};

use capsules_core::driver;

pub const DRIVER_NUM: usize = driver::NUM::IsolatedNvmStorage as usize;

/// Recommended size for the buffer provided to this capsule.
///
/// This is enough space for a buffer to be used for reading/writing userspace
/// data.
pub const BUF_LEN: usize = 512;

/// IDs for subscribed upcalls.
mod upcall {
    /// Get storage size done callback.
    pub const GET_SIZE_DONE: usize = 0;
    /// Read done callback.
    pub const READ_DONE: usize = 1;
    /// Write done callback.
    pub const WRITE_DONE: usize = 2;
    /// Number of upcalls.
    pub const COUNT: u8 = 3;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// Setup a buffer to write bytes to the nonvolatile storage.
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Setup a buffer to read from the nonvolatile storage into.
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
enum HeaderVersion {
    V1 = 0x01,
}

// Current header version to allocate new regions with.
const CURRENT_HEADER_VERSION: HeaderVersion = HeaderVersion::V1;

/// Describes a region of nonvolatile memory that is assigned to a certain app.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppRegion {
    /// The version is based on the capsule version and layout format in use
    /// when the region was created. This is set to a fixed value for all new
    /// regions. An existing region may have been created with a newer or
    /// earlier version of this capsule and therefore might have a different
    /// version than what we currently initialize new regions with.
    version: HeaderVersion,
    /// Absolute address to describe where an app's nonvolatile region starts.
    /// Note that this is the address FOLLOWING the region's header.
    absolute_address: usize,
    /// How many bytes allocated to a certain app. Note that this describes the
    /// length of the usable storage region and does not include the region's
    /// header.
    length: usize,
}

// Metadata to be written before every app's region to describe the owner and
// size of the region.
#[derive(Clone, Copy, Debug)]
struct AppRegionHeader {
    /// An 8 bit version number concatenated with a 24 bit length value.
    version_and_length: u32,
    /// Unique per-app identifier. This comes from the Fixed variant of the
    /// ShortID type.
    shortid: u32,
    /// xor between `version_and_length` and `shortid` fields. This serves as a
    /// checksum.
    xor: u32,
}
/// The size of the `AppRegionHeader` stored in the nonvolatile storage.
const REGION_HEADER_LEN: usize = 3 * core::mem::size_of::<u32>();

impl AppRegionHeader {
    fn new(version: HeaderVersion, shortid: u32, length: usize) -> Option<Self> {
        // check that length will fit in 3 bytes
        if length > (2 << 23) {
            return None;
        }

        let version_and_length = ((version as u8 as u32) << 24) | length as u32;

        let xor = version_and_length ^ shortid;

        Some(AppRegionHeader {
            version_and_length,
            shortid,
            xor,
        })
    }

    fn from_bytes(bytes: [u8; REGION_HEADER_LEN]) -> Option<Self> {
        // first 4 bytes are split between a 8 bit version and 24 bit length
        let version = bytes[0];
        let length_slice = &bytes[1..4];
        let version_and_length_slice = [version, length_slice[0], length_slice[1], length_slice[2]];
        let version_and_length = u32::from_le_bytes(version_and_length_slice);

        let shortid_slice = bytes[4..8].try_into().ok()?;
        let shortid = u32::from_le_bytes(shortid_slice);

        let xor_slice = bytes[8..12].try_into().ok()?;
        let xor = u32::from_le_bytes(xor_slice);

        Some(AppRegionHeader {
            version_and_length,
            shortid,
            xor,
        })
    }

    fn to_bytes(self) -> [u8; REGION_HEADER_LEN] {
        let mut header_slice = [0; REGION_HEADER_LEN];

        // copy version and length
        let version_and_length_slice = u32::to_le_bytes(self.version_and_length);
        let version_and_length_start_idx = 0;
        let version_and_length_end_idx = version_and_length_slice.len();
        header_slice[version_and_length_start_idx..version_and_length_end_idx]
            .copy_from_slice(&version_and_length_slice);

        // copy shortid
        let shortid_slice = u32::to_le_bytes(self.shortid);
        let shortid_start_idx = version_and_length_end_idx;
        let shortid_end_idx = shortid_start_idx + shortid_slice.len();
        header_slice[shortid_start_idx..shortid_end_idx].copy_from_slice(&shortid_slice);

        // copy version and length
        let xor_slice = u32::to_le_bytes(self.xor);
        let xor_start_idx = shortid_end_idx;
        let xor_end_idx = xor_start_idx + xor_slice.len();
        header_slice[xor_start_idx..xor_end_idx].copy_from_slice(&xor_slice);

        header_slice
    }

    fn is_valid(&self) -> bool {
        self.version().is_some() && self.xor == (self.version_and_length ^ self.shortid)
    }

    fn version(&self) -> Option<HeaderVersion> {
        // Need to do this since we can't pattern match against a method call.
        const HEADER_V1: u8 = HeaderVersion::V1 as u8;

        // Extract the 8 most significant bits from the concatenated version and
        // length.
        match (self.version_and_length >> 24) as u8 {
            HEADER_V1 => Some(HeaderVersion::V1),
            _ => None,
        }
    }

    fn length(&self) -> u32 {
        // Extract the 24 least significant bits from the concatenated version
        // and length.
        self.version_and_length & 0x00ffffff
    }
}

/// Operation referencing a particular region.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ManagerTask {
    /// Read the contents of the header in the region. The `usize` is the
    /// address of the start of the header.
    DiscoverRegions(usize),
    /// Write a valid header to the storage.
    WriteHeader(ProcessId, AppRegion),
    /// Erase the contents of a region. This supports using multiple nonvolatile
    /// storage operations to complete the entire erase.
    EraseRegion {
        processid: ProcessId,
        next_erase_start: usize,
        remaining_bytes: usize,
    },
}

/// What is currently using the underlying nonvolatile storage driver.
#[derive(Clone, Copy, Debug)]
pub enum User {
    /// The operation is from a userspace process.
    App { processid: ProcessId },
    /// The operation is from this capsule.
    RegionManager(ManagerTask),
}

/// The operation the process requested.
#[derive(Clone, Copy, Debug)]
pub enum NvmCommand {
    GetSize,
    Read { offset: usize },
    Write { offset: usize },
}

impl NvmCommand {
    fn offset(&self) -> usize {
        match self {
            NvmCommand::Read { offset } => *offset,
            NvmCommand::Write { offset } => *offset,
            NvmCommand::GetSize => 0,
        }
    }

    fn upcall(&self) -> usize {
        match self {
            Self::GetSize => upcall::GET_SIZE_DONE,
            Self::Write { offset: _ } => upcall::WRITE_DONE,
            Self::Read { offset: _ } => upcall::READ_DONE,
        }
    }
}

/// State stored in the grant region on behalf of each app.
#[derive(Default)]
pub struct App {
    /// Describe the location and size of an app's region (if it has been
    /// initialized).
    region: Option<AppRegion>,
    /// Operation that will be handled once init sequence is complete.
    pending_operation: Option<NvmCommand>,
}

/// Helper function to convert create a full, single usize value from two 32-bit
/// values stored in usizes.
///
/// In C this would look like:
///
/// ```c
/// size_t v = (hi << 32) | (uint32_t) lo;
/// ```
///
/// This is useful when passing a machine-sized value (i.e. a `size_t`) via the
/// system call interface in two 32-bit usize values. On a 32-bit machine this
/// essentially has no effect; the full value is stored in the `lo` usize. On a
/// 64-bit machine, this creates a usize by concatenating the hi and lo 32-bit
/// values.
///
/// TODO
/// ----
///
/// This can be more succinctly implemented using
/// [`unbounded_shl()`](https://doc.rust-lang.org/stable/std/primitive.usize.html#method.unbounded_shl).
/// However, that method is currently a nightly-only feature.
#[inline]
pub const fn usize32s_to_usize(lo: usize, hi: usize) -> usize {
    if usize::BITS <= 32 {
        // Just return the lo value since it has the bits we need.
        lo
    } else {
        // Create a 64-bit value.
        (lo & 0xFFFFFFFF) | (hi << 32)
    }
}

pub struct IsolatedNonvolatileStorage<'a, const APP_REGION_SIZE: usize> {
    /// The underlying physical storage device.
    driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'a>,
    /// Per-app state.
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    /// Internal buffer for copying appslices into.
    buffer: TakeCell<'static, [u8]>,
    /// What issued the currently executing call. This can be an app or the
    /// kernel.
    current_user: OptionalCell<User>,

    /// The first byte that is accessible from userspace.
    userspace_start_address: usize,
    /// How many bytes allocated to userspace.
    userspace_length: usize,

    /// Absolute address of the header of the next region of userspace that's
    /// not allocated to an app yet. Each time an app uses this capsule, a new
    /// region of storage will be handed out and this address will point to the
    /// header of a new unallocated region.
    next_unallocated_region_header_address: OptionalCell<usize>,
}

impl<'a, const APP_REGION_SIZE: usize> IsolatedNonvolatileStorage<'a, APP_REGION_SIZE> {
    pub fn new(
        driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'a>,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        userspace_start_address: usize,
        userspace_length: usize,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            driver,
            apps: grant,
            buffer: TakeCell::new(buffer),
            current_user: OptionalCell::empty(),
            userspace_start_address,
            userspace_length,
            next_unallocated_region_header_address: OptionalCell::empty(),
        }
    }

    // Start reading app region headers.
    fn start_region_traversal(&self) -> Result<(), ErrorCode> {
        if self.current_user.is_some() {
            // Can't traverse the regions right now because the underlying
            // driver is already in use.
            return Err(ErrorCode::BUSY);
        }

        let res = self.read_region_header(self.userspace_start_address);
        match res {
            Ok(()) => {
                // Mark that we started the discover operation.
                self.current_user
                    .set(User::RegionManager(ManagerTask::DiscoverRegions(
                        self.userspace_start_address,
                    )));
                Ok(())
            }
            Err(e) => {
                // We did not successfully start the discover, return the error.
                Err(e)
            }
        }
    }

    fn allocate_app_region(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Can't allocate a region if we haven't previously traversed existing
        // regions and found where they stop.
        let new_header_addr = self
            .next_unallocated_region_header_address
            .get()
            .ok_or(ErrorCode::FAIL)?;

        // Get an app's write_id (same as ShortID) for saving to region header.
        // Note that if an app doesn't have the valid permissions, it will be
        // unable to create storage regions.
        let write_id = processid
            .get_storage_permissions()
            .ok_or(ErrorCode::NOSUPPORT)?
            .get_write_id()
            .ok_or(ErrorCode::NOSUPPORT)?;

        let region = AppRegion {
            version: CURRENT_HEADER_VERSION,
            // Have this region start where all the existing regions end.
            // Note that the app's actual region starts after the region header.
            absolute_address: new_header_addr + REGION_HEADER_LEN,
            length: APP_REGION_SIZE,
        };

        // fail if new region is outside userspace area
        if region.absolute_address > self.userspace_start_address + self.userspace_length
            || region.absolute_address + region.length
                > self.userspace_start_address + self.userspace_length
        {
            return Err(ErrorCode::NOMEM);
        }

        let Some(header) = AppRegionHeader::new(region.version, write_id, region.length) else {
            return Err(ErrorCode::FAIL);
        };

        // Write this new region header to the end of the existing regions.
        let res = self.write_region_header(&region, &header, new_header_addr);
        match res {
            Ok(()) => {
                // Mark that we started the initialize region task.
                self.current_user
                    .set(User::RegionManager(ManagerTask::WriteHeader(
                        processid, region,
                    )));
                Ok(())
            }
            Err(e) => {
                // We did not successfully start the region initialization,
                // return the error.
                Err(e)
            }
        }
    }

    // Read the header of an app's storage region. The region_header_address
    // argument describes the start of the **header** and not the usable region
    // itself.
    fn read_region_header(&self, region_header_address: usize) -> Result<(), ErrorCode> {
        self.check_header_access(region_header_address, APP_REGION_SIZE)?;

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            self.driver
                .read(buffer, region_header_address, REGION_HEADER_LEN)
        })
    }

    // Write the header of an app's storage region. The region_header_address
    // argument describes the start of the **header** and not the usable region
    // itself.
    fn write_region_header(
        &self,
        region: &AppRegion,
        region_header: &AppRegionHeader,
        region_header_address: usize,
    ) -> Result<(), ErrorCode> {
        self.check_header_access(region.absolute_address, region.length)?;

        let header_slice = region_header.to_bytes();

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            let _ = buffer
                .get_mut(0..REGION_HEADER_LEN)
                .ok_or(ErrorCode::NOMEM)?
                .copy_from_slice_or_err(
                    header_slice
                        .get(0..REGION_HEADER_LEN)
                        .ok_or(ErrorCode::NOMEM)?,
                );

            self.driver
                .write(buffer, region_header_address, REGION_HEADER_LEN)
        })
    }

    fn erase_region_content(
        &self,
        offset: usize,
        length: usize,
    ) -> Result<(usize, usize), ErrorCode> {
        self.check_header_access(offset, length)?;

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            let active_len = cmp::min(length, buffer.len());

            // Clear the erase buffer in case there was any data
            // remaining from a previous operation.
            for c in buffer.iter_mut() {
                *c = 0xFF;
            }

            // how many more bytes to erase after this operation
            let remaining_len = if length > buffer.len() {
                length - buffer.len()
            } else {
                0
            };

            let next_erase_start = offset + active_len;

            self.driver
                .write(buffer, offset, active_len)
                .and(Ok((next_erase_start, remaining_len)))
        })
    }

    // Returns `Ok()` with the address of the next header to be read if a new
    // header read was started.
    fn header_read_done(&self, region_header_address: usize) -> Result<Option<usize>, ErrorCode> {
        // Cases when a header read completes:
        // 1. Read a valid header
        //     - The valid header belongs to a Tock app (might not be currently
        //       running).
        //     - Search for the owner of the region within the apps.
        //     - Find the owner of the region that has a matching shortid (from
        //       the header).
        //     - Then, startup another read operation to read the header of the
        //       next storage region.
        // 2. Read an invalid header
        //     - We've reached the end of all previously allocated regions.
        //     - Allocate new app region here.

        let header = self.buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
            // Need to copy over bytes since we need to convert a &[u8] into a
            // [u8; REGION_HEADER_LEN]. The &[u8] refers to a slice of size
            // BUF_LEN (which could be different than REGION_HEADER_LEN). Using
            // buffer.try_into() will fail at runtime since the underlying
            // buffer is not the same length as what we're trying to convert
            // into.
            let mut header_buffer = [0; REGION_HEADER_LEN];
            header_buffer
                .copy_from_slice_or_err(&buffer[..REGION_HEADER_LEN])
                .or(Err(ErrorCode::FAIL))?;

            // reconstruct header from bytes we just read
            AppRegionHeader::from_bytes(header_buffer).ok_or(ErrorCode::FAIL)
        })?;

        if header.is_valid() {
            // Find the app with the corresponding shortid.
            for app in self.apps.iter() {
                let processid = app.processid();
                // Skip an app if it doesn't have the proper storage
                // permissions.
                let write_id = match processid.get_storage_permissions() {
                    Some(perms) => match perms.get_write_id() {
                        Some(write_id) => write_id,
                        None => continue,
                    },
                    None => continue,
                };
                if write_id == header.shortid {
                    app.enter(|app, _kernel_data| {
                        if app.region.is_none() {
                            let version = header.version().ok_or(ErrorCode::FAIL)?;
                            let region = AppRegion {
                                version,
                                // The app's actual region starts after the
                                // region header.
                                absolute_address: region_header_address + REGION_HEADER_LEN,
                                length: header.length() as usize,
                            };
                            app.region.replace(region);
                        }
                        Ok::<(), ErrorCode>(())
                    })?;
                    break;
                }
            }

            let next_header_address =
                region_header_address + REGION_HEADER_LEN + header.length() as usize;
            // Kick off another read for the next region.
            self.read_region_header(next_header_address)
                .and(Ok(Some(next_header_address)))
        } else {
            // This is the end of the region traversal. If a header is invalid,
            // we've reached the end of all previously allocated regions.

            // Save this region header address so that we can allocate new
            // regions here later.
            self.next_unallocated_region_header_address
                .set(region_header_address);

            Ok(None)
        }
    }

    fn check_userspace_perms(
        &self,
        processid: ProcessId,
        command: NvmCommand,
    ) -> Result<(), ErrorCode> {
        let perms = processid
            .get_storage_permissions()
            .ok_or(ErrorCode::NOSUPPORT)?;
        let write_id = perms.get_write_id().ok_or(ErrorCode::NOSUPPORT)?;
        match command {
            NvmCommand::Read { offset: _ } => perms
                .check_read_permission(write_id)
                .then_some(())
                .ok_or(ErrorCode::NOSUPPORT),
            NvmCommand::Write { offset: _ } => perms
                .check_modify_permission(write_id)
                .then_some(())
                .ok_or(ErrorCode::NOSUPPORT),
            NvmCommand::GetSize => {
                // If we have a `write_id` then we can return the size.
                Ok(())
            }
        }
    }

    fn check_userspace_access(
        &self,
        offset: usize,
        length: usize,
        region: &AppRegion,
    ) -> Result<(), ErrorCode> {
        // Check that access is within this app's isolated nonvolatile region.
        // This is to prevent an app from reading/writing to another app's
        // nonvolatile storage.

        if offset >= region.length || length > region.length || offset + length > region.length {
            return Err(ErrorCode::INVAL);
        }

        Ok(())
    }

    fn check_header_access(&self, offset: usize, length: usize) -> Result<(), ErrorCode> {
        // Check that we're within the entire userspace region.
        if offset < self.userspace_start_address
            || offset >= self.userspace_start_address + self.userspace_length
            || length > self.userspace_length
            || offset + length >= self.userspace_start_address + self.userspace_length
        {
            return Err(ErrorCode::INVAL);
        }

        Ok(())
    }

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending command
    // completes.
    fn enqueue_userspace_command(
        &self,
        command: NvmCommand,
        processid: ProcessId,
    ) -> Result<(), ErrorCode> {
        self.check_userspace_perms(processid, command)?;

        self.apps
            .enter(processid, |app, _kernel_data| {
                if app.pending_operation.is_some() {
                    return Err(ErrorCode::BUSY);
                }
                app.pending_operation = Some(command);
                Ok(())
            })
            .unwrap_or_else(|err| Err(err.into()))?;

        self.check_queue();
        Ok(())
    }

    fn check_queue(&self) {
        if self.current_user.is_some() {
            // If the driver is busy we can't start a new operation and do not
            // need to check the queue.
            return;
        }

        // If this is none, we haven't traversed the existing regions yet.
        if self.next_unallocated_region_header_address.is_none() {
            match self.start_region_traversal() {
                Ok(()) => {
                    // We started an operation so we can return and let that
                    // operation finish.
                    return;
                }
                Err(_e) => {
                    // We did not start the traversal which is a problem. This
                    // shouldn't happen, but if it does then we could overwrite
                    // existing regions.
                    return;
                }
            }
        }

        // Iterate apps and run an operation if one is pending.
        for app in self.apps.iter() {
            let processid = app.processid();
            let started = app.enter(|app, kernel_data| {
                match app.pending_operation {
                    Some(nvm_command) => {
                        if app.region.is_none() {
                            // This app needs its region allocated.
                            self.allocate_app_region(processid).is_ok()
                        } else {
                            let res = self.handle_syscall(nvm_command, processid, app, kernel_data);
                            match res {
                                Ok(started_operation) => started_operation,
                                Err(e) => {
                                    app.pending_operation = None;
                                    let _ = kernel_data.schedule_upcall(
                                        nvm_command.upcall(),
                                        (into_statuscode(Err(e)), 0, 0),
                                    );

                                    false
                                }
                            }
                        }
                    }
                    None => false,
                }
            });
            if started {
                break;
            }
        }
    }

    fn handle_syscall(
        &self,
        command: NvmCommand,
        processid: ProcessId,
        app: &mut App,
        kernel_data: &kernel::grant::GrantKernelData,
    ) -> Result<bool, ErrorCode> {
        match command {
            NvmCommand::GetSize => {
                match app.region {
                    Some(region) => {
                        // clear pending syscall
                        app.pending_operation = None;
                        // signal app with the result
                        let _ = kernel_data.schedule_upcall(
                            upcall::GET_SIZE_DONE,
                            (into_statuscode(Ok(())), region.length, 0),
                        );
                        Ok(false)
                    }
                    None => Err(ErrorCode::NOMEM),
                }
            }

            NvmCommand::Read { offset: _ } | NvmCommand::Write { offset: _ } => {
                // Get the length of the correct allowed buffer.
                let allow_buf_len = match command {
                    NvmCommand::Read { offset: _ } => kernel_data
                        .get_readwrite_processbuffer(rw_allow::READ)
                        .map_or(0, |read| read.len()),
                    NvmCommand::Write { offset: _ } => kernel_data
                        .get_readonly_processbuffer(ro_allow::WRITE)
                        .map_or(0, |read| read.len()),
                    NvmCommand::GetSize => 0,
                };

                // Check that the matching allowed buffer exists.
                if allow_buf_len == 0 {
                    return Err(ErrorCode::RESERVE);
                }

                // Fail if the app doesn't have a region assigned to it.
                let Some(app_region) = &app.region else {
                    return Err(ErrorCode::NOMEM);
                };

                let command_offset = command.offset();

                self.check_userspace_access(command_offset, allow_buf_len, app_region)?;

                // Need to copy bytes if this is a write!
                if let NvmCommand::Write { offset: _ } = command {
                    let _ = kernel_data
                        .get_readonly_processbuffer(ro_allow::WRITE)
                        .and_then(|write| {
                            write.enter(|app_buffer| {
                                self.buffer.map(|kernel_buffer| {
                                    // Check that the internal buffer and
                                    // the buffer that was allowed are long
                                    // enough.
                                    let write_len = cmp::min(allow_buf_len, kernel_buffer.len());

                                    let d = &app_buffer[0..write_len];
                                    for (i, c) in kernel_buffer[0..write_len].iter_mut().enumerate()
                                    {
                                        *c = d[i].get();
                                    }
                                });
                            })
                        });
                }

                // Calculate where we want to actually read from in the
                // physical storage. Note that the offset for this
                // command is with respect to the app's region address
                // space. This means that userspace accesses start at 0
                // which is the start of the app's region.
                let physical_address = app_region.absolute_address + command_offset;

                let res = self
                    .buffer
                    .take()
                    .map_or(Err(ErrorCode::RESERVE), |buffer| {
                        // Check that the internal buffer and the buffer that was
                        // allowed are long enough.
                        let active_len_buf = cmp::min(allow_buf_len, buffer.len());

                        match command {
                            NvmCommand::Read { offset: _ } => self
                                .driver
                                .read(buffer, physical_address, active_len_buf)
                                .or(Err(ErrorCode::FAIL)),
                            NvmCommand::Write { offset: _ } => self
                                .driver
                                .write(buffer, physical_address, active_len_buf)
                                .or(Err(ErrorCode::FAIL)),
                            NvmCommand::GetSize => Err(ErrorCode::FAIL),
                        }
                    });
                match res {
                    Ok(()) => {
                        self.current_user.set(User::App { processid });
                        Ok(true)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

/// This is the callback client for the underlying physical storage driver.
impl<const APP_REGION_SIZE: usize> hil::nonvolatile_storage::NonvolatileStorageClient
    for IsolatedNonvolatileStorage<'_, APP_REGION_SIZE>
{
    fn read_done(&self, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_user.take().map(|user| {
            match user {
                User::RegionManager(state) => {
                    self.buffer.replace(buffer);
                    if let ManagerTask::DiscoverRegions(address) = state {
                        let res = self.header_read_done(address);
                        match res {
                            Ok(addr) => match addr {
                                Some(next_header_address) => {
                                    self.current_user.set(User::RegionManager(
                                        ManagerTask::DiscoverRegions(next_header_address),
                                    ));
                                }
                                None => {
                                    // We finished the scan of existing
                                    // regions. Now we can check the queue
                                    // to see if there is any work to be
                                    // done.
                                    self.check_queue();
                                }
                            },
                            Err(_e) => {
                                // Not clear what to do here.
                                self.check_queue();
                            }
                        }
                    }
                }
                User::App { processid } => {
                    let _ = self.apps.enter(processid, move |app, kernel_data| {
                        // Need to copy in the contents of the buffer
                        let read_len = kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .and_then(|read| {
                                read.mut_enter(|app_buffer| {
                                    let read_len = cmp::min(app_buffer.len(), length);

                                    let d = &app_buffer[0..read_len];
                                    for (i, c) in buffer[0..read_len].iter().enumerate() {
                                        d[i].set(*c);
                                    }
                                    read_len
                                })
                            })
                            .unwrap_or(0);

                        // Replace the buffer we used to do this read.
                        self.buffer.replace(buffer);

                        // clear pending syscall
                        app.pending_operation = None;
                        // And then signal the app.
                        let _ = kernel_data.schedule_upcall(
                            upcall::READ_DONE,
                            (into_statuscode(Ok(())), read_len, 0),
                        );
                    });

                    self.check_queue();
                }
            }
        });
    }

    fn write_done(&self, buffer: &'static mut [u8], length: usize) {
        // Replace the buffer we used to do this write.
        self.buffer.replace(buffer);

        // Switch on which user of this capsule generated this callback.
        self.current_user.take().map(|user| {
            match user {
                User::RegionManager(state) => {
                    match state {
                        ManagerTask::WriteHeader(processid, region) => {
                            // Now that we have written the header for the app
                            // we can store its region in its grant.
                            let _ = self.apps.enter(processid, |app, _kernel_data| {
                                // set region data in app's grant
                                app.region.replace(region);
                            });

                            // Update our metadata about where the next
                            // unallocated region is.
                            let next_header_addr = region.absolute_address + region.length;
                            self.next_unallocated_region_header_address
                                .set(next_header_addr);

                            // Erase the userspace accessible content of the region
                            // before handing it off to an app.
                            let res =
                                self.erase_region_content(region.absolute_address, region.length);
                            match res {
                                Ok((next_erase_start, remaining_bytes)) => {
                                    self.current_user.set(User::RegionManager(
                                        // need to pass on where the next erase should start
                                        // how long it should be.
                                        ManagerTask::EraseRegion {
                                            processid,
                                            next_erase_start,
                                            remaining_bytes,
                                        },
                                    ));
                                }
                                Err(_e) => {
                                    // Not clear what to do here.
                                    self.current_user.clear();
                                    self.check_queue();
                                }
                            }
                        }
                        ManagerTask::EraseRegion {
                            processid,
                            next_erase_start,
                            remaining_bytes,
                        } => {
                            if remaining_bytes > 0 {
                                // We still have more to erase, so kick off
                                // another one where we left off.
                                let res =
                                    self.erase_region_content(next_erase_start, remaining_bytes);
                                match res {
                                    Ok((next_erase_start, remaining_bytes)) => {
                                        self.current_user.set(User::RegionManager(
                                            ManagerTask::EraseRegion {
                                                processid,
                                                next_erase_start,
                                                remaining_bytes,
                                            },
                                        ));
                                    }
                                    Err(_e) => {
                                        // Not clear what to do here.
                                        self.current_user.clear();
                                        self.check_queue();
                                    }
                                }
                            } else {
                                // Done erasing entire region. Can go on with
                                // normal tasks.
                                self.current_user.clear();
                                self.check_queue();
                            }
                        }
                        _ => {}
                    }
                }
                User::App { processid } => {
                    let _ = self.apps.enter(processid, move |app, kernel_data| {
                        // clear pending syscall
                        app.pending_operation = None;
                        // Notify app that its write has completed.
                        let _ = kernel_data.schedule_upcall(
                            upcall::WRITE_DONE,
                            (into_statuscode(Ok(())), length, 0),
                        );
                    });
                    self.current_user.clear();
                    self.check_queue();
                }
            }
        });
    }
}

/// Provide an interface for userland.
impl<const APP_REGION_SIZE: usize> SyscallDriver
    for IsolatedNonvolatileStorage<'_, APP_REGION_SIZE>
{
    /// Command interface.
    ///
    /// Commands are selected by the lowest 8 bits of the first argument.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Return the number of bytes available to each app.
    /// - `2`: Start a read from the nonvolatile storage.
    /// - `3`: Start a write to the nonvolatile_storage.
    fn command(
        &self,
        command_num: usize,
        offset_lo: usize,
        offset_hi: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            // For the get_bytes, read, and write syscalls we need to first
            // initialize the app's isolated nonvolatile storage. This involves
            // searching the storage area for an existing region that belongs to
            // this app. If we don't find an existing region we allocate a new
            // one. Only once the initialization is complete, can we service the
            // original syscall. So, we can store the syscall data in the app's
            // grant and handle it when initialization finishes.
            1 | 2 | 3 => {
                // We want to handle both 64-bit and 32-bit platforms, but on
                // 32-bit platforms shifting `offset_hi` doesn't make sense.
                let offset: usize = usize32s_to_usize(offset_lo, offset_hi);
                let nvm_command = match command_num {
                    1 => NvmCommand::GetSize,
                    2 => NvmCommand::Read { offset },
                    3 => NvmCommand::Write { offset },
                    _ => return CommandReturn::failure(ErrorCode::NOSUPPORT),
                };

                // Enqueue the operation for the app.
                let res = self.enqueue_userspace_command(nvm_command, processid);
                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
