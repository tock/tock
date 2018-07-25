//! This provides kernel and userspace access to nonvolatile memory.
//!
//! This is an initial implementation that does not provide safety for
//! individual userland applications. Each application has full access to
//! the entire memory space that has been provided to userland. Future revisions
//! should update this to limit applications to only their allocated regions.
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
//! Example instantiation:
//!
//! ```rust
//! let nonvolatile_storage = static_init!(
//!     capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
//!     capsules::nonvolatile_storage_driver::NonvolatileStorage::new(
//!         fm25cl,                      // The underlying storage driver.
//!         kernel::Grant::create(),     // Storage for app-specific state.
//!         3000,                        // The byte start address for the userspace
//!                                      // accessible memory region.
//!         2000,                        // The length of the userspace region.
//!         0,                           // The byte start address of the region
//!                                      // that is accessible by the kernel.
//!         3000,                        // The length of the kernel region.
//!         &mut capsules::nonvolatile_storage_driver::BUFFER));
//! hil::nonvolatile_storage::NonvolatileStorage::set_client(fm25cl, nonvolatile_storage);
//! ```

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x50001;

pub static mut BUFFER: [u8; 512] = [0; 512];

#[derive(Clone, Copy, PartialEq)]
pub enum NonvolatileCommand {
    UserspaceRead,
    UserspaceWrite,
    KernelRead,
    KernelWrite,
}

#[derive(Clone, Copy)]
pub enum NonvolatileUser {
    App { app_id: AppId },
    Kernel,
}

pub struct App {
    callback_read: Option<Callback>,
    callback_write: Option<Callback>,
    pending_command: bool,
    command: NonvolatileCommand,
    offset: usize,
    length: usize,
    buffer_read: Option<AppSlice<Shared, u8>>,
    buffer_write: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback_read: None,
            callback_write: None,
            pending_command: false,
            command: NonvolatileCommand::UserspaceRead,
            offset: 0,
            length: 0,
            buffer_read: None,
            buffer_write: None,
        }
    }
}

pub struct NonvolatileStorage<'a> {
    // The underlying physical storage device.
    driver: &'a hil::nonvolatile_storage::NonvolatileStorage,
    // Per-app state.
    apps: Grant<App>,

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
    kernel_client: OptionalCell<&'static hil::nonvolatile_storage::NonvolatileStorageClient>,
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
}

