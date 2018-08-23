#[no_mangle]
pub unsafe extern "C" fn IOCPortConfigureSet(
    mut ui32IOId: u32,
    mut ui32PortId: u32,
    mut ui32IOConfig: u32,
) {
    let mut ui32Reg: u32;
    ui32Reg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    *(ui32Reg as (*mut usize)) = (ui32IOConfig | ui32PortId) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCPortConfigureGet(mut ui32IOId: u32) -> u32 {
    let mut ui32Reg: u32;
    ui32Reg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    *(ui32Reg as (*mut usize)) as (u32)
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOShutdownSet(mut ui32IOId: u32, mut ui32IOShutdown: u32) {
    let mut ui32Reg: u32;
    let mut ui32Config: u32;
    ui32Reg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32Reg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x18000000i32 as (u32);
    *(ui32Reg as (*mut usize)) = (ui32Config | ui32IOShutdown) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOModeSet(mut ui32IOId: u32, mut ui32IOMode: u32) {
    let mut ui32Reg: u32;
    let mut ui32Config: u32;
    ui32Reg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32Reg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x7000000i32 as (u32);
    *(ui32Reg as (*mut usize)) = (ui32Config | ui32IOMode) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOIntSet(mut ui32IOId: u32, mut ui32Int: u32, mut ui32EdgeDet: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !(0x40000i32 | 0x30000i32) as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config
        | (if ui32Int != 0 { 0x40000i32 } else { 0i32 } as (u32) | ui32EdgeDet))
        as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOEvtSet(mut ui32IOId: u32, mut ui32Evt: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config =
        ui32Config & !(0x800000i32 | 0x400000i32 | 0x200000i32 | 0x80i32 | 0x40i32) as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | ui32Evt) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOPortPullSet(mut ui32IOId: u32, mut ui32Pull: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x6000i32 as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | ui32Pull) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOHystSet(mut ui32IOId: u32, mut ui32Hysteresis: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x40000000i32 as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | ui32Hysteresis) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOInputSet(mut ui32IOId: u32, mut ui32Input: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x20000000i32 as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | ui32Input) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOSlewCtrlSet(mut ui32IOId: u32, mut ui32SlewEnable: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x1000i32 as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | ui32SlewEnable) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIODrvStrengthSet(
    mut ui32IOId: u32,
    mut ui32IOCurrent: u32,
    mut ui32DrvStrength: u32,
) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !(0xc00i32 | 0x300i32) as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | (ui32IOCurrent | ui32DrvStrength)) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIOPortIdSet(mut ui32IOId: u32, mut ui32PortId: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x3fi32 as (u32);
    *(ui32IOReg as (*mut usize)) = (ui32Config | ui32PortId) as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIntEnable(mut ui32IOId: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config | 0x40000u32;
    *(ui32IOReg as (*mut usize)) = ui32Config as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCIntDisable(mut ui32IOId: u32) {
    let mut ui32IOReg: u32;
    let mut ui32Config: u32;
    ui32IOReg = 0x40081000u32.wrapping_add(ui32IOId << 2i32);
    ui32Config = *(ui32IOReg as (*mut usize)) as (u32);
    ui32Config = ui32Config & !0x40000i32 as (u32);
    *(ui32IOReg as (*mut usize)) = ui32Config as (usize);
}

unsafe extern "C" fn GPIO_setOutputEnableDio(mut dioNumber: u32, mut outputEnableValue: u32) {
    *(((0x40022000i32 + 0xd0i32) as (usize) & 0xf0000000usize
        | 0x2000000usize
        | ((0x40022000i32 + 0xd0i32) as (usize) & 0xfffffusize) << 5i32
        | (dioNumber << 2i32) as (usize)) as (*mut usize)) = outputEnableValue as (usize);
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeGpioInput(mut ui32IOId: u32) {
    IOCPortConfigureSet(
        ui32IOId,
        0x0u32,
        (0x0i32
            | 0x0i32
            | 0x6000i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x20000000i32) as (u32),
    );
    GPIO_setOutputEnableDio(ui32IOId, 0x0u32);
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeGpioOutput(mut ui32IOId: u32) {
    IOCPortConfigureSet(
        ui32IOId,
        0x0u32,
        (0x0i32 | 0x0i32 | 0x6000i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32)
            as (u32),
    );
    GPIO_setOutputEnableDio(ui32IOId, 0x1u32);
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeUart(
    mut ui32Base: u32,
    mut ui32Rx: u32,
    mut ui32Tx: u32,
    mut ui32Cts: u32,
    mut ui32Rts: u32,
) {
    if ui32Rx != 0xffffffffu32 {
        IOCPortConfigureSet(
            ui32Rx,
            0xfu32,
            (0x0i32
                | 0x0i32
                | 0x6000i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x20000000i32) as (u32),
        );
    }
    if ui32Tx != 0xffffffffu32 {
        IOCPortConfigureSet(
            ui32Tx,
            0x10u32,
            (0x0i32
                | 0x0i32
                | 0x6000i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32) as (u32),
        );
    }
    if ui32Cts != 0xffffffffu32 {
        IOCPortConfigureSet(
            ui32Cts,
            0x11u32,
            (0x0i32
                | 0x0i32
                | 0x6000i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x20000000i32) as (u32),
        );
    }
    if ui32Rts != 0xffffffffu32 {
        IOCPortConfigureSet(
            ui32Rts,
            0x12u32,
            (0x0i32
                | 0x0i32
                | 0x6000i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32
                | 0x0i32) as (u32),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeSsiMaster(
    mut ui32Base: u32,
    mut ui32Rx: u32,
    mut ui32Tx: u32,
    mut ui32Fss: u32,
    mut ui32Clk: u32,
) {
    if ui32Base == 0x40000000u32 {
        if ui32Rx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Rx,
                0x9u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
        if ui32Tx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Tx,
                0xau32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
        if ui32Fss != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Fss,
                0xbu32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
        if ui32Clk != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Clk,
                0xcu32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
    } else {
        if ui32Rx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Rx,
                0x21u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
        if ui32Tx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Tx,
                0x22u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
        if ui32Fss != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Fss,
                0x23u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
        if ui32Clk != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Clk,
                0x24u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeSsiSlave(
    mut ui32Base: u32,
    mut ui32Rx: u32,
    mut ui32Tx: u32,
    mut ui32Fss: u32,
    mut ui32Clk: u32,
) {
    if ui32Base == 0x40000000u32 {
        if ui32Rx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Rx,
                0x9u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
        if ui32Tx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Tx,
                0xau32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
        if ui32Fss != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Fss,
                0xbu32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
        if ui32Clk != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Clk,
                0xcu32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
    } else {
        if ui32Rx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Rx,
                0x21u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
        if ui32Tx != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Tx,
                0x22u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32) as (u32),
            );
        }
        if ui32Fss != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Fss,
                0x23u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
        if ui32Clk != 0xffffffffu32 {
            IOCPortConfigureSet(
                ui32Clk,
                0x24u32,
                (0x0i32
                    | 0x0i32
                    | 0x6000i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x0i32
                    | 0x20000000i32) as (u32),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeI2c(mut ui32Base: u32, mut ui32Data: u32, mut ui32Clk: u32) {
    let mut ui32IOConfig: u32;
    ui32IOConfig = (0x0i32
        | 0x0i32
        | 0x4000i32
        | 0x0i32
        | 0x0i32
        | 0x0i32
        | 0x0i32
        | 0x4000000i32
        | 0x0i32
        | 0x20000000i32) as (u32);
    IOCPortConfigureSet(ui32Data, 0xdu32, ui32IOConfig);
    IOCPortConfigureSet(ui32Clk, 0xeu32, ui32IOConfig);
}

#[no_mangle]
pub unsafe extern "C" fn IOCPinTypeAux(mut ui32IOId: u32) {
    IOCPortConfigureSet(
        ui32IOId,
        0x8u32,
        (0x0i32
            | 0x0i32
            | 0x6000i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x0i32
            | 0x20000000i32) as (u32),
    );
}
