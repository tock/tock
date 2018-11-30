use enum_primitive::cast::FromPrimitive;

enum_from_primitive! {
pub enum PIN_FN {
    UART0_RX = 2,
    UART0_TX = 3,
    I2C0_SCL = 4,
    I2C0_SDA = 5,
    TDO = 16,
    TDI = 17,
    RED_LED = 6,
    GREEN_LED = 7,
    BUTTON_1 = 13,
    BUTTON_2 = 14,
    GPIO0 = 22,
}
}

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
