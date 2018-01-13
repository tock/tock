//! Radio driver, Bluetooth Low Energy, NRF52
//!
//! The generic radio configuration i.e., not specific to Bluetooth are functions and similar which
//! do not start with `ble`. Moreover, Bluetooth Low Energy specific radio configuration
//! starts with `ble`
//!
//! For more readability the Bluetooth specific configuration may be moved to separate trait
//!
//! ### Author
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: July 18, 2017
//!
//! ### Packet Configuration
//! ```
//! +----------+------+--------+----+--------+----+---------+-----+
//! | Preamble | Base | Prefix | S0 | Length | S1 | Payload | CRC |
//! +----------+------+--------+----+--------+----+---------+-----+
//! ```
//!
//! * Premable - 1 byte
//!
//! * Base and prefix forms together the access address
//!
//! * S0, an optional parameter that is configured to indicate how many bytes of
//! the payload is the PDU Type. Configured as 1 byte!
//!
//! * Length, an optional parameter that is configured to indicate how many bits of the
//! payload is the length field. Configured as 8 bits!
//!
//! * S1, Not used
//!
//! * Payload - 2 to 255 bytes
//!
//! * CRC - 3 bytes

use core::cell::Cell;
use kernel;
use kernel::ReturnCode;
use nrf5x;
use nrf5x::ble_advertising_hil::RadioChannel;
use nrf5x::constants::TxPower;
use peripheral_registers;

// NRF52 Specific Radio Constants
const NRF52_RADIO_PCNF0_S1INCL_MSK: u32 = 0;
const NRF52_RADIO_PCNFO_S1INCL_POS: u32 = 20;
const NRF52_RADIO_PCNF0_PLEN_POS: u32 = 24;
const NRF52_RADIO_PCNF0_PLEN_8BITS: u32 = 0;

static mut PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

pub struct Radio {
    regs: *const peripheral_registers::RADIO,
    tx_power: Cell<TxPower>,
    rx_client: Cell<Option<&'static nrf5x::ble_advertising_hil::RxClient>>,
    tx_client: Cell<Option<&'static nrf5x::ble_advertising_hil::TxClient>>,
}

pub static mut RADIO: Radio = Radio::new();

impl Radio {
    pub const fn new() -> Radio {
        Radio {
            regs: peripheral_registers::RADIO_BASE as *const peripheral_registers::RADIO,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: Cell::new(None),
            tx_client: Cell::new(None),
        }
    }

    fn tx(&self) {
        let regs = unsafe { &*self.regs };
        regs.event_ready.set(0);
        regs.task_txen.set(1);
    }

    fn rx(&self) {
        let regs = unsafe { &*self.regs };
        regs.event_ready.set(0);
        regs.task_rxen.set(1);
    }

