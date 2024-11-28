// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! This provides userspace access to isolated nonvolatile memory.
//!
//! This implementation provides isolation between individual userland
//! applications. Each application only has access to its region of nonvolatile
//! memory and cannot read/write to nonvolatile memory of other applications.
//!
//! Currently, each app is assigned a fixed amount of nonvolatile memory.
//! This number is configurable at capsule creation time. Future implementations
//! should consider giving each app more freedom over configuring the amount
//! of nonvolatile memory they will use.
//!
//! In case, the app previously had data in persistent storage, we need
//! to find where that is. Therefore, there's an initialization sequence
//! that happens on every app's first syscall. 
//! 
//! Here is a general high-level overview of what happens when an
//! app makes their first syscall:
//!  1. App engages with the capsule by making any syscall.
//!  2. Capsule searches through storage to see if that app
//!     has an existing region.
//!  3. a. If the capsule finds a matching region:
//!         - Cache the app's region information in its grant.
//!     b. If the capsule DOESN'T find a matching region:
//!         - Allocate a new region for that app.
//!         - Erase the region's usable area.
//!   4. Handle the syscall that the app originally made. 
//!   5. When the syscall finishes, notify the app via upcall.
//!
//! Here is a diagram of the expected stack with this capsule:
//! Boxes are components and between the boxes are the traits that are the
//! interfaces between components. This capsule only provides a 
//! userspace interface.
//!
//! ```text
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! |                             userspace                           |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//!                             kernel::Driver
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! | capsules::nonvolatile_storage_driver::NonvolatileStorage (this) |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//!            hil::nonvolatile_storage::NonvolatileStorage
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! |               Physical nonvolatile storage driver               |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//! ```
//!
//! Example nonvolatile storage layout:
//!
//! ```text
//!     ╒════════ ← Start of userspace region
//!     ├──────── ← Start of App 1's region header
//!     │ Region version number (8 bits) | Region length (24 bits) (Note that | inidcates bitwise concatenation)
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
//!     │ Region version number (8 bits) | Region length (24 bits) (Note that | inidcates bitwise concatenation)
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
//!
//! Example instantiation:
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let nonvolatile_storage = static_init!(
//!     capsules::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage<'static>,
//!     capsules::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage::new(
//!         fm25cl,                      // The underlying storage driver.
//!         board_kernel.create_grant(&grant_cap),     // Storage for app-specific state.
//!         3000,                        // The byte start address for the userspace
//!                                      // accessible memory region.
//!         2000,                        // The length of the userspace region.
//!         0,                           // The byte start address of the region
//!                                      // that is accessible by the kernel.
//!         &mut [u8; capsules::isolated_nonvolatile_storage_driver::BUF_LEN],    // buffer for reading/writing
//! hil::nonvolatile_storage::NonvolatileStorage::set_client(fm25cl, nonvolatile_storage);
//! ```

use core::cmp;

use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::copy_slice::CopyOrErr;
use kernel::hil;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;

pub const DRIVER_NUM: usize = driver::NUM::IsolatedNvmStorage as usize;

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
enum HeaderVersion {
    V1,
}

impl HeaderVersion {
    const fn value(&self) -> u8 {
        match self {
            HeaderVersion::V1 => 0x01,
        }
    }
}

// Current header version to allocate new regions with.
const CURRENT_HEADER_VERSION: HeaderVersion = HeaderVersion::V1;

/// Describes a region of nonvolatile memory that is assigned to a
/// certain app.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppRegion {
    version: HeaderVersion,
    /// Absolute address to describe where an app's nonvolatile region
    /// starts. Note that this is the address FOLLOWING the region's header.
    offset: usize,
    /// How many bytes allocated to a certain app. Note that this describes
    /// the length of the usable storage region and does not include the
    /// region's header.
    length: usize,
}

// Metadata to be written before every app's region to describe
// the owner and size of the region.
#[derive(Clone, Copy, Debug)]
struct AppRegionHeader {
    // an 8 bit version number concatenated with a 24 bit length value
    version_and_length: u32,
    /// Unique per-app identifier. This comes from
    /// the Fixed variant of the ShortID type.
    shortid: u32,
    // xor between version_and_length and shortid fields
    xor: u32,
}
// Enough space to store the three u32 field of the header
pub const REGION_HEADER_LEN: usize = 3 * core::mem::size_of::<u32>();

