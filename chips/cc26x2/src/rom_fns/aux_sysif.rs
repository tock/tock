static mut g_OpMode_to_order: [u8; 4] = [1u8, 2u8, 0u8, 3u8];

static mut g_Order_to_OpMode: [u8; 4] = [2u8, 0u8, 1u8, 3u8];

#[no_mangle]
pub unsafe extern "C" fn AUXSYSIFOpModeChange(mut targetOpMode: u32) {
    let mut currentOpMode: u32;
    let mut currentOrder: u32;
    let mut nextMode: u32;
    'loop1: loop {
        currentOpMode = *((0x400c6000i32 + 0x0i32) as (*mut usize)) as (u32);
        'loop2: loop {
            if !(currentOpMode as (usize) != *((0x400c6000i32 + 0x4i32) as (*mut usize))) {
                break;
            }
        }
        if currentOpMode != targetOpMode {
            currentOrder = g_OpMode_to_order[currentOpMode as (usize)] as (u32);
            if currentOrder < g_OpMode_to_order[targetOpMode as (usize)] as (u32) {
                nextMode = g_Order_to_OpMode[currentOrder.wrapping_add(1u32) as (usize)] as (u32);
            } else {
                nextMode = g_Order_to_OpMode[currentOrder.wrapping_sub(1u32) as (usize)] as (u32);
            }
            *((0x400c6000i32 + 0x0i32) as (*mut usize)) = nextMode as (usize);
        }
        if !(currentOpMode != targetOpMode) {
            break;
        }
    }
}