    fn set_rx_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.rxaddresses.set(0x01);
    }

    fn set_tx_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.txaddress.set(0x00);
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

    fn set_tx_power(&self) {
        let regs = unsafe { &*self.regs };
        regs.txpower.set(self.tx_power.get() as u32);
    }

    fn set_dma_ptr(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set((&PAYLOAD as *const u8) as u32);
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };
        self.disable_all_interrupts();

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

        // tx or rx finished!
        if regs.event_end.get() == 1 {
            regs.event_end.set(0);

            let result = if regs.crcstatus.get() == 1 {
                ReturnCode::SUCCESS
            } else {
                ReturnCode::FAIL
            };

            match regs.state.get() {
                nrf5x::constants::RADIO_STATE_TXRU
                | nrf5x::constants::RADIO_STATE_TXIDLE
                | nrf5x::constants::RADIO_STATE_TXDISABLE
                | nrf5x::constants::RADIO_STATE_TX => {
                    self.radio_off();
                    self.tx_client
                        .get()
                        .map(|client| client.transmit_event(result));
                }
                nrf5x::constants::RADIO_STATE_RXRU
                | nrf5x::constants::RADIO_STATE_RXIDLE
                | nrf5x::constants::RADIO_STATE_RXDISABLE
                | nrf5x::constants::RADIO_STATE_RX => {
                    self.radio_off();
                    unsafe {
                        self.rx_client.get().map(|client| {
                            client.receive_event(&mut PAYLOAD, PAYLOAD[1] + 1, result)
                        });
                    }
                }
                // Radio state - Disabled
                _ => (),
            }
        }
        self.enable_interrupts();
    }

    pub fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(
            nrf5x::constants::RADIO_INTENSET_READY | nrf5x::constants::RADIO_INTENSET_ADDRESS
                | nrf5x::constants::RADIO_INTENSET_PAYLOAD
                | nrf5x::constants::RADIO_INTENSET_END,
        );
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

    pub fn replace_radio_buffer(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8] {
        // set payload
        for (i, c) in buf.as_ref()[0..len].iter().enumerate() {
            unsafe {
                PAYLOAD[i] = *c;
            }
        }
        buf
    }

    fn ble_initialize(&self, channel: RadioChannel) {
        self.radio_on();

        self.ble_set_tx_power();

        self.ble_set_channel_rate();

        self.ble_set_channel_freq(channel);
        self.ble_set_data_whitening(channel);

        self.set_tx_address();
        self.set_rx_address();

        self.ble_set_packet_config();
        self.ble_set_advertising_access_address();

        self.ble_set_crc_config();

        self.set_dma_ptr();
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.1.1 CRC Generation
    fn ble_set_crc_config(&self) {
        let regs = unsafe { &*self.regs };
        regs.crccnf.set(
            nrf5x::constants::RADIO_CRCCNF_SKIPADDR << nrf5x::constants::RADIO_CRCCNF_SKIPADDR_POS
                | nrf5x::constants::RADIO_CRCCNF_LEN_3BYTES,
        );
        regs.crcinit.set(nrf5x::constants::RADIO_CRCINIT_BLE);
        regs.crcpoly.set(nrf5x::constants::RADIO_CRCPOLY_BLE);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.1.2 Access Address
    // Set access address to 0x8E89BED6
    fn ble_set_advertising_access_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.prefix0.set(0x0000008e);
        regs.base0.set(0x89bed600);
    }

    // Packet configuration
    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.1 Packet Format
    //
    // LSB                                                      MSB
    // +----------+   +----------------+   +---------------+   +------------+
    // | Preamble | - | Access Address | - | PDU           | - | CRC        |
    // | (1 byte) |   | (4 bytes)      |   | (2-255 bytes) |   | (3 bytes)  |
    // +----------+   +----------------+   +---------------+   +------------+
    //
    fn ble_set_packet_config(&self) {
        let regs = unsafe { &*self.regs };

        // sets the header of PDU TYPE to 1 byte
        // sets the header length to 1 byte
        regs.pcnf0.set(
            (nrf5x::constants::RADIO_PCNF0_LFLEN_1BYTE << nrf5x::constants::RADIO_PCNF0_LFLEN_POS)
                | (nrf5x::constants::RADIO_PCNF0_S0_LEN_1BYTE
                    << nrf5x::constants::RADIO_PCNF0_S0LEN_POS)
                | (nrf5x::constants::RADIO_PCNF0_S1_ZERO << nrf5x::constants::RADIO_PCNF0_S1LEN_POS)
                | (NRF52_RADIO_PCNF0_S1INCL_MSK << NRF52_RADIO_PCNFO_S1INCL_POS)
                | (NRF52_RADIO_PCNF0_PLEN_8BITS << NRF52_RADIO_PCNF0_PLEN_POS),
        );

        regs.pcnf1.set(
            (nrf5x::constants::RADIO_PCNF1_WHITEEN_ENABLED
                << nrf5x::constants::RADIO_PCNF1_WHITEEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_ENDIAN_LITTLE
                    << nrf5x::constants::RADIO_PCNF1_ENDIAN_POS)
                | (nrf5x::constants::RADIO_PCNF1_BALEN_3BYTES
                    << nrf5x::constants::RADIO_PCNF1_BALEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_STATLEN_DONT_EXTEND
                    << nrf5x::constants::RADIO_PCNF1_STATLEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_MAXLEN_255BYTES
                    << nrf5x::constants::RADIO_PCNF1_MAXLEN_POS),
        );
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part A], 4.6 REFERENCE SIGNAL DEFINITION
    // Bit Rate = 1 Mb/s ±1 ppm
    fn ble_set_channel_rate(&self) {
        let regs = unsafe { &*self.regs };
        regs.mode.set(nrf5x::constants::RadioMode::Ble1Mbit as u32);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.2 Data Whitening
    // Configure channel index to the LFSR and the hardware solves the rest
    fn ble_set_data_whitening(&self, channel: RadioChannel) {
        let regs = unsafe { &*self.regs };
        regs.datawhiteiv.set(channel.get_channel_index());
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 1.4.1
    // RF Channels:     0 - 39
    // Data:            0 - 36
    // Advertising:     37, 38, 39
    fn ble_set_channel_freq(&self, channel: RadioChannel) {
        let regs = unsafe { &*self.regs };
        regs.frequency.set(channel as u32);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3 TRANSMITTER CHARACTERISTICS
    // Minimum Output Power : -20dBm
    // Maximum Output Power : +10dBm
    //
    // no check is required because the BleConfig::set_tx_power() method ensures that only
    // valid tranmitting power is configured!
    fn ble_set_tx_power(&self) {
        self.set_tx_power();
    }
}

impl nrf5x::ble_advertising_hil::BleAdvertisementDriver for Radio {
    fn transmit_advertisement(
        &self,
        buf: &'static mut [u8],
        len: usize,
        channel: RadioChannel,
    ) -> &'static mut [u8] {
        let res = self.replace_radio_buffer(buf, len);
        self.ble_initialize(channel);
        self.tx();
        self.enable_interrupts();
        res
    }

    fn receive_advertisement(&self, channel: RadioChannel) {
        self.ble_initialize(channel);
        self.rx();
        self.enable_interrupts();
    }

    fn set_receive_client(&self, client: &'static nrf5x::ble_advertising_hil::RxClient) {
        self.rx_client.set(Some(client));
    }

    fn set_transmit_client(&self, client: &'static nrf5x::ble_advertising_hil::TxClient) {
        self.tx_client.set(Some(client));
    }
}

// The BLE Advertising Driver validates that the `tx_power` is between -20 to 10 dBm but then
// underlying chip must validate if the current `tx_power` is supported as well
impl nrf5x::ble_advertising_hil::BleConfig for Radio {
    fn set_tx_power(&self, tx_power: u8) -> kernel::ReturnCode {
        // Convert u8 to TxPower
        // similiar functionlity as the FromPrimitive trait
        match nrf5x::constants::TxPower::from_u8(tx_power) {
            // Invalid transmitting power, propogate error
            TxPower::Error => kernel::ReturnCode::ENOSUPPORT,
            // Valid transmitting power, propogate success
            e @ TxPower::Positive4dBM
            | e @ TxPower::Positive3dBM
            | e @ TxPower::ZerodBm
            | e @ TxPower::Negative4dBm
            | e @ TxPower::Negative8dBm
            | e @ TxPower::Negative12dBm
            | e @ TxPower::Negative16dBm
            | e @ TxPower::Negative20dBm
            | e @ TxPower::Negative40dBm => {
                self.tx_power.set(e);
                kernel::ReturnCode::SUCCESS
            }
        }
    }
}
