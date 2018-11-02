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
    OSC = 8,
    RF_CMD_ACK = 9,
    RF_CORE_CPE0 = 10,
    RF_CORE_CPE1 = 11,
    RF_CORE_HW = 12,
    AUX_ADC = 13,
}
}
