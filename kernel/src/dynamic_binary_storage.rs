// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Dynamic Binary Flasher for application loading and updating at runtime.
//!
//! These functions facilitate dynamic application flashing and process creation
//! during runtime without requiring the user to restart the device.

use core::cell::Cell;

use crate::config;
use crate::debug;
use crate::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};
use crate::process;
use crate::process::{ProcessLoadingAsync, ProcessLoadingAsyncClient};
use crate::process_loading::PaddingRequirement;
use crate::process_loading::ProcessLoadError;
use crate::utilities::cells::{MapCell, OptionalCell, TakeCell};
use crate::utilities::leasable_buffer::SubSliceMut;
use crate::ErrorCode;

/// Expected buffer length for storing application binaries.
pub const BUF_LEN: usize = 512;

/// The number of bytes in the TBF header for a padding app.
const PADDING_TBF_HEADER_LENGTH: usize = 16;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Setup,
    AppWrite,
    Load,
    PaddingWrite,
    Fail,
}

/// Addresses of where the new process will be stored.
#[derive(Clone, Copy, Default)]
pub struct ProcessLoadMetadata {
    pub new_app_start_addr: usize,
    pub new_app_length: usize,
    pub previous_app_end_addr: usize,
    pub next_app_start_addr: usize,
    pub padding_requirement: PaddingRequirement,
}

/// This interface supports flashing binaries at runtime.
pub trait DynamicBinaryStore {
    /// Call to request flashing a new binary.
    ///
    /// This informs the kernel we want to load a process and the  of the
    /// entire process binary. The kernel will try to find a suitable location
    /// in flash to store said process.
    ///
    /// Return value:
    /// - `Ok((length, wait_for_setup))`: If there is a place to load the
    ///   process, the function will return `Ok()` with the size of the region
    ///   to store the process, and whether the process loader is waiting to set
    ///   up. This usually happens when we have to write a post pad app. The
    ///   client app is unable to write new app data until the process loader
    ///   finishes writing the padding app. So if tihs flag is set, then the
    ///   client app has to wait until the setup_done subscribe callback is
    ///   received.
    /// - `Err(ErrorCode)`: If there is nowhere to store the process a suitable
    ///   `ErrorCode` will be returned.
    fn setup(&self, app_length: usize) -> Result<(usize, bool), ErrorCode>;

    /// Instruct the kernel to write data to the flash.
    ///
    /// `offset` is where to start writing within the region allocated
    /// for the new process binary from the `setup()` call.
    ///
    /// The caller must write the first 8 bytes of the process with valid header
    /// data. Writes must either be after the first 8 bytes or include the
    /// entire first 8 bytes.
    ///
    /// Returns an error if the write is outside of the permitted region or is
    /// writing an invalid header.
    fn write_app_data(
        &self,
        buffer: SubSliceMut<'static, u8>,
        offset: usize,
    ) -> Result<(), ErrorCode>;

    /// Sets a client for the DynamicBinaryStore Object
    ///
    /// When the client operation is done, it calls the `setup_done()`
    /// and `write_app_data_done()` functions.
    fn set_storage_client(&self, client: &'static dyn DynamicBinaryStoreClient);

    /// Write a prepad app if required
    fn write_prepad_app(&self);

    /// Call to request kernel to load a new process.
    fn load(&self) -> Result<(), ErrorCode>;
}

/// The callback for dynamic binary flashing.
pub trait DynamicBinaryStoreClient {
    /// Any setup work is done and we are ready to write the process binary.
    fn setup_done(&self);

    /// The provided app binary buffer has been stored.
    fn write_app_data_done(&self, buffer: &'static mut [u8], length: usize);

    /// The new app has been loaded.
    fn load_done(&self);
}

/// Dynamic process loading machine.
pub struct DynamicBinaryStorage<'a> {
    processes: MapCell<&'static mut [Option<&'static dyn process::Process>]>,
    flash_driver: &'a dyn NonvolatileStorage<'a>,
    loader_driver: &'a dyn ProcessLoadingAsync<'a>,
    buffer: TakeCell<'static, [u8]>,
    storage_client: OptionalCell<&'static dyn DynamicBinaryStoreClient>,
    process_metadata: OptionalCell<ProcessLoadMetadata>,
    state: Cell<State>,
}

