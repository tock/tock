static mut G_OP_MODE_TO_ORDER: [u8; 4] = [1u8, 2u8, 0u8, 3u8];

static mut G_ORDER_TO_OPMODE: [u8; 4] = [2u8, 0u8, 1u8, 3u8];

#[no_mangle]
pub unsafe extern "C" fn AUXSYSIFOpModeChange(mut targetOpMode: u32) {
    let mut currentOpMode: u32;
    let mut currentOrder: u32;
    let mut nextMode: u32;
    'loop1: loop {
        currentOpMode = *((0x400c6000i32 + 0x0i32) as (*mut usize)) as (u32);
        while currentOpMode as (usize) != *((0x400c6000i32 + 0x4i32) as (*mut usize)) {}
        /*
        'loop2: loop {
            let currentOpModeAck = *((0x400c6000i32 + 0x4i32) as (*mut usize));
            if currentOpMode as (usize) != currentOpModeAck as (usize) {
                break;
            }
        }
        */
        if currentOpMode != targetOpMode {
            currentOrder = G_OP_MODE_TO_ORDER[currentOpMode as (usize)] as (u32);
            if currentOrder < G_OP_MODE_TO_ORDER[targetOpMode as (usize)] as (u32) {
                nextMode = G_ORDER_TO_OPMODE[currentOrder.wrapping_add(1u32) as (usize)] as (u32);
            } else {
                nextMode = G_ORDER_TO_OPMODE[currentOrder.wrapping_sub(1u32) as (usize)] as (u32);
            }
            *((0x400c6000i32 + 0x0i32) as (*mut usize)) = nextMode as (usize);
        }
        if !(currentOpMode != targetOpMode) {
            break;
        }
    }
}
