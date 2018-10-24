use enum_primitive::cast::FromPrimitive;

enum_from_primitive!{
#[derive(Debug, PartialEq)]
pub enum NVIC_IRQ {
    GPIO = 0,
    I2C0 = 1,
    RF_CORE_CPE1 = 2,
    //UNASSIGNED 3
    AON_RTC = 4,
    UART0 = 5,
    SSI0 = 7,
    SSI1 = 8,
    RF_CORE_CPE0 = 9,
    RF_CORE_HW = 10,
    RF_CMD_ACK = 11,
    I2S = 12,
    //UNASSIGNED 13
    WATCHDOG = 14,
    GPT0A = 15,
    GPT0B = 16,
    GPT1A = 17,
    GPT1B = 18,
    GPT2A = 19,
    GPT2B = 20,
    GPT3A = 21,
    GPT3B = 22,
    CRPYTO = 23,
    DMA_SW = 24,
    DMA_ERROR = 25,
    FLASH = 26,
    SW_EVENT0 = 27,
    AUX_COMBINED = 28,
    AON_PROG = 29,
    DYNAMIC_PROG = 30,
    AUX_COMP_A = 31,
    AUX_ADC = 32,
    TRNG = 33,
    OSC = 34,
    UART1 = 36
}
}
