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
use core::convert::TryFrom;
use kernel;
use kernel::ReturnCode;
use nrf5x;
use nrf5x::ble_advertising_hil::{DisablePHY, PhyTransition, RadioChannel, ReadAction};
use nrf5x::constants::TxPower;
use peripheral_registers;
use ppi;

// NRF52 Specific Radio Constants
const NRF52_RADIO_PCNF0_S1INCL_MSK: u32 = 0;
const NRF52_RADIO_PCNFO_S1INCL_POS: u32 = 20;
const NRF52_RADIO_PCNF0_PLEN_POS: u32 = 24;
const NRF52_RADIO_PCNF0_PLEN_8BITS: u32 = 0;

#[allow(unused)]
const NRF52_RADIO_MODECNF0_RU_DEFAULT: u32 = 0;
const NRF52_RADIO_MODECNF0_RU_FAST: u32 = 1;

const NRF52_FAST_RAMPUP_TIME_TX: u32 = 40;
const NRF52_TX_DELAY: u32 = 3;
const NRF52_TX_END_DELAY: u32 = 3;
const NRF52_RX_END_DELAY: u32 = 7;
const BLE_T_IFS: u32 = 150;

static mut TX_PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

static mut RX_PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

pub struct Radio {
    regs: *const peripheral_registers::RADIO,
    tx_power: Cell<TxPower>,
    rx_client: Cell<Option<&'static nrf5x::ble_advertising_hil::RxClient>>,
    tx_client: Cell<Option<&'static nrf5x::ble_advertising_hil::TxClient>>,
    advertisement_client: Cell<Option<&'static nrf5x::ble_advertising_hil::AdvertisementClient>>,
    state: Cell<RadioState>,
    channel: Cell<Option<RadioChannel>>,
    transition: Cell<PhyTransition>,
}

#[derive(PartialEq, Copy, Clone)]
enum RadioState {
    TX,
    RX,
    Initialized,
    Uninitialized,
}

pub static mut RADIO: Radio = Radio::new();

impl Radio {
    pub const fn new() -> Radio {
        Radio {
            regs: peripheral_registers::RADIO_BASE as *const peripheral_registers::RADIO,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: Cell::new(None),
            tx_client: Cell::new(None),
            advertisement_client: Cell::new(None),
            state: Cell::new(RadioState::Uninitialized),
            channel: Cell::new(None),
            transition: Cell::new(PhyTransition::None),
        }
    }

    pub fn tx(&self) -> bool {
        let regs = unsafe { &*self.regs };

        self.wait_until_disabled();
        self.disable_ppi(nrf5x::constants::PPI_CHEN_CH23 | nrf5x::constants::PPI_CHEN_CH25);
        self.set_dma_ptr_tx();

        self.state.set(RadioState::TX);

        regs.event_ready.set(0);
        regs.event_end.set(0);
        regs.event_disabled.set(0);

        regs.shorts.set(
            nrf5x::constants::RADIO_SHORTS_END_DISABLE | nrf5x::constants::RADIO_SHORTS_READY_START,
        );

        self.enable_interrupt(nrf5x::constants::RADIO_INTENSET_DISABLED);

        let state = regs.state.get();

        if state != nrf5x::constants::RADIO_STATE_TX {
            true
        } else {
            panic!("STATE is not TX\n");
            self.disable_radio();
            false
        }
    }

    fn wait_until_disabled(&self) {
        let regs = unsafe { &*self.regs };

        let state = regs.state.get();

        if state != nrf5x::constants::RADIO_STATE_DISABLE {
            if state == nrf5x::constants::RADIO_STATE_RXDISABLE
                || state == nrf5x::constants::RADIO_STATE_TXDISABLE
            {
                while regs.state.get() == state {
                    // wait until state completes transition with a blocking loop
                }
            }
        }
    }

    pub fn rx(&self) {
        let regs = unsafe { &*self.regs };

        self.wait_until_disabled();
        self.disable_all_interrupts();

        regs.event_end.set(0);
        regs.event_disabled.set(0);

        self.setup_rx();

        self.state.set(RadioState::RX);

        // TODO: if not already going to rx!
        regs.task_rxen.set(1);
    }

