use super::Pinmap;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

pub const CHIP_ID: u32 = 0x2282_f000;

enum_from_primitive! {
pub enum PinFn {
    Uart0Rx = 12,
    Uart0Tx = 13,
    I2c0Scl = 22,
    I2c0Sda = 5,
    Tdo = 16,
    Tdi = 17,
    RedLed = 6,
    GreenLed = 7,
    Button1 = 15,
    Button2 = 14,
    Gpio0 = 21,
    Adc0 = 30,
    Adc1 = 29,
    Adc2 = 28,
    Adc3 = 27,
    Adc4 = 26,
    Adc5 = 25,
    Adc6 = 24,
    Adc7 = 23,
    Pwm0 = 18,
    Pwm1 = 19,
}
}

pub static PINMAP: Pinmap = Pinmap {
    uart0_rx: PinFn::Uart0Rx as usize,
    uart0_tx: PinFn::Uart0Tx as usize,
    i2c0_scl: PinFn::I2c0Scl as usize,
    i2c0_sda: PinFn::I2c0Sda as usize,
    red_led: PinFn::RedLed as usize,
    green_led: PinFn::GreenLed as usize,
    button1: PinFn::Button1 as usize,
    button2: PinFn::Button2 as usize,
    gpio0: PinFn::Gpio0 as usize,
    a0: PinFn::Adc0 as usize,
    a1: PinFn::Adc1 as usize,
    a2: PinFn::Adc2 as usize,
    a3: PinFn::Adc3 as usize,
    a4: PinFn::Adc4 as usize,
    a5: PinFn::Adc5 as usize,
    a6: PinFn::Adc6 as usize,
    a7: PinFn::Adc7 as usize,
    pwm0: PinFn::Pwm0 as usize,
    pwm1: PinFn::Pwm1 as usize,
};