impl<'a> DynamicBinaryStorage<'a> {
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        flash_driver: &'a dyn NonvolatileStorage<'a>,
        loader_driver: &'a dyn ProcessLoadingAsync<'a>,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            processes: MapCell::new(processes),
            flash_driver,
            loader_driver,
            buffer: TakeCell::new(buffer),
            storage_client: OptionalCell::empty(),
            process_metadata: OptionalCell::empty(),
            state: Cell::new(State::Idle),
        }
    }

    /// Function to reset variables and states.
    fn reset_process_loading_metadata(&self) {
        self.state.set(State::Idle);
        self.process_metadata.take();
    }

    /// This function checks whether the new app will fit in the bounds dictated
    /// by the start address and length provided during the setup phase. This
    /// function then also computes where in flash the data should be written
    /// based on whether the call is coming during the app writing phase, or the
    /// padding phase.
    ///
    /// This function returns the physical address in flash where the write is
    /// supposed to happen.
    fn compute_address(&self, offset: usize, length: usize) -> Result<usize, ErrorCode> {
        let mut new_app_len: usize = 0;
        let mut new_app_addr: usize = 0;
        if let Some(metadata) = self.process_metadata.get() {
            new_app_addr = metadata.new_app_start_addr;
            new_app_len = metadata.new_app_length;
        }

        match self.state.get() {
            State::AppWrite => {
                // Check if there is an overflow while adding length and offset.
                match offset.checked_add(length) {
                    Some(result) => {
                        // Check if the length of the new write block goes over
                        // the total size alloted to the new application. We
                        // also check if the new app is trying to write beyond
                        // the bounds of the flash region allocated to it.
                        if length > new_app_len || result > new_app_len {
                            // This means the app is out of bounds.
                            Err(ErrorCode::INVAL)
                        } else {
                            // We compute the new address to write the app
                            // binary segment.
                            Ok(offset + new_app_addr)
                        }
                    }
                    None => Err(ErrorCode::INVAL),
                }
            }
            // If we are going to write the padding header, we already know
            // where to write in flash, so we don't have to add the start
            // address
            State::Setup | State::Load | State::PaddingWrite => Ok(offset),
            // We aren't supposed to be able to write unless we are in one of
            // the first two write states
            _ => Err(ErrorCode::FAIL),
        }
    }

    /// Compute the physical address where we should write the data and then
    /// write it.
    ///
    /// Current limitation: There are two limitations with the current
    /// implementation of how tock looks for and loads apps. Assuming the flash
    /// looks like this:
    ///
    /// ```text
    ///  ____________________________________________________
    /// |             |    |            |          |         |
    /// |     App1    | H? |   NewApp?  |   Pad    |   App2  |
    /// |_____________|____|____________|__________|_________|
    /// ```
    ///
    /// Assuming the new app goes in between App 1 and App 2 which are existing
    /// processes, we write the padding after the new app during setup phase.
    ///
    /// - Issue 1: If there is a power cycle as we try to write the header for
    ///   the new app, during the flash erase part of the flash write, we might
    ///   end up with a break in the linked list when the device reboots and we
    ///   never boot app 2.
    ///     - Potential Fix:  Reserve a section of flash to hold an
    ///       index/repository of valid headers of current processes as a fall
    ///       back mechanism in case we end up with corrupt headers.
    /// - Issue 2: If the header is written successfully, but there is a power
    ///   cycle as the rest of the app binary is being written, we could end up
    ///   with the situation where because the header is valid, we could end up
    ///   with memory fragmentation.
    ///     - Potential Fix:  Create a processes monitoring process that is able
    ///       to clean up after corrupt apps and defragment memory?
    fn write(&self, user_buffer: SubSliceMut<'static, u8>, offset: usize) -> Result<(), ErrorCode> {
        let length = user_buffer.len();
        // Take the buffer to perform tbf header validation and write with.
        let buffer = user_buffer.take();

        let physical_address = self.compute_address(offset, length)?;

        // The kernel needs to check if the app is trying to write/overwrite the
        // header. So the app can only write to the first 8 bytes if the app is
        // writing all 8 bytes. Else, the kernel must raise an error. The app is
        // not allowed to write from say, offset 4 because we have to ensure the
        // validity of the header.
        //
        // This means the app is trying to manipulate the space where the TBF
        // header should go. Ideally, we want the app to only write the complete
        // set of 8 bytes which is used to determine if the header is valid. We
        // don't apps to do this, so we return an error.
        if (offset == 0 && length < 8) || (offset != 0 && offset < 8) {
            return Err(ErrorCode::INVAL);
        }

        // Check if we are writing the start of the TBF header.
        //
        // The app is not allowed to manipulate parts of the TBF header, so if
        // it is trying to write at the very beginning of the promised flash
        // region, we require the app writes the entire 8 bytes of the header.
        // This header is then checked for validity.
        if offset == 0 {
            // Pass the first eight bytes of the tbf header to parse out the
            // length of the header and app. We then use those values to see if
            // the app is going to be valid.
            let test_header_slice = buffer.get(0..8).ok_or(ErrorCode::INVAL)?;
            let header = test_header_slice.try_into().or(Err(ErrorCode::FAIL))?;
            let (_version, _header_length, entry_length) =
                match tock_tbf::parse::parse_tbf_header_lengths(header) {
                    Ok((v, hl, el)) => (v, hl, el),
                    Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => {
                        // If we have an invalid header, so we return an error
                        return Err(ErrorCode::INVAL);
                    }
                    Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                        // If we could not parse the header, then that's an
                        // issue. We return an Error.
                        return Err(ErrorCode::INVAL);
                    }
                };

            // Check if the length in the header is matching what the app
            // requested during the setup phase also check if the kernel
            // version matches the version indicated in the new application.
            let mut new_app_len = 0;
            if let Some(metadata) = self.process_metadata.get() {
                new_app_len = metadata.new_app_length;
            }
            if entry_length as usize != new_app_len {
                return Err(ErrorCode::INVAL);
            }
        }
        self.flash_driver.write(buffer, physical_address, length)
    }

    /// Function to generate the padding header to append after the new app.
    /// This header is created and written to ensure the integrity of the
    /// processes linked list
    fn write_padding_app(&self, padding_app_length: usize, offset: usize) -> Result<(), ErrorCode> {
        // Write the header into the array
        self.buffer.map(|buffer| {
            // First two bytes are the TBF version (2).
            buffer[0] = 2;
            buffer[1] = 0;

            // The next two bytes are the header length (fixed to 16 bytes for
            // padding).
            buffer[2] = (PADDING_TBF_HEADER_LENGTH & 0xff) as u8;
            buffer[3] = ((PADDING_TBF_HEADER_LENGTH >> 8) & 0xff) as u8;

            // The next 4 bytes are the total app length including the header.
            buffer[4] = (padding_app_length & 0xff) as u8;
            buffer[5] = ((padding_app_length >> 8) & 0xff) as u8;
            buffer[6] = ((padding_app_length >> 16) & 0xff) as u8;
            buffer[7] = ((padding_app_length >> 24) & 0xff) as u8;

            // We set the flags to 0.
            for i in 8..12 {
                buffer[i] = 0x00_u8;
            }

            // xor of the previous values
            buffer[12] = buffer[0] ^ buffer[4] ^ buffer[8];
            buffer[13] = buffer[1] ^ buffer[5] ^ buffer[9];
            buffer[14] = buffer[2] ^ buffer[6] ^ buffer[10];
            buffer[15] = buffer[3] ^ buffer[7] ^ buffer[11];
        });

        self.buffer.take().map_or(Err(ErrorCode::BUSY), |buffer| {
            match self
                .loader_driver
                .check_if_within_flash_bounds(offset, PADDING_TBF_HEADER_LENGTH)
            {
                Ok(()) => {
                    // Write the header only if there are more than 16 bytes.
                    // available in the flash.
                    let mut padding_slice = SubSliceMut::new(buffer);
                    padding_slice.slice(..PADDING_TBF_HEADER_LENGTH);
                    // We are only writing the header, so 16 bytes is enough.
                    self.write(padding_slice, offset)
                }
                Err(_e) => Err(ErrorCode::NOMEM),
            }
        })
    }
}