    fn setup_rx(&self) {
        let regs = unsafe { &*self.regs };

        self.set_dma_ptr_rx();

        self.disable_ppi(nrf5x::constants::PPI_CHEN_CH20 | nrf5x::constants::PPI_CHEN_CH23);

        self.state.set(RadioState::RX);

        regs.bcc.set(8); // count one byte

        regs.event_address.set(0);
        regs.event_devmatch.set(0);
        regs.bcmatch.set(0);
        regs.event_rssiend.set(0);
        regs.crcok.set(0);

        regs.shorts.set(
            nrf5x::constants::RADIO_SHORTS_END_DISABLE | nrf5x::constants::RADIO_SHORTS_READY_START
                | nrf5x::constants::RADIO_SHORTS_ADDRESS_BCSTART,
        );

        self.enable_interrupt(nrf5x::constants::RADIO_INTENSET_ADDRESS);
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
        regs.shorts.set(0);
        regs.power.set(0);
    }

    fn set_tx_power(&self) {
        let regs = unsafe { &*self.regs };
        regs.txpower.set(self.tx_power.get() as u32);
    }

    fn set_tifs(&self) {
        let regs = unsafe { &*self.regs };
        regs.tifs.set(150 as u32);
    }

    fn set_dma_ptr_tx(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set((&TX_PAYLOAD as *const u8) as u32);
        }
    }

    fn set_dma_ptr_rx(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set((&RX_PAYLOAD as *const u8) as u32);
        }
    }

    fn set_tx_start_time(&self, usec: u32) {
        let start_time = usec - NRF52_FAST_RAMPUP_TIME_TX - NRF52_TX_DELAY;

        unsafe {
            nrf5x::timer::TIMER0.clear();
            nrf5x::timer::TIMER0.set_cc0(start_time);
            nrf5x::timer::TIMER0.set_events_compare(0, 0);
        }

        self.enable_ppi(nrf5x::constants::PPI_CHEN_CH20);
    }

    fn schedule_tx_after_t_ifs(&self) {
        let end_time = self.get_packet_end_time_value();

        let time =
            end_time + BLE_T_IFS - NRF52_RX_END_DELAY - NRF52_FAST_RAMPUP_TIME_TX - NRF52_TX_DELAY;

        unsafe {
            nrf5x::timer::TIMER0.set_cc0(time);
            nrf5x::timer::TIMER0.set_events_compare(0, 0);
        }

        // CH20: CC[0] => TXEN
        self.enable_ppi(nrf5x::constants::PPI_CHEN_CH20);
    }

    fn schedule_rx_after_t_ifs(&self) {
        let end_time = self.get_packet_end_time_value();
        let earlier_listen = 2;

        let time =
            end_time + BLE_T_IFS - NRF52_TX_END_DELAY - NRF52_FAST_RAMPUP_TIME_TX - earlier_listen;

        unsafe {
            nrf5x::timer::TIMER0.set_cc0(time);
            nrf5x::timer::TIMER0.set_events_compare(0, 0);
        }

        // CH21: CC[0] => RXEN
        self.enable_ppi(nrf5x::constants::PPI_CHEN_CH21);
    }

    fn disable_radio(&self) {
        let regs = unsafe { &*self.regs };

        unsafe {
            nrf5x::timer::TIMER0.stop();
        }

        self.disable_all_interrupts();

        regs.shorts.set(0);
        regs.task_disable.set(1);
        self.disable_ppi(
            nrf5x::constants::PPI_CHEN_CH20 | nrf5x::constants::PPI_CHEN_CH21
                | nrf5x::constants::PPI_CHEN_CH23 | nrf5x::constants::PPI_CHEN_CH25
                | nrf5x::constants::PPI_CHEN_CH31,
        );
        self.state.set(RadioState::Initialized);
    }

    fn handle_address_event(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_address.set(0);

        self.clear_interrupt(
            nrf5x::constants::RADIO_INTENSET_DISABLED | nrf5x::constants::RADIO_INTENSET_ADDRESS,
        );

        // Calculate accurate packets start time?
        // let address_time = self.get_packet_address_time_value();

        loop {
            let state = regs.state.get();

            if regs.bcmatch.get() != 0 {
                break;
            }

            if state == nrf5x::constants::RADIO_STATE_DISABLE {
                self.disable_all_interrupts();
                regs.shorts.set(0);
                return false;
            }
        }

        if let Some(client) = self.rx_client.get() {
            let result = unsafe { client.receive_start(&mut RX_PAYLOAD, RX_PAYLOAD[1] + 2) };

            match result {
                ReadAction::ReadFrameAndStayRX | ReadAction::ReadFrameAndMoveToTX => {
                    // TODO set phy_rx_started = 1
                    self.enable_interrupt(nrf5x::constants::RADIO_INTENSET_END);
                }
                ReadAction::SkipFrame => {
                    self.disable_radio();
                    self.advertisement_client
                        .get()
                        .take()
                        .map(|client| client.advertisement_done());
                }
            }
        } else {
            panic!("No rx_client?\n");
        }

        return true;
    }

    fn handle_rx_end_event(&self) {
        let regs = unsafe { &*self.regs };
        regs.event_end.set(0);

        self.clear_interrupt(nrf5x::constants::RADIO_INTENSET_END);
        self.disable_ppi(nrf5x::constants::PPI_CHEN_CH21);
        let crc_ok = if regs.crcok.get() == 1 {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        };

        // TODO create PDU struct with crc info

        self.schedule_tx_after_t_ifs();

        if let Some(client) = self.rx_client.get() {
            let result = unsafe { client.receive_end(&mut RX_PAYLOAD, RX_PAYLOAD[1] + 2, crc_ok) };

            match result {
                DisablePHY::DisableAfterRX => {
                    self.disable_radio();
                }
                _ => {
                    // Do nothing!
                }
            }
        } else {
            panic!("No rx_client?\n");
        }
    }

    fn handle_tx_end_event(&self) {
        let regs = unsafe { &*self.regs };

        regs.event_disabled.set(0);
        self.clear_interrupt(nrf5x::constants::RADIO_INTENSET_DISABLED);
        regs.event_end.set(0);

        // TODO set wfr_time (= NRF_RADIO->SHORTS)

        // TODO check transition TX_RX?
        if PhyTransition::MoveToRX == self.transition.get() {
            self.setup_rx();

            // TODO wfr_enable
            self.schedule_rx_after_t_ifs();
        } else {
            self.advertisement_client
                .get()
                .take()
                .map(|client| client.advertisement_done());

            // TODO timer->task_stop
            // TODO clear CHEN 4, 5, 20, 31

            assert_eq!(self.transition.get(), PhyTransition::None)
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        let mut enabled_interrupts = regs.intenclr.get();

        if (enabled_interrupts & nrf5x::constants::RADIO_INTENSET_ADDRESS) > 0
            && regs.event_address.get() == 1
        {
            if self.handle_address_event() {
                enabled_interrupts &= !nrf5x::constants::RADIO_INTENSET_DISABLED;
            }
        }

        if (enabled_interrupts & nrf5x::constants::RADIO_INTENSET_DISABLED) > 0
            && regs.event_disabled.get() == 1
        {
            if self.state.get() == RadioState::RX {
                regs.event_disabled.set(0);
                debug!("Timer expired in advertisement client!\n");
                self.advertisement_client
                    .get()
                    .take()
                    .map(|client| client.timer_expired());
            } else if self.state.get() == RadioState::Uninitialized {
                panic!("Oh no!\n");
            } else {
                self.handle_tx_end_event()
            }
        }

        if (enabled_interrupts & nrf5x::constants::RADIO_INTENSET_END) > 0
            && regs.event_end.get() == 1
        {
            self.handle_rx_end_event();
        }
    }

    pub fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(nrf5x::constants::RADIO_INTENSET_ADDRESS);
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
                TX_PAYLOAD[i] = *c;
            }
        }
        buf
    }

    fn get_packet_address_time_value(&self) -> u32 {
        unsafe { nrf5x::timer::TIMER0.get_cc1() }
    }

    fn get_packet_end_time_value(&self) -> u32 {
        unsafe { nrf5x::timer::TIMER0.get_cc2() }
    }

    fn enable_ppi(&self, pins: u32) {
        unsafe {
            ppi::PPI.enable(pins);
        }
    }

    fn disable_ppi(&self, pins: u32) {
        unsafe {
            ppi::PPI.disable(pins);
        }
    }

    pub fn ble_initialize(&self) {
        if self.state.get() == RadioState::Uninitialized {
            self.radio_on();

            self.ble_set_tx_power();
            self.set_tifs();

            self.ble_set_channel_rate();

            self.set_tx_address();
            self.set_rx_address();

            self.ble_set_packet_config();

            self.ble_set_crc_config();

            self.state.set(RadioState::Initialized);

            self.enable_ppi(nrf5x::constants::PPI_CHEN_CH26 | nrf5x::constants::PPI_CHEN_CH27);
        }
        unsafe {
            nrf5x::timer::TIMER0.start();
        }
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.1.1 CRC Generation
    fn ble_set_crc_config(&self) {
        let regs = unsafe { &*self.regs };
        regs.crccnf.set(
            nrf5x::constants::RADIO_CRCCNF_SKIPADDR << nrf5x::constants::RADIO_CRCCNF_SKIPADDR_POS
                | nrf5x::constants::RADIO_CRCCNF_LEN_3BYTES,
        );
        self.ble_set_crcinit(nrf5x::constants::RADIO_CRCINIT_BLE);
        regs.crcpoly.set(nrf5x::constants::RADIO_CRCPOLY_BLE);
    }

    fn ble_set_crcinit(&self, crcinit: u32) {
        let regs = unsafe { &*self.regs };
        regs.crcinit.set(crcinit);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.1.2 Access Address
    // Set access address to 0x8E89BED6
    pub fn ble_set_access_address(&self, aa: u32) {
        let regs = unsafe { &*self.regs };

        regs.prefix0
            .set((regs.prefix0.get() & 0xffffff00) | (aa >> 24));
        regs.base0.set(aa << 8);
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

        regs.modecnf0.set(NRF52_RADIO_MODECNF0_RU_FAST);
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
    fn ble_set_channel(&self, channel: RadioChannel) {
        let regs = unsafe { &*self.regs };
        self.channel.set(Some(channel));
        regs.frequency.set(channel as u32);
        self.ble_set_data_whitening(channel);
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
    fn transmit_advertisement(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8] {
        self.ble_initialize();
        let res = self.set_advertisement_data(buf, len);
        self.set_tx_start_time(300); //TODO - not sure about why tx is delayed by this time
        self.tx();

        res
    }

    fn set_advertisement_data(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8] {
        self.replace_radio_buffer(buf, len) // TODO replace signature to accomondate for a more flexible format of packets
    }

    fn receive_advertisement(&self) {
        self.ble_initialize();
        self.rx();
        self.enable_interrupts();
    }

    fn set_receive_client(&self, client: &'static nrf5x::ble_advertising_hil::RxClient) {
        self.rx_client.set(Some(client));
    }

    fn set_transmit_client(&self, client: &'static nrf5x::ble_advertising_hil::TxClient) {
        self.tx_client.set(Some(client));
    }
    fn set_advertisement_client(
        &self,
        client: &'static nrf5x::ble_advertising_hil::AdvertisementClient,
    ) {
        self.advertisement_client.set(Some(client));
    }
}

impl nrf5x::ble_advertising_hil::BleConfig for Radio {
    // The BLE Advertising Driver validates that the `tx_power` is between -20 to 10 dBm but then
    // underlying chip must validate if the current `tx_power` is supported as well
    fn set_tx_power(&self, tx_power: u8) -> kernel::ReturnCode {
        // Convert u8 to TxPower
        match nrf5x::constants::TxPower::try_from(tx_power) {
            // Invalid transmitting power, propogate error
            Err(_) => kernel::ReturnCode::ENOSUPPORT,
            // Valid transmitting power, propogate success
            Ok(res) => {
                self.tx_power.set(res);
                kernel::ReturnCode::SUCCESS
            }
        }
    }

    fn set_channel(&self, channel: RadioChannel, address: u32, crcinit: u32) {
        self.ble_set_channel(channel);
        self.ble_set_access_address(address);
        self.ble_set_crcinit(crcinit);
    }

    fn set_transition_state(&self, state: PhyTransition) {
        self.transition.set(state);
    }

    fn set_access_address(&self, aa: u32) {
        self.ble_set_access_address(aa)
    }
}
