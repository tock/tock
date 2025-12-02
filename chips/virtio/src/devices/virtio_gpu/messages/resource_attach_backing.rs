// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::ErrorCode;

use super::super::helpers::copy_to_iter;
use super::ctrl_header::{CtrlHeader, CtrlType};
use super::{VirtIOGPUReq, VirtIOGPUResp};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MemEntry {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}

impl MemEntry {
    fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode> {
        // Write out fields to iterator.
        //
        // This struct doesn't need any padding bytes.
        copy_to_iter(dst, u64::to_le_bytes(self.addr).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.length).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.padding).into_iter())?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResourceAttachBackingReq<const ENTRIES: usize> {
    pub ctrl_header: CtrlHeader,
    pub resource_id: u32,
    pub nr_entries: u32,
    pub entries: [MemEntry; ENTRIES],
}

impl<const ENTRIES: usize> VirtIOGPUReq for ResourceAttachBackingReq<ENTRIES> {
    const ENCODED_SIZE: usize = core::mem::size_of::<Self>();
    const CTRL_TYPE: CtrlType = CtrlType::CmdResourceAttachBacking;
    type ExpectedResponse = ResourceAttachBackingResp;

    fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode> {
        // Write out fields to iterator.
        //
        // This struct doesn't need any padding bytes.
        self.ctrl_header.write_to_byte_iter(dst)?;
        copy_to_iter(dst, u32::to_le_bytes(self.resource_id).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.nr_entries).into_iter())?;

        for entry in self.entries {
            entry.write_to_byte_iter(dst)?;
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ResourceAttachBackingResp {
    pub ctrl_header: CtrlHeader,
}

impl VirtIOGPUResp for ResourceAttachBackingResp {
    const ENCODED_SIZE: usize = core::mem::size_of::<Self>();
    const EXPECTED_CTRL_TYPE: CtrlType = CtrlType::RespOkNoData;

    fn from_byte_iter_post_checked_ctrl_header(
        ctrl_header: CtrlHeader,
        _src: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ErrorCode> {
        Ok(ResourceAttachBackingResp { ctrl_header })
    }
}
