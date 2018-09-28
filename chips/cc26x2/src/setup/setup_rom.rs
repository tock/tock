use setup::{ddi, ioc_rom, oscfh};

#[allow(unused_variables, unused_mut)]
unsafe extern "C" fn SetupSignExtendVddrTrimValue(mut ui32VddrTrimVal: u32) -> i32 {
    let mut i32SignedVddrVal: i32 = ui32VddrTrimVal as (i32);
    if i32SignedVddrVal > 0x15i32 {
        i32SignedVddrVal = i32SignedVddrVal - 0x20i32;
    }
    i32SignedVddrVal
}

#[no_mangle]
pub unsafe extern "C" fn SetupStepVddrTrimTo(mut toCode: u32) {
    let mut pmctlResetctl_reg: u32;
    let mut targetTrim: i32;
    let mut currentTrim: i32;
    targetTrim = SetupSignExtendVddrTrimValue(toCode & (0x1fi32 >> 0i32) as (u32));
    currentTrim = SetupSignExtendVddrTrimValue(
        ((*((0x40086200i32 + 0x6i32) as (*mut u8)) as (i32) & 0x1fi32) >> 0i32) as (u32),
    );
    if targetTrim != currentTrim {
        pmctlResetctl_reg =
            (*((0x40090000i32 + 0x28i32) as (*mut usize)) & !0x10i32 as (usize)) as (u32);
        if pmctlResetctl_reg & 0x80u32 != 0 {
            *((0x40090000i32 + 0x28i32) as (*mut usize)) =
                (pmctlResetctl_reg & !0x80i32 as (u32)) as (usize);
            *((0x40092000i32 + 0x2ci32) as (*mut usize));
        }
        'loop3: loop {
            if !(targetTrim != currentTrim) {
                break;
            }
            *((0x40092000i32 + 0x34i32) as (*mut usize));
            if targetTrim > currentTrim {
                currentTrim = currentTrim + 1;
            } else {
                currentTrim = currentTrim - 1;
            }
            *((0x40086200i32 + 0x6i32) as (*mut u8)) =
                ((*((0x40086200i32 + 0x6i32) as (*mut u8)) as (i32) & !0x1fi32) as (u32)
                    | currentTrim as (u32) << 0i32 & 0x1fu32) as (u8);
        }
        *((0x40092000i32 + 0x34i32) as (*mut usize));
        if pmctlResetctl_reg & 0x80u32 != 0 {
            *((0x40092000i32 + 0x34i32) as (*mut usize));
            *((0x40092000i32 + 0x34i32) as (*mut usize));
            *((0x40090000i32 + 0x28i32) as (*mut usize)) = pmctlResetctl_reg as (usize);
            *((0x40092000i32 + 0x2ci32) as (*mut usize));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn SetupAfterColdResetWakeupFromShutDownCfg1(mut ccfg_ModeConfReg: u32) {
    if ccfg_ModeConfReg & 0x2000000u32 == 0u32 && (ccfg_ModeConfReg & 0x1000000u32 != 0u32) {
        *((0x40086200i32 + 0x20i32 + 0x5i32) as (*mut u8)) = 0x80u8;
        *((0x40086200i32 + 0x60i32 + 0x3i32 * 2i32) as (*mut u16)) =
            (0xf8i32 << 8i32 | 0xd8i32) as (u16);
        *((0x40086200i32 + 0x10i32 + 0x5i32) as (*mut u8)) = 0x80u8;
        SetupStepVddrTrimTo(
            ((*((0x50001000i32 + 0x388i32) as (*mut usize)) & 0x1f000000usize) >> 24i32) as (u32),
        );
    }
    if *((0x40090000i32 + 0x10i32) as (*mut usize)) & 0x2usize != 0 {
        ccfg_ModeConfReg = ccfg_ModeConfReg | (0x8000000i32 | 0x4000000i32) as (u32);
    } else {
        *(((0x40095000i32 + 0x24i32) as (usize) & 0xf0000000usize
            | 0x2000000usize
            | ((0x40095000i32 + 0x24i32) as (usize) & 0xfffffusize) << 5i32
            | (5i32 << 2i32) as (usize)) as (*mut usize)) = 0usize;
    }
    *(((0x40090000i32 + 0x10i32) as (usize) & 0xf0000000usize
        | 0x2000000usize
        | ((0x40090000i32 + 0x10i32) as (usize) & 0xfffffusize) << 5i32
        | (0i32 << 2i32) as (usize)) as (*mut usize)) =
        (ccfg_ModeConfReg >> 27i32 & 1u32 ^ 1u32) as (usize);
    *(((0x40090000i32 + 0x10i32) as (usize) & 0xf0000000usize
        | 0x2000000usize
        | ((0x40090000i32 + 0x10i32) as (usize) & 0xfffffusize) << 5i32
        | (2i32 << 2i32) as (usize)) as (*mut usize)) =
        (ccfg_ModeConfReg >> 26i32 & 1u32 ^ 1u32) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn SetupAfterColdResetWakeupFromShutDownCfg2(
    mut ui32Fcfg1Revision: u32,
    mut ccfg_ModeConfReg: u32,
) {
    let mut ui32Trim: u32;
    ui32Trim = SetupGetTrimForAnabypassValue1(ccfg_ModeConfReg);
    ddi::ddi32reg_write(0x400ca000u32, 0x18u32, ui32Trim);
    ui32Trim = SetupGetTrimForRcOscLfRtuneCtuneTrim();
    ddi::ddi16bitfield_write(
        0x400ca000u32,
        0x2cu32,
        (0xffi32 | 0x300i32) as (u32),
        0u32,
        ui32Trim as (u16),
    );
    ui32Trim = SetupGetTrimForXoscHfIbiastherm();
    ddi::ddi32reg_write(0x400ca000u32, 0x1cu32, ui32Trim << 0i32);
    /*
     * f() pointers to ROM functions
    */
    ui32Trim = SetupGetTrimForAmpcompTh2();
    ddi::ddi32reg_write(0x400ca000u32, 0x14u32, ui32Trim);
    ui32Trim = SetupGetTrimForAmpcompTh1();
    ddi::ddi32reg_write(0x400ca000u32, 0x10u32, ui32Trim);
    ui32Trim = SetupGetTrimForAmpcompCtrl(ui32Fcfg1Revision);
    ddi::ddi32reg_write(0x400ca000u32, 0xcu32, ui32Trim);
    ui32Trim = SetupGetTrimForAdcShModeEn(ui32Fcfg1Revision);
    *((0x400ca000i32 + 0x200i32 + 0x24i32 * 2i32 + 1i32) as (*mut u8)) =
        (0x20u32 | ui32Trim << 1i32) as (u8);
    ui32Trim = SetupGetTrimForAdcShVbufEn(ui32Fcfg1Revision);
    *((0x400ca000i32 + 0x200i32 + 0x24i32 * 2i32 + 1i32) as (*mut u8)) =
        (0x10u32 | ui32Trim) as (u8);
    ui32Trim = SetupGetTrimForXoscHfCtl(ui32Fcfg1Revision);
    ddi::ddi32reg_write(0x400ca000u32, 0x28u32, ui32Trim);
    ui32Trim = SetupGetTrimForDblrLoopFilterResetVoltage(ui32Fcfg1Revision);
    *((0x400ca000i32 + 0x200i32 + 0x24i32 * 2i32 + 4i32) as (*mut u8)) =
        (0x60u32 | ui32Trim << 1i32) as (u8);
    ui32Trim = SetupGetTrimForRcOscLfIBiasTrim(ui32Fcfg1Revision);
    *((0x400ca000i32 + 0x200i32 + 0x20i32 * 2i32 + 1i32) as (*mut u8)) =
        (0x80u32 | ui32Trim << 3i32) as (u8);
    ui32Trim = SetupGetTrimForXoscLfRegulatorAndCmirrwrRatio(ui32Fcfg1Revision);
    *((0x400ca000i32 + 0x300i32 + 0x2ci32 * 2i32 + 4i32) as (*mut u16)) =
        (0xfc00u32 | ui32Trim << 2i32) as (u16);
    ui32Trim = SetupGetTrimForRadcExtCfg(ui32Fcfg1Revision);
    ddi::ddi32reg_write(0x400ca000u32, 0x8u32, ui32Trim);
}

unsafe extern "C" fn SysCtrlAonSync() {
    *((0x40092000i32 + 0x2ci32) as (*mut usize));
}

#[no_mangle]
pub unsafe extern "C" fn SetupAfterColdResetWakeupFromShutDownCfg3(mut ccfg_ModeConfReg: u32) {
    let mut _currentBlock;
    let mut fcfg1OscConf: u32;
    let mut ui32Trim: u32;
    let mut currentHfClock: u32;
    let mut ccfgExtLfClk: u32;
    let switch1 = (ccfg_ModeConfReg & 0xc0000u32) >> 18i32;
    /*
    if !(switch1 == 2u32) {
        if switch1 == 1u32 {
            fcfg1OscConf = *((0x50001000i32 + 0x38ci32) as (*mut usize)) as (u32);
            if fcfg1OscConf & 0x20000u32 == 0u32 {
                *((0x400ca000i32 + 0x80i32 + 0x0i32) as (*mut usize)) = 0x4000usize;
                *((0x40086000i32 + 0xci32) as (*mut usize)) =
                    *((0x40086000i32 + 0xci32) as (*mut usize)) & !(0x80i32 | 0xfi32) as (usize)
                        | ((fcfg1OscConf & 0x10000u32) >> 16i32 << 7i32) as (usize)
                        | ((fcfg1OscConf & 0xf000u32) >> 12i32 << 0i32) as (usize);
                *((0x40086000i32 + 0xbi32) as (*mut usize)) =
                    *((0x40086000i32 + 0xbi32) as (*mut usize)) & !0xfi32 as (usize)
                        | ((fcfg1OscConf & 0xf00u32) >> 8i32 << 0i32) as (usize);
                *((0x40086000i32 + 0xai32) as (*mut usize)) = *((0x40086000i32 + 0xai32)
                    as (*mut usize))
                    & !(0x80i32 | 0x60i32 | 0x6i32 | 0x1i32) as (usize)
                    | ((fcfg1OscConf & 0x80u32) >> 7i32 << 7i32) as (usize)
                    | ((fcfg1OscConf & 0x60u32) >> 5i32 << 5i32) as (usize)
                    | ((fcfg1OscConf & 0x6u32) >> 1i32 << 1i32) as (usize)
                    | ((fcfg1OscConf & 0x1u32) >> 0i32 << 0i32) as (usize);
                _currentBlock = 6;
            } else {
                _currentBlock = 4;
            }
        } else {
            _currentBlock = 4;
        }
        if _currentBlock == 6 {
        } else {
            *((0x400ca000i32 + 0x80i32 + 0x0i32) as (*mut usize)) = 0x00000000usize;
        }
    }
    */
    // Set XOSC_HF in bypass mode if CCFG is configured for external TCXO
    if *((0x50003000i32 + 0x1fb0i32) as (*mut usize)) & 0x8usize == 0usize {
        *((0x400ca000i32 + 0x80i32 + 0x28i32) as (*mut usize)) = 0x40usize;
    }
    // Clear DDI_0_OSC_CTL0_CLK_LOSS_EN (ClockLossEventEnable()). This is bit 9 in DDI_0_OSC_O_CTL0.
    // This is typically already 0 except on Lizard where it is set in ROM-boot
    *((0x400ca000i32 + 0x100i32 + 0x0i32) as (*mut usize)) = 0x200usize;
    
    // Setting DDI_0_OSC_CTL1_XOSC_HF_FAST_START according to value found in FCFG1
    ui32Trim = SetupGetTrimForXoscHfFastStart();

    // setup the LF clock based upon CCFG:MODE_CONF:SCLK_LF_OPTION
    *((0x400ca000i32 + 0x200i32 + 0x4i32 * 2i32) as (*mut u8)) = (0x30u32 | ui32Trim) as (u8);
    let switch2 = (ccfg_ModeConfReg & 0xc00000u32) >> 22i32;
    if switch2 == 2u32 {
        _currentBlock = 17;
    } 
    else if switch2 == 1u32 {
        currentHfClock = oscfh::clock_source_get(0x1u32);
        oscfh::clock_source_set(0x4u32, currentHfClock);
        'loop15: loop {
            if oscfh::clock_source_get(0x4u32) != currentHfClock {
                break;
            }
        }
        ccfgExtLfClk = *((0x50003000i32 + 0x1fa8i32) as (*mut usize)) as (u32);
        SetupSetAonRtcSubSecInc((ccfgExtLfClk & 0xffffffu32) >> 0i32);
        // IOC Port configure
        ioc_rom::IOCPortConfigureSet(
            (ccfgExtLfClk & 0xff000000u32) >> 24i32,
            0x7u32,
            (0x0i32
                | 0x0i32
                | 0x6000i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x20000000i32
                | 0x40000000i32) as (u32),
        );

        *((0x400ca000i32 + 0x80i32 + 0x0i32) as (*mut usize)) = 0x400usize;
        _currentBlock = 17;
    } 
    else {
        if switch2 == 0u32 {
            oscfh::clock_source_set(0x4u32, 0x1u32);
            SetupSetAonRtcSubSecInc(0x8637bdu32);
        } 
        else {
            oscfh::clock_source_set(0x4u32, 0x2u32);
        }
        _currentBlock = 18;
    }
    if _currentBlock == 17 {
        oscfh::clock_source_set(0x4u32, 0x3u32);
    }
    *((0x400cb000i32 + 0xbi32) as (*mut u8)) =
        (*((0x50001000i32 + 0x36ci32) as (*mut usize)) >> 0i32 << 0i32 & 0x3fusize) as (u8);
    SysCtrlAonSync();
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForAnabypassValue1(mut ccfg_ModeConfReg: u32) -> u32 {
    let mut ui32Fcfg1Value: u32;
    let mut ui32XoscHfRow: u32;
    let mut ui32XoscHfCol: u32;
    let mut ui32TrimValue: u32;
    ui32Fcfg1Value = *((0x50001000i32 + 0x350i32) as (*mut usize)) as (u32);
    ui32XoscHfRow = (ui32Fcfg1Value & 0x3c000000u32) >> 26i32;
    ui32XoscHfCol = (ui32Fcfg1Value & 0x3fffc00u32) >> 10i32;
    if ccfg_ModeConfReg & 0x20000u32 == 0u32 {
        let mut i32CustomerDeltaAdjust: i32 =
            (ccfg_ModeConfReg << 32i32 - 8i32 - 8i32) as (i32) >> 32i32 - 8i32;
        'loop2: loop {
            if !(i32CustomerDeltaAdjust < 0i32) {
                break;
            }
            ui32XoscHfCol = ui32XoscHfCol >> 1i32;
            if ui32XoscHfCol == 0u32 {
                ui32XoscHfCol = 0xffffu32;
                ui32XoscHfRow = ui32XoscHfRow >> 1i32;
                if ui32XoscHfRow == 0u32 {
                    ui32XoscHfRow = 1u32;
                    ui32XoscHfCol = 1u32;
                }
            }
            i32CustomerDeltaAdjust = i32CustomerDeltaAdjust + 1;
        }
        'loop3: loop {
            if !(i32CustomerDeltaAdjust > 0i32) {
                break;
            }
            ui32XoscHfCol = ui32XoscHfCol << 1i32 | 1u32;
            if ui32XoscHfCol > 0xffffu32 {
                ui32XoscHfCol = 1u32;
                ui32XoscHfRow = ui32XoscHfRow << 1i32 | 1u32;
                if ui32XoscHfRow > 0xfu32 {
                    ui32XoscHfRow = 0xfu32;
                    ui32XoscHfCol = 0xffffu32;
                }
            }
            i32CustomerDeltaAdjust = i32CustomerDeltaAdjust - 1;
        }
    }
    ui32TrimValue = ui32XoscHfRow << 16i32 | ui32XoscHfCol << 0i32;
    ui32TrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForRcOscLfRtuneCtuneTrim() -> u32 {
    let mut ui32TrimValue: u32;
    ui32TrimValue =
        ((*((0x50001000i32 + 0x350i32) as (*mut usize)) & 0x3fcusize) >> 2i32 << 0i32) as (u32);
    ui32TrimValue = (ui32TrimValue as (usize)
        | (*((0x50001000i32 + 0x350i32) as (*mut usize)) & 0x3usize) >> 0i32 << 8i32)
        as (u32);
    ui32TrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForXoscHfIbiastherm() -> u32 {
    let mut ui32TrimValue: u32;
    ui32TrimValue =
        ((*((0x50001000i32 + 0x37ci32) as (*mut usize)) & 0x3fffusize) >> 0i32) as (u32);
    ui32TrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForAmpcompTh2() -> u32 {
    let mut ui32TrimValue: u32;
    let mut ui32Fcfg1Value: u32;
    ui32Fcfg1Value = *((0x50001000i32 + 0x374i32) as (*mut usize)) as (u32);
    ui32TrimValue = (ui32Fcfg1Value & 0xfc000000u32) >> 26i32 << 26i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xfc0000u32) >> 18i32 << 18i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xfc00u32) >> 10i32 << 10i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xfcu32) >> 2i32 << 2i32;
    ui32TrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForAmpcompTh1() -> u32 {
    let mut ui32TrimValue: u32;
    let mut ui32Fcfg1Value: u32;
    ui32Fcfg1Value = *((0x50001000i32 + 0x370i32) as (*mut usize)) as (u32);
    ui32TrimValue = (ui32Fcfg1Value & 0xfc0000u32) >> 18i32 << 18i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xfc00u32) >> 10i32 << 10i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0x3c0u32) >> 6i32 << 6i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0x3fu32) >> 0i32 << 0i32;
    ui32TrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForAmpcompCtrl(mut ui32Fcfg1Revision: u32) -> u32 {
    let mut ui32TrimValue: u32;
    let mut ui32Fcfg1Value: u32;
    let mut ibiasOffset: u32;
    let mut ibiasInit: u32;
    let mut modeConf1: u32;
    let mut deltaAdjust: i32;
    ui32Fcfg1Value = *((0x50001000i32 + 0x378i32) as (*mut usize)) as (u32);
    ibiasOffset = (ui32Fcfg1Value & 0xf00000u32) >> 20i32;
    ibiasInit = (ui32Fcfg1Value & 0xf0000u32) >> 16i32;
    if *((0x50003000i32 + 0x1fb0i32) as (*mut usize)) & 0x1usize == 0usize {
        modeConf1 = *((0x50003000i32 + 0x1faci32) as (*mut usize)) as (u32);
        deltaAdjust = (modeConf1 << 32i32 - 4i32 - 8i32) as (i32) >> 32i32 - 4i32;
        deltaAdjust = deltaAdjust + ibiasOffset as (i32);
        if deltaAdjust < 0i32 {
            deltaAdjust = 0i32;
        }
        if deltaAdjust > 0xf00000i32 >> 20i32 {
            deltaAdjust = 0xf00000i32 >> 20i32;
        }
        ibiasOffset = deltaAdjust as (u32);
        deltaAdjust = (modeConf1 << 32i32 - 4i32 - 12i32) as (i32) >> 32i32 - 4i32;
        deltaAdjust = deltaAdjust + ibiasInit as (i32);
        if deltaAdjust < 0i32 {
            deltaAdjust = 0i32;
        }
        if deltaAdjust > 0xf0000i32 >> 16i32 {
            deltaAdjust = 0xf0000i32 >> 16i32;
        }
        ibiasInit = deltaAdjust as (u32);
    }
    ui32TrimValue = ibiasOffset << 20i32 | ibiasInit << 16i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xff00u32) >> 8i32 << 8i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xf0u32) >> 4i32 << 4i32;
    ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0xfu32) >> 0i32 << 0i32;
    if ui32Fcfg1Revision >= 0x22u32 {
        ui32TrimValue = ui32TrimValue | (ui32Fcfg1Value & 0x40000000u32) >> 30i32 << 30i32;
    }
    ui32TrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForDblrLoopFilterResetVoltage(
    mut ui32Fcfg1Revision: u32,
) -> u32 {
    let mut dblrLoopFilterResetVoltageValue: u32 = 0u32;
    if ui32Fcfg1Revision >= 0x20u32 {
        dblrLoopFilterResetVoltageValue =
            ((*((0x50001000i32 + 0x398i32) as (*mut usize)) & 0x300000usize) >> 20i32) as (u32);
    }
    dblrLoopFilterResetVoltageValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForAdcShModeEn(mut ui32Fcfg1Revision: u32) -> u32 {
    let mut getTrimForAdcShModeEnValue: u32 = 1u32;
    if ui32Fcfg1Revision >= 0x22u32 {
        getTrimForAdcShModeEnValue =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x10000000usize) >> 28i32) as (u32);
    }
    getTrimForAdcShModeEnValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForAdcShVbufEn(mut ui32Fcfg1Revision: u32) -> u32 {
    let mut getTrimForAdcShVbufEnValue: u32 = 1u32;
    if ui32Fcfg1Revision >= 0x22u32 {
        getTrimForAdcShVbufEnValue =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x20000000usize) >> 29i32) as (u32);
    }
    getTrimForAdcShVbufEnValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForXoscHfCtl(mut ui32Fcfg1Revision: u32) -> u32 {
    let mut getTrimForXoschfCtlValue: u32 = 0u32;
    let mut fcfg1Data: u32;
    if ui32Fcfg1Revision >= 0x20u32 {
        fcfg1Data = *((0x50001000i32 + 0x398i32) as (*mut usize)) as (u32);
        getTrimForXoschfCtlValue = (fcfg1Data & 0x18000000u32) >> 27i32 << 8i32;
        getTrimForXoschfCtlValue =
            getTrimForXoschfCtlValue | (fcfg1Data & 0x7000000u32) >> 24i32 << 2i32;
        getTrimForXoschfCtlValue =
            getTrimForXoschfCtlValue | (fcfg1Data & 0xc00000u32) >> 22i32 << 0i32;
    }
    getTrimForXoschfCtlValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForXoscHfFastStart() -> u32 {
    let mut ui32XoscHfFastStartValue: u32;
    ui32XoscHfFastStartValue =
        ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x180000usize) >> 19i32) as (u32);
    ui32XoscHfFastStartValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForRadcExtCfg(mut ui32Fcfg1Revision: u32) -> u32 {
    let mut getTrimForRadcExtCfgValue: u32 = 0x403f8000u32;
    let mut fcfg1Data: u32;
    if ui32Fcfg1Revision >= 0x20u32 {
        fcfg1Data = *((0x50001000i32 + 0x398i32) as (*mut usize)) as (u32);
        getTrimForRadcExtCfgValue = (fcfg1Data & 0xffc00u32) >> 10i32 << 22i32;
        getTrimForRadcExtCfgValue =
            getTrimForRadcExtCfgValue | (fcfg1Data & 0x3f0u32) >> 4i32 << 16i32;
        getTrimForRadcExtCfgValue =
            getTrimForRadcExtCfgValue | (fcfg1Data & 0xfu32) >> 0i32 << 12i32;
    }
    getTrimForRadcExtCfgValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForRcOscLfIBiasTrim(mut ui32Fcfg1Revision: u32) -> u32 {
    let mut trimForRcOscLfIBiasTrimValue: u32 = 0u32;
    if ui32Fcfg1Revision >= 0x22u32 {
        trimForRcOscLfIBiasTrimValue =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x8000000usize) >> 27i32) as (u32);
    }
    trimForRcOscLfIBiasTrimValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupGetTrimForXoscLfRegulatorAndCmirrwrRatio(
    mut ui32Fcfg1Revision: u32,
) -> u32 {
    let mut trimForXoscLfRegulatorAndCmirrwrRatioValue: u32 = 0u32;
    if ui32Fcfg1Revision >= 0x22u32 {
        trimForXoscLfRegulatorAndCmirrwrRatioValue = ((*((0x50001000i32 + 0x38ci32) as (*mut usize))
            & (0x6000000i32 | 0x1e00000i32) as (usize))
            >> 21i32) as (u32);
    }
    trimForXoscLfRegulatorAndCmirrwrRatioValue
}

