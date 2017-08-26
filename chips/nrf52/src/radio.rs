//! Radio driver, Bluetooth Low Energy, nRF52
//!
//! Sending Bluetooth Low Energy advertisement packets with payloads up to 31 bytes
//!
//! Currently all fields in PAYLOAD array are configurable from user-space
//! except the PDU_TYPE.
//!
//! ### Author
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: July 18, 2017

use core::cell::Cell;
use kernel;
use nrf5x;
use peripheral_registers;


pub const PACKET0_S1_SIZE: u32 = 0;
pub const PACKET0_S0_SIZE: u32 = 0;
pub const RADIO_PCNF0_LFLEN_POS: u32 = 0;
pub const RADIO_PCNF0_S0LEN_POS: u32 = 8;
pub const RADIO_PCNF0_S1LEN_POS: u32 = 16;
pub const RADIO_PCNFO_S1INCL_POS: u32 = 20;
pub const RADIO_PCNF0_PLEN_POS: u32 = 24;
pub const RADIO_CRCCNF_SKIPADDR_POS: u32 = 8;
pub const RADIO_PCNF1_WHITEEN_DISABLED: u32 = 0;
pub const RADIO_PCNF1_WHITEEN_ENABLED: u32 = 1;
pub const RADIO_PCNF1_WHITEEN_POS: u32 = 25;
pub const RADIO_PCNF1_BALEN_POS: u32 = 16;
pub const RADIO_PCNF1_STATLEN_POS: u32 = 8;
pub const RADIO_PCNF1_MAXLEN_POS: u32 = 0;
pub const RADIO_PCNF1_ENDIAN_POS: u32 = 24;
pub const RADIO_PCNF1_ENDIAN_BIG: u32 = 1;
pub const RADIO_PCNF1_ENDIAN_LITTLE: u32 = 0;
pub const RADIO_CRCCNF_SKIPADDR_MSK: u32 = 1;


pub const NRF_LFLEN_LEN_1BYTE: u32 = 8;
pub const NRF_S0_LEN_1BYTE: u32 = 1;
pub const NRF_S1_ZERO_LEN: u32 = 0;
pub const NRF_S1INCL_MSK: u32 = 0;
pub const NRF_PLEN_8BITS: u32 = 0;
pub const NRF_MAX_LENGTH: u32 = 37;
pub const NRF_DONT_EXTEND: u32 = 0;
pub const NRF_BALEN: u32 = 3;
pub const NRF_3BYTES_CRC: u32 = 3;
pub const NRF_BLE_1MBIT: u32 = 3;
pub const NRF_FREQ_CH_37: u32 = 2;
pub const NRF_FREQ_CH_38: u32 = 26;
pub const NRF_FREQ_CH_39: u32 = 80;


// Interrupts
pub const NRF_READY_INTR: u32 = 1;
pub const NRF_ADDRESS_INTR: u32 = 2;
pub const NRF_PAYLOAD_INTR: u32 = 4;
pub const NRF_END_INTR: u32 = 8;

// Internal Radio State
pub const RADIO_STATE_DISABLE: u32 = 0;
pub const RADIO_STATE_RXRU: u32 = 1;
pub const RADIO_STATE_RXIDLE: u32 = 2;
pub const RADIO_STATE_RX: u32 = 3;
pub const RADIO_STATE_RXDISABLE: u32 = 4;
pub const RADIO_STATE_TXRU: u32 = 9;
pub const RADIO_STATE_TXIDLE: u32 = 10;
pub const RADIO_STATE_TX: u32 = 11;
pub const RADIO_STATE_TXDISABLE: u32 = 12;


// constants for readability purposes
pub const PAYLOAD_HDR_PDU: usize = 0;
pub const PAYLOAD_HDR_LEN: usize = 1;
pub const PAYLOAD_ADDR_START: usize = 2;
pub const PAYLOAD_ADDR_END: usize = 7;
pub const PAYLOAD_DATA_START: usize = 8;
pub const PAYLOAD_LENGTH: usize = 39;

// Header (2 bytes) which consist of:
//
// Byte #1
// PDU Type (4 bits) - see below for info
// RFU (2 bits)      - don't care
// TXAdd (1 bit)     - don't used yet (use public or private addr)
// RXAdd (1 bit)     - don't care (not used for beacons)
//
// Byte #2
// Length of the total packet

