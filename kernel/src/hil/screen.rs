//! Interface for screens and displays.
use crate::returncode::ReturnCode;

pub enum ScreenRotation {
    Normal,
    Rotated90,
    Rotated180,
    Rotated270,
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
    /// Text pixel format
    TEXT,
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
            Self::TEXT => 8,
        }
    }
}

pub trait ScreenSetup {
    fn set_client(&self, client: Option<&'static dyn ScreenSetupClient>);

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
    /// Note that in the case of `Rotated90` or `Rotated270`, this will swap the width and height.
    fn set_rotation(&self, rotation: ScreenRotation) -> ReturnCode;

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

    /// Send to the screen a driver specific command
    /// When finished, the driver will call the `command_complete()` callback.
    ///
    /// The return values can be:
    /// - `SUCCESS` - the command was sent with success
    /// - `EBUSY` - anoher command is in progress
    /// - `EINVAL` - the parameters of the function were invalid
    fn screen_command(&self, data1: usize, data2: usize, data3: usize) -> ReturnCode;
}

pub trait Screen {
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

    /// Sets the video memory frame.
    /// This function has to be called before the first call to the write function.
    /// This will generate a `command_complete()` callback when finished.
    ///
    /// Return values:
    /// - `SUCCESS`: The write frame is valid.
    /// - `EINVAL`: The parameters of the write frame are not valid.
    /// - `EBUSY`: Unable to set the write frame on the device.
    fn set_write_frame(&self, x: usize, y: usize, width: usize, height: usize) -> ReturnCode;

    /// Sends a write command to write data in the selected video memory frame.
    /// When finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `SUCCESS`: Write is valid and will be sent to the screen.
    /// - `EINVAL`: Write is invalid or length is wrong.
    /// - `EBUSY`: Another write is in progress.
    fn write(&self, buffer: &'static mut [u8], len: usize) -> ReturnCode;

    /// Set the object to receive the asynchronous command callbacks.
    fn set_client(&self, client: Option<&'static dyn ScreenClient>);

    /// Sets the display brightness and/or powers it off
    /// Screens must implement this function for at least two brightness values (in percent)
    ///     0 - power off,
    ///     otherwise - on, set brightness (if available)
    fn set_brightness(&self, brightness: usize) -> ReturnCode;

    /// Inverts the colors.
    fn invert_on(&self) -> ReturnCode;

    /// Reverts the colors to normal.
    fn invert_off(&self) -> ReturnCode;
}

pub trait ScreenAdvanced: Screen + ScreenSetup {}

pub trait ScreenSetupClient {
    /// The screen will call this function to notify that a command has finished.
    fn command_complete(&self, r: ReturnCode);
}

pub trait ScreenClient {
    /// The screen will call this function to notify that a command (except write) has finished.
    fn command_complete(&self, r: ReturnCode);

    /// The screen will call this function to notify that the write command has finished.
    /// This is different from `command_complete` as it has to pass back the write buffer
    fn write_complete(&self, buffer: &'static mut [u8], r: ReturnCode);

    /// Some screens need some time to start, this function is called when the screen is ready
    fn screen_is_ready(&self);
}