/// This is the callback client for the underlying physical storage driver.
impl NonvolatileStorageClient for DynamicBinaryStorage<'_> {
    fn read_done(&self, _buffer: &'static mut [u8], _length: usize) {
        // We will never use this, but we need to implement this anyway.
        unimplemented!();
    }

    fn write_done(&self, buffer: &'static mut [u8], length: usize) {
        match self.state.get() {
            State::AppWrite => {
                self.state.set(State::AppWrite);
                // Switch on which user generated this callback and trigger
                // client callback.
                self.storage_client.map(|client| {
                    client.write_app_data_done(buffer, length);
                });
            }
            State::PaddingWrite => {
                // Replace the buffer after the padding is written.
                self.reset_process_loading_metadata();
                self.buffer.replace(buffer);
            }
            State::Fail => {
                // If we failed at any of writing, we want to set the state to
                // PaddingWrite so that the callback after writing the padding
                // app will get triggererd.
                self.buffer.replace(buffer);
                if let Some(metadata) = self.process_metadata.get() {
                    let _ = self
                        .write_padding_app(metadata.new_app_length, metadata.new_app_start_addr);
                }
                // Clear all metadata specific to this load.
                self.reset_process_loading_metadata();
            }
            State::Setup => {
                // We have finished writing the post app padding.
                self.buffer.replace(buffer);
                self.state.set(State::AppWrite);
                // Let the client know we are done setting up.
                self.storage_client.map(|client| {
                    client.setup_done();
                });
            }
            State::Load => {
                // We finished writing pre-padding and we need to Load the app.
                self.buffer.replace(buffer);
            }
            State::Idle => {
                self.buffer.replace(buffer);
            }
        }
    }
}

