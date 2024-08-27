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
//!     is valid. If it is valid, we move to step 3. If not, we move to step 4. This
//!     header validation is done by XORing the three fields of the header together
//!     (a usize "magic", a u32 shortid, and a usize region length value) and checking
//!     if their result matches one of the known HeaderVersion numbers. This works
//!     since the "magic" value is previously set to the XOR of a HeaderVerion number, the
//!     shortid of the app that owns the region and length of the region.
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
//!     │ App 1's magic value (usize) = HEADER_V1 ^ ShortID ^ length
//!     │ App 1's ShortID (u32)  
//!     │ region 1 length (usize)
//!     ├──────── ← Start of App 1's Region          ═╗
//!     │                                             ║
//!     │
//!     │                                            region 1
//!     │                                            length
//!     │
//!     │                                             ║
//!     │                                            ═╝
//!     ├──────── ← Start of App 2's region header   
//!     │ App 2's magic value (usize) = HEADER_V1 ^ ShortID ^ length
//!     │ App 2's ShortID (u32)                      
//!     │ region 2 length (usize)                    
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
use core::num::NonZeroU32;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::process::ShortId;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{debug, hil};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;

/// enable/disable debug prints
const DEBUG: bool = false;

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

// Constants to distinguish what version of this capsule
// a given header was created with.
const V1: usize = 0x2FA7B3;

// Note that the magic header is versioned to maintain
// compatibility across multiple capsule versions.
#[derive(Clone, Copy, PartialEq, Debug)]
enum HeaderVersion {
    V1,
}