// PDU TYPES
// 0x00 - ADV_IND
// 0x01 - ADV_DIRECT_IND
// 0x02 - ADV_NONCONN_IND
// 0x03 - SCAN_REQ
// 0x04 - SCAN_RSP
// 0x05 - CONNECT_REQ
// 0x06 - ADV_SCAN_IND

//  Advertising Type   Connectable  Scannable   Directed    GAP Name
//  ADV_IND            Yes           Yes         No          Connectable Undirected Advertising
//  ADV_DIRECT_IND     Yes           No          Yes         Connectable Directed Advertising
//  ADV_NONCONN_IND    Yes           No          No          Non-connectible Undirected Advertising
//  ADV_SCAN_IND       Yes           Yes         No          Scannable Undirected Advertising

static mut PAYLOAD: [u8; PAYLOAD_LENGTH] = [0x00; PAYLOAD_LENGTH];

pub struct Radio {
    regs: *const peripheral_registers::RADIO,
    txpower: Cell<usize>,
    client: Cell<Option<&'static nrf5x::ble_advertising_hil::RxClient>>,
    freq: Cell<u32>,
}

pub static mut RADIO: Radio = Radio::new();

impl Radio {
    pub const fn new() -> Radio {
        Radio {
            regs: peripheral_registers::RADIO_BASE as *const peripheral_registers::RADIO,
            txpower: Cell::new(0),
            client: Cell::new(None),
            freq: Cell::new(0),
        }
    }

    // Used configure to radio to send BLE advertisements
    fn start_adv_tx(&self, ch: u32) {
        let regs = unsafe { &*self.regs };

        self.radio_on();

        // ADV_NONCONN_IND
        self.set_payload_header_pdu(0x02);

        // TX Power acc. to twpower variable in the struct
        self.set_txpower();

        // BLE MODE
        self.set_channel_rate(NRF_BLE_1MBIT);

        self.set_channel_freq(ch);
        self.set_datawhiteiv(ch);

        // Set PREFIX | BASE Address
        regs.prefix0.set(0x0000008e);
        regs.base0.set(0x89bed600);

        self.set_tx_address(0x00);
        self.set_rx_address(0x01);
        // regs.RXMATCH.set(0x00);

        // Set Packet Config
        self.set_packet_config(0x00);

        // CRC Config
        self.set_crc_config();

        // Buffer configuration
        self.set_buffer();

        regs.event_ready.set(0);
        regs.task_txen.set(1);

        self.enable_interrupts();
        self.enable_nvic();
    }

    fn start_adv_rx(&self) {
        let regs = unsafe { &*self.regs };

        self.radio_on();

        // BLE MODE
        self.set_channel_rate(NRF_BLE_1MBIT);

        // temporary to listen on all advertising frequencies
        match self.freq.get() {
            37 => self.freq.set(38),
            38 => self.freq.set(39),
            _ => self.freq.set(37),
        }

        self.set_channel_freq(self.freq.get());
        self.set_datawhiteiv(self.freq.get());

        // Set PREFIX | BASE Address
        regs.prefix0.set(0x0000008e);
        regs.base0.set(0x89bed600);

        self.set_tx_address(0x00);
        self.set_rx_address(0x01);
        // regs.RXMATCH.set(0x00);

        // Set Packet Config
        self.set_packet_config(0x00);

        // CRC Config
        self.set_crc_config();

        // Buffer configuration
        self.set_buffer();

        self.enable_interrupts();
        self.enable_nvic();

        regs.event_ready.set(0);
        regs.task_rxen.set(1);
    }


    fn set_crc_config(&self) {
        let regs = unsafe { &*self.regs };
        regs.crccnf.set(RADIO_CRCCNF_SKIPADDR_MSK << RADIO_CRCCNF_SKIPADDR_POS | NRF_3BYTES_CRC);
        regs.crcinit.set(0x555555);
        regs.crcpoly.set(0x00065B);
    }

