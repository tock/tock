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
            v if v == Self::CmdGetDisplayInfo as u32 => Ok(Self::CmdGetDisplayInfo),
            v if v == Self::CmdResourceCreate2d as u32 => Ok(Self::CmdResourceCreate2d),
            v if v == Self::CmdResourceUref as u32 => Ok(Self::CmdResourceUref),
            v if v == Self::CmdSetScanout as u32 => Ok(Self::CmdSetScanout),
            v if v == Self::CmdResourceFlush as u32 => Ok(Self::CmdResourceFlush),
            v if v == Self::CmdTransferToHost2d as u32 => Ok(Self::CmdTransferToHost2d),
            v if v == Self::CmdResourceAttachBacking as u32 => Ok(Self::CmdResourceAttachBacking),
            v if v == Self::CmdResourceDetachBacking as u32 => Ok(Self::CmdResourceDetachBacking),
            v if v == Self::CmdGetCapsetInfo as u32 => Ok(Self::CmdGetCapsetInfo),
            v if v == Self::CmdGetCapset as u32 => Ok(Self::CmdGetCapset),
            v if v == Self::CmdGetEdid as u32 => Ok(Self::CmdGetEdid),

            /* cursor commands */
            v if v == Self::CmdUpdateCursor as u32 => Ok(Self::CmdUpdateCursor),
            v if v == Self::CmdMoveCursor as u32 => Ok(Self::CmdMoveCursor),

            /* success responses */
            v if v == Self::RespOkNoData as u32 => Ok(Self::RespOkNoData),
            v if v == Self::RespOkDisplayInfo as u32 => Ok(Self::RespOkDisplayInfo),
            v if v == Self::RespOkCapsetInfo as u32 => Ok(Self::RespOkCapsetInfo),
            v if v == Self::RespOkCapset as u32 => Ok(Self::RespOkCapset),
            v if v == Self::RespOkEdid as u32 => Ok(Self::RespOkEdid),

            /* error responses */
            v if v == Self::RespErrUnspec as u32 => Ok(Self::RespErrUnspec),
            v if v == Self::RespErrOutOfMemory as u32 => Ok(Self::RespErrOutOfMemory),
            v if v == Self::RespErrInvalidScanoutId as u32 => Ok(Self::RespErrInvalidScanoutId),
            v if v == Self::RespErrInvalidResourceId as u32 => Ok(Self::RespErrInvalidResourceId),
            v if v == Self::RespErrInvalidContextId as u32 => Ok(Self::RespErrInvalidContextId),
            v if v == Self::RespErrInvalidParameter as u32 => Ok(Self::RespErrInvalidParameter),

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

        Ok(Self {
            ctrl_type,
            flags,
            fence_id,
            ctx_id,
            padding,
        })
    }
}
