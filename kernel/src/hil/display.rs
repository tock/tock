//! Interface for screens and displays.
use crate::ErrorCode;
use core::ops::Add;
use core::ops::Sub;

pub const MAX_BRIGHTNESS: u16 = 65536;

#[derive(Copy, Clone, PartialEq)]
pub enum Rotation {
    Normal,
    Rotated90,
    Rotated180,
    Rotated270,
}

impl Add for Rotation {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Rotation::Normal, _) => other,
            (_, Rotation::Normal) => self,
            (Rotation::Rotated90, Rotation::Rotated90) => Rotation::Rotated180,
            (Rotation::Rotated90, Rotation::Rotated180) => Rotation::Rotated270,
            (Rotation::Rotated90, Rotation::Rotated270) => Rotation::Normal,

            (Rotation::Rotated180, Rotation::Rotated90) => Rotation::Rotated270,
            (Rotation::Rotated180, Rotation::Rotated180) => Rotation::Normal,
            (Rotation::Rotated180, Rotation::Rotated270) => Rotation::Rotated90,

            (Rotation::Rotated270, Rotation::Rotated90) => Rotation::Normal,
            (Rotation::Rotated270, Rotation::Rotated180) => Rotation::Rotated90,
            (Rotation::Rotated270, Rotation::Rotated270) => Rotation::Rotated180,
        }
    }
}

impl Sub for Rotation {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (_, Rotation::Normal) => self,

            (Rotation::Normal, Rotation::Rotated90) => Rotation::Rotated270,
            (Rotation::Normal, Rotation::Rotated180) => Rotation::Rotated180,
            (Rotation::Normal, Rotation::Rotated270) => Rotation::Rotated90,

            (Rotation::Rotated90, Rotation::Rotated90) => Rotation::Normal,
            (Rotation::Rotated90, Rotation::Rotated180) => Rotation::Rotated270,
            (Rotation::Rotated90, Rotation::Rotated270) => Rotation::Rotated180,

            (Rotation::Rotated180, Rotation::Rotated90) => Rotation::Rotated90,
            (Rotation::Rotated180, Rotation::Rotated180) => Rotation::Normal,
            (Rotation::Rotated180, Rotation::Rotated270) => Rotation::Rotated270,

