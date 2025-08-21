// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::ErrorCode;

use super::super::helpers::copy_to_iter;
use super::ctrl_header::{CtrlHeader, CtrlType};
use super::{Rect, VirtIOGPUReq, VirtIOGPUResp};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResourceFlushReq {
    pub ctrl_header: CtrlHeader,
    pub r: Rect,
    pub resource_id: u32,
    pub padding: u32,
}

impl VirtIOGPUReq for ResourceFlushReq {
    const ENCODED_SIZE: usize = core::mem::size_of::<Self>();
    const CTRL_TYPE: CtrlType = CtrlType::CmdResourceFlush;
    type ExpectedResponse = ResourceFlushResp;

    fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode> {
        // Write out fields to iterator.
        //
        // This struct doesn't need any padding bytes.
        self.ctrl_header.write_to_byte_iter(dst)?;
        self.r.write_to_byte_iter(dst)?;
        copy_to_iter(dst, u32::to_le_bytes(self.resource_id).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.padding).into_iter())?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResourceFlushResp {
    pub ctrl_header: CtrlHeader,
}

impl VirtIOGPUResp for ResourceFlushResp {
    const ENCODED_SIZE: usize = core::mem::size_of::<Self>();
    const EXPECTED_CTRL_TYPE: CtrlType = CtrlType::RespOkNoData;

    fn from_byte_iter_post_checked_ctrl_header(
        ctrl_header: CtrlHeader,
        _src: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ErrorCode> {
        Ok(ResourceFlushResp { ctrl_header })
    }
}