impl AppRegionHeader {
    fn new(version: HeaderVersion, shortid: u32, length: usize) -> Option<Self> {
        // check that length will fit in 3 bytes
        if length > (2 << 23) {
            return None;
        }

        let version_and_length = ((version.value() as u32) << 24) | length as u32;

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
        // need to do this since we can't pattern match
        // against a method call
        const HEADER_V1: u8 = HeaderVersion::V1.value();

        // extract the 8 most significant bits from
        // the concatenated version and length
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

// Enough space for a buffer to be used for reading/writing userspace data
pub const BUF_LEN: usize = 512;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RegionState {
    ReadHeader(usize),
    WriteHeader(ProcessId, AppRegion),
    EraseRegion {
        processid: ProcessId,
        next_erase_start: usize,
        remaining_bytes: usize,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub enum Command {
    Read { offset: usize, length: usize },
    Write { offset: usize, length: usize },
}

impl Command {
    fn offset(&self) -> usize {
        match self {
            Command::Read { offset, length } => *offset,
            Command::Write { offset, length } => *offset,
        }
    }
    fn length(&self) -> usize {
        match self {
            Command::Read { offset, length } => *length,
            Command::Write { offset, length } => *length,
        }
    }
}

#[derive(Clone, Copy)]
pub enum User {
    App { processid: ProcessId },
    RegionManager(RegionState),
}

#[derive(Clone, Copy)]
pub enum Syscall {
    GetSize,
    Read { offset: usize, length: usize},
    Write { offset: usize, length: usize},
}

impl Syscall {
    fn to_upcall(&self) -> usize {
        match self {
            Self::GetSize => upcall::GET_SIZE_DONE,
            Self::Write { offset: _, length: _ } => upcall::WRITE_DONE,
            Self::Read { offset: _, length: _ } => upcall::READ_DONE,
        }
    }
}

#[derive(Default)]
pub struct App {
    /// The operation the app has requested, if any.
    command: Option<Command>,
    /// Whether this app has previously requested to initialize its nonvolatile
    /// storage.
    has_requested_region: bool,
    /// Describe the location and size of an app's region (if it has been
    /// initialized).
    region: Option<AppRegion>,
    /// Syscall that will be handled once init sequence is complete  
    pending_syscall: Option<Syscall>,
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
    /// What issued the currently executing call. This can be an app or the kernel.
    current_user: OptionalCell<User>,

    /// The first byte that is accessible from userspace.
    userspace_start_address: usize,
    /// How many bytes allocated to userspace.
    userspace_length: usize,

    /// Absolute address of the header of the next region of userspace
    /// that's not allocated to an app yet. Each time an app uses this
    /// capsule, a new region of storage will be handed out and this
    /// address will point to the header of a new unallocated region.
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

    // App-level initialization that allocates a region for an app or fetches
    // an app's existing region from nonvolatile storage
    fn init_app(&self, syscall: Syscall, processid: ProcessId) -> Result<(), ErrorCode> {
        let res = self.apps.enter(processid, |app, _kernel_data| {
            // Mark that this app requested a storage region. If it isn't
            // allocated immediately, it will be handled after previous requests
            // are handled.
            app.has_requested_region = true;

            // can't buffer more than one syscall per app
            if app.pending_syscall.is_some() {
                return Err(ErrorCode::BUSY);
            }

            // Save syscall information to handle after init finishes
            app.pending_syscall = Some(syscall);
            Ok(())
        })?;

        if res.is_err() {
            return res;
        }

        // Start traversing the storage regions to find where the requesting
        // app's storage region is located. If it doesn't exist, a new one will
        // be allocated.
        self.start_region_traversal()
    }

    // Start reading app region headers.
    fn start_region_traversal(&self) -> Result<(), ErrorCode> {
        self.read_region_header(self.userspace_start_address)
    }

    fn allocate_app_region(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // can't allocate a region if we haven't previously traversed existing regions
        // and found where they stop
        let new_header_addr = self
            .next_unallocated_region_header_address
            .get()
            .ok_or(ErrorCode::FAIL)?;

        // Get an app's write_id (same as ShortID) for saving to region header. Note that
        // if an app doesn't have the valid permissions, it will be unable to create
        // storage regions.
        let shortid = processid
            .get_storage_permissions()
            .ok_or(ErrorCode::NOSUPPORT)?
            .get_write_id()
            .ok_or(ErrorCode::NOSUPPORT)?;

        let region = AppRegion {
            version: CURRENT_HEADER_VERSION,
            // Have this region start where all the existing regions end.
            // Note that the app's actual region starts after the region header.
            offset: new_header_addr + REGION_HEADER_LEN,
            length: APP_REGION_SIZE,
        };

        // fail if new region is outside userspace area
        if region.offset > self.userspace_start_address + self.userspace_length
            || region.offset + region.length > self.userspace_start_address + self.userspace_length
        {
            return Err(ErrorCode::NOMEM);
        }

        let Some(header) = AppRegionHeader::new(region.version, shortid, region.length) else {
            return Err(ErrorCode::FAIL);
        };

        // write this new region header to the end of the existing ones
        self.write_region_header(processid, &region, &header, new_header_addr)
    }

    // Read the header of an app's storage region. The region_header_address
    // argument describes the start of the **header** and not the usable region
    // itself.
    fn read_region_header(&self, region_header_address: usize) -> Result<(), ErrorCode> {
        self.check_header_access(region_header_address, APP_REGION_SIZE)?;

        if self.current_user.is_some() {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or_else(
            || {
                Err(ErrorCode::NOMEM)
            },
            |buffer| {
                self.current_user
                    .set(User::RegionManager(RegionState::ReadHeader(
                        region_header_address,
                    )));
                self.driver.read(buffer, region_header_address, REGION_HEADER_LEN)
            },
        )
    }

    // Write the header of an app's storage region. The region_header_address
    // argument describes the start of the **header** and not the usable region
    // itself.
    fn write_region_header(
        &self,
        processid: ProcessId,
        region: &AppRegion,
        region_header: &AppRegionHeader,
        region_header_address: usize,
    ) -> Result<(), ErrorCode> {
        self.check_header_access(region.offset, region.length)?;

        if self.current_user.is_some() {
            return Err(ErrorCode::BUSY);
        }

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

            self.current_user
                .set(User::RegionManager(RegionState::WriteHeader(
                    processid, *region,
                )));
            self.driver.write(buffer, region_header_address, REGION_HEADER_LEN)
        })
    }

    fn erase_region_content(
        &self,
        processid: ProcessId,
        offset: usize,
        length: usize,
    ) -> Result<(), ErrorCode> {
        self.check_header_access(offset, length)?;

        if self.current_user.is_some() {
            return Err(ErrorCode::BUSY);
        }

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

            self.current_user.set(User::RegionManager(
                // need to pass on where the next erase should start
                // how long it should be.
                RegionState::EraseRegion {
                    processid,
                    next_erase_start: offset + active_len,
                    remaining_bytes: remaining_len,
                },
            ));
            self.driver.write(buffer, offset, active_len)
        })
    }

    fn header_read_done(&self, region_header_address: usize) -> Result<(), ErrorCode> {
        // Cases when a header read completes:
        // 1. Read a valid header 
        //     - The valid header belongs to a Tock app (might not be currently running)
        //     - Search for the owner of the region within the apps that have previously 
        //       requested to use nonvolatile storage
        //     - Find the owner of the region that has a matching shortid (from the header) 
        //     - Then, startup another read operation to read the header of the next
        //       storage region.
        // 2. Read an invalid header
        //     - We've reached the end of all previously allocated regions 
        //     - Allocate new app region here

        let header = self.buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
            // Need to copy over bytes since we need to convert a &[u8]
            // into a [u8; REGION_HEADER_LEN]. The &[u8] refers to a
            // slice of size BUF_LEN (which could be different than REGION_HEADER_LEN).
            // Using buffer.try_into() will fail at runtime since the underlying buffer
            // is not the same length as what we're trying to convert into.
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
                // skip an app if it doesn't have the proper storage permissions
                let write_id = match processid.get_storage_permissions() {
                    Some(perms) => match perms.get_write_id() {
                        Some(write_id) => write_id,
                        None => continue,
                    },
                    None => continue,
                };
                if write_id == header.shortid {
                    app.enter(|app, _kernel_data| {
                        // only populate region and signal app that explicitly
                        // requested to initialize storage
                        if app.has_requested_region && app.region.is_none() {
                            let version = header.version().ok_or(ErrorCode::FAIL)?;
                            let region = AppRegion {
                                version,
                                // the app's actual region starts after the
                                // region header
                                offset: region_header_address + REGION_HEADER_LEN,
                                length: header.length() as usize,
                            };
                            app.region.replace(region);
                        }
                        Ok::<(), ErrorCode>(())
                    })?;

                    // Now that we have found the region, app's storage is
                    // initialized and we can handle its original syscall
                    self.check_pending_syscalls();

                    break;
                }
            }

            let next_header_address =
                region_header_address + REGION_HEADER_LEN + header.length() as usize;

            // kick off another read for the next region
            self.read_region_header(next_header_address)
        } else {
            // This is the end of the region traversal. 
            // If a header is invalid, we've reached the end
            // of all previously allocated regions.
            
            // save this region header address so that we can allocate new regions
            // here later
            self.next_unallocated_region_header_address
                .set(region_header_address);

            // Check any pending requests and allocate new regions
            // after any existing regions.
            self.check_unallocated_regions();
            Ok(())
        }
    }

    fn check_userspace_perms(
        &self,
        processid: Option<ProcessId>,
        command: Command,
    ) -> Result<(), ErrorCode> {
        processid.map_or(Err(ErrorCode::FAIL), |processid| {
            let perms = processid
                .get_storage_permissions()
                .ok_or(ErrorCode::NOSUPPORT)?;
            let write_id = perms.get_write_id().ok_or(ErrorCode::NOSUPPORT)?;
            match command {
                Command::Read { offset, length } => perms
                    .check_read_permission(write_id)
                    .then_some(())
                    .ok_or(ErrorCode::NOSUPPORT),
                Command::Write { offset, length } => perms
                    .check_modify_permission(write_id)
                    .then_some(())
                    .ok_or(ErrorCode::NOSUPPORT),
                _ => Err(ErrorCode::FAIL),
            }
        })
    }

    fn check_userspace_access(
        &self,
        offset: usize,
        length: usize,
        processid: Option<ProcessId>,
    ) -> Result<(), ErrorCode> {
        // check that access is within this app's isolated nonvolatile region.
        // this is to prevent an app from reading/writing to another app's
        // nonvolatile storage.
        processid.map_or(Err(ErrorCode::FAIL), |processid| {
            // enter the grant to query what the app's
            // nonvolatile region is
            self.apps
                .enter(processid, |app, _kernel_data| {
                    match &app.region {
                        Some(app_region) => {
                            if offset >= app_region.length
                                || length > app_region.length
                                || offset + length > app_region.length
                            {
                                return Err(ErrorCode::INVAL);
                            }

                            Ok(())
                        }

                        // fail if this app's nonvolatile region hasn't been assigned
                        None => Err(ErrorCode::FAIL),
                    }
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn check_header_access(&self, offset: usize, length: usize) -> Result<(), ErrorCode> {
        // check that we're within the entire userspace region
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
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_userspace_command(
        &self,
        command: Command,
        processid: Option<ProcessId>,
    ) -> Result<(), ErrorCode> {
        self.check_userspace_access(command.offset(), command.length(), processid)?;
        self.check_userspace_perms(processid, command)?;

        processid.map_or(Err(ErrorCode::FAIL), |processid| {
            self.apps
                .enter(processid, |app, kernel_data| {
                    // Get the length of the correct allowed buffer.
                    let allow_buf_len = match command {
                        Command::Read {
                            offset: _,
                            length: _,
                        } => kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .map_or(0, |read| read.len()),
                        Command::Write {
                            offset: _,
                            length: _,
                        } => kernel_data
                            .get_readonly_processbuffer(ro_allow::WRITE)
                            .map_or(0, |read| read.len()),
                        _ => 0,
                    };

                    // Check that the matching allowed buffer exists.
                    if allow_buf_len == 0 {
                        return Err(ErrorCode::RESERVE);
                    }

                    // Fail if the app doesn't have a region assigned to it.
                    let Some(app_region) = &app.region else {
                        return Err(ErrorCode::FAIL);
                    };

                    let (command_offset, command_len) = match command {
                        Command::Read { offset, length } => (offset, length),
                        Command::Write { offset, length } => (offset, length),
                    };

                    // Shorten the length if the application gave us nowhere to
                    // put it.
                    let active_len = cmp::min(command_len, allow_buf_len);

                    // No app is currently using the underlying storage.
                    // Mark this app as active, and then execute the command.
                    self.current_user.set(User::App { processid });

                    // Need to copy bytes if this is a write!
                    if let Command::Write {
                        offset: _,
                        length: _,
                    } = command
                    {
                        let _ = kernel_data
                            .get_readonly_processbuffer(ro_allow::WRITE)
                            .and_then(|write| {
                                write.enter(|app_buffer| {
                                    self.buffer.map(|kernel_buffer| {
                                        // Check that the internal buffer and the buffer that was
                                        // allowed are long enough.
                                        let write_len = cmp::min(active_len, kernel_buffer.len());

                                        let d = &app_buffer[0..write_len];
                                        for (i, c) in
                                            kernel_buffer[0..write_len].iter_mut().enumerate()
                                        {
                                            *c = d[i].get();
                                        }
                                    });
                                })
                            });
                    }

                    // Note that the given offset for this command is with respect to
                    // the app's region address space. This means that userspace accesses
                    // start at 0 which is the start of the app's region.
                    self.userspace_call_driver(
                        command,
                        app_region.offset + command_offset,
                        active_len,
                    )
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn userspace_call_driver(
        &self,
        command: Command,
        offset: usize,
        length: usize,
    ) -> Result<(), ErrorCode> {
        // Calculate where we want to actually read from in the physical
        // storage.
        let physical_address = offset + self.userspace_start_address;

        self.buffer
            .take()
            .map_or(Err(ErrorCode::RESERVE), |buffer| {
                // Check that the internal buffer and the buffer that was
                // allowed are long enough.
                let active_len = cmp::min(length, buffer.len());

                match command {
                    Command::Read { offset, length } => {
                        self.driver.read(buffer, physical_address, active_len)
                    }
                    Command::Write { offset, length } => {
                        self.driver.write(buffer, physical_address, active_len)
                    }
                    _ => Err(ErrorCode::FAIL),
                }
            })
    }

    // Find apps that made syscalls and have a region assigned to them already
    fn find_app_to_handle_syscall(&self) -> Option<(ProcessId, Syscall)> {
        for app in self.apps.iter() {
            let processid = app.processid();
            let pending_syscall = app.enter(|app, _| {
                // hasn't finished init yet, so don't handle syscall
                if app.region.is_none() {
                    None
                } else {
                    app.pending_syscall
                }
            });

            if let Some(syscall) = pending_syscall {
                return Some((processid, syscall));
            }
        };
        None
    }

    // Check for pending events such as unallocated regions or unhandled syscalls
    fn check_unallocated_regions(&self) -> bool {
        // If this is none, we haven't traversed the existing regions yet
        if self.next_unallocated_region_header_address.is_none() {
            return self.start_region_traversal().is_ok();
        }

        // Allocate regions for any apps that we didn't find 
        // while traversing existing regions
        for app in self.apps.iter() {
            let processid = app.processid();
            let started = app.enter(|app, _| {
                if app.has_requested_region && app.region.is_none() {
                    // This app needs its region allocated.
                    self.allocate_app_region(processid).is_ok()
                } else {
                    false
                }
            });
            if started {
                return true;
            }
        }
        false
    }

    fn check_pending_syscalls(&self) -> bool {
        // Handle any pending syscalls 
        if let Some((processid, syscall)) = self.find_app_to_handle_syscall() {
            let res = self.handle_syscall(syscall, processid);

            // Only notify apps if an error occurred.
            // Later on, when the syscall successfully finishes, 
            // they will get an upcall.
            if res.is_err() {
                let _ = self.apps.enter(processid, |_app, kernel_data| {
                    kernel_data.schedule_upcall(syscall.to_upcall(), (into_statuscode(res), 0, 0)).ok();
                });
                return false;
            }

            // no error ocurred when starting up syscall
            return true;
        }
        false
    }

    fn check_queue(&self) {
        let started_allocating = self.check_unallocated_regions();
        // if we didn't need to allocate any new regions, take care of
        // pending syscalls
        if !started_allocating {
            self.check_pending_syscalls();
        }
    }

    fn handle_syscall(&self, syscall: Syscall, processid: ProcessId) -> Result<(), ErrorCode> {
        match syscall {
            Syscall::GetSize => self.handle_get_size_syscall(processid),
            Syscall::Read { offset, length } => self.handle_read_syscall(processid, offset, length),
            Syscall::Write { offset, length } => self.handle_write_syscall(processid, offset, length),
        }
    }


    fn handle_get_size_syscall(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // How many bytes are accessible to each app
        let res = self.apps.enter(processid, |app, kernel_data| 
            match app.region {
                Some(region) => {
                    // clear pending syscall
                    app.pending_syscall = None;
                    // signal app with the result
                    kernel_data
                        .schedule_upcall(upcall::GET_SIZE_DONE, (into_statuscode(Ok(())), region.length, 0))
                        .ok();
                    Ok(())
                },
                None => Err(ErrorCode::FAIL)
            }
        )?;
        if res.is_ok() {
            self.check_queue();
        }
        res
    }

    fn handle_read_syscall(&self, processid: ProcessId, offset: usize, length: usize) -> Result<(), ErrorCode> {
        // Issue a read command
        self
            .enqueue_userspace_command(Command::Read { offset, length }, Some(processid))
    }

    fn handle_write_syscall(&self, processid: ProcessId, offset: usize, length: usize) -> Result<(), ErrorCode> {
        // Issue a write command
        self
            .enqueue_userspace_command(Command::Write { offset, length }, Some(processid))
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
                    let _ = match state {
                        RegionState::ReadHeader(action) => self.header_read_done(action),
                        _ => Err(ErrorCode::FAIL),
                    };
                }
                User::App { processid } => {
                    let res = self.apps.enter(processid, move |app, kernel_data| {
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
                                    return read_len;
                                })
                            })?;

                        // Replace the buffer we used to do this read.
                        self.buffer.replace(buffer);

                        // clear pending syscall
                        app.pending_syscall = None;
                        // And then signal the app.
                        kernel_data
                            .schedule_upcall(upcall::READ_DONE, (into_statuscode(Ok(())), read_len, 0))
                            .ok();
                        Ok::<(), kernel::process::Error>(())
                    });

                    if let Ok(closure_res) = res {
                        if closure_res.is_ok() {
                            self.check_queue();
                        }
                    }
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
                        RegionState::WriteHeader(processid, region) => {
                            // Now that we have written the header for the app we can store its region in its grant.
                            let _ = self.apps.enter(processid, |app, _kernel_data| {
                                // set region data in app's grant
                                app.region.replace(region);
                            });

                            // Update our metadata about where the next unallocated region is.
                            let next_header_addr = region.offset + region.length;
                            self.next_unallocated_region_header_address
                                .set(next_header_addr);

                            // Erase the userspace accessible content of the region
                            // before handing it off to an app.
                            let _ =
                                self.erase_region_content(processid, region.offset, region.length);
                        }
                        RegionState::EraseRegion {
                            processid,
                            next_erase_start,
                            remaining_bytes,
                        } => {
                            if remaining_bytes > 0 {
                                // we still have more to erase, so kick off another one
                                // where we left off
                                let _ = self.erase_region_content(
                                    processid,
                                    next_erase_start,
                                    remaining_bytes,
                                );
                            } else {
                                // done erasing entire region. 
                                self.check_pending_syscalls();
                            }
                        }
                        _ => {}
                    };
                }
                User::App { processid } => {
                    let res = self.apps.enter(processid, move |app, kernel_data| {
                        // clear pending syscall
                        app.pending_syscall = None;
                        // Notify app that its write has completed.
                        kernel_data
                            .schedule_upcall(upcall::WRITE_DONE, (into_statuscode(Ok(())), length, 0))
                            .ok();
                    });

                    if res.is_ok() {
                        self.check_queue();
                    }
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
        offset: usize,
        length: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            // For the get_bytes, read, and write syscalls we need to first
            // initialize the app's isolated nonvolatile storage.
            // This involves searching the storage area for an existing
            // region that belongs to this app. If we don't find an existing region
            // we allocate a new one.
            // Only once the initialization is complete, can we service
            // the original syscall. So, we can store the syscall data
            // in the app's grant and handle it when initialization finishes.
            1 | 2 | 3 => {
                let syscall = match command_num {
                    1 => Syscall::GetSize,
                    2 => Syscall::Read{offset, length},
                    3 => Syscall::Write{offset, length},
                    _ => return CommandReturn::failure(ErrorCode::NOSUPPORT) 
                };

                let res = if let Ok(has_region) = self.apps.enter(processid, |app, _kernel_data| app.region.is_some()) {
                    if has_region {
                        // already initialized this app's region, can directly handle syscall
                        self.handle_syscall(syscall, processid)
                    } else {
                        // need to initialize app region before handling syscall
                        self.init_app(syscall, processid)
                    }
                } else {
                    Err(ErrorCode::NOMEM)
                };

                match res {
                    Ok(_) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e)
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