            (Rotation::Rotated270, Rotation::Rotated90) => Rotation::Rotated180,
            (Rotation::Rotated270, Rotation::Rotated180) => Rotation::Rotated90,
            (Rotation::Rotated270, Rotation::Rotated270) => Rotation::Normal,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum PixelFormat {
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

impl PixelFormat {
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

#[derive(Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum CharacterFormat {
    /// Characters are encoded using 8 bits ASCII, used for monochromatic displays
    ASCII,
    /// Characters are encoded using UTF8, used for monochromatic displays
    UTF8,
    /// Characters are encoded using 16 bits, as in the VGA text mode
    /// https://en.wikipedia.org/wiki/VGA_text_mode
    /// 0   1   2   3   4   5   6   7   8   9   10  11  12  13  14  15  16
    /// +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    /// | B |Background | Foreground    |          Code Point           |
    /// +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    /// B - blink
    VGA,
}

#[derive(Copy, Clone, PartialEq)]
pub struct GraphicsMode {
    pub frame: GraphicsFrame,
    pub pixel_format: PixelFormat,
}

#[derive(Copy, Clone, PartialEq)]
pub struct TextMode {
    pub frame: TextFrame,
    pub character_format: CharacterFormat,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Align {
    horizontal: usize,
    vertical: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub struct GraphicsFrame {
    width: usize,
    height: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub struct TextFrame {
    columns: usize,
    lines: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Tile {
    pub align: Align,
    pub size: GraphicsFrame,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

/* TextBuffer */
pub trait TextBuffer<'a> {
    fn set_client(&self, client: Option<&'a dyn TextBufferClient>);

    /// Returns a tuple (width, height) with the resolution of the
    /// screen that is being used. This function is synchronous as the
    /// resolution is known by the driver at any moment.
    ///
    /// The resolution is constant.
    fn get_mode(&self) -> TextMode;

    /// Sends a write command to the driver, and the buffer to write from
    /// and the len are sent as arguments. When the `write` operation is
    /// finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The write command is valid and will be sent to the driver.
    /// - `BUSY`: The driver is busy with another command.
    fn print(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Sends to the driver a command to set the cursor at a given position
    /// (x_position, y_position). When finished, the driver will call the
    /// `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn set_cursor_position(&self, position: Point) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to show the cursor. When finished,
    /// the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn set_show_cursor(&self, show: bool) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to turn on the blinking cursor. When finished,
    /// the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn set_blink_cursor(&self, blink: bool) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to clear the display of the screen.
    /// When finished, the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn clear(&self) -> Result<(), ErrorCode>;
}

pub trait TextBufferClient {
    /// The driver calls this function when any command (but a write one)
    /// finishes executing.
    fn command_complete(&self, r: Result<(), ErrorCode>);

    /// The driver calls this function when a write command finishes executing.
    fn write_complete(&self, buffer: &'static mut [u8], len: usize, r: Result<(), ErrorCode>);
}

/* FrameBuffer */

pub trait FrameBuffer<'a> {
    /// Returns a tuple (width, height) with the current resolution (in pixels)
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    ///
    /// note that width and height may change due to rotation
    fn get_mode(&self) -> GraphicsMode;

    /// Returns the format of minimum tile (in bytes) that can be written to the framebuffer.
    ///
    /// Due to hardware constraints, framebuffer writes have to be rounded up to a tile size.
    /// This means that the size of the write buffer has to be a multiple of the tile. An
    /// example use case is a framebuffer that has the minimum write unit a full line on
    /// the display. This means that clients can only write entier lines to the framebuffer
    /// as opossed to single pixels.
    fn get_tile_format(&self) -> Tile;

    /// Sets the video memory frame.
    /// This function has to be called before the first call to the write function.
    /// This will generate a `command_complete()` callback when finished.
    ///
    /// Return values:
    /// - `Ok(())`: The write frame is valid.
    /// - `INVAL`: The parameters of the write frame are not valid.
    /// - `BUSY`: Unable to set the write frame on the device.
    fn set_write_frame(&self, origin: Point, size: GraphicsFrame) -> Result<(), ErrorCode>;

    /// Returns the required buffer padding in the format of
    /// a tuble (free bytes before, free bytes after).
    ///
    /// The supplied buffer has to be
    /// +----------------------+------------+---------------------+
    /// | before padding bytes | frame data | after padding bytes |
    /// +----------------------+------------+---------------------+
    ///
    /// Some displays,like the SSD1306, require some command bytes placed before
    /// and after the actual framebuffer data. Withoutn this padding, the display
    /// driver would have to keep another buffer and additional data copy.
    ///
    /// The HIL's user has to fill in data only in between the padding
    /// bytes. Any data written to the padding bytes might be overwritten
    /// by the underlying display driver.
    fn get_buffer_padding(&self) -> (usize, usize);

    /// Sends a write command to write data in the selected video memory frame.
    /// When finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: Write is valid and will be sent to the screen.
    /// - `INVAL`: Write is invalid or length is wrong.
    /// - `BUSY`: Another write is in progress.
    fn write(
        &self,
        buffer: &'static mut [u8],
        len: usize,
        reset_position: bool,
    ) -> Result<(), ErrorCode>;

    /// Flush the framebuffer changes to the hardware device.
    ///
    /// Some framebuffers keep in a temporary memory the changes and require a flush command
    /// to send the changes to the hardware.
    ///
    /// Return values:
    /// - `Ok(())`: Flush is in progress and the client will receive
    ///    a call to `command_complete`.
    /// - `ENOSUPPORT`: Flush has been done synchronous or there is no
    ///    no need to flush the framebuffer.
    /// - `BUSY`: Another write or flush is in progress.
    fn flush(&self) -> Result<(), ErrorCode>;

    /// Set the object to receive the asynchronous command callbacks.
    fn set_client(&self, client: Option<&'a dyn FrameBufferClient>);
}

pub trait FrameBufferClient {
    /// The framebuffer will call this function to notify that the write command has finished.
    /// This is different from `command_complete` as it has to pass back the write buffer
    fn write_complete(&self, buffer: &'static mut [u8], r: Result<(), ErrorCode>);

    /// The framebuffer will call this function to notify that a command (except `write` and
    /// `write_continue`) has finished.
    fn command_complete(&self, r: Result<(), ErrorCode>);
}

pub trait FrameBufferSetup<'a>: FrameBuffer<'a> {
    /// Sets the screen resolution (in pixels). Returns ENOSUPPORT if the resolution is
    /// not supported. The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the resolution.
    fn set_mode(&self, mode: GraphicsMode) -> Result<(), ErrorCode>;

    /// Returns the number of the resolutions supported.
    /// should return at least one (the current resolution)
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen (most screens do not support such a request,
    /// resolutions are described in the data sheet).
    ///
    /// If the screen supports such a feature, the driver should request this information
    /// from the screen upfront.
    fn get_num_supported_modes(&self) -> usize;

    /// Can be called with an index from 0 .. count-1 and will
    /// a tuple (width, height) with the current resolution (in pixels).
    /// note that width and height may change due to rotation
    ///
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_supported_mode(&self, index: usize) -> Option<GraphicsMode>;
}

/* Screen */
pub trait Screen<'a> {
    /// Returns the current rotation.
    /// This function is synchronous as the driver should know this value without
    /// requesting it from the screen.
    fn get_rotation(&self) -> Rotation;

    /// Sets the rotation of the display.
    /// The function should return Ok(()) if the request is registered
    /// and will be sent to the screen.
    /// Upon Ok(()), the caller has to wait for the `command_complete` callback function
    /// that will return the actual Result<(), ErrorCode> after setting the rotation.
    ///
    /// Note that in the case of `Rotated90` or `Rotated270`, this will swap the width and height.
    fn set_rotation(&self, rotation: Rotation) -> Result<(), ErrorCode>;

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
    /// When finished, calls `ScreenClient::screen_is_ready`,
    /// both when power was enabled and disabled.
    fn set_power(&self, enabled: bool) -> Result<(), ErrorCode>;

    /// Set on or off the inversion of colors.
    fn set_invert(&self, inverted: bool) -> Result<(), ErrorCode>;

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
    /// (each increment should change preceived brightness the same).
    /// 1 shall be the minimum supported brightness,
    /// `MAX_BRIGHTNESS` and greater represent the maximum.
    /// Values in between should approximate the intermediate values;
    /// minimum and maximum included (e.g. when there is only 1 level).
    fn set_brightness(&self, brightness: u16) -> Result<(), ErrorCode>;

    /// Set the object to receive the asynchronous command callbacks.
    fn set_client(&self, client: Option<&'a dyn ScreenClient>);
}

pub trait ScreenClient {
    /// The screen will call this function to notify that a command (except write) has finished.
    fn command_complete(&self, r: Result<(), ErrorCode>);
}

/* Text Display */

pub trait TextDisplay<'a>: TextBuffer<'a> + Screen<'a> {}

// Provide blanket implementation for trait group
impl<'a, T: Screen<'a> + TextBuffer<'a>> TextDisplay<'a> for T {}

/* Graphic Display */

pub trait GraphicDisplay<'a>: Screen<'a> + FrameBuffer<'a> {}

// Provide blanket implementations for trait group
impl<'a, T: Screen<'a> + FrameBuffer<'a>> GraphicDisplay<'a> for T {}

/* Display Advanced */

pub trait FullGraphicDisplay<'a>: GraphicDisplay<'a> + FrameBufferSetup<'a> {}

// Provide blanket implementations for trait group
impl<'a, T: GraphicDisplay<'a> + FrameBufferSetup<'a>> FullGraphicDisplay<'a> for T {}