/// Callback client for the async process loader
impl ProcessLoadingAsyncClient for DynamicBinaryStorage<'_> {
    fn process_loaded(&self, result: Result<(), ProcessLoadError>) {
        match result {
            Ok(()) => {
                self.storage_client.map(|client| {
                    client.load_done();
                });
            }
            Err(_e) => {
                if config::CONFIG.debug_load_processes {
                    debug!("Load Failed.");
                }
            }
        }
    }

    fn process_loading_finished(&self) {
        // if config::CONFIG.debug_load_processes {
        debug!("Processes Loaded:");
        self.processes.map(|procs| {
            for (i, proc) in procs.iter().enumerate() {
                proc.map(|p| {
                    debug!("[{}] {}", i, p.get_process_name());
                    debug!("    ShortId: {}", p.short_app_id());
                });
            }
        });
        // }
    }
}

/// Storage interface exposed to the app_loader capsule
impl DynamicBinaryStore for DynamicBinaryStorage<'_> {
    fn set_storage_client(&self, client: &'static dyn DynamicBinaryStoreClient) {
        self.storage_client.set(client);
    }

    fn setup(&self, app_length: usize) -> Result<(usize, bool), ErrorCode> {
        //TODO(?): Check if it is a newer version of an existing app. We can
        // potentially flash the new app and load it before erasing the old one.
        // What happens to the process though? Need to delete it from the
        // process array and load it back in?

        self.process_metadata.set(ProcessLoadMetadata::default());
        let setup_done: bool;

        if self.state.get() == State::Idle {
            self.state.set(State::Setup);
            match self.loader_driver.check_flash_for_new_address(app_length) {
                Ok((
                    new_app_start_address,
                    padding_requirement,
                    previous_app_end_addr,
                    next_app_start_addr,
                )) => {
                    if let Some(mut metadata) = self.process_metadata.get() {
                        metadata.new_app_start_addr = new_app_start_address;
                        metadata.new_app_length = app_length;
                        metadata.previous_app_end_addr = previous_app_end_addr;
                        metadata.next_app_start_addr = next_app_start_addr;
                        metadata.padding_requirement = padding_requirement;
                        self.process_metadata.set(metadata);
                    }

                    match padding_requirement {
                        // If we decided we need to write a padding app after
                        // the new app, we go ahead and do it.
                        PaddingRequirement::PostPad | PaddingRequirement::PreAndPostPad => {
                            // Calculating the distance between our app and
                            // either the next app.
                            let new_app_end_address = new_app_start_address + app_length;
                            let post_pad_length = next_app_start_addr - new_app_end_address;
                            setup_done = false;

                            let padding_result =
                                self.write_padding_app(post_pad_length, new_app_end_address);
                            let _ = match padding_result {
                                Ok(()) => Ok(()),
                                Err(e) => {
                                    // This means we were unable to write the
                                    // padding app.
                                    self.reset_process_loading_metadata();
                                    Err(e)
                                }
                            };
                        }
                        // Otherwise we let the client know we are done with the
                        // setup, and we are ready to write the app to flash.
                        PaddingRequirement::None | PaddingRequirement::PrePad => {
                            self.state.set(State::AppWrite);
                            setup_done = true;
                        }
                    };
                    Ok((app_length, setup_done))
                }
                Err(_err) => {
                    // Reset the state to None because we did not find any
                    // available address for this app.
                    self.reset_process_loading_metadata();
                    Err(ErrorCode::FAIL)
                }
            }
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn write_app_data(
        &self,
        buffer: SubSliceMut<'static, u8>,
        offset: usize,
    ) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::AppWrite => {
                let res = self.write(buffer, offset);
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        // If we fail here, let us erase the app we just wrote.
                        self.state.set(State::Fail);
                        Err(e)
                    }
                }
            }
            _ => {
                // We should never enter write for the rest of the conditions,
                // so return a Busy error.
                Err(ErrorCode::BUSY)
            }
        }
    }

    fn write_prepad_app(&self) {
        if let Some(metadata) = self.process_metadata.get() {
            match metadata.padding_requirement {
                // If we decided we need to write a padding app before the new
                // app, we go ahead and do it.
                PaddingRequirement::PrePad | PaddingRequirement::PreAndPostPad => {
                    // Calculate the distance between our app and the previous
                    // app.
                    let previous_app_end_addr = metadata.previous_app_end_addr;
                    let pre_pad_length = metadata.new_app_start_addr - previous_app_end_addr;
                    self.state.set(State::PaddingWrite);
                    let padding_result =
                        self.write_padding_app(pre_pad_length, previous_app_end_addr);
                    match padding_result {
                        Ok(()) => {
                            if config::CONFIG.debug_load_processes {
                                debug!("Successfully writing prepadding app");
                            }
                        }
                        Err(_e) => {
                            // This means we were unable to write the padding
                            // app.
                            self.reset_process_loading_metadata();
                        }
                    };
                }
                // We should never reach here if we are not writing a prepad
                // app.
                PaddingRequirement::None | PaddingRequirement::PostPad => {
                    if config::CONFIG.debug_load_processes {
                        debug!("No PrePad app to write.");
                    }
                }
            };
        }
    }

    fn load(&self) -> Result<(), ErrorCode> {
        // We have finished writing the last user data segment, next step is to
        // load the process.
        if let Some(metadata) = self.process_metadata.get() {
            let _ = match self
                .loader_driver
                .load_new_applications(metadata.new_app_start_addr, metadata.new_app_length)
            {
                Ok(()) => Ok::<(), ProcessLoadError>(()),
                Err(_e) => {
                    self.reset_process_loading_metadata();
                    return Err(ErrorCode::FAIL);
                }
            };
        } else {
            self.reset_process_loading_metadata();
            return Err(ErrorCode::FAIL);
        }
        self.reset_process_loading_metadata();
        Ok(())
    }
}
