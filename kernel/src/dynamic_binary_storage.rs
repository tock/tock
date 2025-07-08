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
use crate::deferred_call::{DeferredCall, DeferredCallClient};
use crate::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};
use crate::platform::chip::Chip;
use crate::process::ProcessLoadingAsyncClient;
use crate::process_loading::{
    PaddingRequirement, ProcessLoadError, SequentialProcessLoaderMachine,
};
use crate::process_standard::ProcessStandardDebug;
use crate::utilities::cells::{OptionalCell, TakeCell};
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
    Abort,
    PaddingWrite,
    Fail,
}

/// Addresses of where the new process will be stored.
#[derive(Clone, Copy, Default)]
struct ProcessLoadMetadata {
    new_app_start_addr: usize,
    new_app_length: usize,
    previous_app_end_addr: usize,
    next_app_start_addr: usize,
    padding_requirement: PaddingRequirement,
    setup_padding: bool,
}

/// This interface supports flashing binaries at runtime.
pub trait DynamicBinaryStore {
    /// Call to request flashing a new binary.
    ///
    /// This informs the kernel we want to load a process, and the size of the
    /// entire process binary. The kernel will try to find a suitable location
    /// in flash to store said process.
    ///
    /// Return value:
    /// - `Ok(length)`: If there is a place to load the
    ///   process, the function will return `Ok()` with the size of the region
    ///   to store the process.
    /// - `Err(ErrorCode)`: If there is nowhere to store the process a suitable
    ///   `ErrorCode` will be returned.
    fn setup(&self, app_length: usize) -> Result<usize, ErrorCode>;

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
    fn write(&self, buffer: SubSliceMut<'static, u8>, offset: usize) -> Result<(), ErrorCode>;

    /// Signal to the kernel that the requesting process is done writing the new
    /// binary.
    fn finalize(&self) -> Result<(), ErrorCode>;

    /// Call to abort the setup/writing process.
    fn abort(&self) -> Result<(), ErrorCode>;

    /// Sets a client for the SequentialDynamicBinaryStore Object
    ///
    /// When the client operation is done, it calls the `setup_done()`,
    /// `write_done()` and `abort_done()` functions.
    fn set_storage_client(&self, client: &'static dyn DynamicBinaryStoreClient);
}

/// The callback for dynamic binary flashing.
pub trait DynamicBinaryStoreClient {
    /// Any setup work is done and we are ready to write the process binary.
    fn setup_done(&self, result: Result<(), ErrorCode>);

    /// The provided app binary buffer has been stored.
    fn write_done(&self, result: Result<(), ErrorCode>, buffer: &'static mut [u8], length: usize);

    /// The kernel has successfully finished finalizing the new app and is ready
    /// to move to the `load()` phase.
    fn finalize_done(&self, result: Result<(), ErrorCode>);

    /// Canceled any setup or writing operation and freed up reserved space.
    fn abort_done(&self, result: Result<(), ErrorCode>);
}

/// This interface supports loading processes at runtime.
pub trait DynamicProcessLoad {
    /// Call to request kernel to load a new process.
    fn load(&self) -> Result<(), ErrorCode>;

    /// Sets a client for the SequentialDynamicProcessLoading Object
    ///
    /// When the client operation is done, it calls the `load_done()`
    /// function.
    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadClient);
}

/// The callback for dynamic binary flashing.
pub trait DynamicProcessLoadClient {
    /// The new app has been loaded.
    fn load_done(&self, result: Result<(), ProcessLoadError>);
}

/// Dynamic process loading machine.
pub struct SequentialDynamicBinaryStorage<
    'a,
    'b,
    C: Chip + 'static,
    D: ProcessStandardDebug + 'static,
    F: NonvolatileStorage<'b>,
> {
    flash_driver: &'b F,
    loader_driver: &'a SequentialProcessLoaderMachine<'a, C, D>,
    buffer: TakeCell<'static, [u8]>,
    storage_client: OptionalCell<&'static dyn DynamicBinaryStoreClient>,
    load_client: OptionalCell<&'static dyn DynamicProcessLoadClient>,
    process_metadata: OptionalCell<ProcessLoadMetadata>,
    state: Cell<State>,
    deferred_call: DeferredCall,
}

impl<'a, 'b, C: Chip + 'static, D: ProcessStandardDebug + 'static, F: NonvolatileStorage<'b>>
    SequentialDynamicBinaryStorage<'a, 'b, C, D, F>
{
    pub fn new(
        flash_driver: &'b F,
        loader_driver: &'a SequentialProcessLoaderMachine<'a, C, D>,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            flash_driver,
            loader_driver,
            buffer: TakeCell::new(buffer),
            storage_client: OptionalCell::empty(),
            load_client: OptionalCell::empty(),
            process_metadata: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            deferred_call: DeferredCall::new(),
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
                        // Check if the new app is trying to write beyond
                        // the bounds of the flash region allocated to it.
                        if result > new_app_len {
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
            State::Setup | State::Load | State::PaddingWrite | State::Abort => Ok(offset),
            // We aren't supposed to be able to write unless we are in one of
            // the first two write states
            _ => Err(ErrorCode::FAIL),
        }
    }

    /// Compute the physical address where we should write the data and then
    /// write it.
    fn write_buffer(
        &self,
        user_buffer: SubSliceMut<'static, u8>,
        offset: usize,
    ) -> Result<(), ErrorCode> {
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
        // don't want apps to do this, so we return an error.
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
                true => {
                    // Write the header only if there are more than 16 bytes.
                    // available in the flash.
                    let mut padding_slice = SubSliceMut::new(buffer);
                    padding_slice.slice(..PADDING_TBF_HEADER_LENGTH);
                    // We are only writing the header, so 16 bytes is enough.
                    self.write_buffer(padding_slice, offset)
                }
                false => Err(ErrorCode::NOMEM),
            }
        })
    }
}

