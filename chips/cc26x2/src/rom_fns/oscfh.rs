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
use rom_fns::{adi, ddi};

#[derive(Copy)]
#[repr(C)]
pub struct Struct1 {
    pub previousStartupTimeInUs: u32,
    pub timeXoscOff_CV: u32,
    pub timeXoscOn_CV: u32,
    pub timeXoscStable_CV: u32,
    pub tempXoscOff: i32,
}

impl Clone for Struct1 {
    fn clone(&self) -> Self {
        *self
    }
}

#[allow(non_upper_case_globals)]
static mut oscHfGlobals: Struct1 = Struct1 {
    previousStartupTimeInUs: 0u32,
    timeXoscOff_CV: 0u32,
    timeXoscOn_CV: 0u32,
    timeXoscStable_CV: 0u32,
    tempXoscOff: 0i32,
};

#[allow(non_snake_case)]
pub unsafe extern "C" fn clock_source_set(ui32src_clk: u32, ui32osc: u32) {
    if ui32src_clk & 0x1u32 != 0 {
        // ui32Base, ui32Reg, ui32Mask, ui32Shift, ui32Data
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x1u32, 0u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x4u32 != 0 {
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0xcu32, 2u32, ui32osc as (u16));
    }
}

pub unsafe extern "C" fn clock_source_get(ui32src_clk: u32) -> u32 {
    let ui32clock_source: u32;
    if ui32src_clk == 0x4u32 {
        ui32clock_source =
            ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x60000000u32, 29u32) as (u32);
    } else {
        ui32clock_source =
            ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x10000000u32, 28u32) as (u32);
    }
    ui32clock_source
}
#[allow(unused)]
unsafe fn source_ready() -> bool {
    (if ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x1u32, 0u32) != 0 {
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
    adi::safe_hapi_void((*(0x10000048i32 as (*mut RomFuncTable))).HFSourceSafeSwitch);
}

unsafe extern "C" fn AONRTCCurrentCompareValueGet() -> u32 {
    *((0x40092000i32 + 0x30i32) as (*mut usize)) as (u32)
}

#[no_mangle]
pub unsafe extern "C" fn OSCHF_GetStartupTime(mut timeUntilWakeupInMs: u32) -> u32 {
    let mut deltaTimeSinceXoscOnInMs: u32;
    let mut deltaTempSinceXoscOn: i32;
    let mut newStartupTimeInUs: u32;
    deltaTimeSinceXoscOnInMs = 1000u32
        .wrapping_mul(AONRTCCurrentCompareValueGet().wrapping_sub(oscHfGlobals.timeXoscOn_CV))
        >> 16i32;
    deltaTempSinceXoscOn = AONBatMonTemperatureGetDegC() - oscHfGlobals.tempXoscOff;
    /*(*(*(0x10000180i32 as (*mut u32)).offset(27isize) as (*mut u32))
        .offset(0isize) as (unsafe extern "C" fn() -> i32))()
        - oscHfGlobals.tempXoscOff;
        */
    if deltaTempSinceXoscOn < 0i32 {
        deltaTempSinceXoscOn = -deltaTempSinceXoscOn;
    }
    if timeUntilWakeupInMs.wrapping_add(deltaTimeSinceXoscOnInMs) > 3000u32
        || deltaTempSinceXoscOn > 5i32
        || oscHfGlobals.timeXoscStable_CV < oscHfGlobals.timeXoscOn_CV
        || oscHfGlobals.previousStartupTimeInUs == 0u32
    {
        newStartupTimeInUs = 2000u32;
        if *((0x50003000i32 + 0x1fb0i32) as (*mut usize)) & 0x1usize == 0usize {
            newStartupTimeInUs = ((*((0x50003000i32 + 0x1faci32) as (*mut usize)) & 0xffusize)
                >> 0i32)
                .wrapping_mul(125usize) as (u32);
        }
    } else {
        newStartupTimeInUs = 1000000u32.wrapping_mul(
            oscHfGlobals
                .timeXoscStable_CV
                .wrapping_sub(oscHfGlobals.timeXoscOn_CV),
        ) >> 16i32;
        newStartupTimeInUs = newStartupTimeInUs.wrapping_add(newStartupTimeInUs >> 2i32);
        if newStartupTimeInUs < oscHfGlobals.previousStartupTimeInUs {
            newStartupTimeInUs = oscHfGlobals.previousStartupTimeInUs;
        }
    }
    if newStartupTimeInUs < 200u32 {
        newStartupTimeInUs = 200u32;
    }
    if newStartupTimeInUs > 4000u32 {
        newStartupTimeInUs = 4000u32;
    }
    newStartupTimeInUs
}

unsafe extern "C" fn OSCHfSourceReady() -> bool {
    /*
    (if (*(*(0x10000180i32 as (*mut u32)).offset(9isize) as (*mut u32)).offset(3isize)
        as (unsafe extern "C" fn(u32, u32, u32, u32) -> u16))(
        0x400ca000u32, 0x3cu32, 0x1u32, 0u32
    ) != 0
    */
    (if ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x1u32, 0u32) != 0 {
        1i32
    } else {
        0i32
    }) != 0
}

