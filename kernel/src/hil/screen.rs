// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for screens and displays.
//!
//! The interfaces exposed here cover both configurable (`ScreenSetup`), and
//! less configurable hardware (only `Screen`).
//!
//! It's composed of 4 main kinds of requests:
//! - set power,
//! - read configuration (e.g. `get_resolution`),
//! - configure (e.g. `set_invert`),
//! - write buffer.
//!
//! All requests, except for `Screen::set_power`, can return `OFF` under some
//! circumstances.
//!
//! For buffer writes, it's when the display is powered off.
//!
//! While the display is not powered on, the user could try to configure it. In
//! that case, the driver MUST either cache the value, or return `OFF`. This is
//! to let the user power the display in the desired configuration.
//!
//! Configuration reads shall return the actual state of the display. In
//! situations where a parameter cannot be configured (e.g. fixed resolution),
//! they return value may be hardcoded. Otherwise, the driver should query the
//! hardware directly, and return OFF if it's not powered.
//!
//! Configuration sets cause a `command_complete` callback unless noted
//! otherwise.

use crate::utilities::leasable_buffer::SubSliceMut;
use crate::ErrorCode;
use core::ops::Add;
use core::ops::Sub;

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenRotation {
    Normal,
    Rotated90,
    Rotated180,
    Rotated270,
}

impl Add for ScreenRotation {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (ScreenRotation::Normal, _) => other,
            (_, ScreenRotation::Normal) => self,
            (ScreenRotation::Rotated90, ScreenRotation::Rotated90) => ScreenRotation::Rotated180,
            (ScreenRotation::Rotated90, ScreenRotation::Rotated180) => ScreenRotation::Rotated270,
            (ScreenRotation::Rotated90, ScreenRotation::Rotated270) => ScreenRotation::Normal,

            (ScreenRotation::Rotated180, ScreenRotation::Rotated90) => ScreenRotation::Rotated270,
            (ScreenRotation::Rotated180, ScreenRotation::Rotated180) => ScreenRotation::Normal,
            (ScreenRotation::Rotated180, ScreenRotation::Rotated270) => ScreenRotation::Rotated90,

            (ScreenRotation::Rotated270, ScreenRotation::Rotated90) => ScreenRotation::Normal,
            (ScreenRotation::Rotated270, ScreenRotation::Rotated180) => ScreenRotation::Rotated90,
            (ScreenRotation::Rotated270, ScreenRotation::Rotated270) => ScreenRotation::Rotated180,
        }
    }
}

impl Sub for ScreenRotation {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (_, ScreenRotation::Normal) => self,

            (ScreenRotation::Normal, ScreenRotation::Rotated90) => ScreenRotation::Rotated270,
            (ScreenRotation::Normal, ScreenRotation::Rotated180) => ScreenRotation::Rotated180,
            (ScreenRotation::Normal, ScreenRotation::Rotated270) => ScreenRotation::Rotated90,

            (ScreenRotation::Rotated90, ScreenRotation::Rotated90) => ScreenRotation::Normal,
            (ScreenRotation::Rotated90, ScreenRotation::Rotated180) => ScreenRotation::Rotated270,
            (ScreenRotation::Rotated90, ScreenRotation::Rotated270) => ScreenRotation::Rotated180,

            (ScreenRotation::Rotated180, ScreenRotation::Rotated90) => ScreenRotation::Rotated90,
            (ScreenRotation::Rotated180, ScreenRotation::Rotated180) => ScreenRotation::Normal,
            (ScreenRotation::Rotated180, ScreenRotation::Rotated270) => ScreenRotation::Rotated270,

            (ScreenRotation::Rotated270, ScreenRotation::Rotated90) => ScreenRotation::Rotated180,
            (ScreenRotation::Rotated270, ScreenRotation::Rotated180) => ScreenRotation::Rotated90,
            (ScreenRotation::Rotated270, ScreenRotation::Rotated270) => ScreenRotation::Normal,
        }
    }
}

