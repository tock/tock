static mut G_OP_MODE_TO_ORDER: [u8; 4] = [1u8, 2u8, 0u8, 3u8];

static mut G_ORDER_TO_OPMODE: [u8; 4] = [2u8, 0u8, 1u8, 3u8];

use aux;

pub unsafe extern "C" fn AUXSYSIFOpModeChange(mut targetOpMode: u32) {
    let mut currentOpMode: u32;
    let mut currentOrder: u32;
    let mut nextMode: u8;
    'loop1: loop {
        currentOpMode = aux::AUX_CTL.operation_mode_ack().into();
        // currentOpMode = *((0x400c6000u32 + 0x0u32) as (*mut usize)) as (u32);
        if currentOpMode != targetOpMode {
            currentOrder = G_OP_MODE_TO_ORDER[currentOpMode as (usize)] as (u32);
            if currentOrder < G_OP_MODE_TO_ORDER[targetOpMode as (usize)] as (u32) {
                nextMode = G_ORDER_TO_OPMODE[currentOrder.wrapping_add(1u32) as usize];
            } else {
                nextMode = G_ORDER_TO_OPMODE[currentOrder.wrapping_sub(1u32) as usize];
            }
            aux::AUX_CTL.operation_mode_request(nextMode)
            // *((0x400c6000i32 + 0x0i32) as (*mut usize)) = nextMode as (usize);
        }
        if !(currentOpMode != targetOpMode) {
            break;
        }
    }
}
