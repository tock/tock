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
//!         &mut capsules::nonvolatile_storage_driver::BUFFER));
//! hil::nonvolatile_storage::NonvolatileStorage::set_client(fm25cl, nonvolatile_storage);
//! ```

use core::cell::Cell;
use core::cmp;
use core::mem;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::ErrorCode;
use kernel::{
    AppId, Callback, CommandReturn, Driver, Grant, GrantDefault, ProcessCallbackFactory, Read,
    ReadOnlyAppSlice, ReadWrite, ReadWriteAppSlice,
};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::NvmStorage as usize;

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
    callback_read: Callback,
    callback_write: Callback,
    pending_command: bool,
    command: NonvolatileCommand,
    offset: usize,
    length: usize,
    buffer_read: ReadWriteAppSlice,
    buffer_write: ReadOnlyAppSlice,
}

impl GrantDefault for App {
    fn grant_default(_process_id: AppId, cb_factory: &mut ProcessCallbackFactory) -> App {
        App {
            callback_read: cb_factory.build_callback(0).unwrap(),
            callback_write: cb_factory.build_callback(1).unwrap(),
            pending_command: false,
            command: NonvolatileCommand::UserspaceRead,
            offset: 0,
            length: 0,
            buffer_read: ReadWriteAppSlice::default(),
            buffer_write: ReadOnlyAppSlice::default(),
        }
    }
}

pub struct NonvolatileStorage<'a> {
    // The underlying physical storage device.
    driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'static>,
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
    kernel_client:
        OptionalCell<&'static dyn hil::nonvolatile_storage::NonvolatileStorageClient<'static>>,
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

impl<'a> NonvolatileStorage<'a> {
    pub fn new(
        driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'static>,
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
    ) -> Result<(), ErrorCode> {
        // Do bounds check.
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                // Userspace sees memory that starts at address 0 even if it
                // is offset in the physical memory.
                if offset >= self.userspace_length
                    || length > self.userspace_length
                    || offset + length > self.userspace_length
                {
                    return Err(ErrorCode::INVAL);
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
                    return Err(ErrorCode::INVAL);
                }
            }
        }

        // Do very different actions if this is a call from userspace
        // or from the kernel.
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                app_id.map_or(Err(ErrorCode::FAIL), |appid| {
                    self.apps
                        .enter(appid, |app, _| {
                            // Get the length of the correct allowed buffer.
                            let allow_buf_len = match command {
                                NonvolatileCommand::UserspaceRead => app.buffer_read.len(),
                                NonvolatileCommand::UserspaceWrite => app.buffer_write.len(),
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
                                self.current_user
                                    .set(NonvolatileUser::App { app_id: appid });

                                // Need to copy bytes if this is a write!
                                if command == NonvolatileCommand::UserspaceWrite {
                                    app.buffer_write.map_or((), |app_buffer| {
                                        self.buffer.map(|kernel_buffer| {
                                            // Check that the internal buffer and the buffer that was
                                            // allowed are long enough.
                                            let write_len =
                                                cmp::min(active_len, kernel_buffer.len());

                                            let d = &app_buffer[0..write_len];
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
                            if self.kernel_pending_command.get() == true {
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

                // self.current_app.set(Some(appid));
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
                let started_command = cntr.enter(|app, _| {
                    if app.pending_command {
                        app.pending_command = false;
                        self.current_user.set(NonvolatileUser::App {
                            app_id: app.appid(),
                        });
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
impl hil::nonvolatile_storage::NonvolatileStorageClient<'static> for NonvolatileStorage<'_> {
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
                        app.buffer_read.mut_map_or((), |app_buffer| {
                            let read_len = cmp::min(app_buffer.len(), length);

                            let d = &mut app_buffer[0..(read_len as usize)];
                            for (i, c) in buffer[0..read_len].iter().enumerate() {
                                d[i] = *c;
                            }
                        });

                        // Replace the buffer we used to do this read.
                        self.buffer.replace(buffer);

                        // And then signal the app.
                        app.callback_read.schedule(length, 0, 0);
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
                        app.callback_write.schedule(length, 0, 0);
                    });
                }
            }
        });

        self.check_queue();
    }
}

/// Provide an interface for the kernel.
impl hil::nonvolatile_storage::NonvolatileStorage<'static> for NonvolatileStorage<'_> {
    fn set_client(&self, client: &'static dyn hil::nonvolatile_storage::NonvolatileStorageClient) {
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
impl Driver for NonvolatileStorage<'_> {
    /// Setup shared kernel-writable buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Setup a buffer to read from the nonvolatile storage into.
    /// - `1`: Setup a buffer to write bytes to the nonvolatile storage.
    fn allow_readwrite(
        &self,
        appid: AppId,
        allow_num: usize,
        mut slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        let res = match allow_num {
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut slice, &mut app.buffer_read);
                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into())),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    /// Setup shared kernel-readable buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Setup a buffer to write bytes to the nonvolatile storage.
    fn allow_readonly(
        &self,
        appid: AppId,
        allow_num: usize,
        mut slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        let res = match allow_num {
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut slice, &mut app.buffer_write);
                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into())),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
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
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        let res = self
            .apps
            .enter(app_id, |app, _| match subscribe_num {
                0 => {
                    mem::swap(&mut app.callback_read, &mut callback);
                    Ok(())
                }
                1 => {
                    mem::swap(&mut app.callback_write, &mut callback);
                    Ok(())
                }
                _ => Err(ErrorCode::NOSUPPORT),
            })
            .unwrap_or_else(|err| Err(err.into()));

        match res {
            Ok(()) => Ok(callback),
            Err(e) => Err((callback, e)),
        }
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
    fn command(
        &self,
        command_num: usize,
        offset: usize,
        length: usize,
        appid: AppId,
    ) -> CommandReturn {
        match command_num {
            0 /* This driver exists. */ => {
                CommandReturn::success()
            }

            1 /* How many bytes are accessible from userspace */ => {
                // TODO: Would break on 64-bit platforms
                CommandReturn::success_u32(self.userspace_length as u32)
            },

            2 /* Issue a read command */ => {
                let res =
                    self.enqueue_command(
                        NonvolatileCommand::UserspaceRead,
                        offset,
                        length,
                        Some(appid),
                    );

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            3 /* Issue a write command */ => {
                let res =
                    self.enqueue_command(
                        NonvolatileCommand::UserspaceWrite,
                        offset,
                        length,
                        Some(appid),
                    );

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
