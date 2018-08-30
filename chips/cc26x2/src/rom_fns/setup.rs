//! It copies and trims several values from the factory & customer configuration
//! areas into their appropriate places (e.g trims the auxiliary voltages).
//!
//! Source:
//!     - https://github.com/contiki-os/cc26xxware/blob/e816e3508b87744186acae2c5f792ad378836ae3/driverlib/setup_rom.c
//!     - https://github.com/contiki-os/cc26xxware/blob/e816e3508b87744186acae2c5f792ad378836ae3/driverlib/setup.c

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

use rom_fns::setup_rom;
// use rom_fns::aux_sysif;
#[allow(non_snake_case)]
pub fn perform() {
    unsafe { SetupTrimDevice() }
}

#[no_mangle]
pub unsafe extern "C" fn SetupTrimDevice() {
    let mut ui32Fcfg1Revision: u32;
    let mut ui32AonSysResetctl: u32;
    ui32Fcfg1Revision = *((0x50001000i32 + 0x31ci32) as (*mut usize)) as (u32);

    *(((0x40030000i32 + 0x24i32) as (usize) & 0xf0000000usize
        | 0x2000000usize
        | ((0x40030000i32 + 0x24i32) as (usize) & 0xfffffusize) << 5i32
        | (1i32 << 2i32) as (usize)) as (*mut usize)) = 0usize;
    setup_rom::SetupSetCacheModeAccordingToCcfgSetting();
    // Undocumented AON IOC Latch register, found in driverlib AON_IOC.h
    if *(((0x40094000i32 + 0xci32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40094000i32 + 0xci32) as (usize) & 0xfffffusize) << 5i32
        | (0i32 << 2i32) as (usize)) as (*mut usize)) == 0
    {
        TrimAfterColdResetWakeupFromShutDownWakeupFromPowerDown();
    } 
    else if *(((0x40090000i32 + 0x2ci32) as (usize) & 0xf0000000usize
        | 0x2000000usize
        | ((0x40090000i32 + 0x2ci32) as (usize) & 0xfffffusize) << 5i32
        | (0i32 << 2i32) as (usize)) as (*mut usize)) == 0
    {
        TrimAfterColdResetWakeupFromShutDown(ui32Fcfg1Revision);
        TrimAfterColdResetWakeupFromShutDownWakeupFromPowerDown();
    } 
    else {
        TrimAfterColdReset();
        TrimAfterColdResetWakeupFromShutDown(ui32Fcfg1Revision);
        TrimAfterColdResetWakeupFromShutDownWakeupFromPowerDown();
    }
    *((0x40082000i32 + 0x18ci32) as (*mut usize)) = 0usize;
    *((0x40030000i32 + 0x2048i32) as (*mut usize)) = *((0x40030000i32 + 0x2048i32) as (*mut usize))
        & !0xfff0000i32 as (usize)
        | (0x139i32 << 16i32) as (usize);
    if (*((0x40090000i32 + 0x28i32) as (*mut usize)) & (0x2000i32 | 0x1000i32) as (usize)) >> 12i32
        == 1usize
    {
        ui32AonSysResetctl = (*((0x40090000i32 + 0x28i32) as (*mut usize))
            & !(0x2000000i32 | 0x1000000i32 | 0x20000i32 | 0x10000i32 | 0x10i32) as (usize))
            as (u32);
        *((0x40090000i32 + 0x28i32) as (*mut usize)) = (ui32AonSysResetctl | 0x20000u32) as (usize);
        *((0x40090000i32 + 0x28i32) as (*mut usize)) = ui32AonSysResetctl as (usize);
    }
    'loop9: loop {
        if *(((0x40034000i32 + 0x0i32) as (usize) & 0xf0000000usize
            | 0x2000000usize
            | ((0x40034000i32 + 0x0i32) as (usize) & 0xfffffusize) << 5i32
            | (3i32 << 2i32) as (usize)) as (*mut usize)) == 0
        {
            break;
        }
    }
}

unsafe extern "C" fn TrimAfterColdResetWakeupFromShutDownWakeupFromPowerDown() {}

