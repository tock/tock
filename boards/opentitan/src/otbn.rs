// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for collections of Hardware Accelerators.
//!
//! Usage
//! -----
//! ```rust
//!     let _mux_otbn = crate::otbn::AccelMuxComponent::new(&peripherals.otbn)
//!         .finalize(otbn_mux_component_static!());
//!
//!     peripherals.otbn.initialise(
//!         dynamic_deferred_caller
//!             .register(&peripherals.otbn)
//!             .unwrap(), // Unwrap fail = dynamic deferred caller out of slots
//!     );
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use lowrisc::otbn::Otbn;
use lowrisc::virtual_otbn::{MuxAccel, VirtualMuxAccel};

#[macro_export]
macro_rules! otbn_mux_component_static {
    () => {{
        kernel::static_buf!(lowrisc::virtual_otbn::MuxAccel<'static>)
    }};
}

#[macro_export]
macro_rules! otbn_component_static {
    () => {{
        kernel::static_buf!(lowrisc::virtual_otbn::VirtualMuxAccel<'static>)
    }};
}

pub struct AccelMuxComponent {
    otbn: &'static Otbn<'static>,
}

impl AccelMuxComponent {
    pub fn new(otbn: &'static Otbn<'static>) -> AccelMuxComponent {
        AccelMuxComponent { otbn }
    }
}

impl Component for AccelMuxComponent {
    type StaticInput = &'static mut MaybeUninit<MuxAccel<'static>>;
    type Output = &'static MuxAccel<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(MuxAccel::new(self.otbn))
    }
}

pub struct OtbnComponent {
    mux_otbn: &'static MuxAccel<'static>,
}

impl OtbnComponent {
    pub fn new(mux_otbn: &'static MuxAccel<'static>) -> OtbnComponent {
        OtbnComponent { mux_otbn }
    }
}

impl Component for OtbnComponent {
    type StaticInput = &'static mut MaybeUninit<VirtualMuxAccel<'static>>;

    type Output = &'static VirtualMuxAccel<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let virtual_otbn_user = s.write(VirtualMuxAccel::new(self.mux_otbn));

        virtual_otbn_user
    }
}

/// Find the OTBN app in the Tock process list
///
/// This will iterate through the app list inside the `app_flash` looking
/// for a disabled app with the same name as `name`.
/// On success this function will return the following information:
///    * OTBN imem start address
///    * OTBN imem size
///    * OTBN dmem start address
///    * OTBN dmem size
///
/// This function is based on the Tock process loading code
#[allow(dead_code)]
pub fn find_app(name: &str, app_flash: &'static [u8]) -> Result<(usize, usize, usize, usize), ()> {
    let mut remaining_flash = app_flash;

    loop {
        // Get the first eight bytes of flash to check if there is another
        // app.
        let test_header_slice = match remaining_flash.get(0..8) {
            Some(s) => s,
            None => {
                // Not enough flash to test for another app. This just means
                // we are at the end of flash, and there are no more apps to
                // load.
                return Err(());
            }
        };

        // Pass the first eight bytes to tbfheader to parse out the length of
        // the tbf header and app. We then use those values to see if we have
        // enough flash remaining to parse the remainder of the header.
        let (version, header_length, entry_length) = match tock_tbf::parse::parse_tbf_header_lengths(
            test_header_slice.try_into().or(Err(()))?,
        ) {
            Ok((v, hl, el)) => (v, hl, el),
            Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(entry_length)) => {
                // If we could not parse the header, then we want to skip over
                // this app and look for the next one.
                (0, 0, entry_length)
            }
            Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                // Since Tock apps use a linked list, it is very possible the
                // header we started to parse is intentionally invalid to signal
                // the end of apps. This is ok and just means we have finished
                // loading apps.
                return Err(());
            }
        };

        // Now we can get a slice which only encompasses the length of flash
        // described by this tbf header.  We will either parse this as an actual
        // app, or skip over this region.
        let entry_flash = remaining_flash.get(0..entry_length as usize).ok_or(())?;

        // Advance the flash slice for process discovery beyond this last entry.
        // This will be the start of where we look for a new process since Tock
        // processes are allocated back-to-back in flash.
        remaining_flash = remaining_flash.get(entry_flash.len()..).ok_or(())?;

        if header_length > 0 {
            // If we found an actual app header, try to create a `Process`
            // object. We also need to shrink the amount of remaining memory
            // based on whatever is assigned to the new process if one is
            // created.

            // Get a slice for just the app header.
            let header_flash = entry_flash.get(0..header_length as usize).ok_or(())?;

            // Parse the full TBF header to see if this is a valid app. If the
            // header can't parse, we will error right here.
            if let Ok(tbf_header) = tock_tbf::parse::parse_tbf_header(header_flash, version) {
                let process_name = tbf_header.get_package_name().unwrap();

                // If the app is enabled, it's a real app and not what we are looking for.
                if tbf_header.enabled() {
                    continue;
                }

                if name != process_name {
                    continue;
                }

                let dmem_length = tbf_header.get_minimum_app_ram_size();

                let imem_start =
                    unsafe { entry_flash.as_ptr().offset(header_length as isize) as usize };
                let imem_length = entry_length - dmem_length - header_length as u32 - 4;

                let dmem_start = unsafe {
                    entry_flash
                        .as_ptr()
                        .offset(header_length as isize + imem_length as isize)
                        as usize
                };

                return Ok((
                    imem_start,
                    imem_length as usize,
                    dmem_start,
                    dmem_length as usize,
                ));
            }
        }
    }
}