    // Packet configuration
    // Argument unsed atm
    fn set_packet_config(&self, _: u32) {
        let regs = unsafe { &*self.regs };

        // sets the header of PDU TYPE to 1 byte
        // sets the header length to 1 byte
        regs.pcnf0.set((NRF_LFLEN_LEN_1BYTE << RADIO_PCNF0_LFLEN_POS) |
                       (NRF_S0_LEN_1BYTE << RADIO_PCNF0_S0LEN_POS) |
                       (NRF_S1_ZERO_LEN << RADIO_PCNF0_S1LEN_POS) |
                       (NRF_S1INCL_MSK << RADIO_PCNFO_S1INCL_POS) |
                       (NRF_PLEN_8BITS << RADIO_PCNF0_PLEN_POS));

        regs.pcnf1.set((RADIO_PCNF1_WHITEEN_ENABLED << RADIO_PCNF1_WHITEEN_POS) |
                       (RADIO_PCNF1_ENDIAN_LITTLE << RADIO_PCNF1_ENDIAN_POS) |
                       (NRF_BALEN << RADIO_PCNF1_BALEN_POS) |
                       (NRF_DONT_EXTEND << RADIO_PCNF1_STATLEN_POS) |
                       (NRF_MAX_LENGTH << RADIO_PCNF1_MAXLEN_POS));
    }

    // TODO set from capsules?!
    fn set_rx_address(&self, _: u32) {
        let regs = unsafe { &*self.regs };
        regs.rxaddresses.set(0x01);
    }

    // TODO set from capsules?!
    fn set_tx_address(&self, _: u32) {
        let regs = unsafe { &*self.regs };
        regs.txaddress.set(0x00);
    }

    // should not be configured from the capsule i.e.
    // assume always BLE
    fn set_channel_rate(&self, rate: u32) {
        let regs = unsafe { &*self.regs };
        // set channel rate,  3 - BLE 1MBIT/s
        regs.mode.set(rate);
    }

    fn set_datawhiteiv(&self, val: u32) {
        let regs = unsafe { &*self.regs };
        regs.datawhiteiv.set(val);
    }

    fn set_channel_freq(&self, val: u32) {
        let regs = unsafe { &*self.regs };
        //37, 38 and 39 for adv.
        match val {
            37 => regs.frequency.set(NRF_FREQ_CH_37),
            38 => regs.frequency.set(NRF_FREQ_CH_38),
            39 => regs.frequency.set(NRF_FREQ_CH_39),
            _ => regs.frequency.set(NRF_FREQ_CH_37),
        }
    }

    fn radio_on(&self) {
        let regs = unsafe { &*self.regs };
        // reset and enable power
        regs.power.set(0);
        regs.power.set(1);
    }

    fn radio_off(&self) {
        let regs = unsafe { &*self.regs };
        regs.power.set(0);
    }

    // pre-condition validated by the capsule before arriving here
    fn set_txpower(&self) {
        let regs = unsafe { &*self.regs };
        regs.txpower.set(self.txpower.get() as u32);
    }

    fn set_buffer(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set((&PAYLOAD as *const u8) as u32);
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };
        self.disable_nvic();
        self.disable_all_interrupts();
        nrf5x::nvic::clear_pending(nrf5x::peripheral_interrupts::NvicIdx::RADIO);
        let mut end = false;

        if regs.event_ready.get() == 1 {
            regs.event_ready.set(0);
            regs.event_end.set(0);
            regs.task_start.set(1);
        }

        if regs.event_address.get() == 1 {
            regs.event_address.set(0);
        }
        if regs.event_payload.get() == 1 {
            regs.event_payload.set(0);
        }