#[no_mangle]
pub unsafe extern "C" fn OSCHF_TurnOnXosc() {
    /*
     * (*(*(0x10000180i32 as (*mut u32)).offset(24isize) as (*mut u32)).offset(1isize)
        as (unsafe extern "C" fn(u32, u32)))(0x1u32, 0x1u32);
        */
    clock_source_set(0x1u32, 0x1u32);
    oscHfGlobals.timeXoscOn_CV = AONRTCCurrentCompareValueGet();
}

#[no_mangle]
pub unsafe extern "C" fn OSCHF_AttemptToSwitchToXosc() -> bool {
    let mut startupTimeInUs: u32;
    let mut prevLimmit25InUs: u32;
    if clock_source_get(0x1u32) == 0x1u32
    /*
    if (*(*(0x10000180i32 as (*mut u32)).offset(24isize) as (*mut u32)).offset(0isize)
        as (unsafe extern "C" fn(u32) -> u32))(0x1u32) == 0x1u32
        */
    {
        true
    } else if OSCHfSourceReady() {
        // OSCHfSourceSwitch();
        source_switch();
        oscHfGlobals.timeXoscStable_CV = AONRTCCurrentCompareValueGet();
        startupTimeInUs = 1000000u32.wrapping_mul(
            oscHfGlobals
                .timeXoscStable_CV
                .wrapping_sub(oscHfGlobals.timeXoscOn_CV),
        ) >> 16i32;
        prevLimmit25InUs = oscHfGlobals.previousStartupTimeInUs;
        prevLimmit25InUs = prevLimmit25InUs.wrapping_sub(prevLimmit25InUs >> 2i32);
        oscHfGlobals.previousStartupTimeInUs = startupTimeInUs;
        if prevLimmit25InUs > startupTimeInUs {
            oscHfGlobals.previousStartupTimeInUs = prevLimmit25InUs;
        }
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn OSCHF_SwitchToRcOscTurnOffXosc() {
    /*
    (*(*(0x10000180i32 as (*mut u32)).offset(24isize) as (*mut u32)).offset(1isize)
        as (unsafe extern "C" fn(u32, u32)))(0x1u32, 0x0u32);
        */
    clock_source_set(0x1u32, 0x0u32);
    /*
    if (*(*(0x10000180i32 as (*mut u32)).offset(24isize) as (*mut u32)).offset(0isize)
        as (unsafe extern "C" fn(u32) -> u32))(0x1u32) != 0x0u32
        */
    if clock_source_get(0x1u32) != 0x0u32 {
        source_switch();
        // OSCHfSourceSwitch();
    }
    oscHfGlobals.timeXoscOff_CV = AONRTCCurrentCompareValueGet();
    oscHfGlobals.tempXoscOff = AONBatMonTemperatureGetDegC();
    /*
        (*(*(0x10000180i32 as (*mut u32)).offset(27isize) as (*mut u32))
        .offset(0isize) as (unsafe extern "C" fn() -> i32))();
        */
}

#[no_mangle]
pub unsafe extern "C" fn AONBatMonTemperatureGetDegC() -> i32 {
    let mut signedTemp: i32;
    let mut tempCorrection: i32;
    let mut voltageSlope: i8;
    signedTemp = *((0x40095000i32 + 0x30i32) as (*mut usize)) as (i32) << 32i32 - 9i32 - 8i32
        >> 32i32 - 9i32 - 8i32;
    voltageSlope = *((0x50001000i32 + 0x30ci32) as (*mut u8)) as (i8);
    tempCorrection = voltageSlope as (i32)
        * (*((0x40095000i32 + 0x28i32) as (*mut usize)) as (i32) - 0x300i32)
        >> 4i32;
    signedTemp - tempCorrection + 0x80i32 >> 8i32
}
