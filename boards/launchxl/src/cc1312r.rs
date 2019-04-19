use super::Pinmap;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

#[allow(dead_code)]
pub const CHIP_ID: u32 = 0x2082_8000;

enum_from_primitive! {
pub enum PinFn {
    Uart0Rx = 2,
    Uart0Tx = 3,
    I2c0Scl = 4,
    I2c0Sda = 5,
    Tdo = 16,
    Tdi = 17,
    RedLed = 6,
    GreenLed = 7,
    Button1 = 13,
    Button2 = 14,
    Gpio0 = 22,
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

// Booster pack standard pinout
//
// 1  -> 3v3
// 2  -> DIO23 (analog)
// 3  -> DIO3  (UARTRX)
// 4  -> DIO2  (UARTTX)
// 5  -> DIO22 (GPIO)
// 6  -> DIO24 (analog)
// 7  -> DIO10 (SPI CLK)
// 8  -> DIO21 (GPIO)
// 9  -> DIO4  (I2CSCL)
// 10 -> DIO5  (I2CSDA)
//
// 11 -> DIO15 (GPIO)
// 12 -> DIO14 (SPI CS - other)
// 13 -> DIO13 (SPI CS - display)
// 14 -> DIO8  (SPI MISO)
// 15 -> DIO9  (SPI MOSI)
// 16 -> LPRST
// 17 -> unused
// 18 -> DIO11 (SPI CS - RF)
// 19 -> DIO12 (PWM)
// 20 -> GND
//
// 21 -> 5v
// 22 -> GND
// 23 -> DIO25 (analog)
// 24 -> DIO26 (analog)
// 25 -> DIO17 (analog)
// 26 -> DIO28 (analog)
// 27 -> DIO29 (analog)
// 28 -> DIO30 (analog)
// 29 -> DIO0  (GPIO)
// 30 -> DIO1  (GPIO)
//
// 31 -> DIO17
// 32 -> DIO16
// 33 -> TMS
// 34 -> TCK
// 35 -> BPRST
// 36 -> DIO18 (PWM)
// 37 -> DIO19 (PWM)
// 38 -> DIO20 (PWM)
// 39 -> DIO6  (PWM)
// 40 -> DIO7  (PWM)
