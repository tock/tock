//! Hil for FrameBuffer
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
    fn set_resolution(&self, width: usize, height: usize) -> ReturnCode;
    fn set_color_depth(&self, depth: usize) -> ReturnCode;
    fn set_rotation(&self, rotation: ScreenRotation) -> ReturnCode;

    fn get_resolution(&self) -> (usize, usize);
    fn get_color_depth(&self) -> usize;
    fn get_rotation(&self) -> ScreenRotation;

    fn get_resolution_modes(&self) -> usize;
    fn get_resolution_size(&self, index: usize) -> (usize, usize);

    fn get_color_depth_modes(&self) -> usize;
    fn get_color_depth_bits(&self, index: usize) -> usize;

    fn write(&self, x: usize, y: usize, width: usize, height: usize) -> ReturnCode;

    fn set_client(&self, client: Option<&'static dyn ScreenClient>);

    fn on(&self) -> ReturnCode;
    fn off(&self) -> ReturnCode;
}

pub trait ScreenClient {
    fn fill_next_buffer_for_write(&self, buffer: &'a mut [u8]) -> usize;
    fn command_complete(&self, r: ReturnCode);
}
