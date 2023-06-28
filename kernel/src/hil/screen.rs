// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

/*! Interface for screens and displays.

The interfaces exposed here cover both configurable (`ScreenSetup`),
and less configurable hardware (only `Screen`).

It's composed of 4 main kinds of requests:
- set power,
- read configuration (e.g. `get_resolution`),
- configure (e.g. `set_invert`),
- write buffer.

All requests, except for `Screen::set_power`, can return `OFF`
under some circumstances.

For buffer writes, it's when the display is powered off.

While the display is not powered on, the user could try to configure it.
In that case, the driver MUST either cache the value, or return `OFF`.
This is to let the user power the display in the desired configuration.

Configuration reads shall return the actual state of the display.
In situations where a parameter cannot be configured
(e.g. fixed resolution), they return value may be hardcoded.
Otherwise, the driver should query the hardware directly,
and return OFF if it's not powered.

Configuration sets cause a `command_complete` callback
unless noted otherwise.
*/

use crate::ErrorCode;
use core::ops::Add;
use core::ops::Sub;

pub const MAX_BRIGHTNESS: usize = 65536;

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

#[derive(Copy, Clone, PartialEq)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum ScreenPixelFormat {
    /// Pixels encoded as 1-bit, used for monochromatic displays
    Mono,
    /// Pixels encoded as 2-bit red channel, 3-bit green channel, 3-bit blue channel.
    RGB_233,
    /// Pixels encoded as 5-bit red channel, 6-bit green channel, 5-bit blue channel.
    RGB_565,
    /// Pixels encoded as 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    RGB_888,
    /// Pixels encoded as 8-bit alpha channel, 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    ARGB_8888,
    // other pixel formats may be defined.
}

impl ScreenPixelFormat {
    pub fn get_bits_per_pixel(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::RGB_233 => 8,
            Self::RGB_565 => 16,
            Self::RGB_888 => 24,
            Self::ARGB_8888 => 32,
        }
    }
}

pub trait ScreenSetup<'a> {
    fn set_client(&self, client: Option<&'a dyn ScreenSetupClient>);

    /// Sets the screen resolution (in pixels). Returns ENOSUPPORT if the resolution is
    /// not supported. The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the resolution.
    fn set_resolution(&self, resolution: (usize, usize)) -> Result<(), ErrorCode>;

    /// Sets the pixel format. Returns ENOSUPPORT if the pixel format is
    /// not supported. The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the pixel format.
    fn set_pixel_format(&self, depth: ScreenPixelFormat) -> Result<(), ErrorCode>;

    /// Sets the rotation of the display.
    /// The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the rotation.
    ///
    /// Note that in the case of `Rotated90` or `Rotated270`, this will swap the width and height.
    fn set_rotation(&self, rotation: ScreenRotation) -> Result<(), ErrorCode>;

    /// Returns the number of the resolutions supported.
    /// should return at least one (the current resolution)
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen (most screens do not support such a request,
    /// resolutions are described in the data sheet).
    ///
    /// If the screen supports such a feature, the driver should request this information
    /// from the screen upfront.
    fn get_num_supported_resolutions(&self) -> usize;

    /// Can be called with an index from 0 .. count-1 and will
    /// a tuple (width, height) with the current resolution (in pixels).
    /// note that width and height may change due to rotation
    ///
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_supported_resolution(&self, index: usize) -> Option<(usize, usize)>;

    /// Returns the number of the pixel formats supported.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen (most screens do not support such a request,
    /// pixel formats are described in the data sheet).
    ///
    /// If the screen supports such a feature, the driver should request this information
    /// from the screen upfront.
    fn get_num_supported_pixel_formats(&self) -> usize;

    /// Can be called with index 0 .. count-1 and will return
    /// the value of each pixel format mode.
    ///
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_supported_pixel_format(&self, index: usize) -> Option<ScreenPixelFormat>;
}

/// The basic trait for screens
pub trait Screen<'a> {
    /// Returns a tuple (width, height) with the current resolution (in pixels)
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    ///
    /// Note that width and height may change due to rotation.
    fn get_resolution(&self) -> (usize, usize);

    /// Returns the current pixel format
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_pixel_format(&self) -> ScreenPixelFormat;

    /// Returns the current rotation.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_rotation(&self) -> ScreenRotation;

    /// Sets the video memory frame.
    /// This function has to be called before the first call to the write function.
    /// This will generate a `command_complete()` callback when finished.
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

    /// Sends a write command to write data in the selected video memory frame.
    /// When finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: Write is valid and will be sent to the screen.
    /// - `INVAL`: Write is invalid or length is wrong.
    /// - `BUSY`: Another write is in progress.
    fn write(&self, buffer: &'static mut [u8], len: usize) -> Result<(), ErrorCode>;

    /// Sends a write command to write data in the selected video memory frame
    /// without resetting the video memory frame position. It "continues" the
    /// write from the previous position.
    /// This allows using buffers that are smaller than the video mameory frame.
    /// When finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: Write is valid and will be sent to the screen.
    /// - `INVAL`: Write is invalid or length is wrong.
    /// - `BUSY`: Another write is in progress.
    fn write_continue(&self, buffer: &'static mut [u8], len: usize) -> Result<(), ErrorCode>;

    /// Set the object to receive the asynchronous command callbacks.
    fn set_client(&self, client: Option<&'a dyn ScreenClient>);

    /// Sets the display brightness value
    ///
    /// Depending on the display, this may not cause any actual changes
    /// until and unless power is enabled (see `set_power`).
    ///
    /// The following values must be supported:
    /// - 0 - completely no light emitted
    /// - 1..MAX_BRIGHTNESS - on, set brightness to the given level
    ///
    /// The display should interpret the brightness value as *lightness*
    /// (each increment should change perceived brightness the same).
    /// 1 shall be the minimum supported brightness,
    /// `MAX_BRIGHTNESS` and greater represent the maximum.
    /// Values in between should approximate the intermediate values;
    /// minimum and maximum included (e.g. when there is only 1 level).
    fn set_brightness(&self, brightness: usize) -> Result<(), ErrorCode>;

    /// Controls the screen power supply.
    ///
    /// Use it to initialize the display device.
    ///
    /// For screens where display needs nonzero brightness (e.g. LED),
    /// this shall set brightness to a default value
    /// if `set_brightness` was not called first.
    ///
    /// The device may implement power independently from brightness,
    /// so call `set_brightness` to turn on/off the module completely.
    ///
    /// To allow starting in the correct configuration,
    /// the driver is allowed to cache values like brightness or invert mode
    /// and apply them together when power is enabled.
    /// If the display cannot use selected configuration, this call returns `INVAL`.
    ///
    /// When finished, calls `ScreenClient::screen_is_ready`,
    /// both when power was enabled and disabled.
    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode>;

    /// Controls the color inversion mode.
    ///
    /// Pixels already in the frame buffer, as well as newly submitted,
    /// will be inverted. What that means depends on the current pixel format.
    /// May get disabled when switching to another pixel format.
    /// Returns ENOSUPPORT if the device does not accelerate color inversion.
    /// Returns EINVAL if the current pixel format does not support color inversion.
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
    /// The screen will call this function to notify that a command (except write) has finished.
    fn command_complete(&self, r: Result<(), ErrorCode>);

    /// The screen will call this function to notify that the write command has finished.
    /// This is different from `command_complete` as it has to pass back the write buffer
    fn write_complete(&self, buffer: &'static mut [u8], r: Result<(), ErrorCode>);

    /// Some screens need some time to start, this function is called when the screen is ready.
    fn screen_is_ready(&self);
}