impl HeaderVersion {
    fn value(&self) -> usize {
        match self {
            HeaderVersion::V1 => V1,
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
    /// Magic value to determine if a region is valid and what version
    /// of this capsule it was created with. The value stored in
    /// mmeory is an XOR of the header version number, shortid, and length.
    magic: usize,
    /// Unique per-app identifier. This comes from
    /// the Fixed variant of the ShortID type.
    shortid: u32,
    /// How many bytes allocated to a certain app.
    /// Note that the size of the region header is not
    /// included in this length value.
    length: usize,
}
// Enough space to store the magic(usize), shortid (u32) and length (usize) to nonvolatile storage
pub const REGION_HEADER_LEN: usize =
    core::mem::size_of::<usize>() + core::mem::size_of::<u32>() + core::mem::size_of::<usize>();

impl AppRegionHeader {
    fn new(version: HeaderVersion, shortid: u32, length: usize) -> Self {
        let magic = version.value() ^ (shortid as usize) ^ length;
        AppRegionHeader {
            magic,
            shortid,
            length,
        }
    }

    fn is_valid(&self) -> Option<HeaderVersion> {
        let version = self.magic ^ (self.shortid as usize) ^ self.length;
        match version {
            V1 => Some(HeaderVersion::V1),
            _ => None,
        }
    }
}

// Enough space for a buffer to be used for reading/writing userspace data
pub const BUF_LEN: usize = 512;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HeaderState {
    Read(usize),
    Write(ProcessId, AppRegion),
}

#[derive(Clone, Copy, PartialEq)]
pub enum NonvolatileCommand {
    UserspaceRead,
    UserspaceWrite,
    HeaderRead(usize),
    HeaderWrite(ProcessId, AppRegion),
    KernelRead,
    KernelWrite,
}

#[derive(Clone, Copy)]
pub enum NonvolatileUser {
    App { processid: ProcessId },
    HeaderManager(HeaderState),
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

    // Start reading app region headers. The first read will be from the region immediately
    // following the magic header. See the storage layout diagram at the top of this file.
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

        // Apps must have a fixed ShortID value. LocallyUnique apps
        // are not allowed since we need a unique fixed ID to write to
        // storage and use to identify apps.
        let ShortId::Fixed(shortid) = processid.short_app_id() else {
            return Err(ErrorCode::NOSUPPORT);
        };

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
                        return Err(ErrorCode::FAIL);
                    }

                    let header = AppRegionHeader::new(region.version, shortid.get(), region.length);

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
        if DEBUG {
            debug!(
                "[NONVOLATILE_STORAGE_DRIVER]: Reading region header from {:#x}",
                region_header_address
            );
        }
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
        if DEBUG {
            debug!(
                "[NONVOLATILE_STORAGE_DRIVER]: Writing region header to {:#x}",
                region_header_address
            );
        }

        let magic_slice = usize::to_le_bytes(region_header.magic);
        let owner_slice = u32::to_le_bytes(region_header.shortid);
        let length_slice = usize::to_le_bytes(region_header.length);

        self.header_buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
            // copy magic value
            let magic_start_idx = 0;
            let magic_end_idx = core::mem::size_of::<usize>();
            for (i, c) in buffer[magic_start_idx..magic_end_idx]
                .iter_mut()
                .enumerate()
            {
                *c = magic_slice[i];
            }

            // copy region owner
            let owner_start_idx = magic_end_idx;
            let owner_end_idx = owner_start_idx + core::mem::size_of::<u32>();
            for (i, c) in buffer[owner_start_idx..owner_end_idx]
                .iter_mut()
                .enumerate()
            {
                *c = owner_slice[i];
            }

            // copy region length
            let length_start_idx = owner_end_idx;
            let length_end_idx = length_start_idx + core::mem::size_of::<usize>();
            for (i, c) in buffer[length_start_idx..length_end_idx]
                .iter_mut()
                .enumerate()
            {
                *c = length_slice[i];
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

    fn header_read_done(&self, region_header_address: usize) -> Result<(), ErrorCode> {
        // reconstruct header from bytes we just read
        let header = self.header_buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
            // convert first part of header buffer to magic value
            let magic_start_idx = 0;
            let magic_end_idx = core::mem::size_of::<usize>();
            let magic_slice = buffer[magic_start_idx..magic_end_idx]
                .try_into()
                .or(Err(ErrorCode::FAIL))?;
            let magic = usize::from_le_bytes(magic_slice);

            // convert next part of header buffer to region owner
            let owner_start_idx = magic_end_idx;
            let owner_end_idx = owner_start_idx + core::mem::size_of::<u32>();
            let owner_slice = buffer[owner_start_idx..owner_end_idx]
                .try_into()
                .or(Err(ErrorCode::FAIL))?;
            let owner = u32::from_le_bytes(owner_slice);

            // convert next part of header buffer to region length
            let length_start_idx = owner_end_idx;
            let length_end_idx = length_start_idx + core::mem::size_of::<usize>();
            let length_slice = buffer[length_start_idx..length_end_idx]
                .try_into()
                .or(Err(ErrorCode::FAIL))?;
            let length = usize::from_le_bytes(length_slice);

            Ok(AppRegionHeader {
                magic: magic,
                shortid: owner,
                length: length,
            })
        })?;

        // If a header is invalid, we've reached the end
        // of all previously allocated regions.
        match header.is_valid() {
            Some(version) => {
                let shortid =
                    ShortId::Fixed(NonZeroU32::new(header.shortid).ok_or(ErrorCode::NOSUPPORT)?);

                // Find the app with the corresponding shortid.
                for app in self.apps.iter() {
                    if app.processid().short_app_id() == shortid {
                        app.enter(|app, kernel_data| {
                            // only populate region and signal app that explicitly
                            // requested to initialize storage
                            if app.has_requested_region && app.region.is_none() {
                                app.region.replace(AppRegion {
                                    version,
                                    // the app's actual region starts after the
                                    // region header
                                    offset: region_header_address + REGION_HEADER_LEN,
                                    length: header.length,
                                });

                                kernel_data
                                    .schedule_upcall(
                                        upcall::INIT_DONE,
                                        (kernel::errorcode::into_statuscode(Ok(())), 0, 0),
                                    )
                                    .ok();
                            }
                        });

                        break;
                    }
                }

                let next_header_address = region_header_address + REGION_HEADER_LEN + header.length;
                self.read_region_header(next_header_address)
            }
            None => {
                if DEBUG {
                    debug!("[NONVOLATILE_STORAGE_DRIVER]: Read invalid region header. Stopping region traversal. {:#x}", region_header_address);

                    for app in self.apps.iter() {
                        match app.processid().short_app_id() {
                            ShortId::Fixed(id) => {
                                debug!("[NONVOLATILE_STORAGE_DRIVER]: App shortid: {:#x}", id)
                            }
                            ShortId::LocallyUnique => {
                                debug!("[NONVOLATILE_STORAGE_DRIVER]: App shortid: none")
                            }
                        }

                        app.enter(|app, _kernel_data| match app.region {
                            Some(region) => debug!(
                                "\tStorage region:\n\toffset: {:#x}\n\tlength: {:#x}",
                                region.offset, region.length
                            ),
                            None => debug!("\tNo region assigned"),
                        });
                    }
                }

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
    }

    fn header_write_done(&self, processid: ProcessId, region: AppRegion) -> Result<(), ErrorCode> {
        self.apps
            .enter(processid, |app, kernel_data| {
                // set region data in app's grant
                app.region.replace(region);

                // bump the start of the unallocated regions
                let next_header_addr = self
                    .next_unallocated_region_header_address
                    .get()
                    .ok_or(ErrorCode::FAIL)?;

                let next_header_address =
                    next_header_addr + REGION_HEADER_LEN + self.app_region_size;

                self.next_unallocated_region_header_address
                    .set(next_header_address);

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
            }
            NonvolatileCommand::HeaderRead(_) | NonvolatileCommand::HeaderWrite(_, _) => {
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
                                    self.current_user.set(NonvolatileUser::HeaderManager(
                                        HeaderState::Read(address),
                                    ));
                                    self.driver.read(header_buffer, offset, active_len)
                                }
                                NonvolatileCommand::HeaderWrite(processid, region) => {
                                    self.current_user.set(NonvolatileUser::HeaderManager(
                                        HeaderState::Write(processid, region),
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
                NonvolatileUser::HeaderManager(state) => {
                    self.header_buffer.replace(buffer);
                    let res = match state {
                        HeaderState::Read(action) => self.header_read_done(action),
                        _ => Err(ErrorCode::FAIL),
                    };
                    if DEBUG {
                        debug!("[NONVOLATILE_STORAGE_DRIVER]: Header read operation ({:#x?}) finished with {:?}", state, res);
                    }
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
                NonvolatileUser::HeaderManager(state) => {
                    self.header_buffer.replace(buffer);
                    let res = match state {
                        HeaderState::Write(processid, region) => {
                            let write_res = self.header_write_done(processid, region);

                            // signal app if we fail to write its region header
                            if write_res.is_err() {
                                let _ = self.apps.enter(processid, |_, kernel_data| {
                                    kernel_data
                                        .schedule_upcall(upcall::INIT_DONE, (kernel::errorcode::into_statuscode(write_res), 0, 0))
                                        .ok();
                                });
                            }
                            write_res
                        },
                        _ => Err(ErrorCode::FAIL),
                    };
                    if DEBUG {
                        debug!("[NONVOLATILE_STORAGE_DRIVER]: Header write operation ({:#x?}) finished with {:?}", state, res);
                    }
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
