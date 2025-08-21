// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::ErrorCode;

use super::super::helpers::copy_to_iter;
use super::ctrl_header::{CtrlHeader, CtrlType};
use super::{VirtIOGPUReq, VirtIOGPUResp};

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
#[allow(dead_code)]
pub enum VideoFormat {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,
    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,
    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResourceCreate2DReq {
    pub ctrl_header: CtrlHeader,
    pub resource_id: u32,
    pub format: VideoFormat,
    pub width: u32,
    pub height: u32,
}

impl VirtIOGPUReq for ResourceCreate2DReq {
    const ENCODED_SIZE: usize = core::mem::size_of::<Self>();
    const CTRL_TYPE: CtrlType = CtrlType::CmdResourceCreate2d;
    type ExpectedResponse = ResourceCreate2DResp;

    fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode> {
        // Write out fields to iterator.
        //
        // This struct doesn't need any padding bytes.
        self.ctrl_header.write_to_byte_iter(dst)?;
        copy_to_iter(dst, u32::to_le_bytes(self.resource_id).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.format as u32).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.width).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.height).into_iter())?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResourceCreate2DResp {
    pub ctrl_header: CtrlHeader,
}

impl VirtIOGPUResp for ResourceCreate2DResp {
    const ENCODED_SIZE: usize = core::mem::size_of::<Self>();
    const EXPECTED_CTRL_TYPE: CtrlType = CtrlType::RespOkNoData;

    fn from_byte_iter_post_checked_ctrl_header(
        ctrl_header: CtrlHeader,
        _src: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ErrorCode> {
        Ok(ResourceCreate2DResp { ctrl_header })
    }
}
