//! Interfaces for screens and displays.
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

/// Pixel format and color depth information.
///
/// In formats where pixels don't fall on byte boundaries,
/// most significant bits encode pixels more to the left.
#[derive(Copy, Clone, PartialEq)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum ScreenPixelFormat {
    /// Pixels encoded as 1-bit, used for monochromatic displays.
    /// Leftmost pixels occupies the higher bits.
    Mono,
    /// Each pixel is a 4-byte half-word. Each color channel is 1 bit: RGBX,
    /// where X is padding.
    /// Leftmost pixel occupies the higher word.
    RGB_4,
    /// Pixels encoded as 2-bit red channel, 3-bit green channel, 3-bit blue channel.
    RGB_233,
    /// Pixels encoded as 5-bit red channel, 6-bit green channel, 5-bit blue channel.
    RGB_565,
    /// Pixels encoded as 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    RGB_888,
    /// Pixels encoded as 8-bit alpha channel, 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    ARGB_8888,
    /// For cases that were not taken into account. Specifies size in bits.
    Other(u8),
}

impl ScreenPixelFormat {
    const CHOICES_MASK: u32 = 0xffffff;

    pub fn get_bits_per_pixel(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::RGB_4 => 4,
            Self::RGB_233 => 8,
            Self::RGB_565 => 16,
            Self::RGB_888 => 24,
            Self::ARGB_8888 => 32,
            Self::Other(size) => *size as usize,
        }
    }

    /// Helper for encoding.
    pub fn pack(&self) -> u32 {
        match self {
            ScreenPixelFormat::Mono => 0,
            ScreenPixelFormat::RGB_233 => 1,
            ScreenPixelFormat::RGB_565 => 2,
            ScreenPixelFormat::RGB_888 => 3,
            ScreenPixelFormat::ARGB_8888 => 4,
            ScreenPixelFormat::RGB_4 => 5,
            ScreenPixelFormat::Other(depth) => Self::CHOICES_MASK | ((*depth as u32) << 24),
        }
    }

    /// Helper for decoding. Returns None for unsupported formats.
    pub fn unpack(val: u32) -> Option<Self> {
        match val & Self::CHOICES_MASK {
            0 => Some(ScreenPixelFormat::Mono),
            1 => Some(ScreenPixelFormat::RGB_233),
            2 => Some(ScreenPixelFormat::RGB_565),
            3 => Some(ScreenPixelFormat::RGB_888),
            4 => Some(ScreenPixelFormat::ARGB_8888),
            5 => Some(ScreenPixelFormat::RGB_4),
            Self::CHOICES_MASK => Some(ScreenPixelFormat::Other((val >> 24) as u8)),
            _ => None,
        }
    }
}

/// Describes a rectangular area in the frame buffer. Sized in pixels.
pub struct Area {
    /// First column
    pub x: usize,
    /// First row
    pub y: usize,
    /// Column count
    pub width: usize,
    /// Row count
    pub height: usize,
}

/// A frame buffer.
///
/// The frame buffer may be stored on the device itself and inaccessible.
pub trait FrameBuffer {
    /// Returns a tuple (width, height) with the current resolution (in pixels)
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    ///
    /// note that width and height may change due to rotation
    fn get_resolution(&self) -> (usize, usize);

    /// Returns the current pixel format
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_pixel_format(&self) -> ScreenPixelFormat;

    /// Sets the video memory area.
    ///
    /// This function has to be called before the first call to the write function.
    ///
    /// Area's validity depends on the current pixel format:
    /// - the first pixel of the first column must start on a byte boundary, and
    /// - the last pixel of the last column must start on a byte boundary.
    /// This is to reduce ambiguity regarding partial bytes.
    ///
    /// This will generate a `command_complete()` callback when finished.
    ///
    /// Return values:
    /// - `Ok(())`: The write area is valid.
    /// - `INVAL`: The parameters of the write area are not valid.
    /// - `BUSY`: Unable to set the write area on the device.
    fn set_write_area(&self, area: &Area) -> Result<(), ErrorCode>;

