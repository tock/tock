/*
 * Copyright (c) 2015, Texas Instruments Incorporated - http://www.ti.com/
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived
 *    from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
 * FOR A PARTICULAR PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE
 * COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
 * OF THE POSSIBILITY OF SUCH DAMAGE.
*/

// Clock switching and source select code from Texas Instruments
// The registers and fields are undefined in the technical reference
// manual necesistating this component until it is revealed to the world.
#![allow(non_snake_case)]
pub unsafe extern "C" fn clock_source_set(ui32src_clk: u32, ui32osc: u32) {
    if ui32src_clk & 0x1u32 != 0 {
        // ui32Base, ui32Reg, ui32Mask, ui32Shift, ui32Data
        ddi0::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x1u32, 0u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x4u32 != 0 {
        ddi0::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0xcu32, 2u32, ui32osc as (u16));
    }
}

pub unsafe extern "C" fn clock_source_get(ui32src_clk: u32) -> u32 {
    let ui32clock_source: u32;
    if ui32src_clk == 0x4u32 {
        ui32clock_source =
            ddi0::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x60000000u32, 29u32) as (u32);
    } else {
        ui32clock_source =
            ddi0::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x10000000u32, 28u32) as (u32);
    }
    ui32clock_source
}
#[allow(unused)]
unsafe fn source_ready() -> bool {
    (if ddi0::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x1u32, 0u32) != 0 {
        1i32
    } else {
        0i32
    }) != 0
}

#[derive(Copy)]
#[repr(C)]
pub struct RomFuncTable {
    pub _Crc32: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    pub _FlashGetSize: unsafe extern "C" fn() -> u32,
    pub _GetChipId: unsafe extern "C" fn() -> u32,
    pub _ReservedLocation1: unsafe extern "C" fn(u32) -> u32,
    pub _ReservedLocation2: unsafe extern "C" fn() -> u32,
    pub _ReservedLocation3: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    pub _ResetDevice: unsafe extern "C" fn(),
    pub _Fletcher32: unsafe extern "C" fn(*mut u16, u16, u16) -> u32,
    pub _MinValue: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _MaxValue: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _MeanValue: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _StandDeviationValue: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _ReservedLocation4: unsafe extern "C" fn(u32),
    pub _ReservedLocation5: unsafe extern "C" fn(u32),
    pub HFSourceSafeSwitch: unsafe extern "C" fn(),
    pub _SelectCompAInput: unsafe extern "C" fn(u8),
    pub _SelectCompARef: unsafe extern "C" fn(u8),
    pub _SelectADCCompBInput: unsafe extern "C" fn(u8),
    pub _SelectDACVref: unsafe extern "C" fn(u8),
}

impl Clone for RomFuncTable {
    fn clone(&self) -> Self {
        *self
    }
}

pub unsafe fn source_switch() {
    ((*(0x10000048i32 as (*mut RomFuncTable))).HFSourceSafeSwitch);
}

pub mod ddi0 {
    #[no_mangle]
    pub unsafe extern "C" fn ddi32reg_write(ui32Base: u32, ui32Reg: u32, ui32Val: u32) {
        *(ui32Base.wrapping_add(ui32Reg) as (*mut usize)) = ui32Val as (usize);
    }

    #[no_mangle]
    pub unsafe extern "C" fn ddi16bit_write(
        ui32Base: u32,
        ui32Reg: u32,
        mut ui32Mask: u32,
        ui32WrData: u32,
    ) {
        let mut ui32RegAddr: u32;
        let ui32Data: u32;
        ui32RegAddr = ui32Base
            .wrapping_add(ui32Reg << 1i32)
            .wrapping_add(0x400u32);
        if ui32Mask & 0xffff0000u32 != 0 {
            ui32RegAddr = ui32RegAddr.wrapping_add(4u32);
            ui32Mask = ui32Mask >> 16i32;
        }
        ui32Data = if ui32WrData != 0 { ui32Mask } else { 0x0u32 };
        *(ui32RegAddr as (*mut usize)) = (ui32Mask << 16i32 | ui32Data) as (usize);
    }

    #[no_mangle]
    pub unsafe extern "C" fn ddi16bitfield_write(
        ui32Base: u32,
        ui32Reg: u32,
        mut ui32Mask: u32,
        mut ui32Shift: u32,
        ui32Data: u16,
    ) {
        let mut ui32RegAddr: u32;
        let ui32WrData: u32;
        ui32RegAddr = ui32Base
            .wrapping_add(ui32Reg << 1i32)
            .wrapping_add(0x400u32);
        if ui32Shift >= 16u32 {
            ui32Shift = ui32Shift.wrapping_sub(16u32);
            ui32RegAddr = ui32RegAddr.wrapping_add(4u32);
            ui32Mask = ui32Mask >> 16i32;
        }
        ui32WrData = (ui32Data as (i32) << ui32Shift) as (u32);
        *(ui32RegAddr as (*mut usize)) = (ui32Mask << 16i32 | ui32WrData) as (usize);
    }

    #[no_mangle]
    pub unsafe extern "C" fn ddi16bit_read(ui32Base: u32, ui32Reg: u32, mut ui32Mask: u32) -> u16 {
        let mut ui32RegAddr: u32;
        let mut ui16Data: u16;
        ui32RegAddr = ui32Base.wrapping_add(ui32Reg).wrapping_add(0x0u32);
        if ui32Mask & 0xffff0000u32 != 0 {
            ui32RegAddr = ui32RegAddr.wrapping_add(2u32);
            ui32Mask = ui32Mask >> 16i32;
        }
        ui16Data = *(ui32RegAddr as (*mut u16));
        ui16Data = (ui16Data as (u32) & ui32Mask) as (u16);
        ui16Data
    }

    #[no_mangle]
    pub unsafe extern "C" fn ddi16bitfield_read(
        ui32Base: u32,
        ui32Reg: u32,
        mut ui32Mask: u32,
        mut ui32Shift: u32,
    ) -> u16 {
        let mut ui32RegAddr: u32;
        let mut ui16Data: u16;
        ui32RegAddr = ui32Base.wrapping_add(ui32Reg).wrapping_add(0x0u32);
        if ui32Shift >= 16u32 {
            ui32Shift = ui32Shift.wrapping_sub(16u32);
            ui32RegAddr = ui32RegAddr.wrapping_add(2u32);
            ui32Mask = ui32Mask >> 16i32;
        }
        ui16Data = *(ui32RegAddr as (*mut u16));
        ui16Data = (ui16Data as (u32) & ui32Mask) as (u16);
        ui16Data = (ui16Data as (i32) >> ui32Shift) as (u16);
        ui16Data
    }
}

pub mod adi {
    pub unsafe extern "C" fn safe_hapi_void(f_ptr: unsafe extern "C" fn()) {
        'loop1: loop {
            if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
                break;
            }
        }
        f_ptr();
        *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
    }

    #[allow(unused)]
    pub unsafe extern "C" fn safe_hapi_aux_adi_select(
        f_ptr: unsafe extern "C" fn(u8),
        mut ut8signal: u8,
    ) {
        'loop1: loop {
            if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
                break;
            }
        }
        f_ptr(ut8signal);
        *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
    }
}
