//! Radio driver, Bluetooth Low Energy, nRF51
//!
//! Sending Bluetooth Low Energy advertisement packets with payloads up to 31 bytes
//!
//! Currently all fields in PAYLOAD array are configurable from user-space
//! except the PDU_TYPE.
//!
//! ### Authors
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: June 22, 2017

use core::cell::Cell;
use kernel;
use kernel::ReturnCode;
use nrf5x;
use nrf5x::ble_advertising_hil::RadioFrequency;
use nrf5x::constants::TxPower;
use peripheral_registers;

static mut PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

pub struct Radio {
    regs: *const peripheral_registers::RADIO_REGS,
    tx_power: Cell<TxPower>,
    rx_client: Cell<Option<&'static nrf5x::ble_advertising_hil::RxClient>>,
    tx_client: Cell<Option<&'static nrf5x::ble_advertising_hil::TxClient>>,
}

pub static mut RADIO: Radio = Radio::new();

impl Radio {
    pub const fn new() -> Radio {
        Radio {
            regs: peripheral_registers::RADIO_BASE as *const peripheral_registers::RADIO_REGS,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: Cell::new(None),
            tx_client: Cell::new(None),
        }
    }


    fn ble_init(&self, channel: RadioFrequency) {
        let regs = unsafe { &*self.regs };

        self.radio_on();

        // TX Power acc. to twpower variable in the struct
        self.set_tx_power();

        // BLE MODE
        self.set_channel_rate(nrf5x::constants::RadioMode::Ble1Mbit as u32);

        self.set_channel_freq(channel);
        self.set_data_whitening(channel);

        // Set PREFIX | BASE Address
        regs.prefix0.set(0x0000008e);
        regs.base0.set(0x89bed600);

        self.set_tx_address(0x00);
        self.set_rx_address(0x01);

        // Set Packet Config
        self.set_packet_config(0x00);

        // CRC Config
        self.set_crc_config();

        // Buffer configuration
        self.set_buffer();
    }

    fn tx(&self) {
        let regs = unsafe { &*self.regs };
        regs.ready.set(0);
        regs.txen.set(1);
    }

    fn rx(&self) {
        let regs = unsafe { &*self.regs };
        regs.ready.set(0);
        regs.rxen.set(1);
    }

    fn set_crc_config(&self) {
        let regs = unsafe { &*self.regs };
        regs.crccnf.set(
            nrf5x::constants::RADIO_CRCCNF_LEN_3BYTES
                | nrf5x::constants::RADIO_CRCCNF_SKIPADDR
                    << nrf5x::constants::RADIO_CRCCNF_SKIPADDR_POS,
        );
        regs.crcinit.set(nrf5x::constants::RADIO_CRCINIT_BLE);
        regs.crcpoly.set(nrf5x::constants::RADIO_CRCPOLY_BLE);
    }

    // Packet configuration
    fn set_packet_config(&self, _: u32) {
        let regs = unsafe { &*self.regs };
        regs.pcnf0.set(
            (nrf5x::constants::RADIO_PCNF0_S0_LEN_1BYTE << nrf5x::constants::RADIO_PCNF0_S0LEN_POS)
                | (nrf5x::constants::RADIO_PCNF0_LFLEN_1BYTE
                    << nrf5x::constants::RADIO_PCNF0_LFLEN_POS),
        );

        regs.pcnf1.set(
            (nrf5x::constants::RADIO_PCNF1_WHITEEN_ENABLED <<
                nrf5x::constants::RADIO_PCNF1_WHITEEN_POS) |
                 (nrf5x::constants::RADIO_PCNF1_ENDIAN_LITTLE <<
                     nrf5x::constants::RADIO_PCNF1_ENDIAN_POS) |
                 // Total Address is 4 bytes (BASE ADDRESS + PREFIX (1))
                 (nrf5x::constants::RADIO_PCNF1_BALEN_3BYTES <<
                  nrf5x::constants::RADIO_PCNF1_BALEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_STATLEN_DONT_EXTEND
                    << nrf5x::constants::RADIO_PCNF1_STATLEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_MAXLEN_37BYTES
                    << nrf5x::constants::RADIO_PCNF1_MAXLEN_POS),
        );
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

    fn set_data_whitening(&self, channel: RadioFrequency) {
        let regs = unsafe { &*self.regs };
        regs.datawhiteiv.set(channel.get_channel_index());
    }

    fn set_channel_freq(&self, channel: RadioFrequency) {
        let regs = unsafe { &*self.regs };
        //37, 38 and 39 for adv.
        regs.frequency.set(channel as u32);
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

    // pre-condition validated before arriving here
    fn set_tx_power(&self) {
        let regs = unsafe { &*self.regs };
        regs.txpower.set(self.tx_power.get() as u32);
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
        self.disable_interrupts();

        if regs.ready.get() == 1 {
            regs.ready.set(0);
            regs.end.set(0);
            regs.start.set(1);
        }

        if regs.payload.get() == 1 {
            regs.payload.set(0);
        }

        if regs.address.get() == 1 {
            regs.address.set(0);
        }

        if regs.end.get() == 1 {
            regs.end.set(0);
            regs.disable.set(1);

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

    pub fn disable_interrupts(&self) {
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
}

impl nrf5x::ble_advertising_hil::BleAdvertisementDriver for Radio {
    fn transmit_advertisement(&self,
                              buf: &'static mut [u8],
                              len: usize,
                              channel: RadioFrequency)
                              -> &'static mut [u8] {
        let res = self.replace_radio_buffer(buf, len);
        self.ble_init(channel);
        self.tx();
        self.enable_interrupts();
        res
    }

    fn receive_advertisement(&self, channel: RadioFrequency) {
        self.ble_init(channel);
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

// The capsule validates that the `tx_power` is between -20 to 10 dBm but then
// chip must validate if the current `tx_power` is supported as well
impl nrf5x::ble_advertising_hil::BleConfig for Radio {
    fn set_tx_power(&self, power: u8) -> kernel::ReturnCode {
        match nrf5x::constants::TxPower::from_u8(power) {
            TxPower::Error => kernel::ReturnCode::ENOSUPPORT,
            e @ _ => {
                self.tx_power.set(e);
                kernel::ReturnCode::SUCCESS
            }
        }
    }
}
