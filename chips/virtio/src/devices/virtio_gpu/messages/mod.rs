// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

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
