
// registers --- are structs better?
enum RegMap {
    RegFifo                 = 0x00,
    RegOpMode               = 0x01,
    RegFrfMsb               = 0x06,
    RegFrfMid               = 0x07,
    RegFrfLsb               = 0x08,
    RegPaConfig             = 0x09,
    RegOcp                  = 0x0b,
    RegLna                  = 0x0c,
    RegFifoAddrPtr          = 0x0d,
    RegFifoTxBaseAddr       = 0x0e,
    RegFifoRxBaseAddr       = 0x0f,
    RegFifoRxCurrentAddr    = 0x10,
    RegIrqFlags             = 0x12,
    RegRxNbBytes            = 0x13,
    RegPktSnrValue          = 0x19,
    RegPktRssiValue         = 0x1a,
    RegModemConfig_1        = 0x1d,
    RegModemConfig_2        = 0x1e,
    RegPreambleMsb          = 0x20,
    RegPreambleLsb          = 0x21,
    RegPayloadLength        = 0x22,
    RegModemConfig_3        = 0x26,
    RegFreqErrorMsb         = 0x28,
    RegFreqErrorMid         = 0x29,
    RegFreqErrorLsb         = 0x2a,
    RegRssiWideband         = 0x2c,
    RegDetectionOptimize    = 0x31,
    RegInvertiq             = 0x33,
    RegDetectionThreshold   = 0x37,
    RegSyncWord             = 0x39,
    RegInvertiq2            = 0x3b,
    RegDioMapping_1         = 0x40,
    RegVersion              = 0x42,
    RegPaDac                = 0x4d
}

// modes
enum Mode {
    ModeLongRangeMode       = 0x80,
    ModeSleep               = 0x00,
    ModeStdby               = 0x01,
    ModeTx                  = 0x03,
    ModeRxContinuous        = 0x05,
    ModeRxSingle            = 0x06
}


// Irq masks
enum Irq {
    IrqTxDoneMask           = 0x08,
    IrqPayloadCrcErrorMask  = 0x20,
    IrqRxDoneMask           = 0x40
}

// Other config
const PaBoost               = 0x80
const MaxPktLength          = 255

// The modem
// Can possibly expand struct like RF233
struct Radio<'a, S: spi::SpiMasterDevice> {
    //add SPI Settings: LORA_DEFAULT_SPI_FREQUENCY, MSBFIRST, SPI_MODE0
    spi: &'a S,
    spi_busy: Cell<bool>,
    spi_rx: TakeCell<'static, [u8]>,
    spi_tx: TakeCell<'static, [u8]>,
    spi_buf: TakeCell<'static, [u8]>,
    //add Pins: ss(LORA_DEFAULT_SS_PIN), _reset(LORA_DEFAULT_RESET_PIN), _dio0(LORA_DEFAULT_DIO0_PIN),
    // ss_pin: &'a dyn gpio::Pin,
    // reset_pin: &'a dyn gpio::Pin,
    // irq_pin: &'a dyn gpio::InterruptPin,
    //LoRa params
    frequency: u64 = 0,
    packet_index: u32 = 0,
    implicit_header: u8 = 0,
    //Do we need this?
    interrupt_handling: Cell<bool>,
    state: Cell<InternalState>,
    //MAC
    addr: Cell<u16>,
    addr_long: Cell<[u8; 8]>,
    pan: Cell<u16>,
    channel: Cell<u8>,
    crc_valid: Cell<bool>,
}

