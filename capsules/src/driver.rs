use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
// syscall driver numbers
pub enum NUM {
    ADC = 0x00000005,
    ALARM = 0x00000000,
    AMBIENT_LIGHT = 0x60002,
    ANALOG_COMPARATOR = 0x00007,
    APP_FLASH =  0x50000,
    BLE_ADVERTISING = 0x030000,
    BUTTON = 0x00000003,
    CONSOLE = 0x00000001,
    CRC = 0x40002,
    DAC = 0x00000006,
    GPIO = 0x00000004,
    GPIO_ASYNC = 0x80003,
    HUMIDITY = 0x60001,
    I2C_MASTER = 0x40006,
    I2C_MASTER_SLAVE = 0x20006,
    LED = 0x2,
    LPS25HB = 0x70004,
    LTC294X = 0x80000,
    MAX17205 = 0x80001,
    NINEDOF = 0x60004,
    NVM_STORAGE = 0x50001,
    NRF51822_SERIALIZATION = 0x80004,
    PCA9544A = 0x80002,
    RNG = 0x40001,
    SD_CARD = 0x50002,
    SPI = 0x20001,
    TEMPERATURE = 0x60000,
    TMP006 = 0x70001,
    TSL2561 = 0x70000,
    USB_USER = 0x20005,
}
}


enum_from_primitive! {
#[derive(Debug, PartialEq)]
// bit map representing whether an app has permission to that driver or not
// bit assigned in order of ascending driver number (0 indexed, max 63)
pub enum PERMISSION_BIT {
    ADC = 5,
    ALARM = 0,
    AMBIENT_LIGHT = 20,
    ANALOG_COMPARATOR = 7,
    APP_FLASH = 15,
    BLE_ADVERTISING = 11,
    BUTTON = 3,
    CONSOLE = 1,
    CRC = 13,
    DAC = 6,
    GPIO = 4,
    GPIO_ASYNC = 28,
    HUMIDITY = 19,
    I2C_MASTER = 14,
    I2C_MASTER_SLAVE = 10,
    LED = 2,
    LPS25HB = 24,
    LTC294X = 25,
    MAX17205 = 26,
    NINEDOF = 21,
    NRF51822_SERIALIZATION = 29,
    NVM_STORAGE = 16,
    PCA9544A = 27,
    RNG = 12,
    SD_CARD = 17,
    SPI = 8,
    TEMPERATURE = 18,
    TMP006 = 23,
    TSL2561 = 22,
    USB_USER = 9
}
}

pub fn get_permission_bit(driver_num: usize) -> i8 {
    if let Some(num) = NUM::from_usize(driver_num) {
        match num {
            NUM::ADC => PERMISSION_BIT::ADC as i8,
            NUM::ALARM => PERMISSION_BIT::ALARM as i8,
            NUM::AMBIENT_LIGHT => PERMISSION_BIT::AMBIENT_LIGHT as i8,
            NUM::ANALOG_COMPARATOR => PERMISSION_BIT::ANALOG_COMPARATOR as i8,
            NUM::APP_FLASH => PERMISSION_BIT::APP_FLASH as i8,
            NUM::BLE_ADVERTISING => PERMISSION_BIT::BLE_ADVERTISING as i8,
            NUM::BUTTON => PERMISSION_BIT::BUTTON as i8,
            NUM::CONSOLE => PERMISSION_BIT::CONSOLE as i8,
            NUM::CRC => PERMISSION_BIT::CRC as i8,
            NUM::DAC => PERMISSION_BIT::DAC as i8,
            NUM::GPIO => PERMISSION_BIT::GPIO as i8,
            NUM::GPIO_ASYNC => PERMISSION_BIT::GPIO_ASYNC as i8,
            NUM::HUMIDITY => PERMISSION_BIT::HUMIDITY as i8,
            NUM::I2C_MASTER => PERMISSION_BIT::I2C_MASTER as i8,
            NUM::I2C_MASTER_SLAVE => PERMISSION_BIT::I2C_MASTER_SLAVE as i8,
            NUM::LED => PERMISSION_BIT::LED as i8,
            NUM::LPS25HB => PERMISSION_BIT::LPS25HB as i8,
            NUM::LTC294X => PERMISSION_BIT::LTC294X as i8,
            NUM::MAX17205 => PERMISSION_BIT::MAX17205 as i8,
            NUM::NINEDOF => PERMISSION_BIT::NINEDOF as i8,
            NUM::NRF51822_SERIALIZATION => PERMISSION_BIT::NRF51822_SERIALIZATION as i8,
            NUM::NVM_STORAGE => PERMISSION_BIT::NVM_STORAGE as i8,
            NUM::PCA9544A => PERMISSION_BIT::PCA9544A as i8,
            NUM::RNG => PERMISSION_BIT::RNG as i8,
            NUM::SD_CARD => PERMISSION_BIT::SD_CARD as i8,
            NUM::SPI => PERMISSION_BIT::SPI as i8,
            NUM::TEMPERATURE => PERMISSION_BIT::TEMPERATURE as i8,
            NUM::TMP006 => PERMISSION_BIT::TMP006 as i8,
            NUM::TSL2561 => PERMISSION_BIT::TSL2561 as i8,
            NUM::USB_USER => PERMISSION_BIT::USB_USER as i8,
        }
    } else { -1 }
}
