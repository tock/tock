use core::convert::TryFrom;

// PCNF0
pub const RADIO_PCNF0_LFLEN_POS: u32 = 0;
pub const RADIO_PCNF0_S0LEN_POS: u32 = 8;
pub const RADIO_PCNF0_S1LEN_POS: u32 = 16;
pub const RADIO_PCNF0_LFLEN_1BYTE: u32 = 8;
pub const RADIO_PCNF0_S0_LEN_1BYTE: u32 = 1;
pub const RADIO_PCNF0_S1_ZERO: u32 = 0;

// PCNF1
pub const RADIO_PCNF1_WHITEEN_DISABLED: u32 = 0;
pub const RADIO_PCNF1_WHITEEN_ENABLED: u32 = 1;
pub const RADIO_PCNF1_WHITEEN_POS: u32 = 25;
pub const RADIO_PCNF1_BALEN_POS: u32 = 16;
pub const RADIO_PCNF1_STATLEN_POS: u32 = 8;
pub const RADIO_PCNF1_MAXLEN_POS: u32 = 0;
pub const RADIO_PCNF1_ENDIAN_POS: u32 = 24;
pub const RADIO_PCNF1_ENDIAN_BIG: u32 = 1;
pub const RADIO_PCNF1_ENDIAN_LITTLE: u32 = 0;
pub const RADIO_PCNF1_MAXLEN_37BYTES: u32 = 37;
pub const RADIO_PCNF1_MAXLEN_255BYTES: u32 = 255;
pub const RADIO_PCNF1_STATLEN_DONT_EXTEND: u32 = 0;
pub const RADIO_PCNF1_BALEN_3BYTES: u32 = 3;

// CRC
pub const RADIO_CRCCNF_SKIPADDR_POS: u32 = 8;
pub const RADIO_CRCCNF_SKIPADDR: u32 = 1;
pub const RADIO_CRCCNF_LEN_3BYTES: u32 = 3;
pub const RADIO_CRCINIT_BLE: u32 = 0x555555;
pub const RADIO_CRCPOLY_BLE: u32 = 0x00065B;

// MODE
pub const RADIO_MODE_BLE_1MBIT: u32 = 3;

// FREQUENCY
pub const RADIO_FREQ_CH_37: u32 = 2;
pub const RADIO_FREQ_CH_39: u32 = 80;
pub const RADIO_FREQ_CH_38: u32 = 26;

// INTENSET
// There are more INTENSET flags but they differ between nRF51 & nRF51
pub const RADIO_INTENSET_READY: u32 = 1;
pub const RADIO_INTENSET_ADDRESS: u32 = 1 << 1;
pub const RADIO_INTENSET_PAYLOAD: u32 = 1 << 2;
pub const RADIO_INTENSET_END: u32 = 1 << 3;
pub const RADIO_INTENSET_DISABLED: u32 = 1 << 4;

// STATE
pub const RADIO_STATE_DISABLE: u32 = 0;
pub const RADIO_STATE_RXRU: u32 = 1;
pub const RADIO_STATE_RXIDLE: u32 = 2;
pub const RADIO_STATE_RX: u32 = 3;
pub const RADIO_STATE_RXDISABLE: u32 = 4;
pub const RADIO_STATE_TXRU: u32 = 9;
pub const RADIO_STATE_TXIDLE: u32 = 10;
pub const RADIO_STATE_TX: u32 = 11;
pub const RADIO_STATE_TXDISABLE: u32 = 12;

//Shortcuts
pub const RADIO_SHORTS_READY_START: u32 = 1;
pub const RADIO_SHORTS_END_DISABLE: u32 = 1 << 1;
pub const RADIO_SHORTS_DISABLED_TXEN: u32 = 1 << 2;
pub const RADIO_SHORTS_DISABLED_RXEN: u32 = 1 << 3;
pub const RADIO_SHORTS_ADDRESS_RSSISTART: u32 = 1 << 4;
pub const RADIO_SHORTS_END_START: u32 = 1 << 5;
pub const RADIO_SHORTS_ADDRESS_BCSTART: u32 = 1 << 6;
pub const RADIO_SHORTS_DISABLED_RSSISTOP: u32 = 1 << 8;

// Table 2. Pre-programmed PPI channels
pub const PPI_CHEN_CH20: u32 = 1 << 20; // TIMER0->EVENTS_COMPARE[0]    RADIO->TASKS_TXEN
pub const PPI_CHEN_CH21: u32 = 1 << 21; // TIMER0->EVENTS_COMPARE[0]	RADIO->TASKS_RXEN
pub const PPI_CHEN_CH22: u32 = 1 << 22; // TIMER0->EVENTS_COMPARE[1]	RADIO->TASKS_DISABLE
pub const PPI_CHEN_CH23: u32 = 1 << 23; // RADIO->EVENTS_BCMATCH	    AAR->TASKS_START
pub const PPI_CHEN_CH24: u32 = 1 << 24; // RADIO->EVENTS_READY	        CCM->TASKS_KSGEN
pub const PPI_CHEN_CH25: u32 = 1 << 25; // RADIO->EVENTS_ADDRESS	    CCM->TASKS_CRYPT
pub const PPI_CHEN_CH26: u32 = 1 << 26; // RADIO->EVENTS_ADDRESS	    TIMER0->TASKS_CAPTURE[1]
pub const PPI_CHEN_CH27: u32 = 1 << 27; // RADIO->EVENTS_END	        TIMER0->TASKS_CAPTURE[2]
pub const PPI_CHEN_CH28: u32 = 1 << 28; // RTC0->EVENTS_COMPARE[0]	    RADIO->TASKS_TXEN
pub const PPI_CHEN_CH29: u32 = 1 << 29; // RTC0->EVENTS_COMPARE[0]	    RADIO->TASKS_RXEN
pub const PPI_CHEN_CH30: u32 = 1 << 30; // RTC0->EVENTS_COMPARE[0]	    TIMER0->TASKS_CLEAR
pub const PPI_CHEN_CH31: u32 = 1 << 31; // RTC0->EVENTS_COMPARE[0]	    TIMER0->TASKS_START

// BUFFER SIZE
pub const RADIO_PAYLOAD_LENGTH: usize = 255;

pub enum RadioMode {
    Nrf1Mbit = 0,
    Nrf2Mbit = 1,
    Nrt250Kbit = 2,
    Ble1Mbit = 3,
}

#[derive(Debug, Copy, Clone)]
pub enum TxPower {
    Positive4dBM = 0x04,
    Positive3dBM = 0x03,
    ZerodBm = 0x00,
    Negative4dBm = 0xFC,
    Negative8dBm = 0xF8,
    Negative12dBm = 0xF4,
    Negative16dBm = 0xF0,
    Negative20dBm = 0xEC,
    Negative40dBm = 0xD8,
}

//FIXME: use enum-tryfrom-derive, https://docs.rs/crate/enum-tryfrom-derive/0.1.2
impl TryFrom<u8> for TxPower {
    type Error = ();

    fn try_from(val: u8) -> Result<TxPower, ()> {
        match val {
            4 => Ok(TxPower::Positive4dBM),
            3 => Ok(TxPower::Positive3dBM),
            0 => Ok(TxPower::ZerodBm),
            0xFC => Ok(TxPower::Negative4dBm),
            0xF8 => Ok(TxPower::Negative8dBm),
            0xF4 => Ok(TxPower::Negative12dBm),
            0xF0 => Ok(TxPower::Negative16dBm),
            0xEC => Ok(TxPower::Negative20dBm),
            0xD8 => Ok(TxPower::Negative40dBm),
            _ => Err(()),
        }
    }
}
