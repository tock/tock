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

// sorted in order of ascending driver number
// this is so it's easier to avoid conflicts when adding a new driver
pub fn get_permission_bit(driver_num: usize) -> Option<i8> {
    if let Some(num) = NUM::from_usize(driver_num) {
        Some(
            (match num {
                NUM::ALARM => 0,
                NUM::CONSOLE => 1,
                NUM::LED => 2,
                NUM::BUTTON => 3,
                NUM::GPIO => 4,
                NUM::ADC => 5,
                NUM::DAC => 6,
                NUM::ANALOG_COMPARATOR => 7,
                NUM::SPI => 8,
                NUM::USB_USER => 9,
                NUM::I2C_MASTER_SLAVE => 10,
                NUM::BLE_ADVERTISING => 11,
                NUM::RNG => 12,
                NUM::CRC => 13,
                NUM::I2C_MASTER => 14,
                NUM::APP_FLASH => 15,
                NUM::NVM_STORAGE => 16,
                NUM::SD_CARD => 17,
                NUM::TEMPERATURE => 18,
                NUM::HUMIDITY => 19,
                NUM::AMBIENT_LIGHT => 20,
                NUM::NINEDOF => 21,
                NUM::TSL2561 => 22,
                NUM::TMP006 => 23,
                NUM::LPS25HB => 24,
                NUM::LTC294X => 25,
                NUM::MAX17205 => 26,
                NUM::PCA9544A => 27,
                NUM::GPIO_ASYNC => 28,
                NUM::NRF51822_SERIALIZATION => 29,
            }) as i8,
        )
    } else {
        None
    }
}
