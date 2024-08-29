// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! This provides kernel and userspace access to nonvolatile memory.
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
//! Nonvolatile memory is reserved for each app when they explicitly call an
//! initialization syscall. Note that only verified apps can reserve regions
//! since this capsule uses the unique and persistent ShortID to identify
//! the app across reboots. See this page in the Tock book for how to
//! sign apps: <https://book.tockos.org/course/usb-security-key/key-hotp-access#signing-apps>
//!
//! Here is the sequence of events that happen when this initialization syscall is invoked:
//!  1. The capsule starts traversing a "linked-list" of app regions to find
//!     any existing regions that might already exist in storage for the app
//!     making the syscall. This traversal is possible due to headers that exist
//!     right before the start of each app's region. These headers describe the
//!     app that owns the region and size of the region. During traversal of these
//!     regions, the capsule uses the length to determine where the next region
//!     header starts.
//!  2. The capsule reads the header and performs a check to see if the region header
//!     is valid. If it is valid, we move to step 3. If not, we move to step 4.
//!  3. If the capsule finds a valid region header, it tries to save the informaton of the region
//!     to the grant of the app that has the same ShortID of the header it just read. It's
//!     possible that the app previously owned a region but currently isn't running on the
//!     board. If the app is running, it saves the location and length of the region to the
//!     app's grant memory where it can be retrieved later during reads/writes. Then it
//!     continues to traverse the "list" of regions by using length of the previous region
//!     to determine the start of the next one.
//!  4. If the capsule finds an invalid region header, it knows that it has
//!     reached the area of storage where regions have not been initialized.
//!     Therefore, it will begin to allocate regions at the end of the "list"
//!     for any apps that made initialization requests and don't already have a region.
//!  4. Once an app is known to have a valid region (either by discovering it during
//!     traversal or allocating a new one), initialization completes and the app
//!     receives an upcall. Now it can go ahead and start reading/writing only
//!     within its isolated region.
//!
//! However, the kernel accessible memory does not have to be the same range
//! as the userspace accessible address space. The kernel memory can overlap
//! if desired, or can be a completely separate range.
//!
//! Here is a diagram of the expected stack with this capsule:
//! Boxes are components and between the boxes are the traits that are the
//! interfaces between components. This capsule provides both a kernel and
//! userspace interface.
//!
//! ```text
//! +--------------------------------------------+     +--------------+
//! |                                            |     |              |
//! |                  kernel                    |     |  userspace   |
//! |                                            |     |              |
//! +--------------------------------------------+     +--------------+
//!  hil::nonvolatile_storage::NonvolatileStorage       kernel::Driver
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
//!     ╒════════ ← Start of kernel region
//!     │
//!     │
//!     │
//!     ╞════════ ← Start of userspace region
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
//!     ╘════════ ← End of storage
//! ```

//!
//!
//! Example instantiation:
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let nonvolatile_storage = static_init!(
//!     capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
//!     capsules::nonvolatile_storage_driver::NonvolatileStorage::new(
//!         fm25cl,                      // The underlying storage driver.
//!         board_kernel.create_grant(&grant_cap),     // Storage for app-specific state.
//!         3000,                        // The byte start address for the userspace
//!                                      // accessible memory region.
//!         2000,                        // The length of the userspace region.
//!         0,                           // The byte start address of the region
//!                                      // that is accessible by the kernel.
//!         3000,                        // The length of the kernel region.
//!         2048,                        // The length of each region accessible to each app.
//!         &mut [u8; capsules::nonvolatile_storage_driver::BUF_LEN],    // buffer for reading/writing
//!                                                                      // userpace data
//!         &mut [u8; capsules::nonvolatile_storage_driver::REGION_HEADER_LEN])); // buffer for reading/writing
//!                                                                               // header data
//! hil::nonvolatile_storage::NonvolatileStorage::set_client(fm25cl, nonvolatile_storage);
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;

pub const DRIVER_NUM: usize = driver::NUM::NvmStorage as usize;

/// IDs for subscribed upcalls.
mod upcall {
    /// Read done callback.
    pub const READ_DONE: usize = 0;
    /// Write done callback.
    pub const WRITE_DONE: usize = 1;
    /// Initialization done callback.
    pub const INIT_DONE: usize = 2;
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
    /// starts. Note that this is the address following the region's header.
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
            version_and_length: version_and_length,
            shortid: shortid,
            xor: xor,
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
            version_and_length: version_and_length,
            shortid: shortid,
            xor: xor,
        })
    }

    fn to_bytes(&self) -> [u8; REGION_HEADER_LEN] {
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
        // extract the 24 least significant bits from
        // the concatenated verion and length
        self.version_and_length & 0x00ffffff
    }
}

