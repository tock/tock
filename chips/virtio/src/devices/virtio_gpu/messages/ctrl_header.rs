// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use super::super::helpers::{bytes_from_iter, copy_to_iter};
use kernel::ErrorCode;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum CtrlType {
    /* 2d commands */
    CmdGetDisplayInfo = 0x0100,
    CmdResourceCreate2d,
    CmdResourceUref,
    CmdSetScanout,
    CmdResourceFlush,
    CmdTransferToHost2d,
    CmdResourceAttachBacking,
    CmdResourceDetachBacking,
    CmdGetCapsetInfo,
    CmdGetCapset,
    CmdGetEdid,

    /* cursor commands */
    CmdUpdateCursor = 0x0300,
    CmdMoveCursor,

    /* success responses */
    RespOkNoData = 0x1100,
    RespOkDisplayInfo,
    RespOkCapsetInfo,
    RespOkCapset,
    RespOkEdid,

    /* error responses */
    RespErrUnspec = 0x1200,
    RespErrOutOfMemory,
    RespErrInvalidScanoutId,
    RespErrInvalidResourceId,
    RespErrInvalidContextId,
    RespErrInvalidParameter,
}

impl TryFrom<u32> for CtrlType {
    type Error = ();

    fn try_from(int: u32) -> Result<Self, Self::Error> {
        match int {
            /* 2d commands */
            v if v == CtrlType::CmdGetDisplayInfo as u32 => Ok(CtrlType::CmdGetDisplayInfo),
            v if v == CtrlType::CmdResourceCreate2d as u32 => Ok(CtrlType::CmdResourceCreate2d),
            v if v == CtrlType::CmdResourceUref as u32 => Ok(CtrlType::CmdResourceUref),
            v if v == CtrlType::CmdSetScanout as u32 => Ok(CtrlType::CmdSetScanout),
            v if v == CtrlType::CmdResourceFlush as u32 => Ok(CtrlType::CmdResourceFlush),
            v if v == CtrlType::CmdTransferToHost2d as u32 => Ok(CtrlType::CmdTransferToHost2d),
            v if v == CtrlType::CmdResourceAttachBacking as u32 => {
                Ok(CtrlType::CmdResourceAttachBacking)
            }
            v if v == CtrlType::CmdResourceDetachBacking as u32 => {
                Ok(CtrlType::CmdResourceDetachBacking)
            }
            v if v == CtrlType::CmdGetCapsetInfo as u32 => Ok(CtrlType::CmdGetCapsetInfo),
            v if v == CtrlType::CmdGetCapset as u32 => Ok(CtrlType::CmdGetCapset),
            v if v == CtrlType::CmdGetEdid as u32 => Ok(CtrlType::CmdGetEdid),

            /* cursor commands */
            v if v == CtrlType::CmdUpdateCursor as u32 => Ok(CtrlType::CmdUpdateCursor),
            v if v == CtrlType::CmdMoveCursor as u32 => Ok(CtrlType::CmdMoveCursor),

            /* success responses */
            v if v == CtrlType::RespOkNoData as u32 => Ok(CtrlType::RespOkNoData),
            v if v == CtrlType::RespOkDisplayInfo as u32 => Ok(CtrlType::RespOkDisplayInfo),
            v if v == CtrlType::RespOkCapsetInfo as u32 => Ok(CtrlType::RespOkCapsetInfo),
            v if v == CtrlType::RespOkCapset as u32 => Ok(CtrlType::RespOkCapset),
            v if v == CtrlType::RespOkEdid as u32 => Ok(CtrlType::RespOkEdid),

            /* error responses */
            v if v == CtrlType::RespErrUnspec as u32 => Ok(CtrlType::RespErrUnspec),
            v if v == CtrlType::RespErrOutOfMemory as u32 => Ok(CtrlType::RespErrOutOfMemory),
            v if v == CtrlType::RespErrInvalidScanoutId as u32 => {
                Ok(CtrlType::RespErrInvalidScanoutId)
            }
            v if v == CtrlType::RespErrInvalidResourceId as u32 => {
                Ok(CtrlType::RespErrInvalidResourceId)
            }
            v if v == CtrlType::RespErrInvalidContextId as u32 => {
                Ok(CtrlType::RespErrInvalidContextId)
            }
            v if v == CtrlType::RespErrInvalidParameter as u32 => {
                Ok(CtrlType::RespErrInvalidParameter)
            }

            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct CtrlHeader {
    pub ctrl_type: CtrlType,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub padding: u32,
}

impl CtrlHeader {
    pub const ENCODED_SIZE: usize = core::mem::size_of::<Self>();

    pub fn write_to_byte_iter<'a>(
        &self,
        dst: &mut impl Iterator<Item = &'a mut u8>,
    ) -> Result<(), ErrorCode> {
        // Write out fields to iterator.
        //
        // This struct doesn't need any padding bytes.
        copy_to_iter(dst, u32::to_le_bytes(self.ctrl_type as u32).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.flags).into_iter())?;
        copy_to_iter(dst, u64::to_le_bytes(self.fence_id).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.ctx_id).into_iter())?;
        copy_to_iter(dst, u32::to_le_bytes(self.padding).into_iter())?;

        Ok(())
    }

    pub fn from_byte_iter(src: &mut impl Iterator<Item = u8>) -> Result<Self, ErrorCode> {
        let ctrl_type = CtrlType::try_from(u32::from_le_bytes(bytes_from_iter(src)?))
            .map_err(|()| ErrorCode::INVAL)?;
        let flags = u32::from_le_bytes(bytes_from_iter(src)?);
        let fence_id = u64::from_le_bytes(bytes_from_iter(src)?);
        let ctx_id = u32::from_le_bytes(bytes_from_iter(src)?);
        let padding = u32::from_le_bytes(bytes_from_iter(src)?);

        Ok(CtrlHeader {
            ctrl_type,
            flags,
            fence_id,
            ctx_id,
            padding,
        })
    }
}