    /// Sends a write command to write data in the selected video memory area.
    /// When finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: Write is valid and will be sent to the screen.
    /// - `RESERVE`: No write area reserved.
    /// - `INVAL`: Write is invalid or length is wrong.
    /// - `BUSY`: Another write is in progress.
    fn write(&self, buffer: &'static mut [u8], len: usize) -> Result<(), ErrorCode>;

    /// Applies all unapplied write commands.
    ///
    /// When finished, the driver shall call `ScreenClient::write_complete`.
    /// Returns `BUSY` if another operation is in progress,
    /// or `ENOSUPPORT` if the driver automatically applies all writes.
    fn flush(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /// Set the object to receive the asynchronous command callbacks.
    fn set_client(&self, client: Option<&'static dyn ScreenClient>);
}

/// Describes a display module with physical properties.
///
/// Must be implemented on top of a `FrameBuffer`.
pub trait ScreenModule {
    /// Sets the display brightness on a logarithmic scale
    ///
    /// Displays should implement this function for at least 0 and 1.
    /// - 0 - completely no light emitted
    /// - otherwise - on, set brightness to value
    fn set_brightness(&self, brightness: usize) -> Result<(), ErrorCode>;

    /// Controls the screen power supply.
    ///
    /// Use it to initialize the display device.
    ///
    /// Does not control backlight power (if applicable),
    /// so call `set_brightness` to turn on/off the module completely.
    ///
    /// When finished, calls `ScreenClient::screen_is_ready`,
    /// both when power was enabled and disabled.
    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode>;
}

/// A screen which has some extra processing capabilities.
///
/// Only available to drivers implementing FrameBuffer.
pub trait FixedFunctionScreen {
    /// Sets the rotation of the display.
    ///
    /// Pixels already in the frame buffer are not affected,
    /// but newly submitted pixels follow the new directions and dimensions.
    ///
    /// The call to `set_rotation` shall invalidate the selected write area.
    ///
    /// The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the rotation.
    ///
    /// Note that in the case of `Rotated90` or `Rotated270`, this will swap the width and height.
    /// Submitting pixels still respects the same byte boundary rules.
    ///
    /// Returns `ENOSUPPORT` if the device does not accelerate rotation.
    fn set_rotation(&self, rotation: ScreenRotation) -> Result<(), ErrorCode>;

    /// Returns the current rotation.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_rotation(&self) -> ScreenRotation;

    /// Enables color inversion mode.
    ///
    /// Pixels already in the frame buffer, as well as newly submited,
    /// will be inverted.
    /// Returns ENOSUPPORT if the device does not accelerate color inversion.
    fn set_invert(&self, enable: bool) -> Result<(), ErrorCode>;
}

pub trait ScreenSetup {
    fn set_client(&self, client: Option<&'static dyn ScreenSetupClient>);

    /// Sets the screen resolution (in pixels). Returns ENOSUPPORT if the resolution is
    /// not supported. The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// The selected write area shall be immediately invalidated.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the resolution.
    fn set_resolution(&self, resolution: (usize, usize)) -> Result<(), ErrorCode>;

    /// Sets the pixel format. Returns ENOSUPPORT if the pixel format is
    /// not supported. The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// The selected write area shall be immediately invalidated.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the pixel format.
    fn set_pixel_format(&self, depth: ScreenPixelFormat) -> Result<(), ErrorCode>;

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

// Provide blanket implementations for trait group
pub trait Screen: FrameBuffer + ScreenModule {}
impl<T: FrameBuffer + ScreenModule> Screen for T {}

pub trait ScreenAdvanced: Screen + ScreenSetup {}
impl<T: Screen + ScreenSetup> ScreenAdvanced for T {}

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

    /// Some screens need some time to start, this function is called when the screen is ready
    fn screen_is_ready(&self);
}
