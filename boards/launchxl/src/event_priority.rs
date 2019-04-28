//
//  This allows for boards to set custom interrupt priorities
//
use enum_primitive::cast::{FromPrimitive, ToPrimitive};
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EVENT_PRIORITY {
    GPIO = 0,
    UART0 = 1,
    UART1 = 2,
    AON_RTC = 3,
    RTC = 4,
    I2C0 = 6,
    AON_PROG = 7,
    OSC = 8,
}
}

// a default interrupt table can be used that generates the interrupt handlers
// with appropriate event flags based on EVENT_PRIORITY defined in this file
cc26x2::default_interrupt_table!();