impl NonvolatileStorage<'a> {
    pub fn new(
        driver: &'a hil::nonvolatile_storage::NonvolatileStorage,
        grant: Grant<App>,
        userspace_start_address: usize,
        userspace_length: usize,
        kernel_start_address: usize,
        kernel_length: usize,
        buffer: &'static mut [u8],
    ) -> NonvolatileStorage<'a> {
        NonvolatileStorage {
            driver: driver,
            apps: grant,
            buffer: TakeCell::new(buffer),
            current_user: OptionalCell::empty(),
            userspace_start_address: userspace_start_address,
            userspace_length: userspace_length,
            kernel_start_address: kernel_start_address,
            kernel_length: kernel_length,
            kernel_client: OptionalCell::empty(),
            kernel_pending_command: Cell::new(false),
            kernel_command: Cell::new(NonvolatileCommand::KernelRead),
            kernel_buffer: TakeCell::empty(),
            kernel_readwrite_length: Cell::new(0),
            kernel_readwrite_address: Cell::new(0),
        }
    }

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_command(
        &self,
        command: NonvolatileCommand,
        offset: usize,
        length: usize,
        app_id: Option<AppId>,
    ) -> ReturnCode {
        // Do bounds check.
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                // Userspace sees memory that starts at address 0 even if it
                // is offset in the physical memory.
                if offset >= self.userspace_length
                    || length > self.userspace_length
                    || offset + length > self.userspace_length
                {
                    return ReturnCode::EINVAL;
                }
            }
            NonvolatileCommand::KernelRead | NonvolatileCommand::KernelWrite => {
                // Because the kernel uses the NonvolatileStorage interface,
                // its calls are absolute addresses.
                if offset < self.kernel_start_address
                    || offset >= self.kernel_start_address + self.kernel_length
                    || length > self.kernel_length
                    || offset + length > self.kernel_start_address + self.kernel_length
                {
                    return ReturnCode::EINVAL;
                }
            }
        }

        // Do very different actions if this is a call from userspace
        // or from the kernel.
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                app_id.map_or(ReturnCode::FAIL, |appid| {
                    self.apps
                        .enter(appid, |app, _| {
                            // Get the length of the correct allowed buffer.
                            let allow_buf_len = match command {
                                NonvolatileCommand::UserspaceRead => {
                                    app.buffer_read.as_ref().map_or(0, |appbuf| appbuf.len())
                                }
                                NonvolatileCommand::UserspaceWrite => {
                                    app.buffer_write.as_ref().map_or(0, |appbuf| appbuf.len())
                                }
                                _ => 0,
                            };

                            // Check that it exists.
                            if allow_buf_len == 0 || self.buffer.is_none() {
                                return ReturnCode::ERESERVE;
                            }

                            // Shorten the length if the application gave us nowhere to
                            // put it.
                            let active_len = cmp::min(length, allow_buf_len);

                            // First need to determine if we can execute this or must
                            // queue it.
                            if self.current_user.is_none() {
                                // No app is currently using the underlying storage.
                                // Mark this app as active, and then execute the command.
                                self.current_user
                                    .set(NonvolatileUser::App { app_id: appid });

                                // Need to copy bytes if this is a write!
                                if command == NonvolatileCommand::UserspaceWrite {
                                    app.buffer_write.as_mut().map(|app_buffer| {
                                        self.buffer.map(|kernel_buffer| {
                                            // Check that the internal buffer and the buffer that was
                                            // allowed are long enough.
                                            let write_len =
                                                cmp::min(active_len, kernel_buffer.len());

                                            let d = &mut app_buffer.as_mut()[0..write_len];
                                            for (i, c) in
                                                kernel_buffer[0..write_len].iter_mut().enumerate()
                                            {
                                                *c = d[i];
                                            }
                                        });
                                    });
                                }

                                self.userspace_call_driver(command, offset, active_len)
                            } else {
                                // Some app is using the storage, we must wait.
                                if app.pending_command == true {
                                    // No more room in the queue, nowhere to store this
                                    // request.
                                    ReturnCode::ENOMEM
                                } else {
                                    // We can store this, so lets do it.
                                    app.pending_command = true;
                                    app.command = command;
                                    app.offset = offset;
                                    app.length = active_len;
                                    ReturnCode::SUCCESS
                                }
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                })
            }
            NonvolatileCommand::KernelRead | NonvolatileCommand::KernelWrite => {
                self.kernel_buffer
                    .take()
                    .map_or(ReturnCode::ENOMEM, |kernel_buffer| {
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
                                _ => ReturnCode::FAIL,
                            }
                        } else {
                            if self.kernel_pending_command.get() == true {
                                ReturnCode::ENOMEM
                            } else {
                                self.kernel_pending_command.set(true);
                                self.kernel_command.set(command);
                                self.kernel_readwrite_length.set(active_len);
                                self.kernel_readwrite_address.set(offset);
                                self.kernel_buffer.replace(kernel_buffer);
                                ReturnCode::SUCCESS
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
    ) -> ReturnCode {
        // Calculate where we want to actually read from in the physical
        // storage.
        let physical_address = offset + self.userspace_start_address;

        self.buffer.take().map_or(ReturnCode::ERESERVE, |buffer| {
            // Check that the internal buffer and the buffer that was
            // allowed are long enough.
            let active_len = cmp::min(length, buffer.len());

            // self.current_app.set(Some(appid));
            match command {
                NonvolatileCommand::UserspaceRead => {
                    self.driver.read(buffer, physical_address, active_len)
                }
                NonvolatileCommand::UserspaceWrite => {
                    self.driver.write(buffer, physical_address, active_len)
                }
                _ => ReturnCode::FAIL,
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
                    _ => ReturnCode::FAIL,
                }
            });
        } else {
            // If the kernel is not requesting anything, check all of the apps.
            for cntr in self.apps.iter() {
                let started_command = cntr.enter(|app, _| {
                    if app.pending_command {
                        app.pending_command = false;
                        self.current_user.set(NonvolatileUser::App {
                            app_id: app.appid(),
                        });
                        self.userspace_call_driver(app.command, app.offset, app.length)
                            == ReturnCode::SUCCESS
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
impl hil::nonvolatile_storage::NonvolatileStorageClient for NonvolatileStorage<'a> {
    fn read_done(&self, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_user.take().map(|user| {
            match user {
                NonvolatileUser::Kernel => {
                    self.kernel_client.map(move |client| {
                        client.read_done(buffer, length);
                    });
                }
                NonvolatileUser::App { app_id } => {
                    let _ = self.apps.enter(app_id, move |app, _| {
                        // Need to copy in the contents of the buffer
                        app.buffer_read.as_mut().map(|app_buffer| {
                            let read_len = cmp::min(app_buffer.len(), length);

                            let d = &mut app_buffer.as_mut()[0..(read_len as usize)];
                            for (i, c) in buffer[0..read_len].iter().enumerate() {
                                d[i] = *c;
                            }
                        });

                        // Replace the buffer we used to do this read.
                        self.buffer.replace(buffer);

                        // And then signal the app.
                        app.callback_read.map(|mut cb| cb.schedule(length, 0, 0));
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
                NonvolatileUser::App { app_id } => {
                    let _ = self.apps.enter(app_id, move |app, _| {
                        // Replace the buffer we used to do this write.
                        self.buffer.replace(buffer);

                        // And then signal the app.
                        app.callback_write.map(|mut cb| cb.schedule(length, 0, 0));
                    });
                }
            }
        });

        self.check_queue();
    }
}

/// Provide an interface for the kernel.
impl hil::nonvolatile_storage::NonvolatileStorage for NonvolatileStorage<'a> {
    fn set_client(&self, client: &'static hil::nonvolatile_storage::NonvolatileStorageClient) {
        self.kernel_client.set(client);
    }

    fn read(&self, buffer: &'static mut [u8], address: usize, length: usize) -> ReturnCode {
        self.kernel_buffer.replace(buffer);
        self.enqueue_command(NonvolatileCommand::KernelRead, address, length, None)
    }

    fn write(&self, buffer: &'static mut [u8], address: usize, length: usize) -> ReturnCode {
        self.kernel_buffer.replace(buffer);
        self.enqueue_command(NonvolatileCommand::KernelWrite, address, length, None)
    }
}

/// Provide an interface for userland.
impl Driver for NonvolatileStorage<'a> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Setup a buffer to read from the nonvolatile storage into.
    /// - `1`: Setup a buffer to write bytes to the nonvolatile storage.
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                match allow_num {
                    0 => app.buffer_read = slice,
                    1 => app.buffer_write = slice,
                    _ => return ReturnCode::ENOSUPPORT,
                }
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup a read done callback.
    /// - `1`: Setup a write done callback.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                match subscribe_num {
                    0 => app.callback_read = callback,
                    1 => app.callback_write = callback,
                    _ => return ReturnCode::ENOSUPPORT,
                }
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Command interface.
    ///
    /// Commands are selected by the lowest 8 bits of the first argument.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return SUCCESS if this driver is included on the platform.
    /// - `1`: Return the number of bytes available to userspace.
    /// - `2`: Start a read from the nonvolatile storage.
    /// - `3`: Start a write to the nonvolatile_storage.
    fn command(&self, arg0: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        let command_num = arg0 & 0xFF;

        match command_num {
            0 => /* This driver exists. */ ReturnCode::SUCCESS,

            // How many bytes are accessible from userspace.
            1 => ReturnCode::SuccessWithValue { value: self.userspace_length },

            // Issue a read
            2 => {
                let length = (arg0 >> 8) & 0xFFFFFF;
                let offset = arg1;
                self.enqueue_command(NonvolatileCommand::UserspaceRead,
                                     offset,
                                     length,
                                     Some(appid))
            }

            // Issue a write
            3 => {
                let length = (arg0 >> 8) & 0xFFFFFF;
                let offset = arg1;
                self.enqueue_command(NonvolatileCommand::UserspaceWrite,
                                     offset,
                                     length,
                                     Some(appid))
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