impl<'b, C: Chip, D: ProcessStandardDebug, F: NonvolatileStorage<'b>> DeferredCallClient
    for SequentialDynamicBinaryStorage<'_, 'b, C, D, F>
{
    fn handle_deferred_call(&self) {
        // We use deferred call to signal the completion of finalize
        self.storage_client.map(|client| {
            client.finalize_done(Ok(()));
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

/// This is the callback client for the underlying physical storage driver.
impl<'b, C: Chip + 'static, D: ProcessStandardDebug + 'static, F: NonvolatileStorage<'b>>
    NonvolatileStorageClient for SequentialDynamicBinaryStorage<'_, 'b, C, D, F>
{
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
                    client.write_done(Ok(()), buffer, length);
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

                if let Some(mut metadata) = self.process_metadata.get() {
                    if !metadata.setup_padding {
                        // Write padding header to the beginning of the new app address.
                        // This ensures that the linked list is not broken in the event of a
                        // powercycle before the app is fully written and loaded.
                        metadata.setup_padding = true;
                        let _ = self.write_padding_app(
                            metadata.new_app_length,
                            metadata.new_app_start_addr,
                        );
                        self.process_metadata.set(metadata);
                    } else {
                        self.state.set(State::AppWrite);
                        // Let the client know we are done setting up.
                        self.storage_client.map(|client| {
                            client.setup_done(Ok(()));
                        });
                    }
                }
            }
            State::Load => {
                // We finished writing pre-padding and we need to Load the app.
                self.buffer.replace(buffer);
                self.storage_client.map(|client| {
                    client.finalize_done(Ok(()));
                });
            }
            State::Abort => {
                self.buffer.replace(buffer);
                // Reset metadata and let client know we are done aborting.
                self.reset_process_loading_metadata();
                self.storage_client.map(|client| {
                    client.abort_done(Ok(()));
                });
            }
            State::Idle => {
                self.buffer.replace(buffer);
            }
        }
    }
}

/// Callback client for the async process loader
impl<'b, C: Chip + 'static, D: ProcessStandardDebug + 'static, F: NonvolatileStorage<'b>>
    ProcessLoadingAsyncClient for SequentialDynamicBinaryStorage<'_, 'b, C, D, F>
{
    fn process_loaded(&self, result: Result<(), ProcessLoadError>) {
        self.load_client.map(|client| {
            client.load_done(result);
        });
    }

    fn process_loading_finished(&self) {}
}

/// Storage interface exposed to the app_loader capsule
impl<'b, C: Chip + 'static, D: ProcessStandardDebug + 'static, F: NonvolatileStorage<'b>>
    DynamicBinaryStore for SequentialDynamicBinaryStorage<'_, 'b, C, D, F>
{
    fn set_storage_client(&self, client: &'static dyn DynamicBinaryStoreClient) {
        self.storage_client.set(client);
    }

    fn setup(&self, app_length: usize) -> Result<usize, ErrorCode> {
        self.process_metadata.set(ProcessLoadMetadata::default());

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
                            if let Some(mut metadata) = self.process_metadata.get() {
                                if !metadata.setup_padding {
                                    // Write padding header to the beginning of the new app address.
                                    // This ensures that the linked list is not broken in the event of a
                                    // powercycle before the app is fully written and loaded.

                                    metadata.setup_padding = true;
                                    let _ = self.write_padding_app(
                                        metadata.new_app_length,
                                        metadata.new_app_start_addr,
                                    );
                                    self.process_metadata.set(metadata);
                                }
                            }
                        }
                    }
                    Ok(app_length)
                }
                Err(_err) => {
                    // Reset the state to None because we did not find any
                    // available address for this app.
                    self.reset_process_loading_metadata();
                    Err(ErrorCode::FAIL)
                }
            }
        } else {
            // We are in the wrong mode of operation. Ideally we should never reach
            // here, but this error exists as a failsafe. The capsule should send
            // a busy error out to the userland app.
            Err(ErrorCode::INVAL)
        }
    }

    fn write(&self, buffer: SubSliceMut<'static, u8>, offset: usize) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::AppWrite => {
                let res = self.write_buffer(buffer, offset);
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
                // We are in the wrong mode of operation. Ideally we should never reach
                // here, but this error exists as a failsafe. The capsule should send
                // a busy error out to the userland app.
                Err(ErrorCode::INVAL)
            }
        }
    }

    fn finalize(&self) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::AppWrite => {
                if let Some(metadata) = self.process_metadata.get() {
                    match metadata.padding_requirement {
                        // If we decided we need to write a padding app before the new
                        // app, we go ahead and do it.
                        PaddingRequirement::PrePad | PaddingRequirement::PreAndPostPad => {
                            // Calculate the distance between our app and the previous
                            // app.
                            let previous_app_end_addr = metadata.previous_app_end_addr;
                            let pre_pad_length =
                                metadata.new_app_start_addr - previous_app_end_addr;
                            self.state.set(State::Load);
                            let padding_result =
                                self.write_padding_app(pre_pad_length, previous_app_end_addr);
                            match padding_result {
                                Ok(()) => {
                                    if config::CONFIG.debug_load_processes {
                                        debug!("Successfully writing prepadding app");
                                    }
                                    Ok(())
                                }
                                Err(_e) => {
                                    // This means we were unable to write the padding
                                    // app.
                                    self.reset_process_loading_metadata();
                                    Err(ErrorCode::FAIL)
                                }
                            }
                        }
                        // We should never reach here if we are not writing a prepad
                        // app.
                        PaddingRequirement::None | PaddingRequirement::PostPad => {
                            if config::CONFIG.debug_load_processes {
                                debug!("No PrePad app to write.");
                            }
                            self.state.set(State::Load);
                            self.deferred_call.set();
                            Ok(())
                        }
                    }
                } else {
                    Err(ErrorCode::INVAL)
                }
            }
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn abort(&self) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::Setup | State::AppWrite => {
                self.state.set(State::Abort);
                if let Some(metadata) = self.process_metadata.get() {
                    // Write padding header to the beginning of the new app address.
                    // This ensures that the flash space is reclaimed for future use.
                    match self
                        .write_padding_app(metadata.new_app_length, metadata.new_app_start_addr)
                    {
                        Ok(()) => Ok(()),
                        // If abort() returns ErrorCode::BUSY,
                        // the userland app is expected to retry abort.
                        Err(_) => Err(ErrorCode::BUSY),
                    }
                } else {
                    Err(ErrorCode::FAIL)
                }
            }
            _ => {
                // We are in the wrong mode of operation. Ideally we should never reach
                // here, but this error exists as a failsafe. The capsule should send
                // a busy error out to the userland app.
                Err(ErrorCode::INVAL)
            }
        }
    }
}

/// Loading interface exposed to the app_loader capsule
impl<'b, C: Chip + 'static, D: ProcessStandardDebug + 'static, F: NonvolatileStorage<'b>>
    DynamicProcessLoad for SequentialDynamicBinaryStorage<'_, 'b, C, D, F>
{
    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadClient) {
        self.load_client.set(client);
    }

    fn load(&self) -> Result<(), ErrorCode> {
        // We have finished writing the last user data segment, next step is to
        // load the process.
        match self.state.get() {
            State::Load => {
                if let Some(metadata) = self.process_metadata.get() {
                    let _ = match self.loader_driver.load_new_process_binary(
                        metadata.new_app_start_addr,
                        metadata.new_app_length,
                    ) {
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
            _ => Err(ErrorCode::INVAL),
        }
    }
}