/// How pixels are encoded for the screen.
#[derive(Copy, Clone, PartialEq)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum ScreenPixelFormat {
    /// Pixels encoded as 1-bit, used for monochromatic displays.
    Mono = 0,
    /// Pixels encoded as 1-bit, used for monochromatic displays.
    ///
    /// The pixel order uses 8-bit (1-byte) pages where each page is displayed
    /// vertically. That is, buffer[0] bit0 is pixel (0,0), but buffer[0] bit1
    /// is pixel (0,1). The page continues, so buffer[0] bit 7 is pixel
    /// (0,7). Then the page advances horizontally, so buffer[1] bit0 is
    /// pixel (1,0), and buffer[1] bit4 is pixel (1,4).
    ///
    /// An example of a screen driver that uses this format is the SSD1306.
    Mono_8BitPage = 6,
    /// Pixels encoded as 1-bit blue, 1-bit green, 1-bit red,
    /// and 1-bit for opaque (1) vs transparent (0)
    RGB_4BIT = 5,
    /// Pixels encoded as 3-bit red channel, 3-bit green channel, 2-bit blue
    /// channel.
    RGB_332 = 1,
    /// Pixels encoded as 5-bit red channel, 6-bit green channel, 5-bit blue
    /// channel.
    RGB_565 = 2,
    /// Pixels encoded as 8-bit red channel, 8-bit green channel, 8-bit blue
    /// channel.
    RGB_888 = 3,
    /// Pixels encoded as 8-bit alpha channel, 8-bit red channel, 8-bit green
    /// channel, 8-bit blue channel.
    ARGB_8888 = 4,
    // other pixel formats may be defined.
}

impl ScreenPixelFormat {
    pub fn get_bits_per_pixel(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Mono_8BitPage => 1,
            Self::RGB_4BIT => 4,
            Self::RGB_332 => 8,
            Self::RGB_565 => 16,
            Self::RGB_888 => 24,
            Self::ARGB_8888 => 32,
        }
    }
}

