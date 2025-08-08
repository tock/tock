use kernel::ErrorCode;

pub(crate) mod ctrl_header;
pub(crate) mod resource_attach_backing;
pub(crate) mod resource_create_2d;
pub(crate) mod resource_detach_backing;
pub(crate) mod resource_flush;
pub(crate) mod set_scanout;
pub(crate) mod transfer_to_host_2d;

use super::helpers::copy_to_iter;
use ctrl_header::{CtrlHeader, CtrlType};

pub trait VirtIOGPUReq {
    const ENCODED_SIZE: usize;
    const CTRL_TYPE: CtrlType;
    type ExpectedResponse;

    fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode>;
}

pub trait VirtIOGPUResp {
    const ENCODED_SIZE: usize;
    const EXPECTED_CTRL_TYPE: CtrlType;

    fn from_byte_iter_post_checked_ctrl_header(
        ctrl_header: CtrlHeader,
        src: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ErrorCode>
    where
        Self: Sized;

    fn from_byte_iter_post_ctrl_header(
        ctrl_header: CtrlHeader,
        src: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ErrorCode>
    where
        Self: Sized,
    {
        if ctrl_header.ctrl_type == Self::EXPECTED_CTRL_TYPE {
            Self::from_byte_iter_post_checked_ctrl_header(ctrl_header, src)
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    #[allow(dead_code)]
    fn from_byte_iter(src: &mut impl Iterator<Item = u8>) -> Result<Self, ErrorCode>
    where
        Self: Sized,
    {
        let ctrl_header = CtrlHeader::from_byte_iter(src)?;
        Self::from_byte_iter_post_ctrl_header(ctrl_header, src)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn empty() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 && self.height == 0
    }

    // pub fn extend(&self, other: Rect) -> Rect {
    //     use core::cmp::{max, min};

    //     // If either one of the `Rect`s is empty, simply return the other:
    //     if self.is_empty() {
    //         other
    //     } else if other.is_empty() {
    //         *self
    //     } else {
    //         // Determine the "x1" for both self and other, so that we can calculate
    //         // the final width based on the distance of the larger of the two "x0"s
    //         // and the larger of the two "x1"s:
    //         let self_x1 = self.x.saturating_add(self.width);
    //         let other_x1 = other.x.saturating_add(other.width);

    //         // Same for "y1"s:
    //         let self_y1 = self.y.saturating_add(self.height);
    //         let other_y1 = other.y.saturating_add(other.height);

    //         // Now, build the rect:
    //         let new_x0 = min(self.x, other.x);
    //         let new_x1 = max(self_x1, other_x1);
    //         let new_y0 = min(self.y, other.y);
    //         let new_y1 = max(self_y1, other_y1);
    //         Rect {
    //             x: new_x0,
    //             y: new_y0,
    //             width: new_x1.saturating_sub(new_x0),
    //             height: new_y1.saturating_sub(new_y0),
    //         }
    //     }
    // }

    fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode> {
        // Write out fields to iterator.
        //
        // This struct doesn't need any padding bytes.
        copy_to_iter(dst, u32::to_le_bytes(self.x).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.y).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.width).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.height).into_iter())?;

        Ok(())
    }
}
