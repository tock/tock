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

pub unsafe fn clock_source_set(ui32src_clk: u32, ui32osc: u32) {
    if ui32src_clk & 0x1u32 != 0 {
        ddi0::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x1u32, 0u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x2u32 != 0 {
        ddi0::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x2u32, 1u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x4u32 != 0 {
        ddi0::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0xcu32, 2u32, ui32osc as (u16));
    }
}

pub unsafe fn clock_source_get(ui32src_clk: u32) -> u32 {
    let ui32clock_source: u32;
    if ui32src_clk == 0x4u32 {
        ui32clock_source =
            ddi0::ddi16bitfield_read(0x400ca000u32, 0x34u32, 0x60000000u32, 29u32) as (u32);
    } else {
        ui32clock_source =
            ddi0::ddi16bitfield_read(0x400ca000u32, 0x34u32, 0x10000000u32, 28u32) as (u32);
    }
    ui32clock_source
}
#[allow(unused)]
unsafe fn source_ready() -> bool {
    (if ddi0::ddi16bitfield_read(0x400ca000u32, 0x34u32, 0x1u32, 0u32) != 0 {
        1i32
    } else {
        0i32
    }) != 0
}

#[derive(Copy)]
#[repr(C)]
pub struct RomFuncTable {
    pub _crc32: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    pub _flash_get_size: unsafe extern "C" fn() -> u32,
    pub _get_chip_id: unsafe extern "C" fn() -> u32,
    pub _reserved_location1: unsafe extern "C" fn(u32) -> u32,
    pub _reserved_location2: unsafe extern "C" fn() -> u32,
    pub _reserved_location3: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    pub _reset_device: unsafe extern "C" fn(),
    pub _fletcher32: unsafe extern "C" fn(*mut u16, u16, u16) -> u32,
    pub _min_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _max_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _mean_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _stand_deviation_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _reserved_location4: unsafe extern "C" fn(u32),
    pub _reserved_location5: unsafe extern "C" fn(u32),
    pub hfsource_safe_switch: unsafe extern "C" fn(),
    pub _select_comp_ainput: unsafe extern "C" fn(u8),
    pub _select_comp_aref: unsafe extern "C" fn(u8),
    pub _select_adccomp_binput: unsafe extern "C" fn(u8),
    pub _select_comp_bref: unsafe extern "C" fn(u8),
}

impl Clone for RomFuncTable {
    fn clone(&self) -> Self {
        *self
    }
}

/*
    In order to switch oscillator sources we need to call a ROM
    function (proprietary), due to a set of undocumented restrictions.
*/

pub unsafe fn source_switch() {
    adi::safe_hapi_void((*(0x10000048i32 as (*mut RomFuncTable))).hfsource_safe_switch);
}

pub mod ddi0 {

    pub unsafe fn aux_ddi_write(addr: u32, data: u32, size: u32) {
        'loop1: loop {
            if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
                break;
            }
        }
        if size == 2u32 {
            *(addr as (*mut u16)) = data as (u16);
        } else if size == 1u32 {
            *(addr as (*mut u8)) = data as (u8);
        } else {
            *(addr as (*mut usize)) = data as (usize);
        }
        *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
    }

    pub unsafe fn aux_ddi_read(addr: u32, size: u32) -> u32 {
        let ret: u32;

        'loop1: loop {
            if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
                break;
            }
        }
        if size == 2u32 {
            ret = *(addr as (*mut u16)) as (u32);
        } else if size == 1u32 {
            ret = *(addr as (*mut u8)) as (u32);
        } else {
            ret = *(addr as (*mut usize)) as (u32);
        }
        *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
        ret
    }

    pub unsafe fn ddi16bitfield_write(
        ui32base: u32,
        ui32reg: u32,
        mut ui32mask: u32,
        mut ui32shift: u32,
        ui32data: u16,
    ) {
        let mut ui32reg_addr: u32;
        ui32reg_addr = ui32base
            .wrapping_add(ui32reg << 1i32)
            .wrapping_add(0x200u32);
        if ui32shift >= 16u32 {
            ui32shift = ui32shift.wrapping_sub(16u32);
            ui32reg_addr = ui32reg_addr.wrapping_add(4u32);
            ui32mask = ui32mask >> 16i32;
        }
        let ui32wr_data: u32 = (ui32data as (i32) << ui32shift) as (u32);
        aux_ddi_write(ui32reg_addr, ui32mask << 16i32 | ui32wr_data, 4u32);
    }

    pub unsafe fn ddi16bitfield_read(
        ui32base: u32,
        ui32reg: u32,
        mut ui32mask: u32,
        mut ui32shift: u32,
    ) -> u16 {
        let mut ui32reg_addr: u32;
        let mut ui16data: u16;
        ui32reg_addr = ui32base.wrapping_add(ui32reg).wrapping_add(0x0u32);
        if ui32shift >= 16u32 {
            ui32shift = ui32shift.wrapping_sub(16u32);
            ui32reg_addr = ui32reg_addr.wrapping_add(2u32);
            ui32mask = ui32mask >> 16i32;
        }
        ui16data = aux_ddi_read(ui32reg_addr, 2u32) as (u16);
        ui16data = (ui16data as (u32) & ui32mask) as (u16);
        ui16data = (ui16data as (i32) >> ui32shift) as (u16);
        ui16data
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
