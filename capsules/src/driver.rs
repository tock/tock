use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
// syscall driver numbers
pub enum NUM {
    Adc = 0x00000005,
    Alarm = 0x00000000,
    AmbientLight = 0x60002,
    AnalogComparator = 0x00007,
    AppFlash =  0x50000,
    BleAdvertising = 0x030000,
    Button = 0x00000003,
    Console = 0x00000001,
    Crc = 0x40002,
    Dac = 0x00000006,
    Gpio = 0x00000004,
    GpioAsync = 0x80003,
    Humidity= 0x60001,
    I2cMaster = 0x40006,
    I2cMasterSlave = 0x20006,
    Led = 0x2,
    Lps25hb = 0x70004,
    Ltc294x = 0x80000,
    Max17205 = 0x80001,
    NINEDOF = 0x60004,
    NvmStorage = 0x50001,
    Nrf51822Serialization = 0x80004,
    Pca9544a = 0x80002,
    Rng = 0x40001,
    SdCard = 0x50002,
    Spi = 0x20001,
    Temperature = 0x60000,
    Tmp006 = 0x70001,
    Tsl2561 = 0x70000,
    UsbUser = 0x20005,
}
}
