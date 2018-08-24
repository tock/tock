/*
unsafe extern "C" fn aux_adi_ddi_safe_write(n_addr: u32, n_data: u32, n_size: u32) {
    //let mut bIrqEnabled : bool = CPUcpsid() == 0;

    'loop1: loop {
        if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
            break;
        }
    }
    if n_size == 2u32 {
        *(n_addr as (*mut u16)) = n_data as (u16);
    } else if n_size == 1u32 {
        *(n_addr as (*mut u8)) = n_data as (u8);
    } else {
        *(n_addr as (*mut usize)) = n_data as (usize);
    }
    *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;

    /*if bIrqEnabled {
        CPUcpsie();
    }*/
}

unsafe extern "C" fn aux_adi_ddi_safe_read(n_addr: u32, n_size: u32) -> u32 {
    let mut ret: u32;
    //let mut bIrqEnabled: bool = CPUcpsid() == 0;
    'loop1: loop {
        if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
            break;
        }
    }
    if n_size == 2u32 {
        ret = *(n_addr as (*mut u16)) as (u32);
    } else if n_size == 1u32 {
        ret = *(n_addr as (*mut u8)) as (u32);
    } else {
        ret = *(n_addr as (*mut usize)) as (u32);
    }
    *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
    /*if bIrqEnabled {
            CPUcpsie();
        }*/
    ret
}
*/
pub unsafe extern "C" fn ddi32reg_write(ui32Base: u32, ui32Reg: u32, ui32Val: u32) {
    *(ui32Base.wrapping_add(ui32Reg) as (*mut usize)) = ui32Val as (usize);
}

/*
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
*/
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
/*
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
*/
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