unsafe extern "C" fn Step_RCOSCHF_CTRIM(mut toCode: u32) {
    let mut currentRcoscHfCtlReg: u32;
    let mut currentTrim: u32;
    currentRcoscHfCtlReg = *((0x400ca000i32 + 0x30i32) as (*mut u16)) as (u32);
    currentTrim = (currentRcoscHfCtlReg & 0xff00u32) >> 8i32 ^ 0xc0u32;
    'loop1: loop {
        if !(toCode != currentTrim) {
            break;
        }
        *((0x40092000i32 + 0x34i32) as (*mut usize));
        if toCode > currentTrim {
            currentTrim = currentTrim.wrapping_add(1u32);
        } else {
            currentTrim = currentTrim.wrapping_sub(1u32);
        }
        *((0x400ca000i32 + 0x30i32) as (*mut u16)) =
            (currentRcoscHfCtlReg & !0xff00i32 as (u32) | (currentTrim ^ 0xc0u32) << 8i32) as (u16);
    }
}

unsafe extern "C" fn Step_VBG(mut targetSigned: i32) {
    let mut refSysCtl3Reg: u32;
    let mut currentSigned: i32;
    'loop1: loop {
        refSysCtl3Reg = *((0x40086200i32 + 0x5i32) as (*mut u8)) as (u32);
        currentSigned = (refSysCtl3Reg << 32i32 - 6i32 - 0i32) as (i32) >> 32i32 - 6i32;
        *((0x40092000i32 + 0x34i32) as (*mut usize));
        if targetSigned != currentSigned {
            if targetSigned > currentSigned {
                currentSigned = currentSigned + 1;
            } else {
                currentSigned = currentSigned - 1;
            }
            *((0x40086200i32 + 0x5i32) as (*mut u8)) =
                (refSysCtl3Reg & !(0x80i32 | 0x3fi32) as (u32)
                    | currentSigned as (u32) << 0i32 & 0x3fu32) as (u8);
            let _rhs = 0x80i32;
            let _lhs = &mut *((0x40086200i32 + 0x5i32) as (*mut u8));
            *_lhs = (*_lhs as (i32) | _rhs) as (u8);
        }
        if !(targetSigned != currentSigned) {
            break;
        }
    }
}

