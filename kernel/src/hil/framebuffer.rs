//! Interface for FrameBuffer
use crate::returncode::ReturnCode;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
    #[derive(Copy, Clone, PartialEq)]
    pub enum ScreenRotation {
        Normal = 0,
        Rotated90 = 1,
        Rotated180 = 2,
        Rotated270 = 3,
    }
}

impl From<ScreenRotation> for usize {
    fn from(rotation: ScreenRotation) -> usize {
        match rotation {
            ScreenRotation::Normal => 0,
            ScreenRotation::Rotated90 => 1,
            ScreenRotation::Rotated180 => 2,
            ScreenRotation::Rotated270 => 3,
        }
    }
}

enum_from_primitive! {
    #[derive(Copy, Clone, PartialEq)]
    #[allow(non_camel_case_types)]
    pub enum ScreenPixelFormat {
        /// No pixel format
        None = 0,
        /// Pixels encoded as 1-bit, used for monochromatic displays
        Mono = 1,
        /// Pixels encoded as 2-bit red channel, 3-bit green channel, 3-bit blue channel.
        RGB_233 = 2,
        /// Pixels encoded as 5-bit red channel, 6-bit green channel, 5-bit blue channel.
        RGB_565 = 3,
        /// Pixels encoded as 8-bit red channel, 8-bit green channel, 8-bit blue channel.
        RGB_888 = 4,
        /// Pixels encoded as 8-bit alpha channel, 8-bit red channel, 8-bit green channel, 8-bit blue channel.
        ARGB_8888 = 5,
        // other pixel formats may be defined
    }
}

impl ScreenPixelFormat {
    pub fn get_bits_per_pixel(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Mono => 1,
            Self::RGB_233 => 8,
            Self::RGB_565 => 16,
            Self::RGB_888 => 24,
            Self::ARGB_8888 => 32,
        }
    }
}

impl From<ScreenPixelFormat> for usize {
    fn from(pixel_format: ScreenPixelFormat) -> usize {
        match pixel_format {
            ScreenPixelFormat::None => 0,
            ScreenPixelFormat::Mono => 1,
            ScreenPixelFormat::RGB_233 => 2,
            ScreenPixelFormat::RGB_565 => 3,
            ScreenPixelFormat::RGB_888 => 4,
            ScreenPixelFormat::ARGB_8888 => 5,
        }
    }
}

pub trait Screen {
    /// Sets the screen resolution (in pixels). Returns ENOSUPPORT if the resolution is
    /// not supported. The function should return SUCCESS if the request is registered
    /// and will be sent to the screen.
    /// Upon SUCCESS, the caller has to wait for the `command_complete` callback function
    /// that will return the actual ReturnCode after setting the resolution.
    fn set_resolution(&self, resolution: (usize, usize)) -> ReturnCode;

    /// Sets the pixel format. Returns ENOSUPPORT if the pixel format is
    /// not supported. The function should return SUCCESS if the request is registered
    /// and will be sent to the screen.
    /// Upon SUCCESS, the caller has to wait for the `command_complete` callback function
    /// that will return the actual ReturnCode after setting the pixel format.
    fn set_pixel_format(&self, depth: ScreenPixelFormat) -> ReturnCode;

    /// Sets the rotation of the display.
    /// The function should return SUCCESS if the request is registered
    /// and will be sent to the screen.
    /// Upon SUCCESS, the caller has to wait for the `command_complete` callback function
    /// that will return the actual ReturnCode after setting the rotation.
    ///
    /// note this can swap the width with height.
    fn set_rotation(&self, rotation: ScreenRotation) -> ReturnCode;

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

    /// Returns the current rotation.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_rotation(&self) -> ScreenRotation;

    /// Returns the number of the resolutions supported.
    /// should return at least one (the current resolution)
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen (most screens do not support such a request,
    /// resolutions are described in the data sheet).
    ///
    /// If the screen supports such a feature, the driver should request this information
    /// from the screen upfront.
    fn get_supported_resolutions(&self) -> usize;

    /// Can be called with an index from 0 .. count-1 and will
    /// a tuple (width, height) with the current resolution (in pixels).
    /// note that width and height may change due to rotation
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_supported_resolution(&self, index: usize) -> (usize, usize);

    /// Returns the number of the pixel formats supported.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen (most screens do not support such a request,
    /// pixel formats are described in the data sheet).
    ///
    /// If the screen supports such a feature, the driver should request this information
    /// from the screen upfront.
    fn get_supported_pixel_formats(&self) -> usize;

    /// Can be called with index 0 .. count-1 and will returns
    /// the value of each pixel format mode.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_supported_pixel_format(&self, index: usize) -> ScreenPixelFormat;

    /// Sends a write command to write data in the selected video memory window.
    /// The screen will then call ``ScreenClient::fill_next_buffer_for_write`` for
    /// the actual bytes to write. This function will fill the buffer  and return
    /// the number of bytes written. If it returns 0, the write stops and the screen
    /// issues ``ScreenClient::command_complete``.
    /// This avoids triple buffering (an app buffer, a frame buffer buffer and a screen buffer),
    /// data is transfered from the app directly from the AppShare.
    /// This also allows writing a repeated pattern with the app only having to fill a buffer
    /// with one repeated sample. It also allow the screen to have
    /// an internal arbitrary size buffer.
    fn write(&self, x: usize, y: usize, width: usize, height: usize) -> ReturnCode;

    fn set_client(&self, client: Option<&'static dyn ScreenClient>);

    /// Powers up the display.
    fn on(&self) -> ReturnCode;

    /// Powers down the display. The screen should be able to accept data even when the display is off.
    fn off(&self) -> ReturnCode;

    /// Inverts the colors.
    fn invert_on(&self) -> ReturnCode;

    /// Reverts the colors to normal.
    fn invert_off(&self) -> ReturnCode;
}

pub trait ScreenClient {
    /// The screen will then call ``ScreenClient::fill_next_buffer_for_write`` for
    /// the actual bytes to write. This function will fill the buffer  and return
    /// the number of bytes written. If it returns 0, the write stops and the screen
    /// issues ``ScreenClient::command_complete``.
    /// This avoids triple buffering (an app buffer, a frame buffer buffer and a screen buffer),
    /// data is transfered from the app directly from the AppShare.
    /// This also allows writing a repeated pattern with the app only having to fill a buffer
    /// with one repeated sample. It also allow the screen to have
    /// an internal arbitrary size buffer.
    fn fill_next_buffer_for_write(&self, buffer: &'a mut [u8]) -> usize;

    /// The screen will call this function to notify that a command has finished.
    fn command_complete(&self, r: ReturnCode);
}
