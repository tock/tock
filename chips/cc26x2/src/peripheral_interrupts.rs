use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum NvicIrq {
    Gpio = 0,
    I2c0 = 1,
    RfCorePe1 = 2,
    //UNASSIGNED 3
    AonRtc = 4,
    Uart0 = 5,
    Ssi0 = 7,
    Ssi1 = 8,
    RfCorePe2 = 9,
    RfCoreHw = 10,
    RfCmdAck = 11,
    I2s = 12,
    //UNASSIGNED 13
    Watchdog = 14,
    Gpt0a = 15,
    Gpt0b = 16,
    Gpt1a = 17,
    Gpt1b = 18,
    Gpt2a = 19,
    Gpt2b = 20,
    Gpt3a = 21,
    Gpt3b = 22,
    Crypto = 23,
    DmaSu = 24,
    DmaError = 25,
    Flash = 26,
    SwEvent0 = 27,
    AuxCombined = 28,
    AonProg = 29,
    DynamicProg = 30,
    AuxCompA = 31,
    AuxAdc = 32,
    Trng = 33,
    Uart1 = 36
}
}
