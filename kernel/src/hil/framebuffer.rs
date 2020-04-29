//! Hil for FrameBuffer
use crate::returncode::ReturnCode;

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenRotation {
    Normal = 0,
    Rotated90 = 1,
    Rotated180 = 2,
    Rotated270 = 3,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenFormat {
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
    fn size(&self) -> (usize, usize);

    fn format(&self) -> ScreenFormat;
    fn rotation(&self) -> ScreenRotation;

    fn write(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        buffer: &'static [u8],
    ) -> ReturnCode;
}

pub trait Configuration {
    fn set_size(&self, width: usize, height: usize) -> ReturnCode;
    fn set_format(&self, format: ScreenFormat) -> ReturnCode;
    fn set_rotation(&self, format: ScreenRotation) -> ReturnCode;
}

pub trait FrameBufferClient {
    fn write_complete(&self, buffer: &'static [u8], r: ReturnCode);
}