/// Interface to configure the screen.
pub trait ScreenSetup<'a> {
    fn set_client(&self, client: &'a dyn ScreenSetupClient);

    /// Set the screen resolution in pixels with `(X, Y)`.
    ///
    /// Returns `Ok(())` if the request is registered and will be sent to the
    /// screen. A `command_complete` callback function will be triggered when
    /// the resolution change is finished and will provide a `Result<(),
    /// ErrorCode>` to indicate if the resolution change was successful.
    ///
    /// Returns `Err(NOSUPPORT)` if the resolution is not supported. No callback will
    /// be triggered.
    fn set_resolution(&self, resolution: (usize, usize)) -> Result<(), ErrorCode>;

    /// Set the pixel format.
    ///
    /// Returns `Ok(())` if the request is registered and will be sent to the
    /// screen. A `command_complete` callback function will be triggered when
    /// the pixel format change is finished and will provide a `Result<(),
    /// ErrorCode>` to indicate if the pixel format change was successful.
    ///
    /// Returns `Err(NOSUPPORT)` if the pixel format is not supported.
    fn set_pixel_format(&self, format: ScreenPixelFormat) -> Result<(), ErrorCode>;

    /// Set the rotation of the display.
    ///
    /// Returns `Ok(())` if the request is registered and will be sent to the
    /// screen. A `command_complete` callback function will be triggered when
    /// the rotation update is finished and will provide a `Result<(),
    /// ErrorCode>` to indicate if the rotation change was successful.
    ///
    /// Note that `Rotated90` or `Rotated270` will swap the width and height.
    fn set_rotation(&self, rotation: ScreenRotation) -> Result<(), ErrorCode>;

    /// Get the number of supported resolutions.
    ///
    /// This must return at least one (the current resolution).
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen (most screens do not support such
    /// a request, resolutions are described in the data sheet).
    fn get_num_supported_resolutions(&self) -> usize;

    /// Get the resolution specified by the given index.
    ///
    /// `index` is from `0..get_num_supported_resolutions()-1` and this returns
    /// a tuple `(width, height)` of the associated resolution (in pixels). Note
    /// that width and height may change due to rotation. Returns `None` if
    /// `index` is invalid.
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen.
    fn get_supported_resolution(&self, index: usize) -> Option<(usize, usize)>;

    /// Get the number of the pixel formats supported.
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen (most screens do not support such
    /// a request, pixel formats are described in the data sheet).
    fn get_num_supported_pixel_formats(&self) -> usize;

    /// Get the pixel format specified by the given index.
    ///
    /// `index` is from `0..get_num_supported_pixel_formats()-1` and this
    /// returns the associated pixel format. Returns `None` if `index` is
    /// invalid.
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen.
    fn get_supported_pixel_format(&self, index: usize) -> Option<ScreenPixelFormat>;
}

/// Basic interface for screens.
pub trait Screen<'a> {
    /// Set the object to receive the asynchronous command callbacks.
    fn set_client(&self, client: &'a dyn ScreenClient);

    /// Get a tuple `(width, height)` with the current resolution (in pixels).
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen.
    ///
    /// Note that width and height may change due to rotation.
    fn get_resolution(&self) -> (usize, usize);

    /// Get the current pixel format.
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen.
    fn get_pixel_format(&self) -> ScreenPixelFormat;

    /// Get the current rotation.
    ///
    /// This function is synchronous as the driver should know this value
    /// without requesting it from the screen.
    fn get_rotation(&self) -> ScreenRotation;

    /// Sets the write frame.
    ///
    /// This function has to be called before the first call to the write
    /// function. This will generate a `command_complete()` callback when
    /// finished.
    ///
    /// Return values:
    /// - `Ok(())`: The write frame is valid.
    /// - `INVAL`: The parameters of the write frame are not valid.
    /// - `BUSY`: Unable to set the write frame on the device.
    fn set_write_frame(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), ErrorCode>;

    /// Write data from `buffer` to the selected write frame.
    ///
    /// When finished, the driver will call the `write_complete()` callback.
    ///
    /// This function can be called multiple times if the write frame is larger
    /// than the size of the available buffer by setting `continue_write` to
    /// `true`. If `continue_write` is false, the buffer write position will be
    /// reset before the data are written.
    ///
    /// Return values:
    /// - `Ok(())`: Write is valid and will be sent to the screen.
    /// - `SIZE`: The buffer is too long for the selected write frame.
    /// - `BUSY`: Another write is in progress.
    fn write(
        &self,
        buffer: SubSliceMut<'static, u8>,
        continue_write: bool,
    ) -> Result<(), ErrorCode>;

    /// Set the display brightness value.
    ///
    /// Depending on the display, this may not cause any actual changes
    /// until and unless power is enabled (see `set_power`).
    ///
    /// The following values must be supported:
    /// - 0: completely no light emitted
    /// - 1..65536: set brightness to the given level
    ///
    /// The display should interpret the brightness value as *lightness* (each
    /// increment should change perceived brightness the same). 1 shall be the
    /// minimum supported brightness, 65536 is the maximum brightness. Values in
    /// between should approximate the intermediate values; minimum and maximum
    /// included (e.g. when there is only 1 level).
    fn set_brightness(&self, brightness: u16) -> Result<(), ErrorCode>;

    /// Controls the screen power supply.
    ///
    /// Use it to initialize the display device.
    ///
    /// For screens where display needs nonzero brightness (e.g. LED), this
    /// shall set brightness to a default value if `set_brightness` was not
    /// called first.
    ///
    /// The device may implement power independently from brightness, so call
    /// `set_brightness` to turn on/off the module completely.
    ///
    /// To allow starting in the correct configuration, the driver is allowed to
    /// cache values like brightness or invert mode and apply them together when
    /// power is enabled. If the display cannot use selected configuration, this
    /// call returns `INVAL`.
    ///
    /// When finished, calls `ScreenClient::screen_is_ready`, both when power
    /// is enabled and disabled.
    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode>;

    /// Controls the color inversion mode.
    ///
    /// Pixels already in the frame buffer, as well as newly submitted, will be
    /// inverted. What that means depends on the current pixel format. May get
    /// disabled when switching to another pixel format. Returns `NOSUPPORT` if
    /// the device does not accelerate color inversion. Returns `INVAL` if the
    /// current pixel format does not support color inversion.
    fn set_invert(&self, enabled: bool) -> Result<(), ErrorCode>;
}

pub trait ScreenAdvanced<'a>: Screen<'a> + ScreenSetup<'a> {}
// Provide blanket implementations for trait group
impl<'a, T: Screen<'a> + ScreenSetup<'a>> ScreenAdvanced<'a> for T {}

pub trait ScreenSetupClient {
    /// The screen will call this function to notify that a command has finished.
    fn command_complete(&self, r: Result<(), ErrorCode>);
}

pub trait ScreenClient {
    /// The screen will call this function to notify that a command (except
    /// write) has finished.
    fn command_complete(&self, result: Result<(), ErrorCode>);

    /// The screen will call this function to notify that the write command has
    /// finished. This is different from `command_complete` as it has to pass
    /// back the write buffer
    fn write_complete(&self, buffer: SubSliceMut<'static, u8>, result: Result<(), ErrorCode>);

    /// Some screens need some time to start, this function is called when the
    /// screen is ready.
    fn screen_is_ready(&self);
}
