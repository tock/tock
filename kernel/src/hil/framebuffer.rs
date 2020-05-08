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

pub trait Screen {
    /// Sets the screen resolution (in pixels). Returns ENOSUPPORT if the resolution is
    /// not supported. The function should return SUCCESS for at least one resolution.
    fn set_resolution(&self, width: usize, height: usize) -> ReturnCode;

    /// Sets the color depth (in bits per pixel). Returns ENOSUPPORT if the color depth is
    /// not supported. The function should return SUCCESS for at least one color depth.
    fn set_color_depth(&self, depth: usize) -> ReturnCode;

    /// Sets the rotation of the display.
    /// note this can swap the width with height.
    fn set_rotation(&self, rotation: ScreenRotation) -> ReturnCode;

    /// Returns a tuple (width, height) with the current resolution (in pixels)
    /// note that width and height may change due to rotation
    /// This function is synchronous.
    fn get_resolution(&self) -> (usize, usize);

    /// Returns the current color depth (in bits per pixel)
    /// This function is synchronous.
    fn get_color_depth(&self) -> usize;

    /// Returns the current rotation.
    /// This function is synchronous.
    fn get_rotation(&self) -> ScreenRotation;

    /// Returns the number of the resolutions supported.
    /// should return at least one (the current resolution)
    /// This function is synchronous.
    fn get_resolution_modes(&self) -> usize;

    /// Can be called with an index from 0 .. count-1 and will
    /// a tuple (width, height) with the current resolution (in pixels).
    /// note that width and height may change due to rotation
    /// This function is synchronous.
    fn get_resolution_size(&self, index: usize) -> (usize, usize);

    /// Returns the number of the color depths supported.
    /// This function is synchronous.
    fn get_color_depth_modes(&self) -> usize;

    /// Can be called with index 0 .. count-1 and will returns
    /// the value of each color depth mode (in bits per pixel).
    /// This function is synchronous.
    fn get_color_depth_bits(&self, index: usize) -> usize;

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

    /// Inits the screen
    fn init(&self) -> ReturnCode;

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
