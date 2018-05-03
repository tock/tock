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
use kernel::hil::gpio::Pin;
use nrf5x;
use ble_connection::ble_advertising_hil;
use ble_connection::ble_advertising_hil::{DelayStartPoint, PhyTransition, RadioChannel, ReadAction,
                                 TxImmediate};
use nrf5x::constants::TxPower;
use nrf5x::gpio;
use peripheral_registers;
use ppi;
use radio;
use kernel::common::regs::Field;
use kernel::common::regs::FieldValue;

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

const NRF52_DISABLE_TX_DELAY: u32 = NRF52_RX_END_DELAY + NRF52_FAST_RAMPUP_TIME_TX + NRF52_TX_DELAY;
const NRF52_DISABLE_RX_DELAY: u32 = NRF52_TX_END_DELAY + NRF52_FAST_RAMPUP_TIME_TX;

static mut TX_PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

static mut RX_PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

pub struct Radio {
    regs: *const radio::RadioRegisters,
    tx_power: Cell<TxPower>,
    rx_client: Cell<Option<&'static ble_advertising_hil::RxClient>>,
    tx_client: Cell<Option<&'static ble_advertising_hil::TxClient>>,
    advertisement_client: Cell<Option<&'static ble_advertising_hil::AdvertisementClient>>,
    state: Cell<RadioState>,
    channel: Cell<Option<RadioChannel>>,
    debug_bit: Cell<bool>,
    debug_value: Cell<u8>,
    address_receive_time: Cell<Option<u32>>,
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
            regs: radio::RADIO_BASE as *const radio::RadioRegisters,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: Cell::new(None),
            tx_client: Cell::new(None),
            advertisement_client: Cell::new(None),
            state: Cell::new(RadioState::Uninitialized),
            channel: Cell::new(None),
            debug_bit: Cell::new(false),
            debug_value: Cell::new(0),
            address_receive_time: Cell::new(None),
        }
    }

    pub fn tx(&self) {
        let regs = unsafe { &*self.regs };

        self.wait_until_disabled();

        self.setup_tx();

        regs.task_txen.write(radio::Task::ENABLE::SET);
    }

    fn setup_tx(&self) {
        let regs = unsafe { &*self.regs };

        self.set_dma_ptr_tx();
        self.state.set(RadioState::TX);

        regs.event_ready.write(radio::Event::READY::CLEAR);
        regs.event_end.write(radio::Event::READY::CLEAR);
        regs.event_disabled.write(radio::Event::READY::CLEAR);

        regs.shorts.write(
            radio::Shortcut::END_DISABLE::SET + radio::Shortcut::READY_START::SET
        );

        self.enable_interrupt(radio::Interrupt::DISABLED::SET);
    }

    fn setup_rx(&self) {
        let regs = unsafe { &*self.regs };

        self.set_dma_ptr_rx();

        // CH20: TIMER0.EVENTS_COMPARE[0] -> RADIO.TASKS_TXEN
        self.disable_ppi(nrf5x::constants::PPI_CHEN_CH20);

        self.state.set(RadioState::RX);

        regs.bcc.write(radio::BitCounterCompare::BCC.val(8)); // count one byte

        regs.event_address.write(radio::Event::READY::CLEAR);
        // regs.event_devmatch.write(radio::Event::READY::CLEAR);
        regs.event_bcmatch.write(radio::Event::READY::CLEAR);
        // regs.event_rssiend.write(radio::Event::READY::CLEAR);
        regs.event_crcok.write(radio::Event::READY::CLEAR);

        regs.shorts.write(
                radio::Shortcut::END_DISABLE::SET + radio::Shortcut::READY_START::SET
                    + radio::Shortcut::ADDRESS_BCSTART::SET
        );

        self.enable_interrupt(radio::Interrupt::ADDRESS::SET);
    }

    fn wait_until_disabled(&self) {
        let regs = unsafe { &*self.regs };

        let state = regs.state.get();

        if !regs.state.matches_all(radio::State::STATE.val(0)) {
            if regs.state.matches_any(radio::State::STATE.val(4) + radio::State::STATE.val(12)) {
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

        regs.event_end.write(radio::Event::READY::CLEAR);
        regs.event_disabled.write(radio::Event::READY::CLEAR);

        self.setup_rx();

        regs.task_rxen.write(radio::Task::ENABLE::SET);
    }

    fn set_rx_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.rxaddresses.write(radio::ReceiveAddresses::ADDRESS.val(1));
    }

    fn set_tx_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.txaddress.write(radio::TransmitAddress::ADDRESS.val(0));
    }

    fn radio_on(&self) {
        let regs = unsafe { &*self.regs };
        // reset and enable power
        regs.power.write(radio::Task::ENABLE::CLEAR);
        regs.power.write(radio::Task::ENABLE::SET);
    }

    fn radio_off(&self) {
        let regs = unsafe { &*self.regs };
        regs.shorts.set(0);
        regs.power.write(radio::Task::ENABLE::CLEAR);
    }

    fn set_tx_power(&self) {
        let regs = unsafe { &*self.regs };
        regs.txpower.set(self.tx_power.get() as u32);
    }

    fn set_tifs(&self) {
        let regs = unsafe { &*self.regs };
        regs.tifs.set(150u32);
    }

    fn set_dma_ptr_tx(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set(TX_PAYLOAD.as_ptr() as u32);
        }
    }

    fn set_dma_ptr_rx(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set(RX_PAYLOAD.as_ptr() as u32);
        }
    }

    fn get_packet_time_value_with_delay(&self, start_point: DelayStartPoint) -> u32 {
        match start_point {
            DelayStartPoint::PacketEndUsecDelay(_) | DelayStartPoint::PacketEndBLEStandardDelay => {
                self.get_packet_end_time_value() + start_point.value()
            }
            DelayStartPoint::PacketStartUsecDelay(_) => {
                self.get_packet_address_time_value() + start_point.value()
            }
            DelayStartPoint::AbsoluteTimestamp(ab) => ab,
        }
    }

    fn set_cc0(&self, usec: u32) {
        unsafe {
            nrf5x::timer::TIMER0.set_cc0(usec);
            nrf5x::timer::TIMER0.set_events_compare(0, 0);
        }
    }

    fn schedule_tx_after_us(&self, delay: DelayStartPoint) {
        self.setup_tx();

        let now = unsafe { nrf5x::timer::TIMER0.capture(4) };

        let t0 = self.get_packet_time_value_with_delay(delay);
        let time = t0 - NRF52_DISABLE_TX_DELAY;

        self.set_cc0(time);

        // CH20: CC[0] => TXEN
        self.enable_ppi(nrf5x::constants::PPI_CHEN_CH20);
    }

    fn schedule_rx_after_us(&self, delay: DelayStartPoint, timeout: u32) {
        self.setup_rx();

        let earlier_listen: u32 = 2;
        let t0 = self.get_packet_time_value_with_delay(delay);
        let time = t0 - NRF52_DISABLE_RX_DELAY - earlier_listen;

        self.set_cc0(time);

        // CH21: CC[0] => RXEN
        self.enable_ppi(nrf5x::constants::PPI_CHEN_CH21);

        self.set_rx_timeout(t0 + timeout);
    }

    fn set_rx_timeout(&self, usec: u32) {
        //Prepare timer to timeout 'timeout' usec after we have started to rx
        unsafe {
            nrf5x::timer::TIMER0.set_cc1(usec);
            nrf5x::timer::TIMER0.set_events_compare(1, 0);
        }

        self.enable_ppi(nrf5x::constants::PPI_CHEN_CH22 | nrf5x::constants::PPI_CHEN_CH26);
        self.enable_interrupt(radio::Interrupt::DISABLED::SET);
    }

    fn disable_radio(&self) {
        let regs = unsafe { &*self.regs };

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
        self.disable_ppi(nrf5x::constants::PPI_CHEN_CH22);

        self.address_receive_time
            .set(Some(unsafe { nrf5x::timer::TIMER0.get_cc1() }));

        self.clear_interrupt(
            radio::Interrupt::DISABLED::SET + radio::Interrupt::ADDRESS::SET,
        );

        loop {
            let state = regs.state.get();

            if regs.event_bcmatch.get() != 0 {
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
                ReadAction::ReadFrame => {
                    // We want to read packet, enable interrupt on EVENT_END
                    self.enable_interrupt(radio::Interrupt::END::SET);
                }
                ReadAction::SkipFrame => {
                    self.disable_radio();

                    self.handle_advertisement_done();
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

        self.clear_interrupt(radio::Interrupt::END::SET);

        // CH21: TIMER0.EVENTS_COMPARE[0] -> RADIO.RXEN
        self.disable_ppi(nrf5x::constants::PPI_CHEN_CH21);
        let crc_ok = if regs.crcstatus.is_set(radio::Event::READY) {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        };

        if let Some(client) = self.rx_client.get() {
            let result = unsafe {
                client.receive_end(
                    &mut RX_PAYLOAD,
                    RX_PAYLOAD[1] + 2,
                    crc_ok,
                    self.get_packet_address_time_value(),
                )
            };

            match result {
                PhyTransition::MoveToTX(delay) => {
                    self.schedule_tx_after_us(delay);
                }
                PhyTransition::MoveToRX(delay, timeout) => {
                    // Handle connection request
                    self.debug_bit.set(true);

                    let v = self.debug_value.get();
                    self.debug_value.set(v + 1);

                    self.disable_radio();
                    self.wait_until_disabled();
                    self.schedule_rx_after_us(delay, timeout);
                }
                PhyTransition::None => {
                    self.disable_radio();

                    self.handle_advertisement_done();
                }
            }
        } else {
            panic!("No rx_client?\n");
        }
    }

    fn handle_advertisement_done(&self) {
        self.wait_until_disabled();

        if let Some(client) = self.advertisement_client.get() {
            match client.advertisement_done() {
                TxImmediate::TX => self.tx(),
                TxImmediate::RespondAfterTifs => {
                    self.schedule_tx_after_us(DelayStartPoint::PacketEndBLEStandardDelay)
                }
                TxImmediate::GoToSleep => {}
            }
        } else {
            panic!("No advertisement client?");
        }
    }

    fn handle_tx_end_event(&self) {
        let regs = unsafe { &*self.regs };

        regs.event_disabled.write(radio::Event::READY::CLEAR);
        self.clear_interrupt(radio::Interrupt::DISABLED::SET);
        regs.event_end.write(radio::Event::READY::CLEAR);

        let crc_ok = if regs.crcstatus.is_set(radio::Event::READY) {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        };

        if let Some(client) = self.tx_client.get() {
            let result = client.transmit_end(crc_ok);

            match result {
                PhyTransition::MoveToTX(delay) => {
                    self.handle_advertisement_done();
                }
                PhyTransition::MoveToRX(delay, timeout) => {
                    self.schedule_rx_after_us(delay, timeout);
                }
                PhyTransition::None => {
                    self.disable_radio();

                    self.handle_advertisement_done();
                }
            }
        } else {
            panic!("No rx_client?\n");
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        // let current_time = unsafe {nrf5x::timer::TIMER0.capture(4) };
        let mut handled_disabled_event = false;

        if regs.intenclr.matches_any(radio::Interrupt::ADDRESS::SET)
            && regs.event_address.matches_any(radio::Event::READY::SET)
        {
            if self.handle_address_event() {
                handled_disabled_event = true;
            }
        }

        if !handled_disabled_event && regs.intenclr.matches_any(radio::Interrupt::DISABLED::SET)
            && regs.event_disabled.matches_any(radio::Event::READY::SET)
        {
            if self.state.get() == RadioState::RX {
                regs.event_disabled.write(radio::Event::READY::CLEAR);

                if self.debug_value.get() != 1 {
                    let transition = self.advertisement_client
                        .get()
                        .map_or(PhyTransition::None, |client| client.timer_expired());

                    self.wait_until_disabled();

                    match transition {
                        PhyTransition::MoveToTX(delay) => {
                            self.setup_tx();
                            self.tx();
                        }
                        PhyTransition::MoveToRX(delay, timeout) => {
                            self.schedule_rx_after_us(delay, timeout);
                        }
                        PhyTransition::None => {
                            //Do nothing, the device should sleep and wait for timer to fire in BLE
                        }
                    }
                }
            } else if self.state.get() == RadioState::Uninitialized {
                panic!("EVENT_DISABLED while Uninitialized?\n");
            } else {
                self.handle_tx_end_event()
            }
        }

        if regs.intenclr.matches_any(radio::Interrupt::END::SET)
            && regs.event_end.matches_any(radio::Event::READY::SET)
        {
            self.handle_rx_end_event();
        }
    }

    pub fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.write(radio::Interrupt::ADDRESS::SET);
    }

    pub fn enable_interrupt(&self, intr: FieldValue<u32, radio::Interrupt::Register>) {
        let regs = unsafe { &*self.regs };
        regs.intenset.write(intr);
    }

    pub fn clear_interrupt(&self, intr: FieldValue<u32, radio::Interrupt::Register>) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.write(intr);
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
        match self.address_receive_time.get() {
            Some(time) => time,
            None => {
                panic!("Trying to get time for last ADDRESS, but non has been saved\n");
            }
        }
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

            // CH26: RADIO.EVENTS_ADDRESS -> TIMER0.TASKS_CAPTURE[1]
            // CH27: RADIO.EVENTS_END -> TIMER0.TASKS_CAPTURE[2]
            self.enable_ppi(nrf5x::constants::PPI_CHEN_CH26 | nrf5x::constants::PPI_CHEN_CH27);
        }
        unsafe {
            nrf5x::timer::TIMER0.set_prescaler(4);
            nrf5x::timer::TIMER0.set_bitmode(3);
            nrf5x::timer::TIMER0.start();
        }
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.1.1 CRC Generation
    fn ble_set_crc_config(&self) {
        let regs = unsafe { &*self.regs };
        regs.crccnf
            .write(radio::CrcConfiguration::LEN::THREE + radio::CrcConfiguration::SKIPADDR::EXCLUDE);
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
        regs.pcnf0.write(
            radio::PacketConfiguration0::LFLEN.val(8) + radio::PacketConfiguration0::S0LEN.val(1)
                + radio::PacketConfiguration0::S1LEN::CLEAR
                + radio::PacketConfiguration0::S1INCL::CLEAR
                + radio::PacketConfiguration0::PLEN::EIGHT,
        );

        regs.pcnf1.write(
            radio::PacketConfiguration1::WHITEEN::ENABLED + radio::PacketConfiguration1::ENDIAN::LITTLE
                + radio::PacketConfiguration1::BALEN.val(3)
                + radio::PacketConfiguration1::STATLEN::CLEAR
                + radio::PacketConfiguration1::MAXLEN.val(255),
        );

        regs.modecnf0.write(radio::RadioModeConfig::RU.val(1)); // FAST
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

        assert_eq!(nrf5x::constants::RADIO_STATE_DISABLE, regs.state.get());

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

impl ble_advertising_hil::BleAdvertisementDriver for Radio {
    fn transmit_advertisement(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8] {
        self.ble_initialize();
        let res = self.set_advertisement_data(buf, len);
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

    fn set_receive_client(&self, client: &'static ble_advertising_hil::RxClient) {
        self.rx_client.set(Some(client));
    }

    fn set_transmit_client(&self, client: &'static ble_advertising_hil::TxClient) {
        self.tx_client.set(Some(client));
    }
    fn set_advertisement_client(
        &self,
        client: &'static ble_advertising_hil::AdvertisementClient,
    ) {
        self.advertisement_client.set(Some(client));
    }
}

impl ble_advertising_hil::BleConfig for Radio {
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

    fn set_access_address(&self, aa: u32) {
        self.ble_set_access_address(aa)
    }
}