        if regs.event_end.get() == 1 {
            regs.event_end.set(0);
            end = true;
            // this state only verifies that END is received in TX-mode
            // which means that the transmission is finished
            match regs.state.get() {
                RADIO_STATE_TXRU |
                RADIO_STATE_TXIDLE |
                RADIO_STATE_TXDISABLE |
                RADIO_STATE_TX => {
                    match regs.frequency.get() {
                        NRF_FREQ_CH_39 => {
                            self.radio_off();
                        }
                        NRF_FREQ_CH_38 => {
                            self.start_adv_tx(39);
                        }
                        NRF_FREQ_CH_37 => {
                            self.start_adv_tx(38);
                        }
                        // don't care as we only support advertisements at the moment
                        _ => (),
                    }
                }
                RADIO_STATE_RXRU |
                RADIO_STATE_RXIDLE |
                RADIO_STATE_RXDISABLE |
                RADIO_STATE_RX => {
                    if regs.crcstatus.get() == 1 {
                        unsafe {
                            self.client.get().map(|client| {
                                client.receive(&mut PAYLOAD,
                                               PAYLOAD_LENGTH as u8,
                                               kernel::returncode::ReturnCode::SUCCESS)
                            });
                        }
                    }
                }
                // Radio state - Disabled
                _ => (),
            }
        }
        if !end {
            self.enable_nvic();
            self.enable_interrupts();
        }
    }

    pub fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(NRF_READY_INTR | NRF_ADDRESS_INTR | NRF_PAYLOAD_INTR | NRF_END_INTR);
    }

    pub fn enable_interrupt(&self, intr: u32) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(intr);
    }

    pub fn clear_interrupt(&self, intr: u32) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(intr);
    }

    pub fn disable_all_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        // disable all possible interrupts
        regs.intenclr.set(0xffffffff);
    }

    pub fn enable_nvic(&self) {
        nrf5x::nvic::enable(nrf5x::peripheral_interrupts::NvicIdx::RADIO);
    }

    pub fn disable_nvic(&self) {
        nrf5x::nvic::disable(nrf5x::peripheral_interrupts::NvicIdx::RADIO);
    }

    pub fn reset_payload(&self) {}

    // FIXME: Support for other PDU types than ADV_NONCONN_IND
    pub fn set_payload_header_pdu(&self, pdu: u8) {
        unsafe {
            PAYLOAD[PAYLOAD_HDR_PDU] = pdu;
        }
    }

    pub fn set_payload_header_len(&self, len: u8) {
        unsafe {
            PAYLOAD[PAYLOAD_HDR_LEN] = len;
        }
    }
}

impl nrf5x::ble_advertising_hil::BleAdvertisementDriver for Radio {
    fn clear_adv_data(&self) {
        // reset contents except header || address
        for i in PAYLOAD_DATA_START..PAYLOAD_LENGTH {
            unsafe {
                PAYLOAD[i] = 0;
            }
        }
        // configures a payload with only ADV address
        self.set_payload_header_len(6);
    }
    fn set_advertisement_data(&self,
                              ad_type: usize,
                              data: &'static mut [u8],
                              len: usize,
                              offset: usize)
                              -> &'static mut [u8] {
        // set ad type length and type
        unsafe {
            PAYLOAD[offset] = (len + 1) as u8;
            PAYLOAD[offset + 1] = ad_type as u8;
        }
        // set payload
        for (i, c) in data.as_ref()[0..len].iter().enumerate() {
            unsafe {
                PAYLOAD[i + offset + 2] = *c;
            }
        }

        self.set_payload_header_len((offset + len) as u8);
        data
    }
    fn set_advertisement_address(&self, addr: &'static mut [u8]) -> &'static mut [u8] {
        for (i, c) in addr.as_ref()[0..6].iter().enumerate() {
            unsafe {
                PAYLOAD[i + PAYLOAD_ADDR_START] = *c;
            }
        }
        addr
    }
    fn set_advertisement_txpower(&self, power: usize) -> kernel::ReturnCode {
        match power {
            // +4 dBm, 0 dBm, -4 dBm, -8 dBm, -12 dBm, -16 dBm, -20 dBm, -30 dBm
            0x04 | 0x00 | 0xF4 | 0xFC | 0xF8 | 0xF0 | 0xEC | 0xD8 => {
                self.txpower.set(power);
                kernel::ReturnCode::SUCCESS
            }
            _ => kernel::ReturnCode::ENOSUPPORT,
        }
    }
    fn start_advertisement_tx(&self, ch: usize) {
        self.start_adv_tx(ch as u32);
    }
    fn start_advertisement_rx(&self, _ch: usize) {
        self.start_adv_rx();
    }

    fn set_client(&self, client: &'static nrf5x::ble_advertising_hil::RxClient) {
        self.client.set(Some(client));
    }
}