#[no_mangle]
pub unsafe extern "C" fn SetupSetCacheModeAccordingToCcfgSetting() {
    let mut vimsCtlMode0: u32;
    'loop1: loop {
        if *(((0x40034000i32 + 0x0i32) as (usize) & 0xf0000000usize
            | 0x2000000usize
            | ((0x40034000i32 + 0x0i32) as (usize) & 0xfffffusize) << 5i32
            | (3i32 << 2i32) as (usize)) as (*mut usize)) == 0
        {
            break;
        }
    }
    vimsCtlMode0 = (*((0x40034000i32 + 0x4i32) as (*mut usize)) & !0x3i32 as (usize)
        | 0x20000000usize
        | 0x4usize) as (u32);
    if *((0x50003000i32 + 0x1fb0i32) as (*mut usize)) & 0x4usize != 0 {
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = (vimsCtlMode0 | 0x1u32) as (usize);
    } else if *((0x40034000i32 + 0x0i32) as (*mut usize)) & 0x3usize != 0x0usize {
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = (vimsCtlMode0 | 0x3u32) as (usize);
        'loop6: loop {
            if !(*((0x40034000i32 + 0x0i32) as (*mut usize)) & 0x3usize != 0x3usize) {
                break;
            }
        }
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = vimsCtlMode0 as (usize);
    } else {
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = vimsCtlMode0 as (usize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn SetupSetAonRtcSubSecInc(mut subSecInc: u32) {
    *((0x400c6000i32 + 0x7ci32) as (*mut usize)) = (subSecInc & 0xffffu32) as (usize);
    *((0x400c6000i32 + 0x80i32) as (*mut usize)) = (subSecInc >> 16i32 & 0xffu32) as (usize);
    *((0x400c6000i32 + 0x84i32) as (*mut usize)) = 0x1usize;
    'loop1: loop {
        if !(*(((0x400c6000i32 + 0x84i32) as (usize) & 0xf0000000usize
            | 0x2000000usize
            | ((0x400c6000i32 + 0x84i32) as (usize) & 0xfffffusize) << 5i32
            | (1i32 << 2i32) as (usize)) as (*mut usize)) == 0)
        {
            break;
        }
    }
    *((0x400c6000i32 + 0x84i32) as (*mut usize)) = 0usize;
}
