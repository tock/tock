//
//  These are configurable priorities that can be used by ISRs or yields from within kernel space
//

use enum_primitive::cast::FromPrimitive;

pub static mut FLAGS: u32 = 0;

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EVENT_PRIORITY {
    GPIO = 0,
    UART0 = 2,
    UART1 = 1,
    AON_RTC = 3,
    RTC = 4,
    I2C0 = 6,
    AON_PROG = 7,
}
}