unsafe extern "C" fn TrimAfterColdResetWakeupFromShutDown(mut ui32Fcfg1Revision: u32) {
    let mut ccfg_ModeConfReg: u32;
    if *((0x50003000i32 + 0x1fb0i32) as (*mut usize)) & 0x2usize == 0usize {
        *((0x40086200i32 + 0x40i32 + 0xbi32 * 2i32) as (*mut u8)) =
            (0xf0usize | *((0x50003000i32 + 0x1faci32) as (*mut usize)) >> 16i32) as (u8);
    }
    ccfg_ModeConfReg = *((0x50003000i32 + 0x1fb4i32) as (*mut usize)) as (u32);
    setup_rom::SetupAfterColdResetWakeupFromShutDownCfg1(ccfg_ModeConfReg);
    setup_rom::SetupAfterColdResetWakeupFromShutDownCfg2(ui32Fcfg1Revision, ccfg_ModeConfReg);
    /*
    (*(*(0x10000180i32 as (*mut u32)).offset(28isize) as (*mut u32)).offset(1isize)
        as (unsafe extern "C" fn(u32, u32)))(ui32Fcfg1Revision, ccfg_ModeConfReg);
        */
    let mut ui32EfuseData: u32;
    let mut orgResetCtl: u32;
    ui32EfuseData = *((0x50001000i32 + 0x3f8i32) as (*mut usize)) as (u32);
    Step_RCOSCHF_CTRIM((ui32EfuseData & 0xffu32) >> 0i32);
    *((0x40086000i32 + 0x0i32 + 0x3i32) as (*mut u8)) = ((ui32EfuseData & 0xf00u32) >> 8i32 << 4i32
        | (ui32EfuseData & 0xf000u32) >> 12i32 << 0i32)
        as (u8);
    *((0x40086000i32 + 0x0i32 + 0x0i32) as (*mut u8)) =
        ((ui32EfuseData & 0x7c0000u32) >> 18i32 << 0i32) as (u8);
    *((0x40086200i32 + 0x60i32 + (0x4i32 << 1i32)) as (*mut u16)) =
        ((0xf0i32 << 8i32) as (u32) | (ui32EfuseData & 0x7800000u32) >> 23i32 << 4i32) as (u16);
    ui32EfuseData = *((0x50001000i32 + 0x3fci32) as (*mut usize)) as (u32);
    orgResetCtl = (*((0x40090000i32 + 0x28i32) as (*mut usize)) & !0x10i32 as (usize)) as (u32);
    *((0x40090000i32 + 0x28i32) as (*mut usize)) =
        (orgResetCtl & !(0x20i32 | 0x40i32 | 0x80i32 | 0x100i32) as (u32)) as (usize);
    *((0x40092000i32 + 0x2ci32) as (*mut usize));
    if ccfg_ModeConfReg & 0x2000000u32 != 0u32 || ccfg_ModeConfReg & 0x1000000u32 == 0u32 {
        if *((0x40090000i32 + 0x10i32) as (*mut usize)) & 0x2usize != 0 {
            *((0x40086200i32 + 0x60i32 + (0x3i32 << 1i32)) as (*mut u16)) =
                ((0xf8i32 << 8i32) as (u32) | (ui32EfuseData & 0x7c0u32) >> 6i32 << 3i32) as (u16);
        } else {
            *((0x40086200i32 + 0x60i32 + (0x3i32 << 1i32)) as (*mut u16)) =
                ((0xf8i32 << 8i32) as (u32) | (ui32EfuseData & 0xf800u32) >> 11i32 << 3i32)
                    as (u16);
        }
        let _rhs = !0x80i32;
        let _lhs = &mut *((0x40086200i32 + 0x5i32) as (*mut u8));
        *_lhs = (*_lhs as (i32) & _rhs) as (u8);
        let _rhs = 0x80i32;
        let _lhs = &mut *((0x40086200i32 + 0x5i32) as (*mut u8));
        *_lhs = (*_lhs as (i32) | _rhs) as (u8);
        setup_rom::SetupStepVddrTrimTo((ui32EfuseData & 0x1f0000u32) >> 16i32);
    }
    Step_VBG((ui32EfuseData << 32i32 - 6i32 - 0i32) as (i32) >> 32i32 - 6i32);
    *((0x40092000i32 + 0x34i32) as (*mut usize));
    *((0x40092000i32 + 0x34i32) as (*mut usize));
    *((0x40090000i32 + 0x28i32) as (*mut usize)) = orgResetCtl as (usize);
    *((0x40092000i32 + 0x2ci32) as (*mut usize));
    let mut trimReg: u32;
    let mut ui32TrimValue: u32;
    trimReg = *((0x50001000i32 + 0x40ci32) as (*mut usize)) as (u32);
    ui32TrimValue = (trimReg & 0x3f000u32) >> 12i32;
    *((0x400cb000i32 + 0xei32) as (*mut u8)) = (ui32TrimValue << 0i32 & 0x3fu32) as (u8);
    *((0x40086200i32 + 0x10i32 + 0xci32) as (*mut u8)) = 0x40u8;
    *((0x400cb000i32 + 0x60i32 + 0x5i32 * 2i32) as (*mut u16)) =
        (0x38i32 << 8i32 | 3i32 << 3i32) as (u16);
    setup_rom::SetupAfterColdResetWakeupFromShutDownCfg3(ccfg_ModeConfReg);
    /*
    (*(*(0x10000180i32 as (*mut u32)).offset(28isize) as (*mut u32)).offset(2isize)
        as (unsafe extern "C" fn(u32)))(ccfg_ModeConfReg);
        */

    // aux_sysif::AUXSYSIFOpModeChange(0x2u32);
    *(((0x40030000i32 + 0x24i32) as (usize) & 0xf0000000usize
        | 0x2000000usize
        | ((0x40030000i32 + 0x24i32) as (usize) & 0xfffffusize) << 5i32
        | (5i32 << 2i32) as (usize)) as (*mut usize)) = 1usize;
}

unsafe extern "C" fn TrimAfterColdReset() {}