// Enough space for a buffer to be used for reading/writing userspace data
pub const BUF_LEN: usize = 512;

// Enough space for a buffer to be used for holding zeroes that are used
// to
pub const REGION_ERASE_BUF_LEN: usize = 512;

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
pub enum NonvolatileCommand {
    UserspaceRead,
    UserspaceWrite,
    HeaderRead(usize),
    HeaderWrite(ProcessId, AppRegion),
    RegionErase(ProcessId),
    KernelRead,
    KernelWrite,
}

#[derive(Clone, Copy)]
pub enum NonvolatileUser {
    App { processid: ProcessId },
    RegionManager(RegionState),
    Kernel,
}

pub struct App {
    pending_command: bool,
    command: NonvolatileCommand,
    offset: usize,
    length: usize,
    /// if this certain app has previously requested to initialize
    /// its nonvolatile storage.
    has_requested_region: bool,
    /// describe the location and size of an app's region (if it has
    /// been initialized)
    region: Option<AppRegion>,
}

impl Default for App {
    fn default() -> App {
        App {
            pending_command: false,
            command: NonvolatileCommand::UserspaceRead,
            offset: 0,
            length: 0,
            has_requested_region: false,
            region: None,
        }
    }
}

pub struct NonvolatileStorage<'a> {
    // The underlying physical storage device.
    driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'a>,
    // Per-app state.
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    // Internal buffer for copying appslices into.
    buffer: TakeCell<'static, [u8]>,
    // What issued the currently executing call. This can be an app or the kernel.
    current_user: OptionalCell<NonvolatileUser>,

    // The first byte that is accessible from userspace.
    userspace_start_address: usize,
    // How many bytes allocated to userspace.
    userspace_length: usize,
    // The first byte that is accessible from the kernel.
    kernel_start_address: usize,
    // How many bytes allocated to kernel.
    kernel_length: usize,

    // Optional client for the kernel. Only needed if the kernel intends to use
    // this nonvolatile storage.
    kernel_client: OptionalCell<&'a dyn hil::nonvolatile_storage::NonvolatileStorageClient>,
    // Whether the kernel is waiting for a read/write.
    kernel_pending_command: Cell<bool>,
    // Whether the kernel wanted a read/write.
    kernel_command: Cell<NonvolatileCommand>,
    // Holder for the buffer passed from the kernel in case we need to wait.
    kernel_buffer: TakeCell<'static, [u8]>,
    // How many bytes to read/write from the kernel buffer.
    kernel_readwrite_length: Cell<usize>,
    // Where to read/write from the kernel request.
    kernel_readwrite_address: Cell<usize>,

    /// static buffer to store region headers
    /// before they get written to nonvolatile storage
    header_buffer: TakeCell<'static, [u8]>,

    // How many bytes each app should be allocted. Configurable at capsule
    // creation time.
    app_region_size: usize,

    // Absolute address of the header of the next region of userspace
    // that's not allocated to an app yet. Each time an app uses this
    // capsule, a new region of storage will be handed out and this
    // address will point to the header of a new unallocated region.
    next_unallocated_region_header_address: OptionalCell<usize>,

    // static buffer to store zeroes to write to regions when
    // they need to be erased
    region_erase_buffer: TakeCell<'static, [u8]>,
}

