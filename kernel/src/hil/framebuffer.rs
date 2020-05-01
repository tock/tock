//! Hil for FrameBuffer
use crate::returncode::ReturnCode;
use crate::{AppSlice, Shared};

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenRotation {
    Normal = 0,
    Rotated90 = 1,
    Rotated180 = 2,
    Rotated270 = 3,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenColorFormat {
    /// Monochromatic display
    Mono = 0,
    /// 24 bit color display
    Rgb888 = 1,
    /// 16 bit color display
    Rgb565 = 2,
    /// 12 bit color display
    Rgb444 = 3,
}

pub trait Screen {
    fn get_resolution(&self) -> (usize, usize);

    fn get_color_format(&self) -> ScreenColorFormat;
    fn get_rotation(&self) -> ScreenRotation;

    fn write_slice(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        slice: AppSlice<Shared, u8>,
        len: usize
    ) -> ReturnCode;

    fn write_buffer(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        buffer: &'static [u8],
        len: usize
    ) -> ReturnCode;

    fn set_client (&self, client: Option<&'static dyn ScreenClient>);
}

pub trait ScreenConfiguration {
    fn set_resolution(&self, width: usize, height: usize) -> ReturnCode;
    fn set_color_format(&self, format: ScreenColorFormat) -> ReturnCode;
    fn set_rotation(&self, format: ScreenRotation) -> ReturnCode;
}

pub trait ScreenClient {
    fn write_complete(&self, r: ReturnCode);
}
