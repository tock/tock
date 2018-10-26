use super::Pinmap;
use enum_primitive::cast::FromPrimitive;

enum_from_primitive!{
pub enum PIN_FN {
    UART0_RX = 12,
    UART0_TX = 13,
    I2C0_SCL = 22,
    I2C0_SDA = 5,
    TDO = 16,
    TDI = 17,
    RED_LED = 6,
    GREEN_LED = 7,
    BUTTON_1 = 15,
    BUTTON_2 = 14,
    GPIO0 = 21,
    ADC0 = 30,
    ADC1 = 29,
    ADC2 = 28,
    ADC3 = 27,
    ADC4 = 26,
    ADC5 = 25,
    ADC6 = 24,
    ADC7 = 23,
}
}

pub static PINMAP: Pinmap = Pinmap {
    uart0_rx: PIN_FN::UART0_RX as usize,
    uart0_tx: PIN_FN::UART0_TX as usize,
    i2c0_scl: PIN_FN::I2C0_SCL as usize,
    i2c0_sda: PIN_FN::I2C0_SDA as usize,
    red_led: PIN_FN::RED_LED as usize,
    green_led: PIN_FN::GREEN_LED as usize,
    button1: PIN_FN::BUTTON_1 as usize,
    button2: PIN_FN::BUTTON_2 as usize,
    gpio0: PIN_FN::GPIO0 as usize,
    a0: PIN_FN::ADC0 as usize,
    a1: PIN_FN::ADC1 as usize,
    a2: PIN_FN::ADC2 as usize,
    a3: PIN_FN::ADC3 as usize,
    a4: PIN_FN::ADC4 as usize,
    a5: PIN_FN::ADC5 as usize,
    a6: PIN_FN::ADC6 as usize,
    a7: PIN_FN::ADC7 as usize,
};