impl<'a> NonvolatileStorage<'a> {
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
        kernel_start_address: usize,
        kernel_length: usize,
        app_region_size: usize,
        buffer: &'static mut [u8],
        header_buffer: &'static mut [u8],
        region_erase_buffer: &'static mut [u8],
    ) -> NonvolatileStorage<'a> {
        NonvolatileStorage {
            driver,
            apps: grant,
            buffer: TakeCell::new(buffer),
            current_user: OptionalCell::empty(),
            userspace_start_address,
            userspace_length,
            kernel_start_address,
            kernel_length,
            kernel_client: OptionalCell::empty(),
            kernel_pending_command: Cell::new(false),
            kernel_command: Cell::new(NonvolatileCommand::KernelRead),
            kernel_buffer: TakeCell::empty(),
            kernel_readwrite_length: Cell::new(0),
            kernel_readwrite_address: Cell::new(0),
            app_region_size: app_region_size,
            header_buffer: TakeCell::new(header_buffer),
            next_unallocated_region_header_address: OptionalCell::empty(),
            region_erase_buffer: TakeCell::new(region_erase_buffer),
        }
    }

    // App-level initialization that allocates a region for an app or fetches
    // an app's existing region from nonvolatile storage
    fn init_app(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Signal that this app requested a storage region. If it isn't
        // allocated immediately, it will be handled after previous requests
        // are handled.
        self.apps.enter(processid, |app, _kernel_data| {
            app.has_requested_region = true;
        })?;

        // Start traversing the storage regions to find where the requesting app's
        // storage region is located. If it doesn't exist, a new one will be allocated
        self.start_region_traversal()
    }

    // Start reading app region headers.
    fn start_region_traversal(&self) -> Result<(), ErrorCode> {
        self.read_region_header(self.userspace_start_address)
    }

    // Find an app that previously requested a nonvolatile
    // region and doesn't have one assigned.
    fn find_app_to_allocate_region(&self) -> Option<ProcessId> {
        for app in self.apps.iter() {
            let processid = app.processid();
            // find the first app that needs to be allocated
            let app_to_allocate = app.enter(|app, _kernel_data| {
                // if the app previously requested a region and
                // hasn't been allocated one yet
                if app.has_requested_region && app.region.is_none() {
                    Some(processid)
                } else {
                    // no apps need to be allocated
                    None
                }
            });

            if app_to_allocate.is_some() {
                return app_to_allocate;
            }
        }
        None
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

        self.apps
            .enter(processid, |app, _kernel_data| {
                // if the app previously requested a region and
                // hasn't been allocated one yet
                if app.has_requested_region && app.region.is_none() {
                    let region = AppRegion {
                        version: CURRENT_HEADER_VERSION,
                        // Have this region start where all the existing regions end.
                        // Note that the app's actual region starts after the region header.
                        offset: new_header_addr + REGION_HEADER_LEN,
                        // new regions get handed the same size. this can be
                        // configured when the capsule is created.
                        length: self.app_region_size,
                    };

                    // fail if new region is outside userpace area
                    if region.offset > self.userspace_start_address + self.userspace_length
                        || region.offset + region.length
                            > self.userspace_start_address + self.userspace_length
                    {
                        return Err(ErrorCode::NOMEM);
                    }

                    let Some(header) = AppRegionHeader::new(region.version, shortid, region.length)
                    else {
                        return Err(ErrorCode::FAIL);
                    };

                    // write this new region header to the end of the existing ones
                    self.write_region_header(processid, &region, &header, new_header_addr)
                } else {
                    // this app never requested to be allocated or its
                    // region was already allocated
                    Ok(())
                }
            })
            .unwrap_or(Err(ErrorCode::FAIL))
    }

    // Read the header of an app's storage region. The region_header_address argument
    // describes the start of the **header** and not the usable region itself.
    fn read_region_header(&self, region_header_address: usize) -> Result<(), ErrorCode> {
        self.enqueue_command(
            NonvolatileCommand::HeaderRead(region_header_address),
            region_header_address,
            REGION_HEADER_LEN,
            None,
        )
    }

    fn write_region_header(
        &self,
        processid: ProcessId,
        region: &AppRegion,
        region_header: &AppRegionHeader,
        region_header_address: usize,
    ) -> Result<(), ErrorCode> {
        let header_slice = region_header.to_bytes();

        self.header_buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
            for (i, c) in buffer.iter_mut().enumerate() {
                *c = header_slice[i];
            }

            Ok(())
        })?;

        self.enqueue_command(
            NonvolatileCommand::HeaderWrite(processid, *region),
            region_header_address,
            REGION_HEADER_LEN,
            None,
        )
    }

    fn erase_region_content(
        &self,
        processid: ProcessId,
        region: AppRegion,
    ) -> Result<(), ErrorCode> {
        self.region_erase_buffer
            .map_or(Err(ErrorCode::NOMEM), |buffer| {
                // clear the erase buffer in case there was any nonzero value there
                for c in buffer.iter_mut() {
                    *c = 0;
                }
                Ok(())
            })?;

        self.enqueue_command(
            NonvolatileCommand::RegionErase(processid),
            region.offset + REGION_HEADER_LEN,
            region.length,
            None,
        )
    }

    fn region_erase_done(
        &self,
        processid: ProcessId,
        next_erase_start: usize,
        remaining_bytes: usize,
    ) -> Result<(), ErrorCode> {
        if remaining_bytes > 0 {
            // we still have more to erase, so kick off another one
            self.enqueue_command(
                NonvolatileCommand::RegionErase(processid),
                next_erase_start,
                remaining_bytes,
                None,
            )
        } else {
            // done erasing entire region
            self.apps
                .enter(processid, |_app, kernel_data| {
                    // region is erased and we're ready to let the app
                    // know that it's region is ready
                    kernel_data
                        .schedule_upcall(upcall::INIT_DONE, (0, 0, 0))
                        .ok();
                    Ok::<(), ErrorCode>(())
                })
                .unwrap_or_else(|err| Err(err.into()))?;

            // check for apps that haven't had regions allocated
            // for them after requesting one
            match self.find_app_to_allocate_region() {
                Some(processid) => self.allocate_app_region(processid),
                None => Ok(()),
            }
        }
    }

    fn header_read_done(&self, region_header_address: usize) -> Result<(), ErrorCode> {
        // reconstruct header from bytes we just read
        let header = self.header_buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
            let header_buffer = buffer.try_into().or(Err(ErrorCode::FAIL))?;
            AppRegionHeader::from_bytes(header_buffer).ok_or(ErrorCode::FAIL)
        })?;

        // If a header is invalid, we've reached the end
        // of all previously allocated regions.
        if header.is_valid() {
            // Find the app with the corresponding shortid.
            for app in self.apps.iter() {
                // skip an app if it doesn't have the proper storage permissions
                let shortid = match app.processid().get_storage_permissions() {
                    Some(perms) => match perms.get_write_id() {
                        Some(write_id) => write_id,
                        None => continue,
                    },
                    None => continue,
                };
                if shortid == header.shortid {
                    app.enter(|app, kernel_data| {
                        // only populate region and signal app that explicitly
                        // requested to initialize storage
                        if app.has_requested_region && app.region.is_none() {
                            let version = header.version().ok_or(ErrorCode::FAIL)?;
                            app.region.replace(AppRegion {
                                version: version,
                                // the app's actual region starts after the
                                // region header
                                offset: region_header_address + REGION_HEADER_LEN,
                                length: header.length() as usize,
                            });

                            kernel_data
                                .schedule_upcall(
                                    upcall::INIT_DONE,
                                    (kernel::errorcode::into_statuscode(Ok(())), 0, 0),
                                )
                                .ok();
                        }
                        Ok::<(), ErrorCode>(())
                    })?;

                    break;
                }
            }

            let next_header_address =
                region_header_address + REGION_HEADER_LEN + header.length() as usize;
            self.read_region_header(next_header_address)
        } else {
            // save this region header address so that we can allocate new regions
            // here later
            self.next_unallocated_region_header_address
                .set(region_header_address);

            // start allocating any outstanding region allocation requests
            match self.find_app_to_allocate_region() {
                Some(processid) => self.allocate_app_region(processid),
                None => Ok(()),
            }
        }
    }

    fn header_write_done(&self, processid: ProcessId, region: AppRegion) -> Result<(), ErrorCode> {
        self.apps
            .enter(processid, |app, kernel_data| {
                // set region data in app's grant
                app.region.replace(region);

                // bump the start of the unallocated regions
                let curr_header_addr = self
                    .next_unallocated_region_header_address
                    .get()
                    .ok_or(ErrorCode::FAIL)?;

                let next_header_addr = curr_header_addr + REGION_HEADER_LEN + self.app_region_size;

                self.next_unallocated_region_header_address
                    .set(next_header_addr);

                // erase the userpace accessible content of the region
                // before handing it off to an app
                self.erase_region_content(processid, region)
            })
            .unwrap_or_else(|err| Err(err.into()))
    }

    fn check_userspace_perms(
        &self,
        processid: Option<ProcessId>,
        command: NonvolatileCommand,
    ) -> Result<(), ErrorCode> {
        processid.map_or(Err(ErrorCode::FAIL), |processid| {
            let perms = processid
                .get_storage_permissions()
                .ok_or(ErrorCode::NOSUPPORT)?;
            let write_id = perms.get_write_id().ok_or(ErrorCode::NOSUPPORT)?;
            match command {
                NonvolatileCommand::UserspaceRead => {
                    if !perms.check_read_permission(write_id) {
                        Err(ErrorCode::NOSUPPORT)
                    } else {
                        Ok(())
                    }
                }
                NonvolatileCommand::UserspaceWrite => {
                    if !perms.check_modify_permission(write_id) {
                        Err(ErrorCode::NOSUPPORT)
                    } else {
                        Ok(())
                    }
                }
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

    fn check_kernel_access(&self, offset: usize, length: usize) -> Result<(), ErrorCode> {
        // Because the kernel uses the NonvolatileStorage interface,
        // its calls are absolute addresses.
        if offset < self.kernel_start_address
            || offset >= self.kernel_start_address + self.kernel_length
            || length > self.kernel_length
            || offset + length > self.kernel_start_address + self.kernel_length
        {
            return Err(ErrorCode::INVAL);
        }

        Ok(())
    }

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_command(
        &self,
        command: NonvolatileCommand,
        offset: usize,
        length: usize,
        processid: Option<ProcessId>,
    ) -> Result<(), ErrorCode> {
        // Do different bounds check depending on userpace vs kernel accesses
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                self.check_userspace_access(offset, length, processid)?;
                self.check_userspace_perms(processid, command)?;
            }
            NonvolatileCommand::HeaderRead(_)
            | NonvolatileCommand::HeaderWrite(_, _)
            | NonvolatileCommand::RegionErase(_) => {
                self.check_header_access(offset, length)?;
            }
            NonvolatileCommand::KernelRead | NonvolatileCommand::KernelWrite => {
                self.check_kernel_access(offset, length)?;
            }
        }

        // Do very different actions if this is a call from userspace
        // or from the kernel.
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                processid.map_or(Err(ErrorCode::FAIL), |processid| {
                    self.apps
                        .enter(processid, |app, kernel_data| {
                            // Get the length of the correct allowed buffer.
                            let allow_buf_len = match command {
                                NonvolatileCommand::UserspaceRead => kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .map_or(0, |read| read.len()),
                                NonvolatileCommand::UserspaceWrite => kernel_data
                                    .get_readonly_processbuffer(ro_allow::WRITE)
                                    .map_or(0, |read| read.len()),
                                _ => 0,
                            };

                            // Check that it exists.
                            if allow_buf_len == 0 || self.buffer.is_none() {
                                return Err(ErrorCode::RESERVE);
                            }

                            // Shorten the length if the application gave us nowhere to
                            // put it.
                            let active_len = cmp::min(length, allow_buf_len);

                            // First need to determine if we can execute this or must
                            // queue it.
                            if self.current_user.is_none() {
                                // No app is currently using the underlying storage.
                                // Mark this app as active, and then execute the command.
                                self.current_user.set(NonvolatileUser::App { processid });

                                // Need to copy bytes if this is a write!
                                if command == NonvolatileCommand::UserspaceWrite {
                                    let _ = kernel_data
                                        .get_readonly_processbuffer(ro_allow::WRITE)
                                        .and_then(|write| {
                                            write.enter(|app_buffer| {
                                                self.buffer.map(|kernel_buffer| {
                                                    // Check that the internal buffer and the buffer that was
                                                    // allowed are long enough.
                                                    let write_len =
                                                        cmp::min(active_len, kernel_buffer.len());

                                                    let d = &app_buffer[0..write_len];
                                                    for (i, c) in kernel_buffer[0..write_len]
                                                        .iter_mut()
                                                        .enumerate()
                                                    {
                                                        *c = d[i].get();
                                                    }
                                                });
                                            })
                                        });
                                }

                                // Fail if the app doesn't have a region assigned to it.
                                let Some(app_region) = &app.region else {
                                    return Err(ErrorCode::FAIL);
                                };

                                // Note that the given offset for this command is with respect to
                                // the app's region address space. This means that userspace accesses
                                // start at 0 which is the start of the app's region.
                                self.userspace_call_driver(
                                    command,
                                    app_region.offset + offset,
                                    active_len,
                                )
                            } else {
                                // Some app is using the storage, we must wait.
                                if app.pending_command {
                                    // No more room in the queue, nowhere to store this
                                    // request.
                                    Err(ErrorCode::NOMEM)
                                } else {
                                    // We can store this, so lets do it.
                                    app.pending_command = true;
                                    app.command = command;
                                    app.offset = offset;
                                    app.length = active_len;
                                    Ok(())
                                }
                            }
                        })
                        .unwrap_or_else(|err| Err(err.into()))
                })
            }
            NonvolatileCommand::HeaderRead(_) | NonvolatileCommand::HeaderWrite(_, _) => {
                self.header_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |header_buffer| {
                        let active_len = cmp::min(length, header_buffer.len());

                        // Check if there is something going on.
                        if self.current_user.is_none() {
                            match command {
                                NonvolatileCommand::HeaderRead(address) => {
                                    self.current_user.set(NonvolatileUser::RegionManager(
                                        RegionState::ReadHeader(address),
                                    ));
                                    self.driver.read(header_buffer, offset, active_len)
                                }
                                NonvolatileCommand::HeaderWrite(processid, region) => {
                                    self.current_user.set(NonvolatileUser::RegionManager(
                                        RegionState::WriteHeader(processid, region),
                                    ));
                                    self.driver.write(header_buffer, offset, active_len)
                                }
                                _ => Err(ErrorCode::FAIL),
                            }
                        } else {
                            Err(ErrorCode::BUSY)
                        }
                    })
            }
            NonvolatileCommand::RegionErase(_) => {
                self.region_erase_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |buffer| {
                        let active_len = cmp::min(length, buffer.len());

                        // how many more bytes to erase after this operation
                        let remaining_len = if length > buffer.len() {
                            length - buffer.len()
                        } else {
                            0
                        };

                        // Check if there is something going on.
                        if self.current_user.is_none() {
                            match command {
                                NonvolatileCommand::RegionErase(processid) => {
                                    self.current_user.set(NonvolatileUser::RegionManager(
                                        // need to pass on where the next erase should start
                                        // how long it should be.
                                        RegionState::EraseRegion {
                                            processid: processid,
                                            next_erase_start: offset + active_len,
                                            remaining_bytes: remaining_len,
                                        },
                                    ));
                                    self.driver.write(buffer, offset, active_len)
                                }
                                _ => Err(ErrorCode::FAIL),
                            }
                        } else {
                            Err(ErrorCode::BUSY)
                        }
                    })
            }

            NonvolatileCommand::KernelRead | NonvolatileCommand::KernelWrite => {
                self.kernel_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |kernel_buffer| {
                        let active_len = cmp::min(length, kernel_buffer.len());

                        // Check if there is something going on.
                        if self.current_user.is_none() {
                            // Nothing is using this, lets go!
                            self.current_user.set(NonvolatileUser::Kernel);

                            match command {
                                NonvolatileCommand::KernelRead => {
                                    self.driver.read(kernel_buffer, offset, active_len)
                                }
                                NonvolatileCommand::KernelWrite => {
                                    self.driver.write(kernel_buffer, offset, active_len)
                                }
                                _ => Err(ErrorCode::FAIL),
                            }
                        } else {
                            if self.kernel_pending_command.get() {
                                Err(ErrorCode::NOMEM)
                            } else {
                                self.kernel_pending_command.set(true);
                                self.kernel_command.set(command);
                                self.kernel_readwrite_length.set(active_len);
                                self.kernel_readwrite_address.set(offset);
                                self.kernel_buffer.replace(kernel_buffer);
                                Ok(())
                            }
                        }
                    })
            }
        }
    }

    fn userspace_call_driver(
        &self,
        command: NonvolatileCommand,
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

                // self.current_app.set(Some(processid));
                match command {
                    NonvolatileCommand::UserspaceRead => {
                        self.driver.read(buffer, physical_address, active_len)
                    }
                    NonvolatileCommand::UserspaceWrite => {
                        self.driver.write(buffer, physical_address, active_len)
                    }
                    _ => Err(ErrorCode::FAIL),
                }
            })
    }

    fn check_queue(&self) {
        // Check if there are any pending events.
        if self.kernel_pending_command.get() {
            self.kernel_buffer.take().map(|kernel_buffer| {
                self.kernel_pending_command.set(false);
                self.current_user.set(NonvolatileUser::Kernel);

                match self.kernel_command.get() {
                    NonvolatileCommand::KernelRead => self.driver.read(
                        kernel_buffer,
                        self.kernel_readwrite_address.get(),
                        self.kernel_readwrite_length.get(),
                    ),
                    NonvolatileCommand::KernelWrite => self.driver.write(
                        kernel_buffer,
                        self.kernel_readwrite_address.get(),
                        self.kernel_readwrite_length.get(),
                    ),
                    _ => Err(ErrorCode::FAIL),
                }
            });
        } else {
            // If the kernel is not requesting anything, check all of the apps.
            for cntr in self.apps.iter() {
                let processid = cntr.processid();
                let started_command = cntr.enter(|app, _| {
                    if app.pending_command {
                        app.pending_command = false;
                        self.current_user.set(NonvolatileUser::App { processid });
                        if let Ok(()) =
                            self.userspace_call_driver(app.command, app.offset, app.length)
                        {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                if started_command {
                    break;
                }
            }
        }
    }
}

/// This is the callback client for the underlying physical storage driver.
impl hil::nonvolatile_storage::NonvolatileStorageClient for NonvolatileStorage<'_> {
    fn read_done(&self, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_user.take().map(|user| {
            match user {
                NonvolatileUser::Kernel => {
                    self.kernel_client.map(move |client| {
                        client.read_done(buffer, length);
                    });
                }
                NonvolatileUser::RegionManager(state) => {
                    self.header_buffer.replace(buffer);
                    let res = match state {
                        RegionState::ReadHeader(action) => self.header_read_done(action),
                        _ => Err(ErrorCode::FAIL),
                    };
                }
                NonvolatileUser::App { processid } => {
                    let _ = self.apps.enter(processid, move |_, kernel_data| {
                        // Need to copy in the contents of the buffer
                        let _ = kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .and_then(|read| {
                                read.mut_enter(|app_buffer| {
                                    let read_len = cmp::min(app_buffer.len(), length);

                                    let d = &app_buffer[0..read_len];
                                    for (i, c) in buffer[0..read_len].iter().enumerate() {
                                        d[i].set(*c);
                                    }
                                })
                            });

                        // Replace the buffer we used to do this read.
                        self.buffer.replace(buffer);

                        // And then signal the app.
                        kernel_data
                            .schedule_upcall(upcall::READ_DONE, (length, 0, 0))
                            .ok();
                    });
                }
            }
        });

        self.check_queue();
    }

    fn write_done(&self, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_user.take().map(|user| {
            match user {
                NonvolatileUser::Kernel => {
                    self.kernel_client.map(move |client| {
                        client.write_done(buffer, length);
                    });
                }
                NonvolatileUser::RegionManager(state) => {
                    let res = match state {
                        RegionState::WriteHeader(processid, region) => {
                            self.header_buffer.replace(buffer);
                            let write_res = self.header_write_done(processid, region);

                            // signal app if we fail to write its region header
                            if write_res.is_err() {
                                let _ = self.apps.enter(processid, |_, kernel_data| {
                                    kernel_data
                                        .schedule_upcall(
                                            upcall::INIT_DONE,
                                            (kernel::errorcode::into_statuscode(write_res), 0, 0),
                                        )
                                        .ok();
                                });
                            }
                            write_res
                        }
                        RegionState::EraseRegion {
                            processid,
                            next_erase_start,
                            remaining_bytes,
                        } => {
                            self.region_erase_buffer.replace(buffer);
                            self.region_erase_done(processid, next_erase_start, remaining_bytes)
                        }
                        _ => Err(ErrorCode::FAIL),
                    };
                }
                NonvolatileUser::App { processid } => {
                    let _ = self.apps.enter(processid, move |_app, kernel_data| {
                        // Replace the buffer we used to do this write.
                        self.buffer.replace(buffer);

                        // And then signal the app.
                        kernel_data
                            .schedule_upcall(upcall::WRITE_DONE, (length, 0, 0))
                            .ok();
                    });
                }
            }
        });

        self.check_queue();
    }
}

/// Provide an interface for the kernel.
impl<'a> hil::nonvolatile_storage::NonvolatileStorage<'a> for NonvolatileStorage<'a> {
    fn set_client(&self, client: &'a dyn hil::nonvolatile_storage::NonvolatileStorageClient) {
        self.kernel_client.set(client);
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), ErrorCode> {
        self.kernel_buffer.replace(buffer);
        self.enqueue_command(NonvolatileCommand::KernelRead, address, length, None)
    }

    fn write(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), ErrorCode> {
        self.kernel_buffer.replace(buffer);
        self.enqueue_command(NonvolatileCommand::KernelWrite, address, length, None)
    }
}

/// Provide an interface for userland.
impl SyscallDriver for NonvolatileStorage<'_> {
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
    /// - `4`: Initialize an app's nonvolatile_storage.
    fn command(
        &self,
        command_num: usize,
        offset: usize,
        length: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            1 => {
                // How many bytes are accessible to each app
                let res = self.apps.enter(processid, |app, _kernel_data| app.region);

                // handle case where we fail to enter grant
                res.map_or(CommandReturn::failure(ErrorCode::NOMEM), |region| {
                    // handle case where app's region is not assigned
                    region.map_or(CommandReturn::failure(ErrorCode::FAIL), |region| {
                        // TODO: Would break on 64-bit platforms
                        CommandReturn::success_u32(region.length as u32)
                    })
                })
            }

            2 => {
                // Issue a read command
                let res = self.enqueue_command(
                    NonvolatileCommand::UserspaceRead,
                    offset,
                    length,
                    Some(processid),
                );

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            3 => {
                // Issue a write command
                let res = self.enqueue_command(
                    NonvolatileCommand::UserspaceWrite,
                    offset,
                    length,
                    Some(processid),
                );

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }
            4 => {
                // Initialize an app's storage region
                let res = self.init_app(processid);

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
